# Target Platform

## Windows (Win32)

Target native untuk Windows. Build langsung ke PE executable.

```bash
rakit build main.rakit --target win32
# Output: app.exe (native PE binary)
```

Menggunakan Brak codegen (x86_64) + native linker. Tidak perlu cargo/rustc untuk distribusi.

Fitur:
- Native Windows window
- Event handling (click, keyboard, dll)
- Styling via Win32 API
- Debug info (CodeView)

## Linux (GTK4)

Target native untuk Linux menggunakan GTK4.

```bash
rakit build main.rakit --target gtk4 --output app
./app
```

Fitur:
- Native GTK4 window
- CSS styling
- Signal-based events
- Debug info (DWARF)

## Web (WASM)

Target web menggunakan WebAssembly.

```bash
rakit build main.rakit --target wasm --output app.wasm
```

Fitur:
- WebAssembly output
- DOM manipulation via JS bridge
- Event handling via JS bridge
- Hot reload support

## Dev Mode

```bash
rakit dev main.rakit
# Hot reload — ubah file, lihat perubahan langsung
```

Dev mode features:
- File watching dengan auto rebuild
- Hot reload untuk perubahan komponen
- DevTools protocol
- Error overlay
