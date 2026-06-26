# Changelog Rakit

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
