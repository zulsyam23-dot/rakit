# Sistem Komponen Rakit

## Komponen Dasar

Komponen adalah blok pembangun UI di Rakit. Setiap komponen memiliki blok `tampilkan` yang mengembalikan elemen UI.

```rakit
komponen Tombol(props: { teks: String, onClick: () -> Void }) {
    tampilkan {
        <button className="tombol" onClick={props.onClick}>
            {props.teks}
        </button>
    }
}
```

## JSX Syntax

Rakit menggunakan sintaksis mirip JSX:

```rakit
tampilkan {
    <div className="container">
        <h1>"Judul"</h1>
        <p>"Paragraf {variabel}"</p>
        <Tombol teks="Klik" onClick={handler} />
    </div>
}
```

Aturan JSX:
- Satu root element (atau fragment `<></>`)
- Atribut dengan nilai string: `nama="value"`
- Atribut dengan ekspresi: `nama={ekspresi}`
- Children string: `"teks"`
- Children elemen: `<Elemen />`

## Fragment

```rakit
tampilkan {
        <div>"Item 1"</div>
        <div>"Item 2"</div>
    
}
```

## Props

Props adalah parameter yang diteruskan ke komponen:

```rakit
komponen Daftar(props: { items: Array<String> }) {
    tampilkan {
        <ul>
            {props.items.map(item =>
                <li key={item}>{item}</li>
            )}
        </ul>
    }
}

// Penggunaan:
<Daftar items={["A", "B", "C"]} />
```

## Conditional Rendering

```rakit
komponen Pesan(props: { sukses: Bool }) {
    tampilkan {
        <div>
            {props.sukses ?
                <div>"Berhasil!"</div> :
                <div>"Gagal!"</div>
            }
        </div>
    }
}
```

## Keyed Reconciliation

Gunakan atribut `key` untuk optimasi render:

```rakit
<ul>
    {items.map(item =>
        <li key={item.id}>{item.nama}</li>
    )}
</ul>
```
