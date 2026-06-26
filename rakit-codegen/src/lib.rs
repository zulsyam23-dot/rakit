use rakit_bridge::brak_types::*;
use std::cell::RefCell;
use std::collections::HashMap;
use std::collections::HashSet;

pub struct WasmCodegen {
    component_names: Vec<String>,
    app_name: String,
    imported_components: Vec<String>,
    extracted_props: Vec<String>,
    struct_defs: HashMap<String, BrakStructDef>,
    enum_defs: HashMap<String, BrakEnumDef>,
    map_dynamic_items: RefCell<HashSet<String>>,
}

impl WasmCodegen {
    pub fn new() -> Self {
        WasmCodegen {
            component_names: Vec::new(),
            app_name: String::new(),
            imported_components: Vec::new(),
            extracted_props: Vec::new(),
            struct_defs: HashMap::new(),
            enum_defs: HashMap::new(),
            map_dynamic_items: RefCell::new(HashSet::new()),
        }
    }

    pub fn generate(&mut self, program: &BrakProgram, app_name: &str) -> String {
        self.app_name = app_name.to_string();
        let mut code = String::new();

        // First pass: scan user-defined structs and enums so we can check for conflicts with shims
        for item in &program.items {
            if let BrakItem::Struct(s) = item {
                self.struct_defs.insert(s.name.clone(), s.clone());
            }
            if let BrakItem::Enum(e) = item {
                self.enum_defs.insert(e.name.clone(), e.clone());
            }
        }

        let has_user_pengguna = self.struct_defs.contains_key("Pengguna");

        // Emit imports
        code.push_str("use rakit_vdom::node::{VDomNode, AttrValue, ElementNode, ComponentNode, FragmentNode};\n");
        code.push_str("use rakit_vdom::h::{h, text, fragment};\n");
        code.push_str("use rakit_runtime::event::{EventType, EventData, register_global_handler};\n");
        code.push_str("use std::collections::HashMap;\n");
        code.push_str("use std::cell::RefCell;\n");
        code.push_str("use std::rc::Rc;\n");
        code.push_str("use wasm_bindgen::prelude::*;\n");
        code.push_str("use wasm_bindgen::UnwrapThrowExt;\n");
        code.push_str("\n");
        code.push_str("// --- Runtime shims ---\n");
        code.push_str("type Event = ();\n");
        code.push_str("fn rakit_debug<T>(v: &T) -> String { String::new() }\n");
        code.push_str("fn fmt_debug<T: std::fmt::Debug>(v: &T) -> String { format!(\"{:?}\", v) }\n");
        code.push_str("fn attr_debug<T: std::fmt::Debug>(v: &T) -> AttrValue { AttrValue::String(format!(\"{:?}\", v)) }\n");
        code.push_str("fn debug_fn() -> String { \"<fn>\".to_string() }\n");
        code.push_str("fn rakit_boolify(v: &str) -> bool { !v.is_empty() && v != \"false\" && v != \"0\" }\n");
        code.push_str("fn rakit_empty_vec_str() -> Vec<String> { vec![] }\n");

        code.push_str("fn use_context<T: Default>() -> T { T::default() }\n");
        code.push_str("fn ingat<T>(f: impl Fn() -> T, _deps: Vec<AttrValue>) -> T { f() }\n");
        code.push_str("fn parse_json<T: Default>(_s: String) -> T { T::default() }\n");
        code.push_str("fn string(v: impl ToString) -> String { v.to_string() }\n");
        code.push_str("fn rakit_to_bool_val(v: bool) -> bool { v }\n");
        code.push_str("fn refresh_sesi(_refresh_token: String) -> String { String::new() }\n");
        code.push_str("fn muat_ulang() {}\n");
        code.push_str("fn atur_muka(_tema: AttrValue) {}\n");
        code.push_str("fn atur_tema(_tema: HashMap<String, AttrValue>) {}\n");
        code.push_str("#[allow(non_upper_case_globals)]\n");
        code.push_str("static web_socket: WebSocketProvider = WebSocketProvider;\n");
        code.push_str("#[allow(non_upper_case_globals)]\n");
        code.push_str("static timer: TimerProvider = TimerProvider;\n");
        code.push_str("struct WebSocketProvider;\n");
        code.push_str("impl WebSocketProvider { fn baru(&self, _url: &str) -> WebSocketInstance { WebSocketInstance::default() } }\n");
        code.push_str("struct WebSocketInstance { on_message: Box<dyn Fn()> }\n");
        code.push_str("impl Default for WebSocketInstance { fn default() -> Self { Self { on_message: Box::new(|| {}) } } }\n");
        code.push_str("impl WebSocketInstance { fn tutup(&self) {} }\n");
        code.push_str("struct TimerProvider;\n");
        code.push_str("impl TimerProvider { fn baru(&self, _cb: impl Fn() + 'static, _ms: i32) -> TimerInstance { TimerInstance } }\n");
        code.push_str("struct TimerInstance;\n");
        code.push_str("impl TimerInstance { fn berhenti(&self) {} }\n");
        code.push_str("fn rakit_nullish<T: Default + PartialEq>(v: T, default: T) -> T { if v != T::default() { v } else { default } }\n");
        code.push_str("fn rakit_nullish_str(v: String, default: String) -> String { if !v.is_empty() { v } else { default } }\n");
        code.push_str("fn rakit_nullish_bool(v: &str, default: bool) -> bool { v.parse::<bool>().unwrap_or(default) }\n");
        code.push_str("fn rakit_nullish_num(v: &str, default: f64) -> f64 { v.parse::<f64>().unwrap_or(default) }\n");
        code.push_str("fn rakit_to_string(v: &dyn std::fmt::Debug) -> String { format!(\"{:?}\", v) }\n");
        code.push_str("fn rakit_as_node(v: &str) -> VDomNode { VDomNode::text(v) }\n");
        code.push_str("fn rakit_is_truthy(v: &str) -> bool { !v.is_empty() && v != \"false\" && v != \"0\" }\n");
code.push_str("fn rakit_truthy<T: Default + PartialEq>(v: &T) -> bool { *v != T::default() }\n");
        code.push_str("fn gunakan_fetch(_url: &str) -> FetchState { FetchState::default() }\n");
        code.push_str("#[derive(Default)]\n");
        code.push_str("struct FetchState { data: String, muat: bool, muat_ulang: bool, error: Option<String> }\n");
        code.push_str("struct RouterKonteks { path_saat_ini: String, navigasi: Box<dyn Fn(String)>, kembali: Box<dyn Fn()>, history: Vec<String>, params: HashMap<String, AttrValue>, query: HashMap<String, AttrValue> }\n");
        code.push_str("impl Default for RouterKonteks { fn default() -> Self { Self { path_saat_ini: String::new(), navigasi: Box::new(|_| {}), kembali: Box::new(|| {}), history: vec![], params: HashMap::new(), query: HashMap::new() } } }\n");
        code.push_str("struct AuthKonteks { pengguna: Pengguna, muat: bool, error: Option<String>, login: Box<dyn Fn(String, String) -> String>, logout: Box<dyn Fn()>, daftar: Box<dyn Fn(AttrValue) -> String>, refresh_sesi: Box<dyn Fn(String) -> String>, update_profil: Box<dyn Fn(HashMap<String, AttrValue>) -> String> }\n");
        code.push_str("impl Default for AuthKonteks { fn default() -> Self { Self { pengguna: Pengguna::default(), muat: false, error: None, login: Box::new(|_, _| String::new()), logout: Box::new(|| {}), daftar: Box::new(|_| String::new()), refresh_sesi: Box::new(|_| String::new()), update_profil: Box::new(|_| String::new()) } } }\n");
        code.push_str("struct TemaKonteks { tema: Tema, ganti_tema: Box<dyn Fn()>, atur_warna_primer: Box<dyn Fn(AttrValue)> }\n");
        code.push_str("impl Default for TemaKonteks { fn default() -> Self { Self { tema: Tema::default(), ganti_tema: Box::new(|| {}), atur_warna_primer: Box::new(|_| {}) } } }\n");
        code.push_str("struct Sesi { kedaluwarsa: f64, pengguna: HashMap<String, AttrValue>, refresh_token: String, token: String }\n");
        code.push_str("impl Default for Sesi { fn default() -> Self { Self { kedaluwarsa: 0.0, pengguna: HashMap::new(), refresh_token: String::new(), token: String::new() } } }\n");
        if !has_user_pengguna {
            code.push_str("struct Pengguna { nama: String, peran: String }\n");
            code.push_str("impl Default for Pengguna { fn default() -> Self { Self { nama: String::new(), peran: String::new() } } }\n");
            code.push_str("impl Pengguna { fn is_empty(&self) -> bool { self.nama.is_empty() } }\n");
        }
        code.push_str("struct Tema { mode: String, warna_primer: String }\n");
        code.push_str("impl Default for Tema { fn default() -> Self { Self { mode: String::new(), warna_primer: String::new() } } }\n");
        code.push_str("struct State { pengguna: HashMap<String, AttrValue>, muat: bool, error: Option<String>, sesi: HashMap<String, AttrValue> }\n");
        code.push_str("impl Default for State { fn default() -> Self { Self { pengguna: HashMap::new(), muat: false, error: None, sesi: HashMap::new() } } }\n");
        code.push_str("type Omit<T> = T;\n");
        code.push_str("type Array<T> = Vec<T>;\n");
        code.push_str("// --- End runtime shims ---\n");
        code.push_str("\n");

        for item in &program.items {
            match item {
                BrakItem::Function(f) => {
                    if is_component_fn(f) {
                        self.component_names.push(f.name.clone());
                    }
                    self.extracted_props.clear();
                    code.push_str(&self.gen_function(f));
                    code.push_str("\n");
                }
                BrakItem::Struct(s) => {
                    code.push_str(&self.gen_struct(s));
                    code.push_str("\n");
                }
                BrakItem::Enum(e) => {
                    code.push_str(&self.gen_enum(e));
                    code.push_str("\n");
                }
            }
        }

        code.push_str(&self.gen_entry_point());
        code
    }

