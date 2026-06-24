# Hooks Reference

## useState — `keadaan`

```rakit
keadaan(hitung, aturHitung) = 0;
keadaan(nama, aturNama) = "Rakit";
keadaan(aktif, setAktif) = benar;
```

## useEffect — `efek`

```rakit
efek(() => {
    // Efek samping
    cetak("Komponen dimount!");
    () => {
        // Cleanup
        cetak("Komponen unmount!");
    }
}, []);

efek(() => {
    cetak("Nilai berubah: {hitung}");
}, [hitung]);
```

## useMemo — `ingat`

```rakit
ingat(hasil) = () => {
    hitungMahal(props.data)
}, [props.data];
```

## useCallback — `panggil`

```rakit
panggil(handler) = () => {
    aturHitung(hitung + 1)
}, [hitung];
```

## useRef — `acu`

```rakit
acu(inputRef) = batal;

// Mengakses ref
<input ref={inputRef} type="text" />
```

## useContext — `konteks`

```rakit
konteks(Tema) = TemaKonteks;

komponen Tombol() {
    konteks(tema) = TemaKonteks;
    tampilkan {
        <button className={tema}>
            "Tombol"
        </button>
    }
}
```

## useReducer — `pengedger`

```rakit
pengedger(state, dispatch) = (prev, aksi) {
    cocok (aksi.tipe) {
        kasus "tambah" => { hitung: prev.hitung + 1 }
        kasus "kurang" => { hitung: prev.hitung - 1 }
        kasus "reset" => { hitung: 0 }
        kasus _ => prev
    }
}, { hitung: 0 };
```

## Aturan Hooks

1. Panggil hooks hanya di level teratas komponen
2. Panggil hooks hanya dari komponen Rakit
3. Gunakan hooks dengan urutan yang sama setiap render
