use rakit_vdom::node::AttrValue;

pub struct GtkWidgetMap;

impl GtkWidgetMap {
    pub fn tag_to_type(tag: &str) -> &'static str {
        match tag {
            "div" | "container" | "header" | "footer" | "nav" | "main" | "section" => "Box",
            "button" | "tombol" => "Button",
            "text" | "span" | "p" | "label" => "Label",
            "h1" | "h2" | "h3" | "h4" | "h5" | "h6" => "Label",
            "input" | "textbox" => "Entry",
            "checkbox" => "CheckButton",
            "image" | "img" => "Image",
            "progress" | "progressbar" => "ProgressBar",
            "slider" => "Scale",
            "list" | "listbox" => "ListView",
            "dropdown" | "select" | "combobox" => "DropDown",
            "scroll" | "scrollbar" => "ScrolledWindow",
            _ => "Box",
        }
    }

    pub fn set_attr(_widget_type: &str, name: &str, value: &AttrValue) -> GtkAttrOp {
        match name {
            "text" | "teks" | "label" | "value" => {
                if let AttrValue::String(text) = value {
                    return GtkAttrOp::SetText(text.clone());
                }
                GtkAttrOp::None
            }
            "placeholder" => {
                if let AttrValue::String(text) = value {
                    return GtkAttrOp::SetPlaceholder(text.clone());
                }
                GtkAttrOp::None
            }
            "enabled" | "aktif" => {
                let enabled = match value {
                    AttrValue::Bool(b) => *b,
                    _ => true,
                };
                GtkAttrOp::SetSensitive(enabled)
            }
            "visible" | "terlihat" => {
                let visible = match value {
                    AttrValue::Bool(b) => *b,
                    _ => true,
                };
                GtkAttrOp::SetVisible(visible)
            }
            "className" | "class" => {
                if let AttrValue::String(class) = value {
                    return GtkAttrOp::SetCssClass(class.clone());
                }
                GtkAttrOp::None
            }
            _ => GtkAttrOp::None,
        }
    }
}

pub enum GtkAttrOp {
    None,
    SetText(String),
    SetPlaceholder(String),
    SetSensitive(bool),
    SetVisible(bool),
    SetCssClass(String),
    SetValue(f64),
}
