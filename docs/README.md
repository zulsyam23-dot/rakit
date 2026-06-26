# Rakit v1.0.5

**Bahasa UI Reaktif dalam Bahasa Indonesia**

Rakit adalah bahasa pemrograman untuk membangun antarmuka pengguna (UI) reaktif, terinspirasi dari React, yang seluruh sintaksisnya menggunakan Bahasa Indonesia.

```
╔═══════════════════════════════════════════════╗
║       Rakit v1.0.5 — Bahasa Indonesia         ║
║       Pemrograman UI Reaktif                  ║
╠═══════════════════════════════════════════════╣
║  🏗️  Dibangun di atas Brak Language Toolkit   ║
║  🎯  React-like dalam Bahasa Indonesia         ║
║  🖥️  Target: Windows, Linux, macOS, Web       ║
║  🔗  Polyglot: C, Python, Rust                ║
║  ⚡  Kinerja native, zero runtime              ║
║  📦  Package manager built-in                 ║
║  🛠️  LSP, Formatter, Test framework           ║
╚═══════════════════════════════════════════╝
```

## Struktur Project

Mulai v1.0.5, project Rakit mengikuti struktur standar:

```
project/
├── src/                 # Source code (.rakit)
│   ├── main.rakit       # Entry point
│   ├── komponen/        # Komponen
│   ├── index.html       # Web entry (dev server)
│   ├── aset/            # Static assets (CSS, gambar)
│   ├── dist/            # Build output (WASM + JS)
│   └── .rakit-build/    # Temporary Rust build
├── devil.json           # Project manifest
└── .gitignore
```

### Perintah CLI
```
rakit dev src/main.rakit --port 8080    # Dev server
rakit build src/main.rakit --target wasm  # Build WASM
```

## Quick Start

```bash
rakit init aplikasi-saya
cd aplikasi-saya
rakit build main.rakit --target win32
./main.exe
```

## Contoh

```rakit
komponen App() {
    keadaan(hitung, aturHitung) = 0;

    tampilkan {
        <div>
            <h1>"Halo dari Rakit!"</h1>
            <p>"Klik: {hitung}"</p>
            <button onClick={() => aturHitung(hitung + 1)}>
                "Klik aku!"
            </button>
        </div>
    }
}
```

## Instalasi

1. Clone repositori ini
2. Jalankan `cargo build --release`
3. Tambahkan `target/release` ke PATH Anda

## Fitur Utama

- **30+ keyword Bahasa Indonesia**: `fungsi`, `komponen`, `jika`, `ulang`, `cocok`, dll.
- **React-like**: hooks (`keadaan`, `efek`, `ingat`, `acu`, `panggil`, `pengedger`, `konteks`)
- **JSX syntax**: `<div className="...">` dengan ekspresi `{...}`
- **Type system**: static typing dengan type inference
- **Multi-target**: Windows (Win32), Linux (GTK4), Web (WASM)
- **Polyglot FFI**: integrasi C, Python, Rust
- **Developer tools**: LSP, formatter (`rakit fmt`), test framework (`rakit test`)
- **Package manager**: `rakit install`, `rakit search`

"Rakit. Jalankan. Selesai."
