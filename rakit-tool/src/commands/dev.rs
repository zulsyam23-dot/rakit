use crate::commands::build::BuildCommand;
use notify::{Config, Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::path::Path;
use std::path::PathBuf;
use std::sync::mpsc::{self, Sender};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

static INDEX_HTML: &str = r#"<!DOCTYPE html>
<html>
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width, initial-scale=1">
  <title>Rakit App</title>
  <style>
    body { font-family: system-ui, sans-serif; margin: 0; padding: 0; }
    #app { min-height: 100vh; }
  </style>
</head>
<body>
  <div id="app"></div>
  <script type="module">
    import init, { start_app } from "./{app_name}.js";
    init().then(() => {
      if (start_app) start_app();
    });

    const ws = new WebSocket(`ws://${location.hostname}:{ws_port}/`);
    ws.onmessage = (msg) => {
      if (msg.data === "reload") location.reload();
    };
    ws.onclose = () => setTimeout(() => location.reload(), 1000);
  </script>
</body>
</html>"#;

fn mime_type(path: &Path) -> &'static str {
    match path.extension().and_then(|e| e.to_str()) {
        Some("html") => "text/html; charset=utf-8",
        Some("js") => "text/javascript; charset=utf-8",
        Some("wasm") => "application/wasm",
        Some("css") => "text/css; charset=utf-8",
        Some("json") => "application/json",
        Some("png") => "image/png",
        Some("jpg" | "jpeg") => "image/jpeg",
        Some("svg") => "image/svg+xml",
        Some("ico") => "image/x-icon",
        Some("toml") => "text/plain; charset=utf-8",
        Some("txt") => "text/plain; charset=utf-8",
        _ => "application/octet-stream",
    }
}

fn serve_static(
    request: tiny_http::Request,
    root_dir: &Path,
    app_name: &str,
    ws_port: u16,
) {
    let url_path = request.url().to_string();
    let path = if url_path == "/" {
        root_dir.join("index.html")
    } else {
        let cleaned = url_path.trim_start_matches('/');
        root_dir.join(cleaned)
    };

    let response: tiny_http::Response<std::io::Cursor<Vec<u8>>> =
        if path.exists() && path.is_file() {
            let mime = mime_type(&path);
            match std::fs::read(&path) {
                Ok(data) => tiny_http::Response::from_data(data).with_header(
                    tiny_http::Header::from_bytes(&b"Content-Type"[..], mime.as_bytes()).unwrap(),
                ),
                Err(e) => {
                    eprintln!("  Gagal baca {}: {}", path.display(), e);
                    tiny_http::Response::from_string("500 Internal Server Error")
                        .with_status_code(500)
                }
            }
        } else if url_path == "/" || url_path.is_empty() {
            let html = INDEX_HTML
                .replace("{app_name}", app_name)
                .replace("{ws_port}", &ws_port.to_string());
            tiny_http::Response::from_string(html).with_header(
                tiny_http::Header::from_bytes(&b"Content-Type"[..], b"text/html; charset=utf-8")
                    .unwrap(),
            )
        } else {
            tiny_http::Response::from_string("404 Not Found").with_status_code(404)
        };

    let _ = request.respond(response);
}

fn run_ws_server(ws_port: u16, reload_rx: std::sync::mpsc::Receiver<()>) {
    let addr = format!("0.0.0.0:{}", ws_port);
    let listener = match std::net::TcpListener::bind(&addr) {
        Ok(l) => l,
        Err(e) => {
            eprintln!("  WS server gagal bind {}: {}", addr, e);
            return;
        }
    };
    listener.set_nonblocking(true).ok();

    let clients: Arc<Mutex<Vec<tungstenite::WebSocket<std::net::TcpStream>>>> =
        Arc::new(Mutex::new(Vec::new()));

    let clients_clone = clients.clone();
    thread::spawn(move || {
        while let Ok(()) = reload_rx.recv() {
            let mut cs = clients_clone.lock().unwrap();
            use tungstenite::Message;
            let mut i = 0;
            while i < cs.len() {
                if cs[i].send(Message::Text("reload".into())).is_err() {
                    cs.remove(i);
                } else {
                    i += 1;
                }
            }
        }
    });

    for stream in listener.incoming() {
        match stream {
            Ok(s) => {
                s.set_nonblocking(true).ok();
                if let Ok(ws) = tungstenite::accept(s) {
                    clients.lock().unwrap().push(ws);
                }
            }
            Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                thread::sleep(Duration::from_millis(100));
            }
            Err(e) => {
                eprintln!("  WS accept error: {}", e);
                thread::sleep(Duration::from_millis(500));
            }
        }
    }
}

