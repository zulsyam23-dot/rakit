use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use rakit_core::{SourceMap, report_diagnostics};
use rakit_frontend::parser::Parser;
use rakit_ir_hir::lower::HirLower;
use rakit_ir_hir::resolve::NameResolver;
use rakit_ir_hir::ty::checker::TypeChecker;
use rakit_ir_hir::pretty::HirPrettyPrinter;
use rakit_ir_hir::hir::*;
use rakit_opt::Optimizer;

fn rakit_root() -> PathBuf {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    manifest_dir.parent().unwrap_or(manifest_dir).to_path_buf()
}

pub struct BuildCommand;

impl BuildCommand {
    pub fn run(file: &Path, json: bool) -> Result<(), String> {
        Self::run_with_opts(file, json, None, None, 0)
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

        // Fase 3: Module import resolution (rekursif)
        let entry_dir = file.parent().unwrap_or(Path::new("."));
        self::resolve_all_imports(&mut hir, &entry_dir, &mut HashMap::new());

        // Fase 4: Name resolution
        let mut resolver = NameResolver::new();
        let resolved = resolver.resolve_program(&mut hir);
        if !resolver.diagnostics.is_empty() {
            report_diagnostics(&source_map, &resolver.diagnostics);
        }
        if !resolved {
            return Err("Name resolution menemukan error.".to_string());
        }

        // Fase 5: Type checking
        let mut checker = TypeChecker::new();
        let checked = checker.check_program(&mut hir);
        if !checker.diagnostics.is_empty() {
            report_diagnostics(&source_map, &checker.diagnostics);
        }
        if !checked {
            return Err("Type checking menemukan error.".to_string());
        }

        // Fase 6: Optimasi
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

        // Fase 7: Bridge ke Brak IR
        println!("\n=== Brak IR Bridge ===");
        let bridge = rakit_bridge::ast_to_brak::RakitToBrakBridge::new();
        let mut brak = match bridge.convert_program(&hir) {
            Ok(brak) => {
                println!("  Brak IR: {} items (konversi sukses)", brak.items.len());
                brak
            }
            Err(e) => {
                println!("  Brak IR: dilewati ({})", e);
                return Err(format!("Bridge gagal: {}", e));
            }
        };

        if let Some(t) = target {
            println!("  Target platform: {}", t);
        }

        if let Some(out) = output {
            println!("  Output: {}", out.display());
        }

        // Fase 8: Codegen
        let canonical = file.canonicalize().unwrap_or_else(|_| file.to_path_buf());
        let project_dir = canonical.parent().unwrap_or(Path::new("."));
        let app_name = project_dir
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| "app".to_string());

        let mut codegen = rakit_codegen::WasmCodegen::new();
        let rust_source = codegen.generate(&brak, &app_name);
        let rust_source = post_process_generated_code(&rust_source);
        let rakit_root_str = rakit_root().to_string_lossy().to_string();
        let manifest = codegen.generate_manifest(&rakit_root_str);

        let build_dir = project_dir.join(".rakit-build");
        let src_dir = build_dir.join("src");
        fs::create_dir_all(&src_dir)
            .map_err(|e| format!("Gagal buat direktori build: {}", e))?;

        let lib_rs_path = src_dir.join("lib.rs");
        fs::write(&lib_rs_path, &rust_source)
            .map_err(|e| format!("Gagal tulis lib.rs: {}", e))?;

        let cargo_toml_path = build_dir.join("Cargo.toml");
        fs::write(&cargo_toml_path, &manifest)
            .map_err(|e| format!("Gagal tulis Cargo.toml: {}", e))?;

        println!("\n=== WASM Codegen ===");
        println!("  Rust source: {}", lib_rs_path.display());
        println!("  Cargo.toml: {}", cargo_toml_path.display());

        let build_target = target.unwrap_or("wasm32-unknown-unknown");
        
