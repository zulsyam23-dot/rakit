pub struct DomTester {
    inner: String,
}

impl DomTester {
    pub fn new(html: &str) -> Self {
        DomTester {
            inner: html.to_string(),
        }
    }

    pub fn cari_teks(&self) -> &str {
        &self.inner
    }

    pub fn mengandung(&self, teks: &str) -> bool {
        self.inner.contains(teks)
    }

    pub fn panjang(&self) -> usize {
        self.inner.len()
    }
}

impl From<String> for DomTester {
    fn from(s: String) -> Self {
        DomTester::new(&s)
    }
}

impl<'a> From<&'a str> for DomTester {
    fn from(s: &'a str) -> Self {
        DomTester::new(s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dom_tester_contains() {
        let dom = DomTester::new("<div>Halo Dunia</div>");
        assert!(dom.mengandung("Halo"));
        assert!(!dom.mengandung("Tidak ada"));
    }

    #[test]
    fn test_dom_tester_from_string() {
        let dom: DomTester = String::from("<p>Test</p>").into();
        assert_eq!(dom.cari_teks(), "<p>Test</p>");
    }

    #[test]
    fn test_dom_tester_from_str() {
        let dom: DomTester = "<span>Hai</span>".into();
        assert!(dom.mengandung("Hai"));
    }
}