    pub fn generate_manifest(&self, rakit_root: &str) -> String {
        fn norm(p: &str) -> String {
            p.replace('\\', "/")
        }
        let base = norm(rakit_root);
        let vdom = format!("{}/rakit-vdom", base);
        let ui = format!("{}/rakit-ui", base);
        let runtime = format!("{}/rakit-runtime", base);
        let web = format!("{}/rakit-backend-web", base);

        format!(
            r#"[package]
name = "{}"
version = "0.1.0"
edition = "2021"

[dependencies]
rakit-vdom = {{ path = "{}" }}
rakit-ui = {{ path = "{}" }}
rakit-runtime = {{ path = "{}" }}
rakit-backend-web = {{ path = "{}" }}
wasm-bindgen = "0.2"

[lib]
crate-type = ["cdylib"]

[target.'cfg(target_arch = "wasm32")'.dependencies]
wasm-bindgen = "0.2"
web-sys = {{ version = "0.3", features = ["Window", "Document", "Location", "Element", "HtmlElement", "Storage", "WheelEvent", "MouseEvent", "KeyboardEvent", "FocusEvent", "SubmitEvent", "PointerEvent", "TouchEvent", "DragEvent", "ClipboardEvent", "PopStateEvent"] }}
js-sys = "0.3"
"#,
            self.app_name, vdom, ui, runtime, web
        )
    }

    fn gen_function(&mut self, f: &BrakFnDef) -> String {
        let rust_name = to_snake_case(&f.name);
        let is_component = is_component_fn(f);
        let mut params: Vec<String> = f
            .params
            .iter()
            .map(|p| format!("{}: {}", to_snake_case(&p.name), self.gen_ty(&p.ty)))
            .collect();
        if is_component {
            if !params.is_empty() {
                params[0] = "props: &HashMap<String, AttrValue>".to_string();
            } else {
                params.push("_props: &HashMap<String, AttrValue>".to_string());
            }
        }
        let params_str = if params.is_empty() {
            String::new()
        } else {
            params.join(", ")
        };
        let ret = f
            .return_ty
            .as_ref()
            .map(|t| format!(" -> {}", self.gen_ty(t)))
            .unwrap_or_default();

        let mut code = format!("fn {}({}){} {{\n", rust_name, params_str, ret);

        if is_component {
            // Extract use_context let bindings from body and hoist them before hooks
            let (context_lets, remaining_body) = self.extract_context_lets(&f.body);
            code.push_str(&context_lets);
            code.push_str(&self.gen_component_hooks(f));
            code.push_str(&self.gen_component_body_props(f));
            let body_expr = self.gen_component_body(&remaining_body);
            code.push_str(&format!("    {}\n", body_expr));
        } else {
            let len = f.body.stmts.len();
            for (i, stmt) in f.body.stmts.iter().enumerate() {
                let is_last = i == len - 1;
                let stmt_str = self.gen_stmt(stmt, 1);
                if is_last {
                    code.push_str(&format!("    {}\n", stmt_str));
                } else {
                    code.push_str(&format!("    {};\n", stmt_str));
                }
            }
        }

        code.push_str("}\n");
        code
    }

    fn gen_component_hooks(&self, f: &BrakFnDef) -> String {
        let mut code = String::new();
        for hc in &f.hook_calls {
            match &hc.kind {
                BrakHookKind::State { state_var, setter_var, initial, .. } => {
                    let state_snake = to_snake_case(state_var);
                    let setter_snake = to_snake_case(setter_var);
                    let init_str = if let BrakExpr::Object(fields) = initial.as_ref() {
                        // Generate struct literal for object initial values
                        let struct_name = to_pascal_case(state_var);
                        let field_strs: Vec<String> = fields.iter().map(|(k, v)| {
                            let val_str = self.gen_expr(v, 1);
                            let final_val = if matches!(v, BrakExpr::String(_)) {
                                format!("{}.to_string()", val_str)
                            } else {
                                val_str
                            };
                            format!("{}: {}", to_snake_case(k), final_val)
                        }).collect();
                        if fields.is_empty() {
                            format!("{} {{ }}", struct_name)
                        } else {
                            format!("{} {{ {}, ..Default::default() }}", struct_name, field_strs.join(", "))
                        }
                    } else {
                        self.gen_expr(initial, 1)
                    };
                    code.push_str(&format!(
                        "    let {} = {};\n",
                        state_snake, init_str
                    ));
                    let setter_ty = if matches!(initial.as_ref(), BrakExpr::Object(_)) {
                        "val: HashMap<String, AttrValue>".to_string()
                    } else if matches!(initial.as_ref(), BrakExpr::Bool(_)) {
                        "val: bool".to_string()
                    } else if matches!(initial.as_ref(), BrakExpr::String(_)) {
                        "val: String".to_string()
                    } else if matches!(initial.as_ref(), BrakExpr::Number(_)) {
                        "val: f64".to_string()
                    } else if matches!(initial.as_ref(), BrakExpr::Array(_)) {
                        "val: Vec<AttrValue>".to_string()
                    } else {
                        "val: AttrValue".to_string()
                    };
                    code.push_str(&format!(
                        "    let {} = |{}| {{ /* TODO: re-render */ }};\n",
                        setter_snake, setter_ty
                    ));
                }
                BrakHookKind::Effect { callback, .. } => {
                    let cb_str = self.gen_expr(callback, 1);
                    let wrapped = if cb_str.starts_with('{') {
                        format!("|| {}", cb_str)
                    } else {
                        format!("|| {{ {} }}", cb_str)
                    };
                    code.push_str(&format!(
                        "    // effect: call immediately\n    {{ let _ = ({}); }}\n",
                        wrapped
                    ));
                }
                BrakHookKind::Memo { result_var, callback, .. } => {
                    let cb_str = self.gen_expr(callback, 1);
                    let result_snake = to_snake_case(result_var);
                    code.push_str(&format!(
                        "    let {} = {};\n",
                        result_snake, cb_str
                    ));
                }
            }
        }
        code
    }

    fn gen_component_body_props(&mut self, f: &BrakFnDef) -> String {
        let mut code = String::new();
        if f.params.is_empty() {
            return code;
        }
        let first_param = &f.params[0];
        if first_param.ty == BrakTy::Struct(vec![]) {
            return code;
        }
        let fields_to_extract: Vec<(String, BrakTy)> = if let BrakTy::Struct(fields) = &first_param.ty {
            // Check for double-wrapping: a single field "props" containing the actual struct
            if fields.len() == 1 && fields[0].0 == "props" {
                if let BrakTy::Struct(inner_fields) = &fields[0].1 {
                    inner_fields.clone()
                } else if let BrakTy::Named(name) = &fields[0].1 {
                    if let Some(struct_def) = self.struct_defs.get(name) {
                        struct_def.fields.iter().map(|f| (f.name.clone(), f.ty.clone())).collect()
                    } else {
                        return code;
                    }
                } else {
                    return code;
                }
            } else {
                fields.clone()
            }
        } else {
            return code;
        };
        for (name, ty) in &fields_to_extract {
            let snake_name = to_snake_case(name);
            self.extracted_props.push(snake_name.clone());
            if name == "anak" {
                code.push_str(&format!(
                    "    let {}: VDomNode = props.get(\"children\").cloned().and_then(|v| match v {{ AttrValue::String(s) => Some(VDomNode::text(&s)), _ => None }}).unwrap_or(VDomNode::empty());\n",
                    snake_name
                ));
            } else if matches!(ty, BrakTy::Fn(..)) {
                // Event handler / fn props are not serializable through AttrValue
                // Generate a no-op default closure instead
                let default_fn = match ty {
                    BrakTy::Fn(params, ret) => {
                        let params_strs: Vec<String> = params.iter().enumerate()
                            .map(|(i, p)| format!("_{}: {}", i, self.gen_ty(p))).collect();
                        let ret_str = self.gen_ty(ret);
                        let body = match ret_str.as_str() {
                            "()" => "{}".to_string(),
                            "String" => "{ String::new() }".to_string(),
                            "bool" => "{ false }".to_string(),
                            "i32" | "i64" | "u32" | "u64" => "{ 0 }".to_string(),
                            "f64" | "f32" => "{ 0.0 }".to_string(),
                            _ => "{ Default::default() }".to_string(),
                        };
                        if params.is_empty() {
                            format!("|| {}", body)
                        } else {
                            format!("|{}| {}", params_strs.join(", "), body)
                        }
                    }
                    _ => "|| {}".to_string(),
                };
                code.push_str(&format!(
                    "    let {} = {};\n",
                    snake_name, default_fn
                ));
            } else {
                let rust_ty = self.gen_ty(ty);
                let is_array = matches!(ty, BrakTy::Array(_)) 
                    || matches!(ty, BrakTy::Named(n) if n == "Array");
                let extract = if is_array {
                    "vec![]".to_string()
                } else {
                    match ty {
                        BrakTy::Bool => "props.get(\"{name}\").and_then(|v| match v { AttrValue::Bool(b) => Some(*b), AttrValue::String(s) => s.parse::<bool>().ok(), _ => None }).unwrap_or(false)".to_string(),
                        BrakTy::U8 => "props.get(\"{name}\").and_then(|v| if let AttrValue::Number(n) = v { Some(n as u8) } else { None }).unwrap_or(0u8)".to_string(),
                        BrakTy::Int(32) => "props.get(\"{name}\").and_then(|v| if let AttrValue::Number(n) = v { Some(n as i32) } else { None }).unwrap_or(0i32)".to_string(),
                        BrakTy::Int(64) => "props.get(\"{name}\").and_then(|v| if let AttrValue::Number(n) = v { Some(n as i64) } else { None }).unwrap_or(0i64)".to_string(),
                        BrakTy::Float(64) => "props.get(\"{name}\").and_then(|v| if let AttrValue::Number(n) = v { Some(*n) } else { None }).unwrap_or(0.0)".to_string(),
                        BrakTy::Any => "props.get(\"{name}\").and_then(|v| if let AttrValue::String(s) = v { Some(s.clone()) } else { None }).unwrap_or_default()".to_string(),
                        BrakTy::Named(name) if name == "ReactNode" || name == "Node" => "props.get(\"children\").cloned().and_then(|v| match v { AttrValue::String(s) => Some(VDomNode::text(&s)), _ => Some(VDomNode::empty()) }).unwrap_or(VDomNode::empty())".to_string(),
                        BrakTy::Named(name) if name == "Komponen" => "Box::new(|| VDomNode::empty()) as Box<dyn Fn() -> VDomNode>".to_string(),
                        BrakTy::Pointer(inner) if matches!(inner.as_ref(), BrakTy::Void) => "Box::new(|| VDomNode::empty()) as Box<dyn Fn() -> VDomNode>".to_string(),
                        _ => "props.get(\"{name}\").and_then(|v| if let AttrValue::String(s) = v { Some(s.clone()) } else { None }).unwrap_or_default()".to_string(),
                    }
                };
                let extract = extract.replace("{name}", name);
                code.push_str(&format!(
                    "    let {}: {} = {};\n",
                    snake_name, rust_ty, extract
                ));
            }
        }
        code
    }

