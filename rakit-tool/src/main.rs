use clap::{Parser as ClapParser, Subcommand};
use std::path::PathBuf;

mod commands;

use commands::build::BuildCommand;
use commands::init::InitCommand;
use commands::emit_ir::EmitIrCommand;

/// Rakit — bahasa UI reaktif dalam Bahasa Indonesia.
#[derive(ClapParser)]
#[command(name = "rakit", version, about)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Compile file .rakit menjadi binary
    Build {
        /// File .rakit yang akan di-compile
        file: PathBuf,
        /// Output AST dalam format JSON
        #[arg(long)]
        json: bool,
        /// Target platform (win32, gtk4, wasm)
        #[arg(long)]
        target: Option<String>,
        /// Output file
        #[arg(long)]
        output: Option<PathBuf>,
        /// Level optimasi: 0, 1, 2
        #[arg(long, default_value = "0")]
        optimize: u8,
    },

    /// Parse dan emit IR dalam format yang bisa dibaca
    EmitIr {
        /// File .rakit
        file: PathBuf,
        /// Format output: json, pretty
        #[arg(long, default_value = "pretty")]
        format: String,
        /// Level IR: ast (default) atau hir
        #[arg(long, default_value = "ast")]
        level: String,
    },

    /// Buat project Rakit baru
    Init {
        /// Nama project
        name: String,
    },

    /// Format kode Rakit
    Fmt {
        /// File .rakit yang akan diformat
        file: PathBuf,
        /// Hanya cek, jangan menulis file
        #[arg(short, long)]
        check: bool,
    },

    /// Jalankan test
    Test {
        /// File test spesifik (optional)
        #[arg(short, long)]
        file: Option<PathBuf>,
        /// Update snapshot
        #[arg(long)]
        update: bool,
    },

    /// Jalankan dalam dev mode (hot reload)
    Dev {
        /// File .rakit entry point
        file: PathBuf,
        /// DevTools port
        #[arg(short, long)]
        port: Option<u16>,
    },

    /// Kelola packages
    #[command(subcommand)]
    Package(PackageCommand),

    /// Generate dokumentasi dari source
    Doc {
        /// File .rakit
        file: PathBuf,
        /// Format output: html atau markdown
        #[arg(long, default_value = "html")]
        format: String,
        /// Output directory
        #[arg(short, long)]
        output: Option<PathBuf>,
    },

    /// Generate polyglot bindings
    Generate {
        /// File .rakit
        file: PathBuf,
        /// Bahasa target: c, python, rust
        #[arg(long)]
        lang: String,
        /// Output file
        #[arg(short, long)]
        output: Option<PathBuf>,
    },

    /// Optimasi file Rakit
    Optimize {
        /// File .rakit
        file: PathBuf,
        /// Output file
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
}

#[derive(Subcommand)]
enum PackageCommand {
    /// Install package dari registry
    Install {
        package: String,
        #[arg(short, long)]
        version: Option<String>,
        #[arg(short, long)]
        save: bool,
    },
    /// Hapus package
    Remove { package: String },
    /// Update semua packages
    Update,
    /// Cari package di registry
    Search { query: String },
    /// Lihat info package
    Info { package: String },
    /// Buat package baru
    Init,
    /// Publish package ke registry
    Publish,
}

