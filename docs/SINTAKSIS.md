# Sintaksis Rakit

## Daftar Keyword (30+)

| Keyword | Padanan | Deskripsi |
|---------|---------|-----------|
| `fungsi` | `function` | Deklarasi fungsi |
| `komponen` | `component` | Deklarasi komponen UI |
| `tampilkan` | `render` | Blok render komponen |
| `keadaan` | `useState` | Hook state lokal |
| `efek` | `useEffect` | Hook efek samping |
| `ingat` | `useMemo` | Hook memoized value |
| `acu` | `useRef` | Hook ref |
| `panggil` | `useCallback` | Hook callback |
| `konteks` | `useContext` | Hook context |
| `pengedger` | `useReducer` | Hook reducer |
| `jalan` | `async/await` | Eksekusi async |
| `konstan` | `let` | Variabel immutable |
| `ubah` | `let mut` | Variabel mutable |
| `jika` | `if` | Kondisional |
| `lain` | `else` | Else branch |
| `ulang` | `while/for` | Perulangan |
| `cocok` | `match` | Pattern matching |
| `kasus` | `case` | Match arm |
| `coba` | `try` | Try block |
| `tangkap` | `catch` | Catch block |
| `lempar` | `throw` | Throw error |
| `berhenti` | `break` | Break loop |
| `dari` | `from` | Import modul |
| `ke` | `as` | Alias |
| `tipe` | `type` | Type alias |
| `struktur` | `struct` | Struct definition |
| `pilih` | `enum` | Enum definition |
| `benar` | `true` | Boolean true |
| `salah` | `false` | Boolean false |
| `batal` | `null` | Null value |

## Tipe Data

| Tipe | Deskripsi |
|------|-----------|
| `I32` | Integer 32-bit |
| `I64` | Integer 64-bit |
| `U32` | Unsigned 32-bit |
| `U64` | Unsigned 64-bit |
| `F32` | Float 32-bit |
| `F64` | Float 64-bit |
| `Bool` | Boolean |
| `String` | String |
| `Char` | Character |
| `Void` | Unit type |
| `Node` | Tipe elemen UI |

## Fungsi

```rakit
fungsi tambah(a: I32, b: I32) -> I32 {
    a + b
}

// Fungsi tanpa return
fungsi sapa(nama: String) -> Void {
    cetak("Halo, {nama}")
}
```

## Percabangan

```rakit
jika (nilai > 10) {
    cetak("Besar")
} lain jika (nilai > 5) {
    cetak("Sedang")
} lain {
    cetak("Kecil")
}
```

## Perulangan

```rakit
// While loop
ubah i = 0;
ulang (i < 10) {
    cetak("Iterasi {i}")
    i = i + 1
}

// For loop
ulang (item dalam daftar) {
    cetak("Item: {item}")
}
```

## Pattern Matching

```rakit
cocok (nilai) {
    kasus 1 => cetak("satu")
    kasus 2 => cetak("dua")
    kasus _ => cetak("lainnya")
}
```

## Struct dan Enum

```rakit
struktur User {
    nama: String,
    umur: I32,
}

pilih Status {
    Aktif,
    Nonaktif(String),
    Tertunda,
}

struktur User {
    nama: String,
    umur: I32,
}
```