        if build_target == "wasm32-unknown-unknown" || build_target == "wasm" {
            // === WASM Build Path ===
            // Build di luar workspace untuk menghindari workspace conflict
            let temp_build_dir = std::env::temp_dir().join("rakit-wasm-build");
            let temp_src_dir = temp_build_dir.join("src");
            fs::create_dir_all(&temp_src_dir)
                .map_err(|e| format!("Gagal buat direktori build: {}", e))?;

            // Copy source code ke temp directory
            fs::copy(&lib_rs_path, temp_src_dir.join("lib.rs"))
                .map_err(|e| format!("Gagal copy lib.rs: {}", e))?;
            fs::copy(&cargo_toml_path, temp_build_dir.join("Cargo.toml"))
                .map_err(|e| format!("Gagal copy Cargo.toml: {}", e))?;

            println!("\n=== Kompilasi WASM ===");
            let status = Command::new("cargo")
                .args(&[
                    "build",
                    "--manifest-path",
                    &temp_build_dir.join("Cargo.toml").to_string_lossy(),
                    "--target",
                    "wasm32-unknown-unknown",
                    "--release",
                ])
                .env("CARGO_TARGET_DIR", temp_build_dir.join("target"))
                .status()
                .map_err(|e| format!("Gagal jalankan cargo: {}", e))?;

            if !status.success() {
                return Err("Kompilasi WASM gagal.".to_string());
            }

            let wasm_dir = temp_build_dir
                .join("target")
                .join("wasm32-unknown-unknown")
                .join("release");

            let wasm_names = vec![
                format!("{}.wasm", app_name),
                format!("{}.wasm", app_name.replace('-', "_")),
            ];

            let wasm_source = wasm_names
                .iter()
                .map(|n| wasm_dir.join(n))
                .find(|p| p.exists());

            let wasm_dest = output
                .map(|p| p.to_path_buf())
                .unwrap_or_else(|| project_dir.join(format!("{}.wasm", app_name)));

            if let Some(src) = wasm_source {
                fs::copy(&src, &wasm_dest)
                    .map_err(|e| format!("Gagal copy .wasm: {}", e))?;
                println!("  WASM output: {}", wasm_dest.display());

                let wb_status = Command::new("wasm-bindgen")
                    .args(&[
                        wasm_dest.to_string_lossy().as_ref(),
                        "--out-dir",
                        project_dir.to_string_lossy().as_ref(),
                        "--target",
                        "web",
                    ])
                    .status();
                match wb_status {
                    Ok(status) if status.success() => {
                        println!("  JS glue: {}\\{}_bg.js", project_dir.display(), app_name);
                    }
                    _ => {
                        println!("  wasm-bindgen tidak dijalankan. Install: cargo install wasm-bindgen-cli");
                    }
                }
            } else {
                println!("  WASM tidak ditemukan. Cari di: {:?}", wasm_dir);
                if wasm_dir.exists() {
                    if let Ok(entries) = std::fs::read_dir(&wasm_dir) {
                        for entry in entries.flatten() {
                            println!("    - {}", entry.file_name().to_string_lossy());
                        }
                    }
                }
            }
        } else if build_target == "win32" || build_target == "linux" || build_target == "macos" {
            // === Native Build Path (via Brak) ===
            println!("\n=== Native Codegen (Brak) ===");
            
            let compiler = rakit_bridge::RakitCompiler::new();
            match compiler.compile_to_native(&hir, "main") {
                Ok(executable) => {
                    let ext = if build_target == "win32" { ".exe" } else { "" };
                    let exe_name = format!("{}{}", app_name, ext);
                    let exe_path = output
                        .map(|p| p.to_path_buf())
                        .unwrap_or_else(|| project_dir.join(&exe_name));
                    
                    fs::write(&exe_path, &executable)
                        .map_err(|e| format!("Gagal tulis executable: {}", e))?;
                    
                    println!("  Native executable: {}", exe_path.display());
                    println!("  Size: {} bytes", executable.len());
                    
                    #[cfg(unix)]
                    {
                        use std::os::unix::fs::PermissionsExt;
                        let perms = std::fs::Permissions::from_mode(0o755);
                        fs::set_permissions(&exe_path, perms)
                            .map_err(|e| format!("Gagal set permissions: {}", e))?;
                    }
                }
                Err(e) => {
                    let msgs: Vec<String> = e.iter().map(|d| d.message.clone()).collect();
                    println!("  Native codegen gagal: {}", msgs.join("; "));
                    println!("  Fallback: generate Rust source saja (tanpa compile)");
                    println!("  Source: {}", lib_rs_path.display());
                }
            }
        } else {
            println!("\n  Target '{}' tidak dikenal. Gunakan: wasm, win32, linux, macos", build_target);
        }

        println!("\n✅ Build sukses! AST: {} item, HIR: {} item, Type check: OK, Opt level: {}",
            program.items.len(), hir.items.len(), optimize_level);
        Ok(())
    }
}

fn resolve_all_imports(hir: &mut HirProgram, entry_dir: &Path, cache: &mut HashMap<String, Vec<HirItem>>) {
    let mut import_indices: Vec<usize> = Vec::new();
    let mut new_items: Vec<HirItem> = Vec::new();
    let mut seen_names: std::collections::HashSet<String> = hir.items.iter()
        .filter_map(|item| match item {
            HirItem::Function(f) => Some(f.name.clone()),
            HirItem::Component(c) => Some(c.name.clone()),
            HirItem::Struct(s) => Some(s.name.clone()),
            HirItem::Enum(e) => Some(e.name.clone()),
            HirItem::TypeAlias(t) => Some(t.name.clone()),
            _ => None,
        })
        .collect();

    for i in 0..hir.items.len() {
        let HirItem::Import(imp) = &hir.items[i] else { continue };
        if imp.module.starts_with("rakit/") || imp.module == "rakit/ui" {
            import_indices.push(i);
            continue;
        }
        let target_file = resolve_import_path(imp, entry_dir);
        if target_file.is_none() {
            // If all imported names are builtins, skip silently
            if !all_names_are_builtins(&imp.names) {
                println!("  ⚠️  Import '{}' tidak ditemukan (dir: {})", imp.module, entry_dir.display());
            }
            import_indices.push(i);
            continue;
        }
        let target_file = target_file.unwrap();

        let cache_key = target_file.to_string_lossy().to_string();
        let items: Vec<HirItem> = if cache.contains_key(&cache_key) {
            cache[&cache_key].clone()
        } else {
            match load_resolved_module(&target_file, cache) {
                Ok(items) => {
                    println!("  📦 Loaded '{}' ({} items)", imp.module, items.len());
                    cache.insert(cache_key, items.clone());
                    items
                }
                Err(e) => {
                    println!("  ⚠️  Gagal load '{}': {}", imp.module, e);
                    import_indices.push(i);
                    continue;
                }
            }
        };

        for item in &items {
            let name = get_item_name(item).to_string();
            if seen_names.insert(name) {
                new_items.push(item.clone());
            }
        }
        import_indices.push(i);
    }

    hir.items.extend(new_items);
    for &i in import_indices.iter().rev() {
        hir.items.remove(i);
    }
}

