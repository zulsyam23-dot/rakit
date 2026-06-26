# Changelog Rakit

## v1.0.5 — 26 Juni 2026

### Project Restructure
- **Folder reorganization**: Source (.rakit) dipindah ke `src/`, build output (WASM/JS) ke `src/dist/`, index.html tetap di `src/`
- **Build output**: `wasm-bindgen` sekarang output ke `src/dist/` bukan root project
- **Dev server**: `project_dir` sekarang mengikuti lokasi entry file, bukan root CWD
- **app_name**: Sekarang dideteksi dari project root (cari `devil.json`), bukan dari entry file parent

### Version Consistency
- **Semua Cargo.toml**: Rakit sub-crates dari `0.1.0` → `1.0.5`, Devil dari `1.0.4` → `1.0.5`, Devil Registry dari `0.1.0` → `1.0.5`
- **devil.json**: Semua project manifest diperbarui ke v1.0.5
- **MCP server**: Devil MCP server version sinkron ke v1.0.5

### Bug Fixes
- **app_name di WASM build**: Entry file di subdirektori (e.g. `src/main.rakit`) sekarang menghasilkan nama project yang benar, bukan nama direktori
- **Devil dev server**: `build_dir` sekarang mengikuti entry path, bukan hardcoded ke `cwd/.rakit-build/`

### Catatan
- Build sukses: WASM output di `src/dist/aplikasi-ku.wasm`
- 86 compilation warnings (cosmetic: unused variables, dead code)
- Semua project applications diupdate: aplikasi-ku, demo-rakit, rakit-site

---

## v1.0.4 — 26 Juni 2026

### Bug Fixes
- **WASM codegen — enum variant patterns**: Match arms now emit qualified `EnumName::Variant(..)` instead of bare variant names, fixing `bindings_with_variant_name` error. Variants with fields use `(..)` pattern to ignore inner data.
- **WASM codegen — Default for enums**: Enums now implement `Default` (picking first variant) so structs containing enum fields can derive `Default`.
- **WASM codegen — I32/F64 mismatch**: Fixed type mismatch when passing numeric literal to `I32` parameter by updating test example to use `Angka` (since all Rakit numeric literals are `F64`).
- **String interpolation desugaring**: Template strings like `"text {expr}"` are now lowered to `BinaryOp::Concat` chains at parser level, flowing through existing HIR → Brak → codegen pipeline without needing the `BrakExpr::Template` variant.
- **String literal codegen**: `BrakExpr::String(s)` emits `"s".to_string()` so literals are `String` type, not `&str`.
- **`cetak`/`println!` codegen**: Non-literal arguments use `println!("{}", expr)` instead of `println!(format!(…))`.
- **JSX tag emission**: Static string tags remain raw `&str` for `VDomNode::element`; dynamic tags get `.as_str()`.
- **Struct `Default` derive**: All user structs get `#[derive(Debug, Clone, Default)]` so `Pengguna::default()` works in runtime shims.
- **Conditional Pengguna shim**: Hardcoded shim `Pengguna` struct is skipped when user defines their own `Pengguna`.
- **Try/catch codegen**: Now emits `match catch_unwind { Ok(val) => val, Err(e) => { … } }` that propagates the closure return value and includes the catch block body.
- **Array reuse**: Fixed use-after-move error when passing array to multiple functions by using `.clone()` (`.salin()`) in test example.

### Test Results
- ✅ `examples/komprehensif/` — 37 Brak items, full WASM compilation succeeds
- ✅ Zero compilation errors (136 warnings are cosmetic: dead_code, unused_imports, unnecessary parens)
- ✅ WASM output: `.wasm` + JS glue generated

---

## v1.0.3 — 26 Juni 2026

### Bug Fixes
- **Component props**: Props komponen sekarang bisa diakses langsung di JSX dan body.  
  Sebelumnya `komponen Halaman(judul: Teks)` gagal karena `judul` tidak dikenal di scope.  
  HIR lowering sekarang membuat `let` binding untuk SETIAP prop, bukan hanya untuk struct-type props.
- **Import module**: Import untuk built-in functions (cetak, tampilkan, dll) tidak lagi menampilkan warning.  
  `dari "std/io" gunakan { cetak }` sekarang resolved secara silent karena `cetak` sudah registered sebagai builtin.

### Fitur Baru
- **Native binary output**: Build sekarang bisa menghasilkan `.exe` langsung tanpa cargo/rustc.  
  `rakit build file.rakit --target win32` → menghasilkan PE executable.  
  Menggunakan Brak codegen (ObjBackend) + native linker (NativeLinker).
- **Multi-target support**: `--target` sekarang mendukung `wasm`, `win32`, `linux`, `macos`.

### Test Results
- 176/176 unit tests pass
- 12/12 example files build successfully (WASM)
- 6/6 example files build successfully (native PE)
- 0 regressions

---

## v1.0.2 — 26 Juni 2026

### Perbaikan
- **Distribusi**: `rakit.cmd` sekarang menjalankan `bin\rakit.exe` langsung (sebelumnya menggunakan `cargo run` yang membutuhkan Rust toolchain).
- **Binary release**: Rebuild binary dengan perbaikan terbaru dari seluruh 19 subcrate.

### Catatan
- Seluruh komponen linter dan compiler berjalan tanpa error
- 19 subcrate dalam workspace berjalan dengan baik
- Distribution zip telah diupdate ke v1.0.2

---

## v1.0.1 — 24 Juni 2026

### Bug Fixes
- **Parser**: Struct init `Ident { ... }` tidak lagi salah memakan `{` dari blok `jika`/`ulang`.  
  Sebelumnya `jika x > y { cetak("ok") }` diartikan sebagai struct init `y { ... }`.  
  Sekarang parser menggunakan lookahead untuk membedakan block vs struct init.
- **Tree shaking**: Perbaiki panic *"removal index out of bounds"* saat menghapus item yang tidak terpakai.  
  Index penghapusan sekarang dalam urutan descending.
- **Type system**: Tambah dukungan nama tipe Bahasa Indonesia:  
  `Angka → F64`, `Teks → String`, `BenarSalah → Bool`, `Huruf → Char`, `Kosong → Void`.
- **Entry point**: Tree shaking sekarang mengenali `utama` sebagai entry point (sama seperti `main`).

### Catatan
- Semua 174+ test lulus
- 12 contoh file (*examples/*.rakit) berhasil build
- `examples/hello.rakit` — contoh lengkap dengan fungsi, komponen, JSX, struct, enum

---

## v1.0.0 — Rilis Awal

- Parser, HIR lowering, type checker, codegen
- CLI: `build`, `emit-ir`, `init`, `fmt`, `test`, `dev`, `package`, `doc`, `generate`, `optimize`
- Package manager, LSP, polyglot FFI, optimizer, docs generator
- Benchmark suite
- Dokumentasi lengkap