fn start_file_watcher(
    file: &Path,
    rebuild_tx: Sender<()>,
) -> notify::Result<RecommendedWatcher> {
    let (event_tx, event_rx) = mpsc::channel();

    let mut watcher = RecommendedWatcher::new(
        move |res: Result<Event, notify::Error>| {
            if let Ok(event) = res {
                let _ = event_tx.send(event);
            }
        },
        Config::default(),
    )?;

    let watch_dir = if file.is_dir() {
        file.to_path_buf()
    } else {
        file.parent().unwrap_or(Path::new(".")).to_path_buf()
    };
    watcher.watch(&watch_dir, RecursiveMode::Recursive)?;

    thread::spawn(move || {
        let mut last_build = std::time::Instant::now();
        for event in event_rx {
            if matches!(
                event.kind,
                EventKind::Create(_) | EventKind::Modify(_) | EventKind::Remove(_)
            ) {
                if event
                    .paths
                    .iter()
                    .any(|p| p.extension().map_or(false, |e| e == "rakit"))
                {
                    if last_build.elapsed() > Duration::from_millis(500) {
                        println!("  perubahan terdeteksi, rebuild...");
                        let _ = rebuild_tx.send(());
                        last_build = std::time::Instant::now();
                    }
                }
            }
        }
    });

    Ok(watcher)
}

fn rebuild_project(file: &Path) -> Result<(), String> {
    BuildCommand::run_with_opts(file, false, Some("wasm"), None, 0)
}

pub fn run_dev_server(file: &Path, port: Option<u16>) -> Result<(), String> {
    let http_port = port.unwrap_or(8080);
    let ws_port = http_port + 1;

    let project_dir = if file.is_dir() {
        file.to_path_buf()
    } else {
        file.parent().unwrap_or(Path::new(".")).to_path_buf()
    };

    let app_name = find_project_root_for_dev(&file)
        .or_else(|| std::env::current_dir().ok())
        .and_then(|root| root.file_name().map(|s| s.to_string_lossy().to_string()))
        .unwrap_or_else(|| "app".to_string());

    println!("╔══════════════════════════════════════╗");
    println!("║  Rakit Dev Server                    ║");
    println!("╠══════════════════════════════════════╣");
    println!("║  Entry: {}", file.display());
    println!("║  HTTP:  http://localhost:{}/", http_port);
    println!("║  WS:    ws://localhost:{}/", ws_port);
    println!("╚══════════════════════════════════════╝");

    println!("");
    println!("Build awal...");

    rebuild_project(file)?;

    println!("  ✅ Build awal selesai");
    println!("  Menunggu perubahan... (Ctrl+C untuk berhenti)");
    println!("");

    let (rebuild_tx, rebuild_rx) = mpsc::channel();
    let (reload_tx, reload_rx) = mpsc::channel();

    let _watcher = start_file_watcher(file, rebuild_tx)
        .map_err(|e| format!("Gagal start file watcher: {}", e))?;

    let _ws_thread = thread::spawn(move || {
        run_ws_server(ws_port, reload_rx);
    });

    let http_dir = project_dir.clone();
    let http_name = app_name.clone();
    let _http_thread = thread::spawn(move || {
        let addr = format!("0.0.0.0:{}", http_port);
        let server = match tiny_http::Server::http(&addr) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("  Gagal start HTTP server: {}", e);
                return;
            }
        };
        loop {
            match server.recv() {
                Ok(request) => serve_static(request, &http_dir, &http_name, ws_port),
                Err(e) => {
                    eprintln!("  HTTP server error: {}", e);
                    break;
                }
            }
        }
    });

    let file = file.to_path_buf();
    loop {
        if rebuild_rx.recv().is_err() {
            return Ok(());
        }
        while rebuild_rx.try_recv().is_ok() {}

        print!("  ⏳ Rebuild...");
        let _ = std::io::Write::flush(&mut std::io::stdout());

        match rebuild_project(&file) {
            Ok(()) => {
                println!(" ✅");
                let _ = reload_tx.send(());
            }
            Err(e) => {
                println!(" ❌");
                eprintln!("  {}", e);
            }
        }
    }
}

fn find_project_root_for_dev(entry: &Path) -> Option<PathBuf> {
    let mut dir = entry.parent()?;
    loop {
        if dir.join("devil.json").exists() {
            return Some(dir.to_path_buf());
        }
        match dir.parent() {
            Some(p) if p != dir => dir = p,
            _ => return None,
        }
    }
}
