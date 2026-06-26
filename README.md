# Rakit v1.0.3

Bahasa pemrograman modern dengan kompiler, LSP, package manager, dan polyglot FFI.

## 🚀 Instalasi

### Windows
1. Download **rakit-v1.0.3-windows.zip** dari [GitHub Releases](https://github.com/zulsyam23-dot/rakit/releases)
2. Extract, lalu jalankan `rakit.cmd` atau tambahkan `bin/` ke PATH

### Dari Source
```
git clone https://github.com/zulsyam23-dot/rakit.git
cd rakit
cargo build --release
./rakit.cmd
```

### Prasyarat
- [Rust](https://rustup.rs/) 1.85+

## 📦 Package Manager
```
rakit package install <nama>
rakit package remove <nama>
rakit package search <kata_kunci>
```

## 📖 Dokumentasi
Buka [docs/README.md](docs/README.md) untuk dokumentasi lengkap (filosofi, sintaksis, komponen, hooks, platform, polyglot, FAQ).

## 🔧 Perintah CLI
| Perintah | Deskripsi |
|----------|-----------|
| `rakit build <file>` | Kompilasi ke native/WASM |
| `rakit build <file> --target win32` | Build PE executable |
| `rakit run <file>` | Jalankan program |
| `rakit doc <file>` | Generate dokumentasi |
| `rakit optimize <file>` | Optimasi kode |
| `rakit generate --lang c` | Generate binding C/Python/Rust |

## 📜 Lisensi
Penggunaan bebas. Modifikasi harus dengan izin. Lihat [LICENSE](LICENSE).
