# FAQ — Tanya Jawab

## Apa itu Rakit?

Rakit adalah bahasa pemrograman untuk membangun antarmuka pengguna (UI) reaktif, mirip React, tetapi dengan sintaksis dalam Bahasa Indonesia.

## Mengapa Bahasa Indonesia?

Untuk menurunkan hambatan bagi developer Indonesia dan memperkenalkan pemrograman UI reaktif dengan cara yang lebih mudah dipahami.

## Platform apa saja yang didukung?

Windows (Win32 native), Linux (GTK4), dan Web (WASM).

## Apakah Rakit siap untuk production?

Ya, Rakit v1.0.5 memiliki semua fitur yang diperlukan untuk production: type system, optimasi compiler, polyglot FFI, package manager, LSP, formatter, dan test framework. Build native langsung ke .exe tanpa cargo.

## Bagaimana performanya?

Rakit menggunakan arsitektur zero-runtime — kompiler mengubah komponen Rakit langsung ke native code melalui Brak Language Toolkit. Hasilnya adalah binary native dengan performa tinggi.

## Apakah saya perlu tahu React untuk menggunakan Rakit?

Tidak, tetapi pengalaman dengan React akan membantu karena Rakit menggunakan konsep yang sama (komponen, hooks, VDOM) hanya dengan sintaksis Bahasa Indonesia.

## Bagaimana cara berkontribusi?

Fork repositori di GitHub, buat branch fitur, dan kirim pull request. Lihat CONTRIBUTING.md untuk detailnya.

## Lisensi apa yang digunakan?

MIT License — bebas digunakan untuk proyek komersial dan pribadi.

## Di mana saya bisa mendapatkan bantuan?

- Dokumentasi: docs/
- Issues: GitHub Issues
- Discord: discord.gg/rakit