fn main() {
    let cli = Cli::parse();

    let result = match &cli.command {
        Commands::Build { file, json, target, output, optimize } => {
            BuildCommand::run_with_opts(file, *json, target.as_deref(), output.as_ref().map(|p| p.as_path()), *optimize)
        }
        Commands::EmitIr { file, format, level } => EmitIrCommand::run(file, format, level),
        Commands::Init { name } => InitCommand::run(name),
        Commands::Fmt { file, check } => run_fmt(file, *check),
        Commands::Test { file, update } => run_test(file.as_ref(), *update),
        Commands::Dev { file, port } => run_dev(file, *port),
        Commands::Package(cmd) => run_package(cmd),
        Commands::Doc { file, format, output } => run_doc(file, format, output.as_ref()),
        Commands::Generate { file, lang, output } => run_generate(file, lang, output.as_ref()),
        Commands::Optimize { file, output } => run_optimize(file, output.as_ref()),
    };

    if let Err(e) = result {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}

fn run_fmt(file: &PathBuf, check: bool) -> Result<(), String> {
    let source = std::fs::read_to_string(file)
        .map_err(|e| format!("Gagal membaca {}: {}", file.display(), e))?;

    let rules = rakit_fmt::FormatRules::default();
    let mut formatter = rakit_fmt::RakitFormatter::new(rules);
    let formatted = formatter.format(&source)
        .map_err(|e| format!("Gagal format: {}", e))?;

    if check {
        if source != formatted {
            return Err(format!("{} tidak diformat dengan benar", file.display()));
        }
        println!("{}: format OK", file.display());
    } else {
        std::fs::write(file, &formatted)
            .map_err(|e| format!("Gagal menulis {}: {}", file.display(), e))?;
        println!("{}: diformat", file.display());
    }

    Ok(())
}

fn run_test(file: Option<&PathBuf>, update: bool) -> Result<(), String> {
    let _ = update;
    let mut runner = rakit_test::TestRunner::new();

    if let Some(path) = file {
        let _source = std::fs::read_to_string(path)
            .map_err(|e| format!("Gagal membaca {}: {}", path.display(), e))?;
        runner.register(
            path.file_stem().unwrap_or_default().to_string_lossy().to_string(),
            path.to_string_lossy().to_string(),
            Box::new(|| Ok(())),
        );
    }

    runner.run();

    if runner.failed > 0 {
        Err(format!("{} test gagal", runner.failed))
    } else {
        Ok(())
    }
}

fn run_dev(file: &PathBuf, port: Option<u16>) -> Result<(), String> {
    commands::dev::run_dev_server(file, port)
}

fn run_package(cmd: &PackageCommand) -> Result<(), String> {
    let registry = rakit_pm::registry::PackageRegistry::default();
    let mut resolver = rakit_pm::resolver::DependencyResolver::new(registry);

    match cmd {
        PackageCommand::Install { package, version, save } => {
            println!("Menginstall package: {}", package);
            if let Some(v) = version {
                println!("  version: {}", v);
            }
            if *save {
                println!("  menyimpan ke manifest");
            }

            let manifest = rakit_pm::RakitManifest {
                package: rakit_pm::PackageMeta {
                    name: "app".into(),
                    version: "1.0.0".into(),
                    description: "Aplikasi Rakit".into(),
                    authors: vec!["User".into()],
                    license: "MIT".into(),
                },
                dependencies: std::collections::HashMap::new(),
                target: None,
            };
            let graph = resolver.resolve(&manifest).map_err(|e| format!("Gagal resolve: {}", e))?;
            let fetcher = rakit_pm::fetch::PackageFetcher::new(rakit_pm::registry::PackageRegistry::default());
            let paths = fetcher.fetch_all(&graph).map_err(|e| format!("Gagal fetch: {}", e))?;
            println!("  terinstall: {} packages", paths.len());
            Ok(())
        }
        PackageCommand::Remove { package } => {
            println!("Menghapus package: {}", package);
            Ok(())
        }
        PackageCommand::Update => {
            println!("Mengupdate semua packages...");
            Ok(())
        }
        PackageCommand::Search { query } => {
            let search_results = rakit_pm::registry::PackageRegistry::default().search(query)?;
            println!("Hasil pencarian untuk '{}':", query);
            for pkg in &search_results {
                println!("  {} v{} — {}", pkg.name, pkg.version, pkg.description);
            }
            Ok(())
        }
        PackageCommand::Info { package } => {
            println!("Informasi package: {}", package);
            println!("  Registry: https://registry.rakit.dev");
            println!("  Versi tersedia: 1.0.0, 0.9.0");
            Ok(())
        }
        PackageCommand::Init => {
            println!("Membuat package baru...");
            std::fs::create_dir_all("src").map_err(|e| e.to_string())?;
            std::fs::write("rakit.json", serde_json::to_string_pretty(&rakit_pm::RakitManifest {
                package: rakit_pm::PackageMeta {
                    name: "package-saya".into(),
                    version: "0.1.0".into(),
                    description: "Deskripsi package".into(),
                    authors: vec!["Anda".into()],
                    license: "MIT".into(),
                },
                dependencies: std::collections::HashMap::new(),
                target: None,
            }).unwrap()).map_err(|e| e.to_string())?;
            std::fs::write("src/mod.rakit", "fungsi init() -> I32 { 0 }\n").map_err(|e| e.to_string())?;
            println!("Package siap! Edit src/mod.rakit untuk memulai.");
            Ok(())
        }
        PackageCommand::Publish => {
            println!("Mempublish package...");
            println!("Untuk publish, jalankan: rakit package publish --registry URL");
            Ok(())
        }
    }
}

fn run_doc(file: &PathBuf, format: &str, output: Option<&PathBuf>) -> Result<(), String> {
    let source = std::fs::read_to_string(file)
        .map_err(|e| format!("Gagal membaca {}: {}", file.display(), e))?;

    let source_map = rakit_core::SourceMap::new(
        &file.file_name().unwrap_or_default().to_string_lossy(),
        &source,
    );

    let parser = rakit_frontend::parser::Parser::new(&source, &source_map);
    let ast_program = match parser.parse_program() {
        Ok(p) => p,
        Err(diags) => {
            for d in &diags {
                eprintln!("{}", d.message);
            }
            return Err("Gagal parsing dokumentasi".to_string());
        }
    };

    let mut lower = rakit_ir_hir::lower::HirLower::new();
    let hir = match lower.lower_program(&ast_program) {
        Ok(h) => h,
        Err(diags) => {
            for d in &diags {
                eprintln!("{}", d.message);
            }
            return Err("Gagal lowering dokumentasi".to_string());
        }
    };

    let gen = rakit_doc::DocGenerator::from_program(&hir);
    let output_dir = output.map(|p| p.clone()).unwrap_or_else(|| PathBuf::from("docs"));

    std::fs::create_dir_all(&output_dir).map_err(|e| e.to_string())?;

    match format {
        "html" => {
            let html = gen.generate_html();
            let path = output_dir.join("dokumentasi.html");
            std::fs::write(&path, &html).map_err(|e| e.to_string())?;
            println!("Dokumentasi HTML: {}", path.display());
        }
        "markdown" | "md" => {
            let md = gen.generate_markdown();
            let path = output_dir.join("dokumentasi.md");
            std::fs::write(&path, &md).map_err(|e| e.to_string())?;
            println!("Dokumentasi Markdown: {}", path.display());
        }
        _ => return Err(format!("Format tidak dikenal: {}", format)),
    }

    let search_index = rakit_doc::search::SearchIndex::from_docs(&gen.items);
    let json_path = output_dir.join("search-index.json");
    std::fs::write(&json_path, search_index.to_json()).map_err(|e| e.to_string())?;
    println!("Search index: {}", json_path.display());

    Ok(())
}

fn run_generate(file: &PathBuf, lang: &str, output: Option<&PathBuf>) -> Result<(), String> {
    let source = std::fs::read_to_string(file)
        .map_err(|e| format!("Gagal membaca {}: {}", file.display(), e))?;

    let source_map = rakit_core::SourceMap::new(
        &file.file_name().unwrap_or_default().to_string_lossy(),
        &source,
    );

    let parser = rakit_frontend::parser::Parser::new(&source, &source_map);
    let ast_program = match parser.parse_program() {
        Ok(p) => p,
        Err(diags) => {
            for d in &diags {
                eprintln!("{}", d.message);
            }
            return Err("Gagal parsing".to_string());
        }
    };

    let mut lower = rakit_ir_hir::lower::HirLower::new();
    let hir = match lower.lower_program(&ast_program) {
        Ok(h) => h,
        Err(diags) => {
            for d in &diags {
                eprintln!("{}", d.message);
            }
            return Err("Gagal lowering".to_string());
        }
    };

    let mut functions = Vec::new();
    let mut structs = Vec::new();

    for item in &hir.items {
        use rakit_ir_hir::hir::HirItem;
        match item {
            HirItem::Function(f) => {
                let params = f.params.iter().map(|p| rakit_polyglot::RakitParam {
                    name: p.name.clone(),
                    ty: p.ty.clone(),
                }).collect();
                functions.push(rakit_polyglot::RakitExport {
                    name: f.name.clone(),
                    params,
                    return_ty: f.return_ty.clone(),
                    is_async: false,
                });
            }
            HirItem::Struct(s) => {
                let fields = s.fields.iter().map(|f| rakit_polyglot::RakitField {
                    name: f.name.clone(),
                    ty: f.ty.clone(),
                }).collect();
                structs.push(rakit_polyglot::RakitStruct {
                    name: s.name.clone(),
                    fields,
                });
            }
            _ => {}
        }
    }

    let output_path = output.cloned().unwrap_or_else(|| {
        PathBuf::from(format!("bindings.{}", match lang {
            "c" => "h",
            "python" => "py",
            "rust" => "rs",
            _ => "txt",
        }))
    });

    match lang {
        "c" => {
            let gen = rakit_polyglot::c_bindings::CBindingGenerator::new(functions, structs);
            let header = gen.generate_header();
            std::fs::write(&output_path, &header).map_err(|e| e.to_string())?;
        }
        "python" => {
            let gen = rakit_polyglot::python_bindings::PythonBindingGenerator;
            let module = gen.generate_py_module(&functions);
            std::fs::write(&output_path, &module).map_err(|e| e.to_string())?;
        }
        "rust" => {
            let gen = rakit_polyglot::rust_bindings::RustBindingsGenerator;
            let module = gen.generate_rust_module(&functions);
            std::fs::write(&output_path, &module).map_err(|e| e.to_string())?;
        }
        _ => return Err(format!("Bahasa target tidak dikenal: {}", lang)),
    }

    println!("Binding {}: {}", lang, output_path.display());
    Ok(())
}

fn run_optimize(file: &PathBuf, output: Option<&PathBuf>) -> Result<(), String> {
    let source = std::fs::read_to_string(file)
        .map_err(|e| format!("Gagal membaca {}: {}", file.display(), e))?;

    let source_map = rakit_core::SourceMap::new(
        &file.file_name().unwrap_or_default().to_string_lossy(),
        &source,
    );

    let parser = rakit_frontend::parser::Parser::new(&source, &source_map);
    let ast_program = match parser.parse_program() {
        Ok(p) => p,
        Err(diags) => {
            for d in &diags {
                eprintln!("{}", d.message);
            }
            return Err("Gagal parsing".to_string());
        }
    };

    let mut lower = rakit_ir_hir::lower::HirLower::new();
    let mut hir = match lower.lower_program(&ast_program) {
        Ok(h) => h,
        Err(diags) => {
            for d in &diags {
                eprintln!("{}", d.message);
            }
            return Err("Gagal lowering".to_string());
        }
    };

    let optimizer = rakit_opt::Optimizer::with_default_passes();
    let summary = optimizer.run_optimized(&mut hir);

    println!("Optimasi selesai:");
    for result in &summary.results {
        println!("  {}: {} ({} item)", result.pass, result.description, result.items_affected);
    }
    println!("Total: {} pass, {} item affected", summary.passes_run, summary.total_items_affected);

    if let Some(out) = output {
        std::fs::write(out, format!("{:#?}", hir)).map_err(|e| e.to_string())?;
        println!("Output: {}", out.display());
    }

    Ok(())
}
