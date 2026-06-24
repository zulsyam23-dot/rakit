use crate::extract::{DocItem, DocItemKind};

pub fn render_html(items: &[DocItem]) -> String {
    let mut html = String::new();
    html.push_str("<!DOCTYPE html>\n<html lang=\"id\">\n<head>\n");
    html.push_str("<meta charset=\"UTF-8\">\n");
    html.push_str("<title>Dokumentasi Rakit</title>\n");
    html.push_str("<style>\n");
    html.push_str("body { font-family: -apple-system, sans-serif; max-width: 960px; margin: 0 auto; padding: 20px; }\n");
    html.push_str("h1 { border-bottom: 2px solid #ddd; }\n");
    html.push_str(".item { margin: 16px 0; padding: 12px; background: #f5f5f5; border-radius: 8px; }\n");
    html.push_str(".item h3 { margin: 0 0 8px 0; }\n");
    html.push_str("pre { background: #eee; padding: 8px; border-radius: 4px; overflow-x: auto; }\n");
    html.push_str("</style>\n</head>\n<body>\n");
    html.push_str("<h1>Dokumentasi Rakit</h1>\n");
    html.push_str("<p>Dokumentasi otomatis dari source code Rakit.</p>\n");

    for item in items {
        html.push_str("<div class=\"item\">\n");
        let kind_str = match item.kind {
            DocItemKind::Function => "Fungsi",
            DocItemKind::Component => "Komponen",
            DocItemKind::Struct => "Struktur",
            DocItemKind::Enum => "Enum",
            DocItemKind::TypeAlias => "Alias Tipe",
        };
        html.push_str(&format!("<h3>[{}] {}</h3>\n", kind_str, item.name));
        html.push_str(&format!("<p>{}</p>\n", item.description));

        if !item.params.is_empty() {
            html.push_str("<h4>Parameter</h4>\n<ul>\n");
            for p in &item.params {
                html.push_str(&format!("<li><code>{}</code>: {}</li>\n", p.name, p.description));
            }
            html.push_str("</ul>\n");
        }

        if let Some(ref ret) = item.returns {
            html.push_str(&format!("<p><strong>Return:</strong> <code>{}</code></p>\n", ret));
        }

        html.push_str("</div>\n");
    }

    html.push_str("</body>\n</html>");
    html
}

pub fn render_markdown(items: &[DocItem]) -> String {
    let mut md = String::new();
    md.push_str("# Dokumentasi Rakit\n\n");
    md.push_str("Dokumentasi otomatis dari source code.\n\n");

    for item in items {
        let kind_str = match item.kind {
            DocItemKind::Function => "Fungsi",
            DocItemKind::Component => "Komponen",
            DocItemKind::Struct => "Struktur",
            DocItemKind::Enum => "Enum",
            DocItemKind::TypeAlias => "Alias Tipe",
        };
        md.push_str(&format!("## {}: `{}`\n\n", kind_str, item.name));
        md.push_str(&format!("{}\n\n", item.description));

        if !item.params.is_empty() {
            md.push_str("### Parameter\n\n");
            for p in &item.params {
                md.push_str(&format!("- **`{}`**: {}\n", p.name, p.description));
            }
            md.push_str("\n");
        }

        if let Some(ref ret) = item.returns {
            md.push_str(&format!("**Return:** `{}`\n\n", ret));
        }

        md.push_str("---\n\n");
    }

    md
}
