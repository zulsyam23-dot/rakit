# Polyglot Integration

Rakit mendukung integrasi dengan C, Python, dan Rust melalui polyglot FFI.

## C Bindings

Generate header file C dari fungsi Rakit:

```bash
rakit generate main.rakit --lang c --output rakit_bindings.h
```

```c
#include "rakit_bindings.h"

int main() {
    int32_t hasil = rakit_hitung(42);
    return hasil;
}
```

## Python Bindings

Generate modul Python:

```bash
rakit generate main.rakit --lang python --output rakit_module.py
```

```python
import rakit_module

hasil = rakit_module.hitung(42)
print(hasil)
```

## Rust FFI

Generate binding Rust:

```bash
rakit generate main.rakit --lang rust --output rakit_ffi.rs
```

```rust
mod rakit_ffi;

fn main() {
    let hasil = rakit_ffi::hitung(42);
    println!("{}", hasil);
}
```

## ABI Normalization

Rakit secara otomatis menormalisasi calling convention untuk cross-language calls:

```rust
// System V AMD64 ABI (Linux/macOS)
// Microsoft x64 ABI (Windows)
let abi = RakitAbi::for_target("win32");
```

## Export Function

Untuk mengekspor fungsi ke bahasa lain, gunakan:

```rakit
// Fungsi ini akan di-export
fungsi hitung(x: I32) -> I32 {
    x * 2
}
```