    fn scan_props_accesses(&self, block: &BrakBlock) -> Vec<(String, BrakTy)> {
        let mut fields: Vec<(String, BrakTy)> = Vec::new();
        self.scan_block_for_props(block, &mut fields);
        fields
    }

    fn scan_block_for_props(&self, block: &BrakBlock, fields: &mut Vec<(String, BrakTy)>) {
        for stmt in &block.stmts {
            match stmt {
                BrakStmt::Let(let_stmt) => self.scan_expr_for_props(&let_stmt.value, fields),
                BrakStmt::Expr(expr) => self.scan_expr_for_props(expr, fields),
                BrakStmt::Return(Some(expr)) => self.scan_expr_for_props(expr, fields),
                BrakStmt::If(if_stmt) => {
                    self.scan_expr_for_props(&if_stmt.condition, fields);
                    self.scan_block_for_props(&if_stmt.then_block, fields);
                    if let Some(else_block) = &if_stmt.else_block {
                        self.scan_block_for_props(else_block, fields);
                    }
                }
                BrakStmt::Block(b) => self.scan_block_for_props(b, fields),
                _ => {}
            }
        }
    }

    fn scan_expr_for_props(&self, expr: &BrakExpr, fields: &mut Vec<(String, BrakTy)>) {
        match expr {
            BrakExpr::Member(obj, field) => {
                if let BrakExpr::Ident(name) = obj.as_ref() {
                    if name == "props" && !fields.iter().any(|(f, _)| f == field) {
                        fields.push((field.clone(), BrakTy::Any));
                    }
                }
                self.scan_expr_for_props(obj, fields);
            }
            BrakExpr::Call(callee, args) => {
                self.scan_expr_for_props(callee, fields);
                for arg in args {
                    self.scan_expr_for_props(arg, fields);
                }
            }
            BrakExpr::Binary(_, lhs, rhs) => {
                self.scan_expr_for_props(lhs, fields);
                self.scan_expr_for_props(rhs, fields);
            }
            BrakExpr::Unary(_, expr) => self.scan_expr_for_props(expr, fields),
            BrakExpr::Ternary(c, t, e) => {
                self.scan_expr_for_props(c, fields);
                self.scan_expr_for_props(t, fields);
                self.scan_expr_for_props(e, fields);
            }
            BrakExpr::Block(b) => self.scan_block_for_props(b, fields),
            BrakExpr::Array(items) => {
                for item in items {
                    self.scan_expr_for_props(item, fields);
                }
            }
            BrakExpr::Object(pairs) => {
                for (_, v) in pairs {
                    self.scan_expr_for_props(v, fields);
                }
            }
            BrakExpr::Assign(lhs, rhs) => {
                self.scan_expr_for_props(lhs, fields);
                self.scan_expr_for_props(rhs, fields);
            }
            _ => {}
        }
    }

    fn gen_component_body(&self, block: &BrakBlock) -> String {
        self.gen_component_body_stmts(&block.stmts)
    }

    fn gen_component_body_stmts(&self, stmts: &[BrakStmt]) -> String {
        if stmts.len() == 1 {
            if let BrakStmt::Return(Some(expr)) = &stmts[0] {
                return self.gen_expr(expr, 1);
            }
            if let BrakStmt::Expr(expr) = &stmts[0] {
                if matches!(expr, BrakExpr::Call(..)) || matches!(expr, BrakExpr::Block(..)) {
                    if let BrakExpr::Block(inner_block) = expr {
                        let stmts_str: Vec<String> = inner_block
                            .stmts
                            .iter()
                            .map(|s| self.gen_component_stmt(s))
                            .collect();
                        return stmts_str.join("\n");
                    }
                    return self.gen_expr(expr, 1);
                }
            }
        }
        let mut code = String::new();
        let len = stmts.len();
        for (i, stmt) in stmts.iter().enumerate() {
            let is_last = i == len - 1;
            let stmt_str = self.gen_stmt(stmt, 1);
            if is_last {
                code.push_str(&format!("    {}\n", stmt_str));
            } else {
                code.push_str(&format!("    {};\n", stmt_str));
            }
        }
        code
    }

    fn extract_context_lets(&self, block: &BrakBlock) -> (String, BrakBlock) {
        let mut context_code = String::new();
        let mut remaining: Vec<BrakStmt> = Vec::new();
        let stmts = if let Some(inner) = block.stmts.first()
            .filter(|_| block.stmts.len() == 1)
            .and_then(|s| if let BrakStmt::Expr(BrakExpr::Block(b)) = s { Some(b) } else { None })
        {
            &inner.stmts
        } else {
            &block.stmts
        };
        for stmt in stmts {
            let is_context_let = matches!(stmt, BrakStmt::Let(let_stmt)
                if matches!(&let_stmt.value, BrakExpr::Call(callee, _)
                    if matches!(callee.as_ref(), BrakExpr::Ident(name) if name == "konteks")));
            if is_context_let {
                let stmt_str = self.gen_stmt(stmt, 1);
                context_code.push_str(&format!("    {};\n", stmt_str));
            } else {
                remaining.push(stmt.clone());
            }
        }
        (context_code, BrakBlock { stmts: remaining })
    }

    fn gen_component_stmt(&self, stmt: &BrakStmt) -> String {
        match stmt {
            BrakStmt::Return(Some(expr)) => self.gen_expr(expr, 1),
            BrakStmt::Expr(expr) => self.gen_expr(expr, 1),
            _ => self.gen_stmt(stmt, 1).trim_end_matches(';').to_string(),
        }
    }

    fn gen_struct(&self, s: &BrakStructDef) -> String {
        let rust_name = to_pascal_case(&s.name);
        let mut code = format!("#[derive(Debug, Clone, Default)]\npub struct {} {{\n", rust_name);
        for field in &s.fields {
            let field_name = to_snake_case(&field.name);
            code.push_str(&format!(
                "    pub {}: {},\n",
                field_name,
                self.gen_ty(&field.ty)
            ));
        }
        code.push_str("}\n");
        code
    }

    fn gen_enum(&self, e: &BrakEnumDef) -> String {
        let rust_name = to_pascal_case(&e.name);
        let mut code = format!("#[derive(Debug, Clone)]\npub enum {} {{\n", rust_name);
        for variant in &e.variants {
            let variant_name = to_pascal_case(&variant.name);
            if variant.fields.is_empty() {
                code.push_str(&format!("    {},\n", variant_name));
            } else {
                let tys: Vec<String> = variant.fields.iter().map(|t| self.gen_ty(t)).collect();
                code.push_str(&format!("    {}({}),\n", variant_name, tys.join(", ")));
            }
        }
        code.push_str("}\n");
        if let Some(first) = e.variants.first() {
            let first_name = to_pascal_case(&first.name);
            if first.fields.is_empty() {
                code.push_str(&format!("impl Default for {} {{ fn default() -> Self {{ Self::{} }} }}\n", rust_name, first_name));
            } else {
                let defaults: Vec<String> = first.fields.iter().map(|t| {
                    match self.gen_ty(t).as_str() {
                        "f64" => "0.0".to_string(),
                        "i32" => "0".to_string(),
                        "String" => "String::new()".to_string(),
                        "bool" => "false".to_string(),
                        _ => "Default::default()".to_string(),
                    }
                }).collect();
                code.push_str(&format!("impl Default for {} {{ fn default() -> Self {{ Self::{}({}) }} }}\n", rust_name, first_name, defaults.join(", ")));
            }
        }
        code
    }

    fn gen_block(&self, block: &BrakBlock, indent: u32) -> String {
        let mut code = String::new();
        let ind = "    ".repeat(indent as usize);
        let len = block.stmts.len();
        for (i, stmt) in block.stmts.iter().enumerate() {
            let is_last = i == len - 1;
            let stmt_str = self.gen_stmt(stmt, indent);
            let is_let = matches!(stmt, BrakStmt::Let(_));
            if is_last && !is_let {
                code.push_str(&format!("{}{}\n", ind, stmt_str));
            } else {
                code.push_str(&format!("{}{};\n", ind, stmt_str));
            }
        }
        code
    }

