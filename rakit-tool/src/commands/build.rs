use std::fs;
use std::path::Path;
use rakit_core::{SourceMap, report_diagnostics};
use rakit_frontend::parser::Parser;
use rakit_ir_hir::lower::HirLower;
use rakit_ir_hir::resolve::NameResolver;
use rakit_ir_hir::ty::checker::TypeChecker;
use rakit_ir_hir::pretty::HirPrettyPrinter;
use rakit_opt::Optimizer;

pub struct BuildCommand;

impl BuildCommand {
    pub fn run(file: &Path, json: bool) -> Result<(), String> {
        Self::run_with_opts(file, json, None, None, 1)
    }

    pub fn run_with_opts(
        file: &Path,
        json: bool,
        target: Option<&str>,
        output: Option<&Path>,
        optimize_level: u8,
    ) -> Result<(), String> {
        let source = fs::read_to_string(file)
            .map_err(|e| format!("Gagal membaca file '{}': {}", file.display(), e))?;

        let source_map = SourceMap::new(
            &file.file_name().unwrap_or_default().to_string_lossy(),
            &source,
        );

        let parser = Parser::new(&source, &source_map);
        let program = match parser.parse_program() {
            Ok(program) => program,
            Err(diagnostics) => {
                report_diagnostics(&source_map, &diagnostics);
                return Err("Build gagal karena ada kesalahan parsing.".to_string());
            }
        };

        if json {
            let json_str = serde_json::to_string_pretty(&program)
                .map_err(|e| format!("Gagal serialize JSON: {}", e))?;
            println!("{}", json_str);
            return Ok(());
        }

        // Fase 1: Cetak AST
        let mut ast_printer = rakit_ir_ast::AstPrettyPrinter::new();
        println!("=== AST ===\n{}", ast_printer.print(&program));

        // Fase 2: Lowering ke HIR
        let mut lower = HirLower::new();
        let mut hir = lower.lower_program(&program)
            .map_err(|diags| {
                report_diagnostics(&source_map, &diags);
                "Lowering HIR gagal.".to_string()
            })?;

        // Fase 3: Name resolution
        let mut resolver = NameResolver::new();
        let resolved = resolver.resolve_program(&mut hir);
        if !resolver.diagnostics.is_empty() {
            report_diagnostics(&source_map, &resolver.diagnostics);
        }
        if !resolved {
            return Err("Name resolution menemukan error.".to_string());
        }

        // Fase 4: Type checking
        let mut checker = TypeChecker::new();
        let checked = checker.check_program(&mut hir);
        if !checker.diagnostics.is_empty() {
            report_diagnostics(&source_map, &checker.diagnostics);
        }
        if !checked {
            return Err("Type checking menemukan error.".to_string());
        }

        // Fase 5: Optimasi
        if optimize_level > 0 {
            let optimizer = Optimizer::with_default_passes();
            let summary = optimizer.run_optimized(&mut hir);
            println!("\n=== Optimasi ===");
            for result in &summary.results {
                println!("  {}: {}", result.pass, result.description);
            }
        }

        // Cetak HIR
        let mut hir_printer = HirPrettyPrinter::new();
        println!("\n=== HIR (desugared + type-checked) ===");
        println!("{}", hir_printer.print_program(&hir));

        // Fase 6: Bridge ke Brak IR
        println!("\n=== Brak IR Bridge ===");
        let bridge = rakit_bridge::ast_to_brak::RakitToBrakBridge::new();
        match bridge.convert_program(&hir) {
            Ok(brak) => {
                println!("  Brak IR: {} items (konversi sukses)", brak.items.len());
            }
            Err(e) => {
                println!("  Brak IR: dilewati ({})", e);
            }
        }

        if let Some(t) = target {
            println!("  Target platform: {}", t);
        }

        if let Some(out) = output {
            println!("  Output: {}", out.display());
        }

        println!("\n✅ Build sukses! AST: {} item, HIR: {} item, Type check: OK, Opt level: {}",
            program.items.len(), hir.items.len(), optimize_level);
        Ok(())
    }
}