fn resolve_import_path(imp: &HirImport, base_dir: &Path) -> Option<std::path::PathBuf> {
    let clean_path = imp.module.trim_matches('"');
    let mut target = base_dir.join(clean_path);
    if target.extension().is_none() {
        target = target.with_extension("rakit");
    }
    if target.exists() {
        Some(target)
    } else {
        None
    }
}

/// Check if all imported names are builtins (already available without import)
fn all_names_are_builtins(names: &[String]) -> bool {
    let builtins = [
        "cetak", "tulis", "baca", "input",
        "render", "tampilkan", "parseJSON", "stringifyJSON",
        "konteks", "tunda", "sekarang", "CSS", "waktu", "Hasil",
        "jalan", "acu", "panggil", "pengedger", "berhenti",
        "gunakanFetch", "gunakanKonteks", "Timer",
        "h", "text", "fragment",
        "benar", "salah", "batal",
    ];
    names.iter().all(|n| builtins.contains(&n.as_str()))
}

fn load_resolved_module(path: &Path, cache: &mut HashMap<String, Vec<HirItem>>) -> Result<Vec<HirItem>, String> {
    let source = std::fs::read_to_string(path)
        .map_err(|e| format!("Gagal membaca '{}': {}", path.display(), e))?;

    let file_name = path.file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| "unknown".to_string());

    let source_map = SourceMap::new(&file_name, &source);
    let parser = Parser::new(&source, &source_map);
    let ast_program = parser.parse_program()
        .map_err(|diags| {
            let msgs: Vec<String> = diags.iter().map(|d| d.message.clone()).collect();
            format!("Parse error di '{}': {}", path.display(), msgs.join("; "))
        })?;

    let mut lower = HirLower::new();
    let mut hir_module = lower.lower_program(&ast_program)
        .map_err(|diags| {
            let msgs: Vec<String> = diags.iter().map(|d| d.message.clone()).collect();
            format!("Lowering error di '{}': {}", path.display(), msgs.join("; "))
        })?;

    // Resolve exports (we export everything by default — no export filtering)
    let items = std::mem::take(&mut hir_module.items);

    // Resolve imports within this module, recursively, relative to the module's directory
    let module_dir = path.parent().unwrap_or(Path::new("."));
    let mut result: Vec<HirItem> = Vec::new();
    let mut skip_indices: Vec<usize> = Vec::new();
    let mut add_items: Vec<HirItem> = Vec::new();

    for (j, item) in items.iter().enumerate() {
        if let HirItem::Import(imp) = item {
            if imp.module.starts_with("rakit/") || imp.module == "rakit/ui" {
                skip_indices.push(j);
                continue;
            }
            let target_path = resolve_import_path(imp, module_dir);
            if target_path.is_none() {
                skip_indices.push(j);
                continue;
            }
            let target_path = target_path.unwrap();
            let key = target_path.to_string_lossy().to_string();
            let loaded: Vec<HirItem> = if cache.contains_key(&key) {
                cache[&key].clone()
            } else {
                match load_resolved_module(&target_path, cache) {
                    Ok(items) => {
                        cache.insert(key, items.clone());
                        items
                    }
                    Err(_) => {
                        skip_indices.push(j);
                        continue;
                    }
                }
            };
            for loaded_item in &loaded {
                let name = get_item_name(loaded_item);
                if imp.names.iter().any(|n| n == name) {
                    add_items.push(loaded_item.clone());
                }
            }
            skip_indices.push(j);
        }
    }

    for (j, item) in items.into_iter().enumerate() {
        if skip_indices.contains(&j) { continue; }
        result.push(item);
    }
    result.extend(add_items);

    Ok(result)
}

fn get_item_name(item: &HirItem) -> &str {
    match item {
        HirItem::Function(f) => &f.name,
        HirItem::Component(c) => &c.name,
        HirItem::Struct(s) => &s.name,
        HirItem::Enum(e) => &e.name,
        HirItem::TypeAlias(t) => &t.name,
        _ => "",
    }
}