    fn gen_stmt(&self, stmt: &BrakStmt, indent: u32) -> String {
        match stmt {
            BrakStmt::Let(let_stmt) => {
                let mutability = if let_stmt.mutable { "mut " } else { "" };
                // Skip destructuring `props = props.props` — gen_component_body_props handles it
                if let_stmt.name == "props" && let_stmt.ty.as_ref().map_or(false, |t| matches!(t, BrakTy::Struct(_))) {
                    if let BrakExpr::Member(obj, _) = &let_stmt.value {
                        if let BrakExpr::Ident(name) = obj.as_ref() {
                            if name == "props" {
                                return format!("let {} = HashMap::<String, AttrValue>::new();", to_snake_case(&let_stmt.name));
                            }
                        }
                    }
                }
                let ty = let_stmt
                    .ty
                    .as_ref()
                    .map(|t| format!(": {}", self.gen_ty(t)))
                    .unwrap_or_default();
                let value_str = match (&let_stmt.ty, &let_stmt.value) {
                    (Some(BrakTy::Fn(params, ret)), BrakExpr::Null) => {
                        let param_strs: Vec<String> = params.iter().enumerate().map(|(i, _)| format!("_{}", i)).collect();
                        let ret_str = self.gen_ty(ret);
                        let body = match ret_str.as_str() {
                            "()" => "{}".to_string(),
                            "String" => "{ String::new() }".to_string(),
                            "bool" => "{ false }".to_string(),
                            "i32" | "i64" | "u32" | "u64" => "{ 0 }".to_string(),
                            "f64" | "f32" => "{ 0.0 }".to_string(),
                            "VDomNode" => "{ VDomNode::empty() }".to_string(),
                            "AttrValue" => "{ AttrValue::String(String::new()) }".to_string(),
                            _ => "{ Default::default() }".to_string(),
                        };
                        if params.is_empty() {
                            format!("|| {}", body)
                        } else {
                            format!("|{}| {}", param_strs.join(", "), body)
                        }
                    }
                    (Some(BrakTy::Named(name)), BrakExpr::Null) if name == "Komponen" => {
                        "Box::new(|| VDomNode::empty())".to_string()
                    }
                    _ => self.gen_expr(&let_stmt.value, indent),
                };
                format!(
                    "let {}{}{} = {}",
                    mutability,
                    to_snake_case(&let_stmt.name),
                    ty,
                    value_str
                )
            }
            BrakStmt::Expr(expr) => {
                self.gen_expr(expr, indent)
            }
            BrakStmt::Return(opt) => match opt {
                Some(expr) => format!("return {}", self.gen_expr(expr, indent)),
                None => "return".to_string(),
            },
            BrakStmt::If(if_stmt) => {
                self.gen_if(if_stmt, indent)
            }
            BrakStmt::While(while_stmt) => {
                let ind = "    ".repeat(indent as usize);
                format!(
                    "while {} {{\n{}{}}}",
                    self.gen_expr(&while_stmt.condition, indent),
                    self.gen_block(&while_stmt.body, indent + 1),
                    ind
                )
            }
            BrakStmt::Block(block) => {
                let ind = "    ".repeat(indent as usize);
                format!(
                    "{{\n{}{}}}",
                    self.gen_block(block, indent + 1),
                    ind
                )
            }
            BrakStmt::Match(m) => {
                format!("match {} {{\n{}}}",
                    self.gen_expr(&m.expr, indent),
                    m.arms.iter().map(|arm| {
                        let pat = self.gen_pattern(&arm.pattern);
                        format!("{} => {}", pat, self.gen_expr(&arm.body, indent))
                    }).collect::<Vec<_>>().join(",\n")
                )
            }
            BrakStmt::Try(t) => {
                let ind = "    ".repeat(indent as usize);
                let catch_ind = "    ".repeat((indent + 1) as usize);
                let catch_body = self.gen_block(&t.catch_block, indent + 2);
                format!(
                    "match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {{\n{}{}}})) {{\n{}Ok(val) => val,\n{}Err({}) => {{\n{}{}}}\n{}}}\n",
                    self.gen_block(&t.try_block, indent + 1),
                    ind,
                    catch_ind,
                    catch_ind,
                    to_snake_case(&t.catch_var),
                    catch_body,
                    ind,
                    ind
                )
            }
            BrakStmt::Throw(e) => {
                format!("panic!({})", self.gen_expr(e, indent))
            }
        }
    }

    fn qualify_enum_variant(&self, name: &str) -> Option<(String, bool)> {
        for (enum_name, enum_def) in &self.enum_defs {
            for variant in &enum_def.variants {
                if variant.name == name {
                    return Some((enum_name.clone(), !variant.fields.is_empty()));
                }
            }
        }
        None
    }

    fn gen_pattern(&self, pattern: &BrakPattern) -> String {
        match pattern {
            BrakPattern::Wildcard => "_".to_string(),
            BrakPattern::Literal(lit) => self.gen_literal(lit),
            BrakPattern::Ident(name) => {
                if let Some((enum_name, has_fields)) = self.qualify_enum_variant(name) {
                    let qualified = format!("{}::{}", to_pascal_case(&enum_name), to_pascal_case(name));
                    if has_fields {
                        format!("{}(..)", qualified)
                    } else {
                        qualified
                    }
                } else {
                    name.clone()
                }
            }
        }
    }

    fn gen_literal(&self, lit: &BrakLiteral) -> String {
        match lit {
            BrakLiteral::Number(n) => {
                // Always generate f64 format (Rakit Angka maps to f64)
                if *n == (*n as i64) as f64 {
                    format!("{}.0", *n as i64)
                } else {
                    format!("{}", n)
                }
            }
            BrakLiteral::String(s) => format!("\"{}\"", escape_string(s)),
            BrakLiteral::Bool(b) => b.to_string(),
            BrakLiteral::Null => "Default::default()".to_string(),
        }
    }

    fn gen_if(&self, if_stmt: &BrakIf, indent: u32) -> String {
        let ind = "    ".repeat(indent as usize);
        let cond_str = self.gen_expr(&if_stmt.condition, indent);
        let is_comparison = cond_str.contains("==") || cond_str.contains("!=") || cond_str.contains("<") || cond_str.contains(">") || cond_str.contains("&&") || cond_str.contains("||");
        let is_literal = cond_str == "true" || cond_str == "false";
        let needs_truthy = !is_comparison && !is_literal
            && (cond_str.contains("props.get(")
                || cond_str.starts_with(|c: char| c.is_ascii_lowercase()));
        let wrapped_cond = if needs_truthy {
            format!("rakit_truthy(&{})", cond_str)
        } else {
            cond_str
        };
        let then_content = self.gen_block(&if_stmt.then_block, indent + 1);
        let mut code = format!(
            "if {} {{\n{}{}}}",
            wrapped_cond,
            then_content,
            ind
        );

        if let Some(else_block) = &if_stmt.else_block {
            if else_block.stmts.len() == 1 {
                if let BrakStmt::If(inner) = &else_block.stmts[0] {
                    code.push_str(&format!(" else {}\n", self.gen_if(inner, indent)));
                    return code;
                }
            }
            let else_content = self.gen_block(else_block, indent + 1);
            code.push_str(&format!(
                " else {{\n{}{}}}",
                else_content,
                ind
            ));
        }

        code
    }

