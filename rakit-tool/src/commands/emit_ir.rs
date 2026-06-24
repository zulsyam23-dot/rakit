use std::fs;
use std::path::Path;
use rakit_core::{SourceMap, report_diagnostics};
use rakit_frontend::parser::Parser;
use rakit_ir_hir::lower::HirLower;
use rakit_ir_hir::pretty::HirPrettyPrinter;

pub struct EmitIrCommand;

impl EmitIrCommand {
    pub fn run(file: &Path, format: &str, level: &str) -> Result<(), String> {
        let source = fs::read_to_string(file)
            .map_err(|e| format!("Gagal membaca file '{}': {}", file.display(), e))?;

        let source_map = SourceMap::new(
            &file.file_name().unwrap_or_default().to_string_lossy(),
            &source,
        );

        let parser = Parser::new(&source, &source_map);
        let program = parser.parse_program();

        match program {
            Ok(program) => {
                match level {
                    "hir" => {
                        let mut lower = HirLower::new();
                        let hir = lower.lower_program(&program)
                            .map_err(|diags| {
                                report_diagnostics(&source_map, &diags);
                                "Lowering HIR gagal.".to_string()
                            })?;

                        let mut printer = HirPrettyPrinter::new();
                    println!("{}", printer.print_program(&hir));
                    }
                    "ast" | _ => {
                        match format {
                            "json" => {
                                let json_str = serde_json::to_string_pretty(&program)
                                    .map_err(|e| format!("Gagal serialize JSON: {}", e))?;
                                println!("{}", json_str);
                            }
                            "pretty" | _ => {
                                let mut printer = rakit_ir_ast::AstPrettyPrinter::new();
                                println!("{}", printer.print(&program));
                            }
                        }
                    }
                }
                Ok(())
            }
            Err(diagnostics) => {
                report_diagnostics(&source_map, &diagnostics);
                Err("Emit IR gagal karena ada kesalahan parsing.".to_string())
            }
        }
    }
}
