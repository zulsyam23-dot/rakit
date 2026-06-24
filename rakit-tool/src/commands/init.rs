use std::fs;
use std::path::Path;

pub struct InitCommand;

impl InitCommand {
    pub fn run(name: &str) -> Result<(), String> {
        let dir = Path::new(name);
        if dir.exists() {
            return Err(format!("Direktori '{}' sudah ada.", name));
        }

        fs::create_dir_all(dir.join("src"))
            .map_err(|e| format!("Gagal membuat direktori: {}", e))?;

        // Buat rakit.json
        let config = serde_json::json!({
            "name": name,
            "version": "0.1.0",
            "rakit-version": "0.1.0",
            "source-dir": "src",
            "target": "native",
            "dependencies": {}
        });
        let config_str = serde_json::to_string_pretty(&config)
            .map_err(|e| format!("Gagal membuat konfigurasi: {}", e))?;
        fs::write(dir.join("rakit.json"), config_str)
            .map_err(|e| format!("Gagal menulis rakit.json: {}", e))?;

        // Buat main.rakit
        let main_content = format!(r#"// Rakit — {name}
// Selamat datang di Rakit! Bahasa UI reaktif dalam Bahasa Indonesia.

komponen Halaman(judul: String) {{
    konstan pesan = "Halo dari Rakit!"

    tampilkan <div kelas="container">
        <h1>{{judul}}</h1>
        <p>{{pesan}}</p>
    </div>
}}

fungsi utama() {{
    // Entry point
}}
"#);
        fs::write(dir.join("src").join("main.rakit"), main_content)
            .map_err(|e| format!("Gagal menulis main.rakit: {}", e))?;

        println!("Project Rakit '{}' berhasil dibuat!", name);
        println!("  cd {}", name);
        println!("  rakit build src/main.rakit");

        Ok(())
    }
}