    fn gen_expr(&self, expr: &BrakExpr, indent: u32) -> String {
        match expr {
            BrakExpr::Number(n) => {
                if *n == (*n as i64) as f64 {
                    format!("{}.0", *n as i64)
                } else {
                    format!("{}", n)
                }
            }
            BrakExpr::String(s) => format!("\"{}\".to_string()", escape_string(s)),
            BrakExpr::Bool(b) => b.to_string(),
            BrakExpr::Null => "VDomNode::empty()".to_string(),
            BrakExpr::Ident(name) => {
                if name == "h" {
                    name.clone()
                } else if name == "fragment" {
                    name.clone()
                } else if name == "render" || name == "tampilkan" {
                    "render".to_string()
                } else if name == "cetak" {
                    "println!".to_string()
                } else if name == "Math" {
                    "math_shim".to_string()
                } else if name == "window" || name == "jendela" {
                    "web_sys::window().unwrap_throw()".to_string()
                } else if name == "document" || name == "dokumen" {
                    "web_sys::window().unwrap_throw().document().unwrap_throw()".to_string()
                } else if name == "localStorage" || name == "penyimpanan_lokal" || name == "local_storage" {
                    "web_sys::window().unwrap_throw().local_storage().unwrap_throw().unwrap_throw()".to_string()
                } else if name == "navigator" {
                    "web_sys::window().unwrap_throw().navigator()".to_string()
                } else if name == "history" || name == "riwayat" {
                    "web_sys::window().unwrap_throw().history().unwrap_throw()".to_string()
                } else if name == "fetch" || name == "ambil" {
                    "web_sys::window().unwrap_throw().fetch_with_str".to_string()
                } else if name == "konteks" {
                    "use_context".to_string()
                } else if name == "sekarang" {
                    "js_sys::Date::now".to_string()
                } else if name == "batal" {
                    "None".to_string()
                } else if name == "benar" {
                    "true".to_string()
                } else if name == "salah" {
                    "false".to_string()
                } else if name == "berhenti" {
                    "return".to_string()
                } else {
                    to_snake_case(name)
                }
            }
            BrakExpr::Binary(op, lhs, rhs) => {
                let is_string_add = matches!(op, BrakBinaryOp::Add) && matches!(lhs.as_ref(), BrakExpr::String(_));
                let op_str = match op {
                    BrakBinaryOp::Add => " + ",
                    BrakBinaryOp::Sub => " - ",
                    BrakBinaryOp::Mul => " * ",
                    BrakBinaryOp::Div => " / ",
                    BrakBinaryOp::Mod => " % ",
                    BrakBinaryOp::And => " && ",
                    BrakBinaryOp::Or => " || ",
                    BrakBinaryOp::Eq => " == ",
                    BrakBinaryOp::Ne => " != ",
                    BrakBinaryOp::Lt => " < ",
                    BrakBinaryOp::Gt => " > ",
                    BrakBinaryOp::Le => " <= ",
                    BrakBinaryOp::Ge => " >= ",
                    BrakBinaryOp::Concat => " + &",
                    BrakBinaryOp::NullCoalescing => ".unwrap_or(",
                };
                if matches!(op, BrakBinaryOp::NullCoalescing) {
                    match rhs.as_ref() {
                        BrakExpr::String(s) => {
                            format!(
                                "rakit_nullish_str({}, {})",
                                self.gen_expr(lhs, indent),
                                self.gen_expr(rhs, indent)
                            )
                        }
                        BrakExpr::Bool(b) => {
                            // After prop extraction, values are already typed bools
                            format!(
                                "({} || {})",
                                self.gen_expr(lhs, indent),
                                b
                            )
                        }
                        BrakExpr::Number(n) => {
                            // After prop extraction, values are already typed f64
                            let n_str = if *n == (*n as i64) as f64 { format!("{}.0", *n as i64) } else { format!("{}", n) };
                            format!(
                                "({}).to_string().parse::<f64>().unwrap_or({})",
                                self.gen_expr(lhs, indent),
                                n_str
                            )
                        }
                        _ => {
                            format!(
                                "rakit_nullish({}, {})",
                                self.gen_expr(lhs, indent),
                                self.gen_expr(rhs, indent)
                            )
                        }
                    }
                } else if matches!(op, BrakBinaryOp::Concat) {
                    format!(
                        "format!(\"{{}}{{}}\", {}, {})",
                        self.gen_expr(lhs, indent),
                        self.gen_expr(rhs, indent)
                    )
                } else if is_string_add {
                    format!(
                        "format!(\"{{}}{{}}\", {}, {})",
                        self.gen_expr(lhs, indent),
                        self.gen_expr(rhs, indent)
                    )
                } else {
                    format!(
                        "({}{}{})",
                        self.gen_expr(lhs, indent),
                        op_str,
                        self.gen_expr(rhs, indent)
                    )
                }
            }
            BrakExpr::Unary(op, rhs) => {
                match op {
                    BrakUnaryOp::Not => {
                        // Check if the inner expression is an ident that might be a HashMap
                        if let BrakExpr::Ident(name) = rhs.as_ref() {
                            if name == "pengguna" || name == "tema" || name == "sesi" {
                                return format!("{}.is_empty()", self.gen_expr(rhs, indent));
                            }
                        }
                        format!("!{}", self.gen_expr(rhs, indent))
                    }
                    BrakUnaryOp::Neg => format!("-{}", self.gen_expr(rhs, indent)),
                }
            }
            BrakExpr::Assign(lhs, rhs) => {
                if let BrakExpr::Member(obj, field) = lhs.as_ref() {
                    if field == "pathname" || field == "hash" || field == "href" || field == "search" {
                        format!("{}.set_{}({})", self.gen_expr(obj, indent), field, self.gen_expr(rhs, indent))
                    } else {
                        format!(
                            "{} = {}",
                            self.gen_expr(lhs, indent),
                            self.gen_expr(rhs, indent)
                        )
                    }
                } else {
                    format!(
                        "{} = {}",
                        self.gen_expr(lhs, indent),
                        self.gen_expr(rhs, indent)
                    )
                }
            }
            BrakExpr::Call(callee, args) => {
                let callee_raw = self.gen_expr(callee, indent);
                // Wrap member field callees in parens for field-as-function calls e.g. (router.navigasi)(args)
                let field_map_keys = ["panjang", "petakan", "filter", "cari", "kurangi", "urutkan", "gabung", "salin", "potong", "dorong", "mengandung", "acak", "buat", "amat", "lepas", "saat_ini", "gaya", "cegah_default", "hentikan_propagasi", "add_event_listener", "addEventListener", "remove_event_listener", "removeEventListener", "get_item", "getItem", "set_item", "setItem", "location", "pathname"];
                let callee_str = if let BrakExpr::Member(obj, field) = callee.as_ref() {
                    // Only wrap if field is a known Rust method name (methods can't be taken as values)
                    let known_methods = ["get", "contains", "starts_with", "ends_with", "split", "join", "push", "pop", "len", "is_empty", "remove", "insert", "clone", "to_string", "clear", "into_iter", "iter", "keys", "values", "map", "filter", "find", "fold", "collect", "as_ref", "deref", "rev", "reverse", "unwrap", "unwrap_or", "unwrap_or_default", "and_then", "or_else", "cmp", "eq", "ne", "baru", "tutup", "berhenti", "location", "pathname", "add_event_listener", "addEventListener", "remove_event_listener", "removeEventListener", "get_item", "getItem", "set_item", "setItem", "on_message", "onmessage", "prevent_default", "preventDefault", "stop_propagation", "stopPropagation", "style", "observe", "disconnect", "random", "new", "sort", "as_slice"];
                    if known_methods.contains(&field.as_str()) || field_map_keys.contains(&field.as_str()) {
                        callee_raw
                    } else if let BrakExpr::Ident(_) = obj.as_ref() {
                        format!("({})", callee_raw)
                    } else {
                        callee_raw
                    }
                } else {
                    callee_raw
                };
                let args_strs: Vec<String> = if callee_str.ends_with(".into_iter().map") && !args.is_empty() {
                    if let BrakExpr::Block(block) = &args[0] {
                        let mut closure_params = Vec::new();
                        let mut remaining_stmts = Vec::new();
                        for stmt in &block.stmts {
                            if let BrakStmt::Let(let_stmt) = stmt {
                                if matches!(&let_stmt.value, BrakExpr::Null) {
                                    closure_params.push(to_snake_case(&let_stmt.name));
                                    continue;
                                }
                            }
                            remaining_stmts.push(stmt.clone());
                        }
                        if !closure_params.is_empty() {
                            // Mark map params as dynamic items so member access uses .get()
                            let mut dyn_items = self.map_dynamic_items.borrow_mut();
                            for p in &closure_params {
                                dyn_items.insert(p.clone());
                            }
                            drop(dyn_items);
                            let body_block = BrakBlock { stmts: remaining_stmts };
                            let body_str = self.gen_block(&body_block, indent + 1);
                            let ind = "    ".repeat(indent as usize);
                            let mut dyn_items = self.map_dynamic_items.borrow_mut();
                            for p in &closure_params {
                                dyn_items.remove(p);
                            }
                            // Shadow AttrValue item with HashMap for .get() field access
                            let shadow_lets: String = closure_params.iter()
                                .map(|p| format!("{}let {}: HashMap<String, AttrValue> = HashMap::new();\n", ind, p))
                                .collect();
                            vec![format!("|{}| {{\n{}{}{}}}", closure_params.join(", "), shadow_lets, body_str, ind)]
                        } else {
                            args.iter().map(|a| self.gen_expr(a, indent)).collect()
                        }
                    } else {
                        args.iter().map(|a| self.gen_expr(a, indent)).collect()
                    }
                } else {
                    args.iter().map(|a| self.gen_expr(a, indent)).collect()
                };

                if callee_str == "h" && args.len() >= 2 {
                    self.gen_jsx_call(&args, indent)
                } else if callee_str == "fragment" {
                    if args.len() >= 3 {
                        self.gen_children(&args[2], indent)
                    } else {
                        "VDomNode::fragment(vec![])".to_string()
                    }
                } else if callee_str == "render" && args.len() >= 2 {
                    // render(<App />, document.getElementById("root")) -> just return the app
                    self.gen_expr(&args[0], indent)
                } else if callee_str == "println!" || callee_str == "cetak" {
                    if args_strs.len() > 1 {
                        let fmt = args_strs[0].trim_matches('"');
                        let placeholders: Vec<String> = args_strs[1..].iter().map(|_| "{}".to_string()).collect();
                        format!("println!(\"{} {}\", {})", fmt, placeholders.join(" "), args_strs[1..].join(", "))
                    } else {
                        let arg = &args_strs[0];
                        if arg.starts_with('"') && arg.ends_with('"') && !arg[1..arg.len()-1].contains('\"') {
                            format!("println!({})", arg)
                        } else {
                            format!("println!(\"{{}}\", {})", arg)
                        }
                    }
                } else if callee_str == "stringifyJSON" {
                    if args_strs.is_empty() { "String::new()".to_string() } else { format!("format!(\"{:?}\", {})", args_strs[0], args_strs[0]) }
                } else if callee_str == "parseJSON" {
                    if args_strs.is_empty() { "String::new()".to_string() } else { format!("{}", args_strs[0]) }
                } else if callee_str == "console" {
                    format!("web_sys::console::log_1(&wasm_bindgen::JsValue::from({}))", args_strs.join(", "))
                } else if callee_str == "Math" {
                    if !args_strs.is_empty() { args_strs[0].clone() } else { "0.0".to_string() }
                } else if callee_str == "ingat" && !args_strs.is_empty() {
                    // Wrap first argument (callback block) with || closure syntax
                    let wrapped_cb = if args_strs[0].starts_with('{') {
                        format!("|| {}", args_strs[0])
                    } else {
                        format!("|| {{ {} }}", args_strs[0])
                    };
                    let rest_args = if args_strs.len() > 1 {
                        args_strs[1..].join(", ")
                    } else {
                        String::new()
                    };
                    if rest_args.is_empty() {
                        format!("ingat({})", wrapped_cb)
                    } else {
                        format!("ingat({}, {})", wrapped_cb, rest_args)
                    }
                } else if callee_str == "use_context" && !args_strs.is_empty() {
                    let type_name = to_pascal_case(&args_strs[0].trim_matches('"'));
                    format!("use_context::<{}>()", type_name)
                } else {
                    let call_str = if callee_str.ends_with(".starts_with") || callee_str.ends_with(".contains") || callee_str.ends_with(".ends_with") {
                        let as_str_args: Vec<String> = args_strs.iter().map(|a| format!("{}.as_str()", a)).collect();
                        format!("{}({})", callee_str, as_str_args.join(", "))
                    } else if (callee_str.contains("timer.baru") || callee_str.contains("web_socket.baru")) && args.len() >= 2 {
                        let cb = if args_strs[0].starts_with('{') {
                            format!("|| {}", args_strs[0])
                        } else {
                            args_strs[0].clone()
                        };
                        let rest = args_strs[1..].join(", ");
                        format!("{}({}, {})", callee_str, cb, rest)
                    } else {
                        let raw_call = format!("{}({})", callee_str, args_strs.join(", "));
                        if callee_str.ends_with(".map") {
                            format!("VDomNode::fragment({}.collect::<Vec<VDomNode>>())", raw_call)
                        } else {
                            raw_call
                        }
                    };
                    if callee_str.starts_with("__state") || callee_str.starts_with("__set_state") || callee_str.starts_with("__memo") {
                        call_str
                    } else if is_component_ref(&callee_str) {
                        format!("VDomNode::component(\"{}\", std::collections::HashMap::new(), None)", callee_str)
                    } else {
                        call_str
                    }
                }
            }
            BrakExpr::Member(obj, field) => {
                let obj_str = self.gen_expr(obj, indent);
                let field_snake = to_snake_case(field);
                // If accessing props.field, generate inline HashMap extraction
                if let BrakExpr::Ident(name) = obj.as_ref() {
                    if name == "props" {
                        // Check if it's a method name (used as callee) vs a prop name
                        let methods = ["get", "contains_key", "insert", "remove", "len", "is_empty", "iter", "keys", "values", "clone", "clear", "entry"];
                        if methods.contains(&field.as_str()) {
                            return format!("props.{}", to_snake_case(field));
                        }
                        // Use extracted variable if available
                        if self.extracted_props.contains(&field_snake) {
                            return field_snake.clone();
                        }
                        return format!("props.get(\"{}\").and_then(|v| if let AttrValue::String(s) = v {{ Some(s.clone()) }} else {{ None }}).unwrap_or_default()", field);
                    }
                    // Enum variant access: Status.Aktif → Status::Aktif
                    if let Some(enum_def) = self.enum_defs.get(name) {
                        if enum_def.variants.iter().any(|v| v.name == *field) {
                            return format!("{}::{}", to_pascal_case(name), to_pascal_case(field));
                        }
                    }
                }
                let field_map: HashMap<&str, &str> = [
                    ("panjang", "len() as f64"),
                    ("length", "len() as f64"),
                    ("petakan", "into_iter().map"),
                    ("filter", "into_iter().filter"),
                    ("cari", "into_iter().find"),
                    ("kurangi", "into_iter().fold"),
                    ("urutkan", "sort"),
                    ("gabung", "join"),
                    ("salin", "clone"),
                    ("potong", "as_slice"),
                    ("dorong", "push"),
                    ("mengandung", "contains"),
                    ("acak", "random"),
                    ("buat", "new"),
                    ("amat", "observe"),
                    ("lepas", "disconnect"),
                    ("saat_ini", "as_ref().map(|r| r.deref())"),
                    ("gaya", "style()"),
                    ("cegah_default", "prevent_default"),
                    ("hentikan_propagasi", "stop_propagation"),
                    ("add_event_listener", "add_event_listener_with_callback"),
                    ("addEventListener", "add_event_listener_with_callback"),
                    ("remove_event_listener", "remove_event_listener_with_callback"),
                    ("removeEventListener", "remove_event_listener_with_callback"),
                    ("get_item", "get_item"),
                    ("set_item", "set_item"),
                    ("location", "location()"),
                    ("pathname", "pathname()"),
                ].iter().map(|(k, v)| (*k, *v)).collect();

                if let Some(rust_fn) = field_map.get(field.as_str()) {
                    format!("{}.{}", obj_str, rust_fn)
                } else if field == "toString" || field == "to_string" {
                    format!("rakit_to_string(&{})", obj_str)
                } else if field == "startsWith" || field == "mulaiDengan" {
                    format!("{}.starts_with", obj_str)
                } else if field == "endsWith" || field == "akhiriDengan" {
                    format!("{}.ends_with", obj_str)
                } else if field == "includes" || field == "mengandung" {
                    format!("{}.contains", obj_str)
                } else {
                    if let BrakExpr::Ident(name) = obj.as_ref() {
                        if self.map_dynamic_items.borrow().contains(name) {
                            format!("{}.get(\"{}\").and_then(|v| if let AttrValue::String(s) = v {{ Some(s.clone()) }} else {{ None }}).unwrap_or_default()", obj_str, field_snake)
                        } else {
                            format!("{}.{}", obj_str, field_snake)
                        }
                    } else {
                        format!("{}.{}", obj_str, field_snake)
                    }
                }
            }
            BrakExpr::Index(obj, index) => {
                format!(
                    "{}[{}]",
                    self.gen_expr(obj, indent),
                    self.gen_expr(index, indent)
                )
            }
            BrakExpr::Array(items) => {
                let items_strs: Vec<String> =
                    items.iter().map(|i| self.gen_expr(i, indent)).collect();
                if items_strs.is_empty() {
                    "Vec::<AttrValue>::new()".to_string()
                } else {
                    format!("vec![{}]", items_strs.join(", "))
                }
            }
            BrakExpr::StructInit(name, fields) => {
                if let Some(struct_def) = self.struct_defs.get(name) {
                    let field_strs: Vec<String> = fields
                        .iter()
                        .map(|(k, v)| {
                            let snake = to_snake_case(k);
                            let val_str = self.gen_expr(v, indent);
                            format!("{}: {}", snake, val_str)
                        })
                        .collect();
                    let struct_name = to_pascal_case(name);
                    if fields.is_empty() {
                        format!("{} {{}}", struct_name)
                    } else {
                        format!("{} {{ {} }}", struct_name, field_strs.join(", "))
                    }
                } else {
                    let fields_strs: Vec<String> = fields
                        .iter()
                        .map(|(k, v)| {
                            format!(
                                "(\"{}\", {})",
                                k,
                                self.gen_attr_value(v, indent)
                            )
                        })
                        .collect();
                    format!("vec![{}]", fields_strs.join(", "))
                }
            }
            BrakExpr::Block(block) => {
                let ind = "    ".repeat(indent as usize);
                format!(
                    "{{\n{}{}}}",
                    self.gen_block(block, indent + 1),
                    ind
                )
            }
            BrakExpr::Ternary(cond, then_expr, else_expr) => {
                let cond_str = self.gen_expr(cond, indent);
                let is_comparison = cond_str.contains("==") || cond_str.contains("!=") || cond_str.contains("<") || cond_str.contains(">") || cond_str.contains("&&") || cond_str.contains("||");
                let is_literal = cond_str == "true" || cond_str == "false";
                let needs_truthy = !is_comparison && !is_literal
                    && (cond_str.contains("props.get(")
                        || cond_str.starts_with(|c: char| c.is_ascii_lowercase()));
                let wrapped_cond = if needs_truthy {
                    format!("rakit_truthy(&{})", cond_str)
                } else {
                    cond_str
                };
                format!(
                    "if {} {{ {} }} else {{ {} }}",
                    wrapped_cond,
                    self.gen_expr(then_expr, indent),
                    self.gen_expr(else_expr, indent)
                )
            }
            BrakExpr::ArrowFn(params, body) => {
                let params_str = if params.is_empty() {
                    "||".to_string()
                } else {
                    format!("|{}|", params.join(", "))
                };
                format!("{} {}", params_str, self.gen_expr(body, indent))
            }
            BrakExpr::Object(fields) => {
                if fields.is_empty() {
                    "HashMap::<String, AttrValue>::new()".to_string()
                } else {
                    let fields_strs: Vec<String> = fields.iter()
                        .map(|(k, v)| {
                            if k.is_empty() {
                                self.gen_expr(v, indent)
                            } else {
                                format!("(\"{}\".to_string(), {})", k, self.gen_attr_str_value(v, indent))
                            }
                        })
                        .collect();
                    format!("[{}].into_iter().collect::<HashMap<String, AttrValue>>()", fields_strs.join(", "))
                }
            }
            BrakExpr::Spread(inner) => {
                self.gen_expr(inner, indent)
            }
            BrakExpr::Template(parts) => {
                if parts.is_empty() {
                    "String::new()".to_string()
                } else if parts.len() == 1 {
                    self.gen_expr(&parts[0], indent)
                } else {
                    let parts_strs: Vec<String> = parts.iter()
                        .map(|p| self.gen_expr(p, indent))
                        .collect();
                    format!("format!(\"{}\", {})", 
                        parts_strs.iter().map(|_| "{}").collect::<Vec<_>>().join(""),
                        parts_strs.join(", "))
                }
            }
            BrakExpr::Match(m) => {
                let arms_strs: Vec<String> = m.arms.iter()
                    .map(|arm| {
                        let pat = self.gen_pattern(&arm.pattern);
                        format!("{} => {}", pat, self.gen_expr(&arm.body, indent))
                    })
                    .collect();
                format!("match {} {{ {} }}", self.gen_expr(&m.expr, indent), arms_strs.join(", "))
            }
        }
    }

