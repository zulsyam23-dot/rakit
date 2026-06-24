# Pendahuluan Rakit

## Filosofi

Rakit lahir dari keyakinan bahwa bahasa pemrograman tidak harus selalu menggunakan bahasa Inggris. Dengan sintaksis dalam Bahasa Indonesia, Rakit bertujuan untuk:

1. **Menurunkan hambatan** bagi developer Indonesia yang tidak fasih bahasa Inggris
2. **Meningkatkan produktivitas** dengan kode yang lebih natural dibaca
3. **Memperkenalkan pemrograman UI reaktif** dengan cara yang lebih mudah dipahami

Rakit menggabungkan konsep React (komponen, hooks, VDOM) dengan sintaksis yang familiar bagi penutur Bahasa Indonesia.

## Instalasi

### Prasyarat

- Rust toolchain (rustup, cargo)
- Untuk target Win32: Visual Studio Build Tools
- Untuk target GTK4: GTK4 development libraries
- Untuk target WASM: wasm-pack

### Build dari Source

```bash
git clone https://github.com/rakit/rakit
cd rakit/rakit
cargo build --release
cp target/release/rakit.exe /usr/local/bin/
```

### Verifikasi

```bash
rakit --version
# Rakit v1.0.0
```

## Struktur Project

```
project/
├── main.rakit          # Entry point
├── komponen/           # Komponen-komponen UI
│   ├── Tombol.rakit
│   └── Daftar.rakit
├── rakit.json           # Manifest file (untuk package)
└── .rakit-cache/       # Cache compiler
```

## Hello World

```rakit
// main.rakit
fungsi main() -> I32 {
    cetak("Halo, Rakit!")
    0
}
```

Build dan jalankan:

```bash
rakit build main.rakit
./main.exe
```
