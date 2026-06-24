use std::collections::HashMap;

pub struct HoverEngine {
    docs: HashMap<String, String>,
}

impl HoverEngine {
    pub fn new() -> Self {
        let mut docs = HashMap::new();
        docs.insert("fungsi".into(), "Deklarasi fungsi: `fungsi nama(param: Tipe) -> TipeKembali { }`".into());
        docs.insert("komponen".into(), "Deklarasi komponen UI: `komponen Nama(props: Tipe) { tampilkan { ... } }`".into());
        docs.insert("tampilkan".into(), "Blok tampilan UI komponen: `tampilkan { <div>...</div> }`".into());
        docs.insert("keadaan".into(), "Hook state: `keadaan(nilai, aturNilai) = initial;`".into());
        docs.insert("efek".into(), "Hook efek samping: `efek(() => { ... }, [deps])`".into());
        docs.insert("ingat".into(), "Hook memoized value: `ingat(() => expr, [deps])`".into());
        docs.insert("panggil".into(), "Hook memoized callback: `panggil(() => fn, [deps])`".into());
        docs.insert("acu".into(), "Hook ref: `acu(nilaiAwal)` — memegang referensi mutable".into());
        docs.insert("konteks".into(), "Hook context: `konteks(Konteks)` — membaca nilai dari context provider".into());
        docs.insert("pengedger".into(), "Hook reducer: `pengedger(state, dispatch) = reducer, initial`".into());
        docs.insert("jalan".into(), "Blok async: `jalan { konstan data = await fetch(...); ... }`".into());
        docs.insert("coba".into(), "Try-catch: `coba { ... } tangkap(err) { ... }`".into());
        docs.insert("tangkap".into(), "Catch block: `coba { ... } tangkap(err) { ... }`".into());
        docs.insert("lempar".into(), "Throw error: `lempar Error(\"pesan\")`".into());
        docs.insert("konstan".into(), "Variabel immutable: `konstan nama: Tipe = nilai;`".into());
        docs.insert("ubah".into(), "Variabel mutable: `ubah nama: Tipe = nilai;`".into());
        docs.insert("jika".into(), "Kondisional: `jika (kondisi) { ... } lain { ... }`".into());
        docs.insert("ulang".into(), "Perulangan: `ulang (i dalam 0..10) { ... }` atau `ulang (kondisi) { ... }`".into());
        docs.insert("cocok".into(), "Pattern matching: `cocok (nilai) { kasus 1 => ..., kasus _ => ... }`".into());
        docs.insert("struktur".into(), "Definisi struct: `struktur Nama { field: Tipe, ... }`".into());
        docs.insert("pilih".into(), "Definisi enum: `pilih Nama { Varian1, Varian2(Tipe), ... }`".into());
        docs.insert("tipe".into(), "Type alias: `tipe Nama = TipeAsli;`".into());
        docs.insert("benar".into(), "Nilai boolean true".into());
        docs.insert("salah".into(), "Nilai boolean false".into());
        docs.insert("batal".into(), "Nilai null/void: `batal`".into());
        HoverEngine { docs }
    }

    pub fn hover_for(&self, word: &str) -> Option<&str> {
        self.docs.get(word).map(|s| s.as_str())
    }

    pub fn add_doc(&mut self, keyword: String, doc: String) {
        self.docs.insert(keyword, doc);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hover_for_keyword() {
        let engine = HoverEngine::new();
        let hover = engine.hover_for("fungsi");
        assert!(hover.is_some());
        assert!(hover.unwrap().contains("fungsi"));
    }

    #[test]
    fn test_hover_for_unknown() {
        let engine = HoverEngine::new();
        let hover = engine.hover_for("unknown_keyword");
        assert!(hover.is_none());
    }

    #[test]
    fn test_hover_add_doc() {
        let mut engine = HoverEngine::new();
        engine.add_doc("custom".into(), "Dokumentasi custom".into());
        assert_eq!(engine.hover_for("custom"), Some("Dokumentasi custom"));
    }
}