    fn gen_attr_value(&self, expr: &BrakExpr, indent: u32) -> String {
        match expr {
            BrakExpr::String(s) => format!("AttrValue::String(\"{}\".into())", escape_string(s)),
            BrakExpr::Number(n) => {
                let n_str = n.to_string();
                if n_str.contains('.') || n_str.contains('e') || n_str.contains('E') {
                    format!("AttrValue::Number({})", n_str)
                } else {
                    format!("AttrValue::Number({}.0)", n_str)
                }
            }
            BrakExpr::Bool(b) => format!("AttrValue::Bool({})", b),
            BrakExpr::Null => "AttrValue::Expression(String::new())".to_string(),
            BrakExpr::Ident(name) => {
                let snake = to_snake_case(name);
                if snake == "true" || snake == "false" {
                    format!("AttrValue::Bool({})", snake)
                } else {
                    format!("AttrValue::String(rakit_debug(&{}))", snake)
                }
            }
            _ => format!("AttrValue::String(rakit_debug(&{}))", self.gen_expr(expr, indent)),
        }
    }

    fn gen_jsx_call(&self, args: &[BrakExpr], indent: u32) -> String {
        if args.len() < 2 {
            let args_strs: Vec<String> =
                args.iter().map(|a| self.gen_expr(a, indent)).collect();
            return format!("h({})", args_strs.join(", "));
        }

        // Tag must be &str for VDomNode::element, so keep raw string literal if static
        let tag = if let BrakExpr::String(s) = &args[0] {
            format!("\"{}\"", escape_string(s))
        } else {
            format!("{}.as_str()", self.gen_expr(&args[0], indent))
        };
        let attrs = &args[1];
        let children = if args.len() > 2 { &args[2] } else { &BrakExpr::Array(vec![]) };
        let has_children = match children {
            BrakExpr::Array(arr) => !arr.is_empty(),
            _ => true,
        };

        // Check if this is a component or an HTML element
        let tag_trimmed = tag.trim_matches('"');
        let is_component_like = !tag_trimmed.starts_with(|c: char| c.is_ascii_lowercase());

        if is_component_like {
            let attrs_str = self.gen_component_attrs(attrs, indent);
            if has_children {
                let children_str = self.gen_children(children, indent);
                format!("VDomNode::component_with_children({}, {}, {}, None)", tag, attrs_str, children_str)
            } else {
                format!("VDomNode::component({}, {}, None)", tag, attrs_str)
            }
        } else {
            let has_events = self.has_event_attrs(attrs);
            if has_events {
                let (attr_map_str, events_str) = self.gen_attrs_with_events(attrs, indent);
                let children_str = self.gen_children(children, indent);
                format!("VDomNode::element_with_attrs({}, {}, {}, {}, None)", tag, attr_map_str, events_str, children_str)
            } else {
                let (attr_vec_str, _) = self.gen_attrs_simple(attrs, indent);
                let children_str = self.gen_children(children, indent);
                format!("VDomNode::element({}, {}, {})", tag, attr_vec_str, children_str)
            }
        }
    }