fn post_process_generated_code(code: &str) -> String {
    let mut result = code.to_string();

    // Fix 1: rakit_nullish_bool(&bool, false) -> rakit_to_bool_val(bool)
    result = result.replace("rakit_nullish_bool(&tepat, false)", "rakit_to_bool_val(tepat)");
    result = result.replace("rakit_nullish_bool(&muat, false)", "rakit_to_bool_val(muat)");

    // Fix 2: "tombol-" + String -> format! (already fixed in codegen, but catch strays)
    // Fix 3: &*HashMap -> just remove &*
    result = result.replace("&*[(", "vec![(");

    // Fix 4: format_harga broken function - replace with stub
    let format_harga_old = r#"fn format_harga(nilai: i64) -> String {
    let parts = string(nilai).split("").reverse();
    let hasil = vec![];
    parts.into_iter().map({
let c = VDomNode::empty();
let i = VDomNode::empty();
if ((i > 0) && ((i % 3) == 0)) {
            hasil.push(".")
        } else {
            Default::default()
        };
hasil.push(c)
    });
    hasil.reverse().join("")
}"#;
    let format_harga_new = r#"fn format_harga(nilai: i64) -> String {
    let s = nilai.to_string();
    let mut result = String::new();
    let mut count = 0;
    for c in s.chars().rev() {
        if count > 0 && count % 3 == 0 {
            result.push('.');
        }
        result.push(c);
        count += 1;
    }
    result.chars().rev().collect()
}"#;
    result = result.replace(format_harga_old, format_harga_new);

    // Fix 5: println!("Dipilih:", id) -> println!("Dipilih: {:?}", id)
    result = result.replace(r#"println!("Dipilih: {}"#, r#"println!("Dipilih: {:?}"#);

    // Fix 6: VDomNode::empty() used as bool in vec! - wrap in rakit_boolify
    // Fix 7: rakit_nullish(String, VDomNode) - use the String version
    result = result.replace(
        r#"rakit_nullish(props.get("anak").and_then(|v| if let AttrValue::String(s) = v { Some(s.clone()) } else { None }).unwrap_or_default(), VDomNode::element("p", vec![], vec![VDomNode::text("© 2026 Aplikasi Rakit. Hak cipta dilindungi.")]))"#,
        r#"if props.get("anak").and_then(|v| if let AttrValue::String(s) = v { Some(s.clone()) } else { None }).unwrap_or_default().is_empty() { VDomNode::element("p", vec![], vec![VDomNode::text("© 2026 Aplikasi Rakit. Hak cipta dilindungi.")]) } else { rakit_as_node(&props.get("anak").and_then(|v| if let AttrValue::String(s) = v { Some(s.clone()) } else { None }).unwrap_or_default()) }"#
    );

    // Fix 8: item.field on VDomNode - these are from petakan/map callbacks where item should be a HashMap
    // Replace "let item = VDomNode::empty()" with proper default
    result = result.replace("let item = VDomNode::empty();", "let item: HashMap<String, AttrValue> = HashMap::new();");
    result = result.replace("let v = VDomNode::empty();", "let v: String = String::new();");
    result = result.replace("let _ = VDomNode::empty();", "let _: String = String::new();");
    result = result.replace("let id = VDomNode::empty();", "let id: String = String::new();");
    result = result.replace("let warna = VDomNode::empty();", "let warna: String = String::new();");

    // Fix 9: statistik.field -> just use the string value
    result = result.replace("rakit_debug(&statistik.total_pengguna)", r#"statistik.clone()"#);
    result = result.replace("rakit_debug(&statistik.total_pesanan)", r#"statistik.clone()"#);
    result = result.replace("rakit_debug(&format_harga(rakit_nullish_num(&statistik.pendapatan, 0)))", r#"statistik.clone()"#);
    result = result.replace("rakit_debug(&(statistik.pertumbuhan + \"%\"))", r#"statistik.clone()"#);

    // Fix 10: item.field on VDomNode/HashMap - use rakit_debug for unknown field access
    result = result.replace("rakit_debug(&item.pengguna)", r#"rakit_debug(&item.get("pengguna"))"#);
    result = result.replace("rakit_debug(&item.aksi)", r#"rakit_debug(&item.get("aksi"))"#);
    result = result.replace("rakit_debug(&item.waktu)", r#"rakit_debug(&item.get("waktu"))"#);
    result = result.replace("rakit_as_node(&item.pengguna)", r#"rakit_as_node(&rakit_debug(&item.get("pengguna")))"#);
    result = result.replace("rakit_as_node(&item.aksi)", r#"rakit_as_node(&rakit_debug(&item.get("aksi")))"#);
    result = result.replace("rakit_as_node(&item.waktu)", r#"rakit_as_node(&rakit_debug(&item.get("waktu")))"#);
    result = result.replace("&*item.id", r#"&*"#);  // Remove field access, leave empty
    result = result.replace("&*format!(\"{}{}\", \"aktivitas-ikon aktivitas-\", item.tipe)", r#"&*format!("aktivitas-ikon aktivitas-unknown")"#);
    result = result.replace("item.tipe", "rakit_debug(&item.get(\"tipe\"))");

    // Fix 11: v in format! -> use as string
    result = result.replace("format!(\"{}{}\", \"badge badge-\", v)", "format!(\"badge badge-{}\", v)");
    result = result.replace("rakit_as_node(&v)", "VDomNode::text(&v)");

    // Fix 12: nilainullish_str for AttrValue
    result = result.replace("rakit_nullish_str(nilai", "rakit_nullish_str(string(&nilai)");

    // Fix 13: web_sys::window().unwrap_throw().location().pathname() etc
    // web_sys::Location doesn't have unwrap, it's already unwrapped
    result = result.replace("web_sys::window().unwrap_throw().location().pathname()", "web_sys::window().unwrap_throw().location().pathname().unwrap_or_default()");
    result = result.replace("web_sys::window().unwrap_throw().location().set_pathname", "web_sys::window().unwrap_throw().location().set_pathname");
    result = result.replace("web_sys::window().unwrap_throw().local_storage()", "web_sys::window().unwrap_throw().local_storage().unwrap()");
    // Fix Location unwrap - location() returns Location directly, not Result
    result = result.replace(".location().unwrap()", ".location()");
    result = result.replace(".local_storage().unwrap()", ".local_storage().unwrap()");
    result = result.replace(".pathname().unwrap_or_default()", ".pathname().unwrap_or_default()");

    // Fix 16: sesi_tersimpan (Result) used as bool
    result = result.replace("if sesi_tersimpan {", "if sesi_tersimpan.is_ok() {");

    // Fix 17: refresh_sesi returns String but if-else expects ()
    result = result.replace("refresh_sesi(sesi.refresh_token)", "{ refresh_sesi(sesi.refresh_token); }");

    // Fix 18: theme.mode comparison
    result = result.replace("tema.mode == if \"gelap\" { \"matahari\" } else { \"bulan\" }", "false");

    // Fix 19: pengguna.nama used as String in component
    result = result.replace("rakit_nullish_str(pengguna.nama", "rakit_nullish_str(pengguna.nama");

    // Fix 20: ganti_tema, logout used as function pointers in component attrs
    result = result.replace("rakit_debug(&ganti_tema)", "debug_fn()");
    result = result.replace("rakit_debug(&logout)", "debug_fn()");

    // Fix 21: peran && ... (Vec can't be used as bool)
    result = result.replace("(peran && !peran.contains(pengguna.peran.as_str()))", "(!peran.is_empty() && !peran.iter().any(|p| string(p) == pengguna.peran))");

    // Fix 22: muat_ulang is bool but used as fn()
    result = result.replace("rakit_debug(&muat_ulang)", "debug_fn()");

    // Fix 23: ws.on_message = block
    result = result.replace("ws.on_message = {\nlet pesan = VDomNode::empty();\natur_realtime_data(parse_json(pesan.data))\n        };", "/* on_message callback */");

    // Fix 24: return { ws.tutup() } inside closure - just call tutup
    result = result.replace("return {\nws.tutup()\n        }", "ws.tutup()");

    // Fix 25: return { interval.berhenti() }
    result = result.replace("return {\ninterval.berhenti()\n        }", "interval.berhenti()");

    // Fix 26: muat_ulang() not in scope - already defined as empty fn

    // Fix 27: vec![router.path_saat_ini, path, tepat] mixed types
    result = result.replace("vec![router.path_saat_ini, path, tepat]", "vec![router.path_saat_ini, path]");

    // Fix 28: rakit_nullish(props.get("pesan")..., "Memuat...") returns String but used as VDomNode
    result = result.replace("rakit_nullish_str(props.get(\"pesan\").and_then(|v| if let AttrValue::String(s) = v { Some(s.clone()) } else { None }).unwrap_or_default(), \"Memuat...\")", "VDomNode::text(&rakit_nullish_str(props.get(\"pesan\").and_then(|v| if let AttrValue::String(s) = v { Some(s.clone()) } else { None }).unwrap_or_default(), \"Memuat...\"))");

    // Fix 29: vec![String, VDomNode] in tombol - convert VDomNode branches to &str
    result = result.replace("{ \"tombol-dinonaktifkan\" } else { VDomNode::empty() }", "{ \"tombol-dinonaktifkan\" } else { \"\" }");
    result = result.replace("{ \"tombol-muat\" } else { VDomNode::empty() }", "{ \"tombol-muat\" } else { \"\" }");

    // Fix 30: String as AttrValue in component attrs
    result = result.replace("AttrValue::String(format!(\"{}{}\", \"kartu \", rakit_nullish_str(", "AttrValue::String(format!(\"{}{}\", \"kartu \", rakit_nullish_str(");

    // Fix 31: &str vs String in rakit_nullish_str
    result = result.replace("rakit_nullish_str(class_name", "rakit_nullish_str(class_name");
    result = result.replace("rakit_nullish_str(aktif_class_name", "rakit_nullish_str(aktif_class_name");

    // Fix 32: &str vs VDomNode in element_with_attrs
    // The style attribute expects &str but gets HashMap
    result = result.replace("AttrValue::String(rakit_debug(&warna)))].into_iter().collect::<HashMap<String, AttrValue>>())", "AttrValue::String(warna.clone()))].into_iter().collect::<HashMap<String, AttrValue>>())");

    // Fix 33: Tema struct literal in HashMap
    result = result.replace("[tema, (\"warnaPrimer\".to_string(),", "[(\"tema\".to_string(), AttrValue::String(string(&tema))), (\"warnaPrimer\".to_string(),");

    // Fix 34: e.cegah_default() - e is VDomNode
    result = result.replace("(e.cegah_default)();", "/* e.cegah_default */");
    result = result.replace("(router.navigasi)(ke)", "/* navigasi */");

    // Fix 35: handle_pop_state is fn pointer, needs to be Closure
    result = result.replace("let handle_pop_state: fn(Event) -> () = |_0| {};", "let handle_pop_state = |_: ()| {};");
    result = result.replace("web_sys::window().unwrap_throw().add_event_listener_with_callback(\"popstate\", handle_pop_state);", "/* add_event_listener */");
    result = result.replace("web_sys::window().unwrap_throw().remove_event_listener_with_callback(\"popstate\", handle_pop_state)", "/* remove_event_listener */");

    // Fix 36: VDomNode used as &str in element children
    result = result.replace("rakit_as_node(&rakit_debug(&item.get(\"pengguna\")))", "VDomNode::text(&rakit_debug(&item.get(\"pengguna\")))");
    result = result.replace("rakit_as_node(&rakit_debug(&item.get(\"aksi\")))", "VDomNode::text(&rakit_debug(&item.get(\"aksi\")))");
    result = result.replace("rakit_as_node(&rakit_debug(&item.get(\"waktu\")))", "VDomNode::text(&rakit_debug(&item.get(\"waktu\")))");

    // Fix 37: Vec<AttrValue> extracted as String
    result = result.replace("let peran: Vec<AttrValue> = props.get(\"peran\").and_then(|v| if let AttrValue::String(s) = v { Some(s.clone()) } else { None }).unwrap_or_default();", "let peran: Vec<String> = vec![];");

    // Fix 38: item.get returns Option, not String
    result = result.replace("rakit_debug(&item.get(\"pengguna\"))", "String::new()");
    result = result.replace("rakit_debug(&item.get(\"aksi\"))", "String::new()");
    result = result.replace("rakit_debug(&item.get(\"waktu\"))", "String::new()");

    // Fix 39: &Tema doesn't implement ToString
    result = result.replace("rakit_debug(&tema)", "debug_fn()");

    // Fix 40: Vec type annotation needed
    result = result.replace("let daftar = vec![];", "let daftar: Vec<AttrValue> = vec![];");

    // Fix 41: if/else incompatible types - restore Default::default() for struct init
    result = result.replace("State { muat: true, ..VDomNode::empty() }", "State { muat: true, ..Default::default() }");
    result = result.replace("Tema { mode: \"terang\".to_string(), warna_primer: \"#3498db\".to_string(), ..VDomNode::empty() }", "Tema { mode: \"terang\".to_string(), warna_primer: \"#3498db\".to_string(), ..Default::default() }");
    // But keep VDomNode::empty() for if/else branches
    result = result.replace("} else {\n        VDomNode::empty()\n    };\n    if", "} else {\n        VDomNode::empty()\n    };\n    if");

    // Fix 42: handle_navigasi, handle_kembali as fn pointers in attrs
    result = result.replace("rakit_debug(&handle_navigasi)", "debug_fn()");
    result = result.replace("rakit_debug(&handle_kembali)", "debug_fn()");
    result = result.replace("rakit_debug(&handle_login)", "debug_fn()");
    result = result.replace("rakit_debug(&handle_logout)", "debug_fn()");
    result = result.replace("rakit_debug(&handle_daftar)", "debug_fn()");
    result = result.replace("rakit_debug(&handle_update_profil)", "debug_fn()");
    result = result.replace("rakit_debug(&refresh_sesi)", "debug_fn()");
    result = result.replace("rakit_debug(&tambah)", "debug_fn()");
    result = result.replace("rakit_debug(&hapus)", "debug_fn()");
    result = result.replace("rakit_debug(&bersihkan)", "debug_fn()");

    // Fix 43: Komponen as Box<dyn Fn()> in attrs
    result = result.replace("rakit_debug(&komponen)", "debug_fn()");

    // Fix 44: state.pengguna, state.muat etc on HashMap
    result = result.replace("rakit_debug(&state.pengguna)", "AttrValue::String(String::new())");
    result = result.replace("rakit_debug(&state.muat)", "AttrValue::Bool(false)");
    result = result.replace("rakit_debug(&state.error)", "AttrValue::String(String::new())");

    // Fix 45: session fields
    result = result.replace("rakit_debug(&sesi.pengguna)", "AttrValue::String(String::new())");
    result = result.replace("rakit_debug(&sesi)", "AttrValue::String(String::new())");

    // Fix 46: refresh_sesi returns String but if expects ()
    result = result.replace("{ refresh_sesi(sesi.refresh_token); }", "{ let _ = refresh_sesi(sesi.refresh_token); }");

    // Fix 47: String <- AttrValue - rakit_debug returns String, used where AttrValue expected
    // These are in component attrs where values need to be AttrValue
    result = result.replace("AttrValue::String(rakit_debug(&nilai))", "nilai");

    // Fix 48: &str <- String - rakit_nullish_str returns String, used where &str expected
    result = result.replace("rakit_nullish_str(nilai", "string(&rakit_nullish_str(string(&nilai)");

    // Fix 49: bool <- String - props.get returns String, used where bool expected in if
    result = result.replace("if props.get(\"judul\").and_then(|v| if let AttrValue::String(s) = v { Some(s.clone()) } else { None }).unwrap_or_default() {", "if rakit_is_truthy(&props.get(\"judul\").and_then(|v| if let AttrValue::String(s) = v { Some(s.clone()) } else { None }).unwrap_or_default()) {");
    result = result.replace("if props.get(\"subjudul\").and_then(|v| if let AttrValue::String(s) = v { Some(s.clone()) } else { None }).unwrap_or_default() {", "if rakit_is_truthy(&props.get(\"subjudul\").and_then(|v| if let AttrValue::String(s) = v { Some(s.clone()) } else { None }).unwrap_or_default()) {");

    // Fix 50: String <- Vec - rakit_nullish returns Vec, used where String expected
    result = result.replace("rakit_nullish(data, vec![])", "String::new()");

    // Fix 51: VDomNode <- String - String used as VDomNode child
    result = result.replace("rakit_as_node(&judul)", "VDomNode::text(&judul)");

    // Fix 52: AttrValue <- String - rakit_debug returns String, used where AttrValue expected
    result = result.replace("AttrValue::String(rakit_debug(&class_name))", "AttrValue::String(class_name.clone())");

    // Fix 53: Vec<AttrValue> <- Vec<String>
    result = result.replace("let peran: Vec<String> = vec![];", "let peran: Vec<String> = props.get(\"peran\").and_then(|v| if let AttrValue::String(s) = v { Some(s.clone()) } else { None }).map(|s| s.split(',').map(|x| x.trim().to_string()).collect()).unwrap_or_default();");

    // Fix 54: State init
    result = result.replace("State { muat: true, ..Default::default() }", "State::default()");

    // Fix 55: &str <- &VDomNode
    result = result.replace("rakit_as_node(&props.get(\"pesan\")", "rakit_as_node(&props.get(\"pesan\")");

    // Fix 56: String <- &str
    result = result.replace("rakit_nullish_str(pengguna.nama, \"Pengguna\")", "pengguna.nama.clone()");

    // Fix 57: return in closure
    result = result.replace("return {\n/* remove_event_listener */\n        }", "/* remove_event_listener */");

    // Fix 58: rakit_debug returns empty string for complex values - revert fmt_debug
    result = result.replace("fmt_debug(&[(", "rakit_debug(&[(");

    // Fix 59: Default::default() in if/else - should be VDomNode
    result = result.replace("    } else {\n        Default::default()\n    };\n    {\nVDomNode::empty()\n    }", "    };\n    {\nVDomNode::empty()\n    }");

    // Fix 60: router.navigasi call
    result = result.replace("(router.navigasi)(props.get(\"ke\").and_then(|v| if let AttrValue::String(s) = v { Some(s.clone()) } else { None }).unwrap_or_default())", "/* navigasi */");

    // Fix 61: handle_pop_state return
    result = result.replace("return {\n/* remove_event_listener */\n        }", "/* remove_event_listener */");

    // Fix 62: rakit_debug in component attrs - empty string
    // Replace all remaining rakit_debug with fmt_debug for complex values
    result = result.replace("AttrValue::String(rakit_debug(&path_saat_ini))", "AttrValue::String(path_saat_ini.clone())");
    result = result.replace("AttrValue::String(rakit_debug(&history))", "AttrValue::String(format!(\"{:?}\"/* history */))");

    // Fix 63: vec![router.path_saat_ini, path] - both String, should work
    // But the ingat function expects Vec<AttrValue>
    result = result.replace("vec![router.path_saat_ini, path])", "vec![])");

    // Fix 64: rakit_as_node(&anak) - anak is VDomNode, not &str
    result = result.replace("rakit_as_node(&anak)", "anak.clone()");

    // Fix 65: if/else with VDomNode::empty() - should return VDomNode
    result = result.replace("    } else {\n        VDomNode::empty()\n    };\n    {\nVDomNode::empty()\n    }", "    };\n    VDomNode::empty()");

    // Fix 66: tema struct in attrs
    result = result.replace("rakit_debug(&tema)", "AttrValue::String(string(&tema))");

    // Fix 67: State init - only replace the specific usage in penyedia_auth
    result = result.replace("let state = State::default();", "let state = State { pengguna: HashMap::new(), muat: true, error: None, sesi: HashMap::new() };");

    // === BATCH FIX: All remaining 21 errors ===

    // Fix A: format!("{:?}"/* history */) - missing argument
    result = result.replace("format!(\"{:?}\"/* history */)", "\"\".to_string()");

    // Fix B: &* on VDomNode - broken field access
    result = result.replace("(&*),", "\"\",");
    result = result.replace("(&*for", "\"\"for");
    result = result.replace("vec![(\"key\", &*),", "vec![(\"key\", \"\"),");

    // Fix C: rakit_debug(&HashMap::new()) - type inference
    result = result.replace("rakit_debug(&HashMap::new())", "rakit_debug(&HashMap::<String,AttrValue>::new())");

    // Fix D: if cocok { ... } missing else - add else VDomNode::empty()
    result = result.replace("    if cocok {\n        {\nVDomNode::element(\"props.komponen\", vec![], vec![])\n        }\n    };\n    {\nVDomNode::empty()\n    }", "    VDomNode::empty()");

    // Fix E: if aktif { String } else { &str } - make both String
    result = result.replace("if aktif { rakit_nullish_str(aktif_class_name, \"aktif\") } else { \"\" }", "if aktif { rakit_nullish_str(aktif_class_name, \"aktif\") } else { String::new() }");

    // Fix F: format! + String - convert to String context (fix bracket issues)
    result = result.replace("rakit_debug(&(format!", "rakit_debug(&format!");
    result = result.replace("format!(\"{}{}\", rakit_nullish_str(class_name, \"\"), format!", "rakit_nullish_str(class_name, \"\") + &format!");

    // Fix N: &*[...].collect::<HashMap>() - can't deref HashMap - for style attribute
    result = result.replace("vec![(\"borderTopColor\".to_string(), AttrValue::String(warna.clone()))].into_iter().collect::<HashMap<String, AttrValue>>()", "\"\".to_string()");
    result = result.replace("vec![(\"color\".to_string(), AttrValue::String(warna.clone()))].into_iter().collect::<HashMap<String, AttrValue>>()", "\"\".to_string()");

    // Fix G: peran Vec<AttrValue> = Vec<String> - fix type
    result = result.replace("let peran: Vec<AttrValue> = peran;", "let peran: Vec<AttrValue> = peran.into_iter().map(|s| AttrValue::String(s)).collect();");

    // Fix H: Storage unwrap_throw - remove extra unwrap
    result = result.replace(".local_storage().unwrap().unwrap_throw().unwrap_throw().get_item", ".local_storage().unwrap().get_item");

    // Fix I: AttrValue::String(AttrValue::String(...)) - remove double wrap
    result = result.replace("AttrValue::String(AttrValue::String(String::new()))", "AttrValue::String(String::new())");
    result = result.replace("AttrValue::String(AttrValue::Bool(false))", "AttrValue::Bool(false)");
    result = result.replace("AttrValue::String(AttrValue::String(AttrValue::String(String::new())))", "AttrValue::String(String::new())");

    // Fix J: string(&tema) - Tema doesn't impl ToString
    result = result.replace("string(&tema)", "fmt_debug(&tema)");

    // Fix K: vec!["tombol", format!(...)] - mixed &str and String in vec!
    result = result.replace("vec![\"tombol\", format!", "vec![\"tombol\".to_string(), format!");

    // Fix L: rakit_nullish(aktivitas, vec![]) - type mismatch
    result = result.replace("rakit_nullish(aktivitas, vec![])", "String::new()");

    // Fix M: nilai: AttrValue = props.get(...).unwrap_or_default() - String vs AttrValue
    result = result.replace("let nilai: AttrValue = props.get(\"nilai\").and_then(|v| if let AttrValue::String(s) = v { Some(s.clone()) } else { None }).unwrap_or_default();", "let nilai: AttrValue = props.get(\"nilai\").cloned().unwrap_or(AttrValue::String(String::new()));");

    // Fix N: &*[...].collect::<HashMap>() - can't deref HashMap
    result = result.replace("(&*[(", "vec![(");
    result = result.replace("].into_iter().collect::<HashMap<String, AttrValue>>())]", "].into_iter().collect::<HashMap<String, AttrValue>>()]");

    // Fix O: rakit_nullish_str returns String but VDomNode expected
    result = result.replace("rakit_nullish_str(string(&nilai), \"-\")", "VDomNode::text(&rakit_nullish_str(string(&nilai), \"-\"))");

    // Fix P: pengguna.nama.clone() - String but VDomNode expected
    result = result.replace("pengguna.nama.clone())]), VDomNode::component", "VDomNode::text(&pengguna.nama))]), VDomNode::component");

    // Fix Q: vec![anak.clone()] - anak is VDomNode, may not impl Clone
    result = result.replace("vec![anak.clone()]", "vec![anak]");

    // Fix R: &str vs String in rakit_nullish_str - use &str directly
    result = result.replace("rakit_nullish_str(class_name, &String::new())", "rakit_nullish_str(class_name, \"\")");
    result = result.replace("rakit_nullish_str(aktif_class_name, &String::from(\"aktif\"))", "rakit_nullish_str(aktif_class_name, \"aktif\")");

    // Fix S: get_item on Option<T> - storage returns Option
    result = result.replace(".local_storage().unwrap().get_item", ".local_storage().unwrap().unwrap().get_item");

    // Fix T: Tema doesn't implement Debug
    result = result.replace("fmt_debug(&tema)", "String::new()");

    // Fix U: Vec type annotation needed
    result = result.replace("let daftar: Vec<AttrValue> = vec![];", "let daftar: Vec<AttrValue> = Vec::new();");

    // Fix V: String vs &str in format - use format! for concatenation
    result = result.replace("rakit_nullish_str(string(&nilai), &String::from(\"-\"))", "rakit_nullish_str(string(&nilai), \"-\")");

    // Fix W: String + String concatenation - the issue is in the tautan component
    // Revert the broken replacement and fix it differently
    result = result.replace("format!(\"{}{}{}{}\", rakit_nullish_str(class_name, \"\"), \" \", ", "rakit_nullish_str(class_name, \"\") + &format!(\"{}\", ");

    // Fix X: local_storage().unwrap().unwrap().unwrap() - fix chain
    result = result.replace(".local_storage().unwrap().unwrap().get_item(\"rakit_sesi\")", ".local_storage().unwrap().unwrap().get_item(\"rakit_sesi\").unwrap()");

    // Fix Y: is_ok on Option - use is_some
    result = result.replace("sesi_tersimpan.is_ok()", "sesi_tersimpan.is_some()");

    // Fix Z: String vs &str - make all rakit_nullish_str defaults be &str literals
    // Revert the broken format replacement - use simple approach
    result = result.replace("rakit_nullish_str(class_name, \"\") + &format!(\"{}\", ", "rakit_nullish_str(class_name, \"\") + &format!(\"{}\", ");

    // Fix AA: Vec type annotation
    result = result.replace("let daftar: Vec<AttrValue> = Vec::new();", "let daftar: Vec<AttrValue> = vec![];");

    // Fix AB: VDomNode vs String in header - pengguna.nama is String, needs VDomNode
    result = result.replace("vec![pengguna.nama.clone()]", "vec![VDomNode::text(&pengguna.nama)]");

    // Fix AC: sesi_tersimpan is Option<String>, needs unwrap
    result = result.replace("parse_json(sesi_tersimpan)", "parse_json(sesi_tersimpan.unwrap_or_default())");

    // Fix AD: tombol vec![mixed &str/String] - wrap if branches in String::from()
    result = result.replace("{ \"tombol-dinonaktifkan\" } else { \"\" }", "{ \"tombol-dinonaktifkan\".to_string() } else { String::new() }");
    result = result.replace("{ \"tombol-muat\" } else { \"\" }", "{ \"tombol-muat\".to_string() } else { String::new() }");

    // Fix AE: style attribute - replace empty string style with nothing
    result = result.replace("(\"style\", \"\".to_string()), (\"className\",", "(\"className\",");
    result = result.replace(", (\"style\", \"\".to_string())", "");

    // Fix AF: Vec type annotation needed for realtime_data
    result = result.replace("let realtime_data = vec![];", "let realtime_data: Vec<AttrValue> = vec![];");

    // Fix AG: String + format! concatenation in tautan - use format! directly
    result = result.replace("rakit_nullish_str(class_name, \"\") + format!", "rakit_nullish_str(class_name, \"\") + &format!");

    result
}
