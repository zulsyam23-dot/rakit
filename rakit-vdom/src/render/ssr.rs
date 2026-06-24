use crate::node::*;

pub struct SsrRenderer {
    output: String,
    pretty: bool,
    indent: usize,
}

impl SsrRenderer {
    pub fn new(pretty: bool) -> Self {
        SsrRenderer {
            output: String::new(),
            pretty,
            indent: 0,
        }
    }

    pub fn render(&mut self, node: &VDomNode) -> String {
        self.output.clear();
        self.render_node(node);
        self.output.clone()
    }

    fn render_node(&mut self, node: &VDomNode) {
        match node {
            VDomNode::Element(elem) => self.render_element(elem),
            VDomNode::Text(text) => self.render_text(&text.value),
            VDomNode::Fragment(frag) => {
                for child in &frag.children {
                    self.render_node(child);
                }
            }
            VDomNode::Component(comp) => {
                self.output.push_str("<!-- component: ");
                self.output.push_str(&comp.name);
                self.output.push_str(" -->");
            }
            VDomNode::Empty => {}
        }
    }

    fn render_element(&mut self, elem: &ElementNode) {
        if self.pretty {
            self.output.push('\n');
            for _ in 0..self.indent {
                self.output.push_str("  ");
            }
        }
        self.indent += 1;

        self.output.push('<');
        self.output.push_str(&elem.tag);

        for (name, value) in &elem.attrs {
            self.output.push(' ');
            self.output.push_str(name);
            self.output.push('=');
            self.output.push('"');
            match value {
                AttrValue::String(s) => self.output.push_str(&escape_html(s)),
                AttrValue::Bool(b) => {
                    if *b {
                        self.output.push_str(name);
                    }
                }
                AttrValue::Number(n) => self.output.push_str(&n.to_string()),
                AttrValue::Expression(e) => self.output.push_str(&escape_html(e)),
            }
            self.output.push('"');
        }

        if elem.children.is_empty() && SELF_CLOSING_TAGS.contains(&elem.tag.as_str()) {
            self.output.push_str(" />");
            self.indent -= 1;
            return;
        }

        self.output.push('>');

        if elem.children.is_empty() {
            // empty tag
        } else if elem.children.len() == 1 && matches!(elem.children[0], VDomNode::Text(_)) {
            self.render_node(&elem.children[0]);
        } else {
            for child in &elem.children {
                self.render_node(child);
            }
        }

        self.indent -= 1;
        if self.pretty && !elem.children.is_empty() {
            self.output.push('\n');
            for _ in 0..self.indent {
                self.output.push_str("  ");
            }
        }
        self.output.push_str("</");
        self.output.push_str(&elem.tag);
        self.output.push('>');
    }

    fn render_text(&mut self, text: &str) {
        self.output.push_str(&escape_html(text));
    }
}

const SELF_CLOSING_TAGS: &[&str] = &[
    "area", "base", "br", "col", "embed", "hr", "img", "input", "link", "meta", "param",
    "source", "track", "wbr",
];

fn escape_html(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}