    fn gen_component_attrs(&self, expr: &BrakExpr, indent: u32) -> String {
        match expr {
            BrakExpr::StructInit(_, fields) => {
                if fields.is_empty() {
                    "HashMap::new()".to_string()
                } else {
                    let fields_strs: Vec<String> = fields.iter()
                        .map(|(k, v)| {
                            let val_str = self.gen_attr_value(v, indent);
                            format!("(\"{}\".to_string(), {})", k, val_str)
                        })
                        .collect();
                    format!("[{}].into_iter().collect::<HashMap<String, AttrValue>>()", fields_strs.join(", "))
                }
            }
            _ => "HashMap::new()".to_string(),
        }
    }

    fn has_event_attrs(&self, expr: &BrakExpr) -> bool {
        if let BrakExpr::StructInit(_, fields) = expr {
            fields.iter().any(|(k, _)| is_event_attr(k))
        } else {
            false
        }
    }

    fn gen_attrs_simple(&self, expr: &BrakExpr, indent: u32) -> (String, bool) {
        match expr {
            BrakExpr::StructInit(_, fields) => {
                let attr_pairs: Vec<String> = fields.iter()
                    .filter(|(k, _)| !is_event_attr(k))
                    .map(|(k, v)| {
                        let val = self.gen_attr_simple_value(v, indent);
                        format!("(\"{}\", {})", k, val)
                    })
                    .collect();
                if attr_pairs.is_empty() {
                    ("vec![]".to_string(), false)
                } else {
                    (format!("vec![{}]", attr_pairs.join(", ")), true)
                }
            }
            _ => ("vec![]".to_string(), false),
        }
    }

    fn gen_attr_simple_value(&self, expr: &BrakExpr, indent: u32) -> String {
        match expr {
            BrakExpr::String(s) => format!("\"{}\"", escape_string(s)),
            BrakExpr::Number(n) => format!("\"{}\"", n),
            BrakExpr::Bool(b) => format!("\"{}\"", b),
            BrakExpr::Ident(name) => format!("&*{}", to_snake_case(name)),
            _ => {
                let ex = self.gen_expr(expr, indent);
                format!("&*{}", ex)
            }
        }
    }

    fn gen_attrs_with_events(&self, expr: &BrakExpr, indent: u32) -> (String, String) {
        match expr {
            BrakExpr::StructInit(_, fields) => {
                let mut attr_entries = Vec::new();
                let mut event_entries = Vec::new();
                for (k, v) in fields {
                    if is_event_attr(k) {
                        let event_name = k.trim_start_matches("on").to_lowercase();
                        event_entries.push(format!(
                            "(EventType::from(\"{}\"), {})",
                            event_name,
                            self.gen_event_handler(k, v, indent)
                        ));
                    } else {
                        attr_entries.push(format!(
                            "(\"{}\".to_string(), {})",
                            k,
                            self.gen_attr_str_value(v, indent)
                        ));
                    }
                }
                let attrs_str = if attr_entries.is_empty() {
                    "HashMap::new()".to_string()
                } else {
                    format!("[{}].into_iter().collect::<HashMap<String, AttrValue>>()",
                        attr_entries.join(", "))
                };
                let events_str = if event_entries.is_empty() {
                    "HashMap::new()".to_string()
                } else {
                    format!("[{}].into_iter().collect::<HashMap<EventType, u64>>()",
                        event_entries.join(", "))
                };
                (attrs_str, events_str)
            }
            _ => ("HashMap::new()".to_string(), "HashMap::new()".to_string()),
        }
    }

    fn gen_event_handler(&self, attr_name: &str, expr: &BrakExpr, indent: u32) -> String {
        let handler = self.gen_expr(expr, indent);
        // Register a global handler and return its ID (u64)
        // For now, use a placeholder ID since dynamic registration is complex
        // TODO: implement proper handler registration
        format!("0u64 /* {} */", handler)
    }

    fn gen_attr_str_value(&self, expr: &BrakExpr, indent: u32) -> String {
        match expr {
            BrakExpr::String(s) => format!("AttrValue::String(\"{}\".into())", escape_string(s)),
            BrakExpr::Number(n) => format!("AttrValue::Number({})", n),
            BrakExpr::Bool(b) => format!("AttrValue::Bool({})", b),
            BrakExpr::Null => "AttrValue::Expression(String::new())".to_string(),
            BrakExpr::Ident(name) => {
                let snake = to_snake_case(name);
                format!("AttrValue::String(rakit_debug(&{}))", snake)
            }
            BrakExpr::Binary(op, lhs, rhs) if matches!(op, BrakBinaryOp::Concat) => {
                format!("AttrValue::String(format!(\"{{}}{{}}\", {}, {}))", self.gen_expr(lhs, indent), self.gen_expr(rhs, indent))
            }
            BrakExpr::Ternary(cond, then_expr, else_expr) => {
                format!(
                    "if {} {{ {} }} else {{ {} }}",
                    self.gen_expr(cond, indent),
                    self.gen_attr_str_value(then_expr, indent),
                    self.gen_attr_str_value(else_expr, indent)
                )
            }
            _ => format!("AttrValue::String(rakit_debug(&{}))", self.gen_expr(expr, indent)),
        }
    }

    fn gen_children(&self, expr: &BrakExpr, indent: u32) -> String {
        match expr {
            BrakExpr::Array(items) => {
                if items.is_empty() {
                    "vec![]".to_string()
                } else {
                    let items_strs: Vec<String> = items
                        .iter()
                        .map(|i| self.gen_child_item(i, indent))
                        .collect();
                    format!("vec![{}]", items_strs.join(", "))
                }
            }
            BrakExpr::Call(callee, _) => {
                let callee_str = self.gen_expr(callee, indent);
                if callee_str == "h" || callee_str.starts_with("VDomNode::") {
                    format!("vec![{}]", self.gen_expr(expr, indent))
                } else {
                    format!("vec![{}]", self.gen_child_item(expr, indent))
                }
            }
            _ => {
                format!("vec![{}]", self.gen_child_item(expr, indent))
            }
        }
    }

    fn gen_children_for_component(&self, expr: &BrakExpr, indent: u32) -> String {
        match expr {
            BrakExpr::Array(items) => {
                if items.is_empty() {
                    "HashMap::new()".to_string()
                } else {
                    let items_strs: Vec<String> = items
                        .iter()
                        .map(|i| self.gen_child_item(i, indent))
                        .collect();
                    let children_str = format!("vec![{}]", items_strs.join(", "));
                    format!("[(\"children\".into(), AttrValue::String(\"{}\".into()))].into_iter().collect()", children_str)
                }
            }
            _ => "HashMap::new()".to_string(),
        }
    }

