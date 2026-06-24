# Changelog Rakit

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