    fn gen_child_item(&self, expr: &BrakExpr, indent: u32) -> String {
        match expr {
            BrakExpr::String(s) => {
                format!("VDomNode::text(\"{}\")", escape_string(s))
            }
            BrakExpr::Number(n) => {
                format!("VDomNode::text(&format!(\"{}\", {}))", "{}", n)
            }
            BrakExpr::Null => {
                "VDomNode::empty()".to_string()
            }
            BrakExpr::Bool(b) => {
                self.gen_expr(expr, indent)
            }
            BrakExpr::Call(callee, _) => {
                let callee_str = self.gen_expr(callee, indent);
                if callee_str == "h" || callee_str.starts_with("VDomNode::") || callee_str == "fragment" {
                    self.gen_expr(expr, indent)
                } else if is_component_ref(&callee_str) {
                    let args_strs: Vec<String> = if let BrakExpr::Call(_, ref args) = expr {
                        args.iter().map(|a| self.gen_expr(a, indent)).collect()
                    } else {
                        vec![]
                    };
                    if !args_strs.is_empty() {
                        format!("VDomNode::component(\"{}\", {}, None)", 
                            callee_str.trim_matches('"'),
                            args_strs[0])
                    } else {
                        format!("VDomNode::component(\"{}\", HashMap::new(), None)", 
                            callee_str.trim_matches('"'))
                    }
                } else {
                    let result = self.gen_expr(expr, indent);
                    if result.starts_with("VDomNode::") || result.starts_with("rakit_as_node") || result.starts_with("rakit_nullish") {
                        result
                    } else {
                        format!("rakit_as_node(&{})", result)
                    }
                }
            }
            BrakExpr::Ternary(cond, then_expr, else_expr) => {
                let then_str = self.gen_child_item(then_expr, indent);
                let else_str = self.gen_child_item(else_expr, indent);
                let needs_wrap = |s: &str| !s.starts_with("VDomNode::") && !s.starts_with("rakit_") && !s.starts_with("if ") && !s.starts_with("{") && !s.starts_with("|| ") && s != "Default::default()";
                let then_wrapped = if needs_wrap(&then_str) { format!("rakit_as_node(&{})", then_str) } else { then_str };
                let else_wrapped = if needs_wrap(&else_str) { format!("rakit_as_node(&{})", else_str) } else { else_str };
                let cond_str = self.gen_expr(cond, indent);
                let is_comparison = cond_str.contains("==") || cond_str.contains("!=") || cond_str.contains("<") || cond_str.contains(">") || cond_str.contains("&&") || cond_str.contains("||");
                let is_literal = cond_str == "true" || cond_str == "false";
                let needs_truthy = !is_comparison && !is_literal
                    && (cond_str.contains("props.get(")
                        || cond_str.starts_with(|c: char| c.is_ascii_lowercase()));
                let wrapped_cond = if needs_truthy {
                    format!("rakit_truthy(&{})", cond_str)
                } else {
                    cond_str
                };
                format!(
                    "if {} {{ {} }} else {{ {} }}",
                    wrapped_cond,
                    then_wrapped,
                    else_wrapped
                )
            }
            BrakExpr::Ident(name) => {
                let snake = to_snake_case(name);
                if snake == "null" || snake == "batal" {
                    "Default::default()".to_string()
                } else {
                    format!("rakit_as_node(&{})", snake)
                }
            }
            BrakExpr::Binary(op, lhs, rhs) if matches!(op, BrakBinaryOp::Concat) => {
                format!("VDomNode::text(&({}))", self.gen_expr(expr, indent))
            }
            BrakExpr::Array(items) => {
                let items_strs: Vec<String> = items.iter()
                    .map(|i| self.gen_child_item(i, indent))
                    .collect();
                format!("VDomNode::fragment(vec![{}])", items_strs.join(", "))
            }
            _ => {
                let expr_str = self.gen_expr(expr, indent);
                let needs_wrap = !expr_str.starts_with("VDomNode::") && !expr_str.starts_with("rakit_") && !expr_str.starts_with("if ") && !expr_str.starts_with("{") && !expr_str.starts_with("|| ") && expr_str != "Default::default()";
                if needs_wrap {
                    format!("rakit_as_node(&{})", expr_str)
                } else {
                    expr_str
                }
            }
    }
    }

    fn gen_ty(&self, ty: &BrakTy) -> String {
        match ty {
            BrakTy::Int(bits) => format!("i{}", bits),
            BrakTy::UInt(bits) => format!("u{}", bits),
            BrakTy::Float(bits) => format!("f{}", bits),
            BrakTy::Bool => "bool".to_string(),
            BrakTy::U8 => "u8".to_string(),
            BrakTy::Void => "()".to_string(),
            BrakTy::Any => "AttrValue".to_string(),
            BrakTy::Pointer(inner) => {
                match inner.as_ref() {
                    BrakTy::U8 => "String".to_string(),
                    BrakTy::Void => "VDomNode".to_string(),
                    t => format!("Box<{}>", self.gen_ty(t)),
                }
            }
            BrakTy::Array(inner) => format!("Vec<{}>", self.gen_ty(inner)),
            BrakTy::Optional(inner) => format!("Option<{}>", self.gen_ty(inner)),
            BrakTy::Fn(params, ret) => {
                let params_strs: Vec<String> =
                    params.iter().map(|p| self.gen_ty(p)).collect();
                format!("fn({}) -> {}", params_strs.join(", "), self.gen_ty(ret))
            }
            BrakTy::Struct(fields) => {
                if fields.is_empty() {
                    "HashMap<String, AttrValue>".to_string()
                } else {
                    // Inline struct types are not valid in Rust; use HashMap instead
                    // since all component props pass through the attr system at runtime
                    "HashMap<String, AttrValue>".to_string()
                }
            }
            BrakTy::Enum(variants) => {
                // Enum/union types from Rakit type aliases (e.g. "primer" | "sekunder")
                // Use String at runtime since variants are string literals
                "String".to_string()
            }
            BrakTy::Named(name) => {
                match name.as_str() {
                    "Node" => "VDomNode".to_string(),
                    "String" => "String".to_string(),
                    "Int" => "i64".to_string(),
                    "Float" => "f64".to_string(),
                    "Bool" => "bool".to_string(),
                    "ReactNode" => "VDomNode".to_string(),
                    "Komponen" => "Box<dyn Fn() -> VDomNode>".to_string(),
                    "Record" => "HashMap<String, AttrValue>".to_string(),
                    "Promise" => "String".to_string(),
                    "Hasil" => "String".to_string(),
                    "Error" => "String".to_string(),
                    "Any" => "AttrValue".to_string(),
                    "Partial" => "HashMap<String, AttrValue>".to_string(),
                    "Omit" => "HashMap<String, AttrValue>".to_string(),
                    "Array" => "Vec<AttrValue>".to_string(),
                    _ => {
                        // Check if it's a known user-defined struct/enum
                        if self.struct_defs.contains_key(name) || self.enum_defs.contains_key(name) {
                            to_pascal_case(name)
                        } else {
                            // Unknown named types default to String for WASM interop
                            "String".to_string()
                        }
                    }
                }
            }
        }
    }

    fn gen_entry_point(&self) -> String {
        let component_registrations: Vec<String> = self
            .component_names
            .iter()
            .map(|name| {
                let rust_name = to_snake_case(name);
                format!(
                    r#"    engine.register_component("{}", |props| {}(props));"#,
                    name, rust_name
                )
            })
            .collect();

        let root: &str = if self.component_names.is_empty() {
            "RootComponent"
        } else {
            // Prefer "App" as root (the main entry point component)
            self.component_names.iter().find(|n| *n == "App")
                .map(|s| s.as_str())
                .unwrap_or(&self.component_names[0])
        };

        let registrations = component_registrations.join("\n");

        format!(
            r#"#[cfg(target_arch = "wasm32")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn start_app() {{
    let backend = rakit_backend_web::WebBackend::new("root");
    let mut engine = rakit_ui::RakitApp::new(backend, "{}");
{}
    engine.init(&rakit_ui::AppConfig::new("{}", "com.rakit", "0.1.0")).unwrap();
    engine.mount(&rakit_ui::WindowConfig::new("{}", 800, 600)).unwrap();
    std::mem::forget(engine);
}}
"#,
            root, registrations, self.app_name, self.app_name
        )
    }
}

fn is_component_fn(f: &BrakFnDef) -> bool {
    f.is_component || f.return_ty
        .as_ref()
        .map(|t| matches!(t, BrakTy::Named(n) if n == "Node"))
        .or_else(|| {
            f.return_ty.as_ref().map(|t| match t {
                BrakTy::Pointer(inner) => matches!(inner.as_ref(), BrakTy::Void),
                _ => false,
            })
        })
        .unwrap_or(false)
}

fn is_component_ref(name: &str) -> bool {
    name.chars().next().map(|c| c.is_uppercase()).unwrap_or(false)
}

fn is_event_attr(name: &str) -> bool {
    name.starts_with("on") && name.len() > 2
        && name[2..].chars().next().map(|c| c.is_uppercase()).unwrap_or(false)
}

fn is_render_call(expr: &BrakExpr) -> bool {
    if let BrakExpr::Call(callee, _) = expr {
        if let BrakExpr::Ident(name) = callee.as_ref() {
            return name == "h" || name == "render" || name == "fragment";
        }
    }
    false
}

fn has_render_return(body: &BrakBlock) -> bool {
    if body.stmts.len() == 1 {
        if let BrakStmt::Return(Some(expr)) = &body.stmts[0] {
            return is_render_call(expr) || true;
        }
        if let BrakStmt::Expr(expr) = &body.stmts[0] {
            return is_render_call(expr) || matches!(expr, BrakExpr::Block(_));
        }
    }
    false
}

fn extract_body_expr(body: &BrakBlock) -> Option<&BrakExpr> {
    if body.stmts.len() == 1 {
        match &body.stmts[0] {
            BrakStmt::Return(Some(expr)) => Some(expr),
            BrakStmt::Expr(expr) => {
                if matches!(expr, BrakExpr::Call(..)) || matches!(expr, BrakExpr::Block(..)) {
                    Some(expr)
                } else {
                    None
                }
            }
            _ => None,
        }
    } else {
        None
    }
}

fn to_snake_case(name: &str) -> String {
    let mut result = String::new();
    let mut prev_upper = false;
    for (i, ch) in name.chars().enumerate() {
        if ch.is_uppercase() {
            if i > 0 && !prev_upper {
                result.push('_');
            }
            result.push(ch.to_ascii_lowercase());
            prev_upper = true;
        } else {
            result.push(ch);
            prev_upper = false;
        }
    }
    result
}

fn to_pascal_case(name: &str) -> String {
    let mut result = String::new();
    let mut capitalize = true;
    for ch in name.chars() {
        if ch == '_' {
            capitalize = true;
        } else if capitalize {
            result.push(ch.to_ascii_uppercase());
            capitalize = false;
        } else {
            result.push(ch);
        }
    }
    result
}

fn escape_string(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
        .replace('\t', "\\t")
}
