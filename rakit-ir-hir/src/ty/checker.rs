use std::collections::HashMap;
use rakit_core::Diagnostic;
use crate::hir::*;
use super::*;

pub struct TypeChecker {
    scope: Vec<Scope>,
    struct_defs: HashMap<String, StructType>,
    enum_defs: HashMap<String, EnumType>,
    fn_sigs: HashMap<String, FnType>,
    pub diagnostics: Vec<Diagnostic>,
    errors: usize,
}

struct Scope {
    bindings: HashMap<String, TypeInfo>,
    depth: usize,
}

impl TypeChecker {
    pub fn new() -> Self {
        let mut globals = HashMap::new();
        Self::register_builtins(&mut globals);
        TypeChecker {
            scope: vec![Scope { bindings: globals, depth: 0 }],
            struct_defs: HashMap::new(),
            enum_defs: HashMap::new(),
            fn_sigs: HashMap::new(),
            diagnostics: Vec::new(),
            errors: 0,
        }
    }

    fn register_builtins(scope: &mut HashMap<String, TypeInfo>) {
        let blank_obj = || TypeInfo::Struct(StructType {
            name: String::new(), fields: vec![], generics: vec![],
        });
        let field = |name: &str, ty: TypeInfo| FieldType { name: name.into(), ty };
        let struct_ty = |name: &str, flds: Vec<FieldType>| TypeInfo::Struct(StructType {
            name: name.into(), fields: flds, generics: vec![],
        });
        let fn_ty = |params: Vec<TypeInfo>, ret: TypeInfo| TypeInfo::Fn(FnType::new(params, ret));
        let opt = |inner: TypeInfo| TypeInfo::Optional(Box::new(inner));

        // ── Primitives ──────────────────────────────────────────────
        scope.insert("String".into(), TypeInfo::String);
        scope.insert("Number".into(), TypeInfo::F64);
        scope.insert("Boolean".into(), TypeInfo::Bool);
        scope.insert("BigInt".into(), TypeInfo::I64);
        scope.insert("Symbol".into(), TypeInfo::String);
        // Nullish / error
        scope.insert("batal".into(), opt(TypeInfo::Infer));
        scope.insert("benar".into(), TypeInfo::Bool);
        scope.insert("salah".into(), TypeInfo::Bool);

        // ── Error ───────────────────────────────────────────────────
        scope.insert("Error".into(), struct_ty("Error", vec![
            field("message", TypeInfo::String),
            field("name", TypeInfo::String),
            field("stack", opt(TypeInfo::String)),
        ]));

        // ── Promise ─────────────────────────────────────────────────
        let promise_ctor = |inner: TypeInfo| -> TypeInfo { struct_ty("Promise", vec![
            field("then", fn_ty(vec![fn_ty(vec![inner.clone()], TypeInfo::Infer)], struct_ty("Promise", vec![]))),
            field("catch", fn_ty(vec![fn_ty(vec![TypeInfo::Infer], TypeInfo::Infer)], struct_ty("Promise", vec![]))),
            field("finally", fn_ty(vec![fn_ty(vec![], TypeInfo::Void)], struct_ty("Promise", vec![]))),
        ])};
        scope.insert("Promise".into(), promise_ctor(TypeInfo::Infer));

        // ── Array / Object / Map / Set ──────────────────────────────
        scope.insert("Array".into(), TypeInfo::Array(Box::new(TypeInfo::Infer)));
        scope.insert("Object".into(), struct_ty("Object", vec![
            field("keys", fn_ty(vec![TypeInfo::Infer], TypeInfo::Array(Box::new(TypeInfo::String)))),
            field("values", fn_ty(vec![TypeInfo::Infer], TypeInfo::Array(Box::new(TypeInfo::Infer)))),
            field("entries", fn_ty(vec![TypeInfo::Infer], TypeInfo::Array(Box::new(TypeInfo::Array(Box::new(TypeInfo::Infer)))))),
            field("assign", fn_ty(vec![TypeInfo::Infer, TypeInfo::Infer], TypeInfo::Infer)),
            field("create", fn_ty(vec![TypeInfo::Infer], TypeInfo::Infer)),
            field("defineProperty", fn_ty(vec![TypeInfo::Infer, TypeInfo::String, TypeInfo::Infer], TypeInfo::Infer)),
            field("freeze", fn_ty(vec![TypeInfo::Infer], TypeInfo::Infer)),
            field("seal", fn_ty(vec![TypeInfo::Infer], TypeInfo::Infer)),
            field("hasOwnProperty", fn_ty(vec![TypeInfo::String], TypeInfo::Bool)),
            field("getPrototypeOf", fn_ty(vec![TypeInfo::Infer], TypeInfo::Infer)),
            field("setPrototypeOf", fn_ty(vec![TypeInfo::Infer, TypeInfo::Infer], TypeInfo::Infer)),
            field("is", fn_ty(vec![TypeInfo::Infer, TypeInfo::Infer], TypeInfo::Bool)),
            field("preventExtensions", fn_ty(vec![TypeInfo::Infer], TypeInfo::Infer)),
            field("isExtensible", fn_ty(vec![TypeInfo::Infer], TypeInfo::Bool)),
            field("isSealed", fn_ty(vec![TypeInfo::Infer], TypeInfo::Bool)),
            field("isFrozen", fn_ty(vec![TypeInfo::Infer], TypeInfo::Bool)),
        ]));
        scope.insert("RegExp".into(), TypeInfo::String);
        scope.insert("Date".into(), struct_ty("Date", vec![
            // Static methods
            field("now", fn_ty(vec![], TypeInfo::I64)),
            field("parse", fn_ty(vec![TypeInfo::String], TypeInfo::I64)),
            field("UTC", fn_ty(vec![TypeInfo::I32, TypeInfo::I32], TypeInfo::I64)),
            // Instance methods
            field("getFullYear", fn_ty(vec![], TypeInfo::I32)),
            field("getMonth", fn_ty(vec![], TypeInfo::I32)),
            field("getDate", fn_ty(vec![], TypeInfo::I32)),
            field("getDay", fn_ty(vec![], TypeInfo::I32)),
            field("getHours", fn_ty(vec![], TypeInfo::I32)),
            field("getMinutes", fn_ty(vec![], TypeInfo::I32)),
            field("getSeconds", fn_ty(vec![], TypeInfo::I32)),
            field("getMilliseconds", fn_ty(vec![], TypeInfo::I32)),
            field("getTime", fn_ty(vec![], TypeInfo::I64)),
            field("setFullYear", fn_ty(vec![TypeInfo::I32], TypeInfo::I64)),
            field("setMonth", fn_ty(vec![TypeInfo::I32], TypeInfo::I64)),
            field("setDate", fn_ty(vec![TypeInfo::I32], TypeInfo::I64)),
            field("setHours", fn_ty(vec![TypeInfo::I32], TypeInfo::I64)),
            field("setMinutes", fn_ty(vec![TypeInfo::I32], TypeInfo::I64)),
            field("setSeconds", fn_ty(vec![TypeInfo::I32], TypeInfo::I64)),
            field("setMilliseconds", fn_ty(vec![TypeInfo::I32], TypeInfo::I64)),
            field("setTime", fn_ty(vec![TypeInfo::I64], TypeInfo::I64)),
            field("toISOString", fn_ty(vec![], TypeInfo::String)),
            field("toJSON", fn_ty(vec![], TypeInfo::String)),
            field("toLocaleDateString", fn_ty(vec![], TypeInfo::String)),
            field("toLocaleTimeString", fn_ty(vec![], TypeInfo::String)),
            field("toString", fn_ty(vec![], TypeInfo::String)),
        ]));
        scope.insert("Map".into(), blank_obj());
        scope.insert("Set".into(), blank_obj());
        scope.insert("WeakMap".into(), blank_obj());
        scope.insert("WeakSet".into(), blank_obj());

        // ── Math ────────────────────────────────────────────────────
        scope.insert("Math".into(), struct_ty("Math", vec![
            field("PI", TypeInfo::F64), field("E", TypeInfo::F64),
            field("LN2", TypeInfo::F64), field("LN10", TypeInfo::F64),
            field("LOG2E", TypeInfo::F64), field("LOG10E", TypeInfo::F64),
            field("SQRT1_2", TypeInfo::F64), field("SQRT2", TypeInfo::F64),
            field("abs", fn_ty(vec![TypeInfo::F64], TypeInfo::F64)),
            field("floor", fn_ty(vec![TypeInfo::F64], TypeInfo::F64)),
            field("ceil", fn_ty(vec![TypeInfo::F64], TypeInfo::F64)),
            field("round", fn_ty(vec![TypeInfo::F64], TypeInfo::F64)),
            field("acak", fn_ty(vec![], TypeInfo::F64)), // random
            field("random", fn_ty(vec![], TypeInfo::F64)),
            field("pow", fn_ty(vec![TypeInfo::F64, TypeInfo::F64], TypeInfo::F64)),
            field("sqrt", fn_ty(vec![TypeInfo::F64], TypeInfo::F64)),
            field("cbrt", fn_ty(vec![TypeInfo::F64], TypeInfo::F64)),
            field("sin", fn_ty(vec![TypeInfo::F64], TypeInfo::F64)),
            field("cos", fn_ty(vec![TypeInfo::F64], TypeInfo::F64)),
            field("tan", fn_ty(vec![TypeInfo::F64], TypeInfo::F64)),
            field("asin", fn_ty(vec![TypeInfo::F64], TypeInfo::F64)),
            field("acos", fn_ty(vec![TypeInfo::F64], TypeInfo::F64)),
            field("atan", fn_ty(vec![TypeInfo::F64], TypeInfo::F64)),
            field("atan2", fn_ty(vec![TypeInfo::F64, TypeInfo::F64], TypeInfo::F64)),
            field("exp", fn_ty(vec![TypeInfo::F64], TypeInfo::F64)),
            field("log", fn_ty(vec![TypeInfo::F64], TypeInfo::F64)),
            field("log10", fn_ty(vec![TypeInfo::F64], TypeInfo::F64)),
            field("log2", fn_ty(vec![TypeInfo::F64], TypeInfo::F64)),
            field("trunc", fn_ty(vec![TypeInfo::F64], TypeInfo::F64)),
            field("sign", fn_ty(vec![TypeInfo::F64], TypeInfo::F64)),
            field("max", fn_ty(vec![TypeInfo::Array(Box::new(TypeInfo::F64))], TypeInfo::F64)),
            field("min", fn_ty(vec![TypeInfo::Array(Box::new(TypeInfo::F64))], TypeInfo::F64)),
            field("clz32", fn_ty(vec![TypeInfo::I32], TypeInfo::I32)),
            field("imul", fn_ty(vec![TypeInfo::I32, TypeInfo::I32], TypeInfo::I32)),
            field("fround", fn_ty(vec![TypeInfo::F64], TypeInfo::F32)),
            field("hypot", fn_ty(vec![TypeInfo::Array(Box::new(TypeInfo::F64))], TypeInfo::F64)),
        ]));

        // ── JSON ────────────────────────────────────────────────────
        scope.insert("JSON".into(), struct_ty("JSON", vec![
            field("parse", fn_ty(vec![TypeInfo::String], TypeInfo::Infer)),
            field("stringify", fn_ty(vec![TypeInfo::Infer], TypeInfo::String)),
        ]));

        // ── Atomics / Reflect / Proxy ────────────────────────────────
        scope.insert("Atomics".into(), blank_obj());
        scope.insert("Reflect".into(), blank_obj());
        scope.insert("Proxy".into(), blank_obj());

        // ── console ─────────────────────────────────────────────────
        scope.insert("console".into(), struct_ty("console", vec![
            field("log", fn_ty(vec![TypeInfo::Infer], TypeInfo::Void)),
            field("warn", fn_ty(vec![TypeInfo::Infer], TypeInfo::Void)),
            field("error", fn_ty(vec![TypeInfo::Infer], TypeInfo::Void)),
            field("info", fn_ty(vec![TypeInfo::Infer], TypeInfo::Void)),
            field("debug", fn_ty(vec![TypeInfo::Infer], TypeInfo::Void)),
            field("table", fn_ty(vec![TypeInfo::Infer], TypeInfo::Void)),
            field("group", fn_ty(vec![TypeInfo::String], TypeInfo::Void)),
            field("groupEnd", fn_ty(vec![], TypeInfo::Void)),
            field("time", fn_ty(vec![TypeInfo::String], TypeInfo::Void)),
            field("timeEnd", fn_ty(vec![TypeInfo::String], TypeInfo::Void)),
            field("count", fn_ty(vec![TypeInfo::String], TypeInfo::Void)),
            field("assert", fn_ty(vec![TypeInfo::Bool, TypeInfo::Infer], TypeInfo::Void)),
            field("clear", fn_ty(vec![], TypeInfo::Void)),
            field("dir", fn_ty(vec![TypeInfo::Infer], TypeInfo::Void)),
            field("trace", fn_ty(vec![], TypeInfo::Void)),
        ]));

        // ── window ──────────────────────────────────────────────────
        let location_struct = struct_ty("Location", vec![
            field("href", TypeInfo::String), field("pathname", TypeInfo::String),
            field("search", TypeInfo::String), field("hash", TypeInfo::String),
            field("host", TypeInfo::String), field("hostname", TypeInfo::String),
            field("port", TypeInfo::String), field("protocol", TypeInfo::String),
            field("origin", TypeInfo::String),
            field("assign", fn_ty(vec![TypeInfo::String], TypeInfo::Void)),
            field("replace", fn_ty(vec![TypeInfo::String], TypeInfo::Void)),
            field("reload", fn_ty(vec![], TypeInfo::Void)),
        ]);
        let history_struct = struct_ty("History", vec![
            field("length", TypeInfo::I32),
            field("scrollRestoration", TypeInfo::String),
            field("state", TypeInfo::Infer),
            field("back", fn_ty(vec![], TypeInfo::Void)),
            field("forward", fn_ty(vec![], TypeInfo::Void)),
            field("go", fn_ty(vec![TypeInfo::I32], TypeInfo::Void)),
            field("pushState", fn_ty(vec![TypeInfo::Infer, TypeInfo::String, opt(TypeInfo::String)], TypeInfo::Void)),
            field("replaceState", fn_ty(vec![TypeInfo::Infer, TypeInfo::String, opt(TypeInfo::String)], TypeInfo::Void)),
        ]);
        let navigator_struct = struct_ty("Navigator", vec![
            field("userAgent", TypeInfo::String), field("platform", TypeInfo::String),
            field("language", TypeInfo::String), field("languages", TypeInfo::Array(Box::new(TypeInfo::String))),
            field("online", TypeInfo::Bool), field("cookieEnabled", TypeInfo::Bool),
            field("geolocation", blank_obj()), field("clipboard", blank_obj()),
            field("storage", blank_obj()), field("serviceWorker", blank_obj()),
        ]);
        scope.insert("window".into(), struct_ty("Window", vec![
            field("location", location_struct.clone()),
            field("history", history_struct.clone()),
            field("navigator", navigator_struct.clone()),
            field("document", blank_obj()),
            field("console", struct_ty("console", vec![
                field("log", fn_ty(vec![TypeInfo::Infer], TypeInfo::Void)),
            ])),
            field("localStorage", struct_ty("Storage", vec![
                field("getItem", fn_ty(vec![TypeInfo::String], opt(TypeInfo::String))),
                field("setItem", fn_ty(vec![TypeInfo::String, TypeInfo::String], TypeInfo::Void)),
                field("removeItem", fn_ty(vec![TypeInfo::String], TypeInfo::Void)),
                field("clear", fn_ty(vec![], TypeInfo::Void)),
                field("key", fn_ty(vec![TypeInfo::I32], opt(TypeInfo::String))),
                field("length", TypeInfo::I32),
            ])),
            field("sessionStorage", struct_ty("Storage", vec![
                field("getItem", fn_ty(vec![TypeInfo::String], opt(TypeInfo::String))),
                field("setItem", fn_ty(vec![TypeInfo::String, TypeInfo::String], TypeInfo::Void)),
                field("removeItem", fn_ty(vec![TypeInfo::String], TypeInfo::Void)),
                field("clear", fn_ty(vec![], TypeInfo::Void)),
                field("key", fn_ty(vec![TypeInfo::I32], opt(TypeInfo::String))),
                field("length", TypeInfo::I32),
            ])),
            field("Math", struct_ty("Math", vec![
                field("acak", fn_ty(vec![], TypeInfo::F64)),
                field("random", fn_ty(vec![], TypeInfo::F64)),
            ])),
            field("JSON", struct_ty("JSON", vec![
                field("parse", fn_ty(vec![TypeInfo::String], TypeInfo::Infer)),
                field("stringify", fn_ty(vec![TypeInfo::Infer], TypeInfo::String)),
            ])),
            field("Date", struct_ty("Date", vec![
                field("now", fn_ty(vec![], TypeInfo::I64)),
                field("parse", fn_ty(vec![TypeInfo::String], TypeInfo::I64)),
            ])),
            field("fetch", fn_ty(vec![TypeInfo::String, opt(blank_obj())], struct_ty("Promise", vec![]))),
            field("setTimeout", fn_ty(vec![fn_ty(vec![], TypeInfo::Void), TypeInfo::I32], TypeInfo::I32)),
            field("setInterval", fn_ty(vec![fn_ty(vec![], TypeInfo::Void), TypeInfo::I32], TypeInfo::I32)),
            field("clearTimeout", fn_ty(vec![TypeInfo::I32], TypeInfo::Void)),
            field("clearInterval", fn_ty(vec![TypeInfo::I32], TypeInfo::Void)),
            field("requestAnimationFrame", fn_ty(vec![fn_ty(vec![TypeInfo::F64], TypeInfo::Void)], TypeInfo::I32)),
            field("cancelAnimationFrame", fn_ty(vec![TypeInfo::I32], TypeInfo::Void)),
            field("alert", fn_ty(vec![TypeInfo::String], TypeInfo::Void)),
            field("confirm", fn_ty(vec![TypeInfo::String], TypeInfo::Bool)),
            field("prompt", fn_ty(vec![TypeInfo::String, TypeInfo::String], TypeInfo::String)),
            field("open", fn_ty(vec![TypeInfo::String, TypeInfo::String, TypeInfo::String], blank_obj())),
            field("close", fn_ty(vec![], TypeInfo::Void)),
            field("innerWidth", TypeInfo::I32), field("innerHeight", TypeInfo::I32),
            field("outerWidth", TypeInfo::I32), field("outerHeight", TypeInfo::I32),
            field("scrollX", TypeInfo::F64), field("scrollY", TypeInfo::F64),
            field("pageXOffset", TypeInfo::F64), field("pageYOffset", TypeInfo::F64),
            field("scrollTo", fn_ty(vec![TypeInfo::I32, TypeInfo::I32], TypeInfo::Void)),
            field("scrollBy", fn_ty(vec![TypeInfo::I32, TypeInfo::I32], TypeInfo::Void)),
            field("addEventListener", fn_ty(vec![TypeInfo::String, fn_ty(vec![TypeInfo::Infer], TypeInfo::Void)], TypeInfo::Void)),
            field("removeEventListener", fn_ty(vec![TypeInfo::String, fn_ty(vec![TypeInfo::Infer], TypeInfo::Void)], TypeInfo::Void)),
            field("dispatchEvent", fn_ty(vec![TypeInfo::Infer], TypeInfo::Bool)),
            field("name", TypeInfo::String), field("opener", blank_obj()),
            field("parent", blank_obj()), field("top", blank_obj()),
            field("frameElement", blank_obj()), field("frames", blank_obj()),
            field("self", blank_obj()), field("window", blank_obj()),
            field("screen", blank_obj()), field("screenLeft", TypeInfo::I32),
            field("screenTop", TypeInfo::I32), field("screenX", TypeInfo::I32),
            field("screenY", TypeInfo::I32),
            field("performance", blank_obj()),
            field("crypto", blank_obj()),
        ]));

        // ── document ────────────────────────────────────────────────
        scope.insert("document".into(), struct_ty("Document", vec![
            field("getElementById", fn_ty(vec![TypeInfo::String], TypeInfo::Node)),
            field("querySelector", fn_ty(vec![TypeInfo::String], opt(TypeInfo::Node))),
            field("querySelectorAll", fn_ty(vec![TypeInfo::String], TypeInfo::Array(Box::new(TypeInfo::Node)))),
            field("createElement", fn_ty(vec![TypeInfo::String], TypeInfo::Node)),
            field("createTextNode", fn_ty(vec![TypeInfo::String], TypeInfo::Node)),
            field("createDocumentFragment", fn_ty(vec![], TypeInfo::Node)),
            field("body", TypeInfo::Node), field("head", TypeInfo::Node),
            field("documentElement", TypeInfo::Node),
            field("title", TypeInfo::String), field("URL", TypeInfo::String),
            field("domain", TypeInfo::String), field("referrer", TypeInfo::String),
            field("cookie", TypeInfo::String),
            field("addEventListener", fn_ty(vec![TypeInfo::String, fn_ty(vec![TypeInfo::Infer], TypeInfo::Void)], TypeInfo::Void)),
            field("removeEventListener", fn_ty(vec![TypeInfo::String, fn_ty(vec![TypeInfo::Infer], TypeInfo::Void)], TypeInfo::Void)),
            field("dispatchEvent", fn_ty(vec![TypeInfo::Infer], TypeInfo::Bool)),
            field("hasFocus", fn_ty(vec![], TypeInfo::Bool)),
            field("execCommand", fn_ty(vec![TypeInfo::String], TypeInfo::Bool)),
        ]));

        // ── Browser API globals ─────────────────────────────────────
        scope.insert("history".into(), history_struct.clone());
        scope.insert("location".into(), location_struct.clone());
        scope.insert("navigator".into(), navigator_struct.clone());
        scope.insert("localStorage".into(), struct_ty("Storage", vec![
            field("getItem", fn_ty(vec![TypeInfo::String], opt(TypeInfo::String))),
            field("setItem", fn_ty(vec![TypeInfo::String, TypeInfo::String], TypeInfo::Void)),
            field("removeItem", fn_ty(vec![TypeInfo::String], TypeInfo::Void)),
            field("clear", fn_ty(vec![], TypeInfo::Void)),
            field("key", fn_ty(vec![TypeInfo::I32], opt(TypeInfo::String))),
            field("length", TypeInfo::I32),
        ]));
        scope.insert("sessionStorage".into(), struct_ty("Storage", vec![
            field("getItem", fn_ty(vec![TypeInfo::String], opt(TypeInfo::String))),
            field("setItem", fn_ty(vec![TypeInfo::String, TypeInfo::String], TypeInfo::Void)),
            field("removeItem", fn_ty(vec![TypeInfo::String], TypeInfo::Void)),
            field("clear", fn_ty(vec![], TypeInfo::Void)),
            field("key", fn_ty(vec![TypeInfo::I32], opt(TypeInfo::String))),
            field("length", TypeInfo::I32),
        ]));

        scope.insert("fetch".into(), fn_ty(
            vec![TypeInfo::String, opt(blank_obj())],
            struct_ty("Promise", vec![]),
        ));
        scope.insert("WebSocket".into(), fn_ty(
            vec![TypeInfo::String],
            struct_ty("WebSocket", vec![
                field("send", fn_ty(vec![TypeInfo::String], TypeInfo::Void)),
                field("close", fn_ty(vec![], TypeInfo::Void)),
                field("readyState", TypeInfo::I32),
                field("bufferedAmount", TypeInfo::I32),
                field("extensions", TypeInfo::String),
                field("protocol", TypeInfo::String),
                field("url", TypeInfo::String),
                field("onopen", fn_ty(vec![TypeInfo::Infer], TypeInfo::Void)),
                field("onclose", fn_ty(vec![TypeInfo::Infer], TypeInfo::Void)),
                field("onmessage", fn_ty(vec![TypeInfo::Infer], TypeInfo::Void)),
                field("onerror", fn_ty(vec![TypeInfo::Infer], TypeInfo::Void)),
            ]),
        ));
        scope.insert("Worker".into(), fn_ty(
            vec![TypeInfo::String],
            struct_ty("Worker", vec![
                field("postMessage", fn_ty(vec![TypeInfo::Infer], TypeInfo::Void)),
                field("terminate", fn_ty(vec![], TypeInfo::Void)),
                field("onmessage", fn_ty(vec![TypeInfo::Infer], TypeInfo::Void)),
                field("onerror", fn_ty(vec![TypeInfo::Infer], TypeInfo::Void)),
            ]),
        ));
        scope.insert("Headers".into(), fn_ty(vec![],
            struct_ty("Headers", vec![
                field("append", fn_ty(vec![TypeInfo::String, TypeInfo::String], TypeInfo::Void)),
                field("delete", fn_ty(vec![TypeInfo::String], TypeInfo::Void)),
                field("get", fn_ty(vec![TypeInfo::String], opt(TypeInfo::String))),
                field("has", fn_ty(vec![TypeInfo::String], TypeInfo::Bool)),
                field("set", fn_ty(vec![TypeInfo::String, TypeInfo::String], TypeInfo::Void)),
                field("forEach", fn_ty(vec![fn_ty(vec![TypeInfo::String, TypeInfo::String], TypeInfo::Void)], TypeInfo::Void)),
            ]),
        ));
        scope.insert("Request".into(), fn_ty(
            vec![TypeInfo::String, opt(blank_obj())],
            struct_ty("Request", vec![
                field("url", TypeInfo::String),
                field("method", TypeInfo::String),
                field("headers", blank_obj()),
                field("body", blank_obj()),
                field("json", fn_ty(vec![], struct_ty("Promise", vec![]))),
                field("text", fn_ty(vec![], struct_ty("Promise", vec![]))),
            ]),
        ));
        scope.insert("Response".into(), struct_ty("Response", vec![
            field("status", TypeInfo::I32),
            field("ok", TypeInfo::Bool),
            field("statusText", TypeInfo::String),
            field("headers", blank_obj()),
            field("body", blank_obj()),
            field("url", TypeInfo::String),
            field("redirected", TypeInfo::Bool),
            field("type", TypeInfo::String),
            field("json", fn_ty(vec![], struct_ty("Promise", vec![]))),
            field("text", fn_ty(vec![], struct_ty("Promise", vec![]))),
            field("blob", fn_ty(vec![], struct_ty("Promise", vec![]))),
            field("arrayBuffer", fn_ty(vec![], struct_ty("Promise", vec![]))),
            field("formData", fn_ty(vec![], struct_ty("Promise", vec![]))),
            field("clone", fn_ty(vec![], blank_obj())),
        ]));
        scope.insert("URL".into(), fn_ty(
            vec![TypeInfo::String, opt(TypeInfo::String)],
            struct_ty("URL", vec![
                field("href", TypeInfo::String),
                field("origin", TypeInfo::String),
                field("protocol", TypeInfo::String),
                field("host", TypeInfo::String),
                field("hostname", TypeInfo::String),
                field("port", TypeInfo::String),
                field("pathname", TypeInfo::String),
                field("search", TypeInfo::String),
                field("hash", TypeInfo::String),
                field("searchParams", blank_obj()),
                field("toString", fn_ty(vec![], TypeInfo::String)),
            ]),
        ));
        scope.insert("URLSearchParams".into(), fn_ty(
            vec![opt(TypeInfo::String)],
            struct_ty("URLSearchParams", vec![
                field("append", fn_ty(vec![TypeInfo::String, TypeInfo::String], TypeInfo::Void)),
                field("delete", fn_ty(vec![TypeInfo::String], TypeInfo::Void)),
                field("get", fn_ty(vec![TypeInfo::String], opt(TypeInfo::String))),
                field("getAll", fn_ty(vec![TypeInfo::String], TypeInfo::Array(Box::new(TypeInfo::String)))),
                field("has", fn_ty(vec![TypeInfo::String], TypeInfo::Bool)),
                field("set", fn_ty(vec![TypeInfo::String, TypeInfo::String], TypeInfo::Void)),
                field("sort", fn_ty(vec![], TypeInfo::Void)),
                field("forEach", fn_ty(vec![fn_ty(vec![TypeInfo::String, TypeInfo::String], TypeInfo::Void)], TypeInfo::Void)),
                field("toString", fn_ty(vec![], TypeInfo::String)),
            ]),
        ));
        scope.insert("FileReader".into(), blank_obj());
        scope.insert("FormData".into(), fn_ty(vec![],
            struct_ty("FormData", vec![
                field("append", fn_ty(vec![TypeInfo::String, TypeInfo::Infer], TypeInfo::Void)),
                field("delete", fn_ty(vec![TypeInfo::String], TypeInfo::Void)),
                field("get", fn_ty(vec![TypeInfo::String], TypeInfo::Infer)),
                field("has", fn_ty(vec![TypeInfo::String], TypeInfo::Bool)),
                field("set", fn_ty(vec![TypeInfo::String, TypeInfo::Infer], TypeInfo::Void)),
                field("forEach", fn_ty(vec![fn_ty(vec![TypeInfo::Infer, TypeInfo::String], TypeInfo::Void)], TypeInfo::Void)),
            ]),
        ));
        scope.insert("AbortController".into(), struct_ty("AbortController", vec![
            field("abort", fn_ty(vec![], TypeInfo::Void)),
            field("signal", blank_obj()),
        ]));
        scope.insert("AbortSignal".into(), blank_obj());

        // Observers
        scope.insert("IntersectionObserver".into(), fn_ty(
            vec![fn_ty(vec![TypeInfo::Array(Box::new(blank_obj())), blank_obj()], TypeInfo::Void)],
            struct_ty("IntersectionObserver", vec![
                field("observe", fn_ty(vec![TypeInfo::Node], TypeInfo::Void)),
                field("unobserve", fn_ty(vec![TypeInfo::Node], TypeInfo::Void)),
                field("disconnect", fn_ty(vec![], TypeInfo::Void)),
            ]),
        ));
        scope.insert("MutationObserver".into(), fn_ty(
            vec![fn_ty(vec![TypeInfo::Array(Box::new(blank_obj())), blank_obj()], TypeInfo::Void)],
            struct_ty("MutationObserver", vec![
                field("observe", fn_ty(vec![TypeInfo::Node, blank_obj()], TypeInfo::Void)),
                field("disconnect", fn_ty(vec![], TypeInfo::Void)),
            ]),
        ));
        scope.insert("ResizeObserver".into(), fn_ty(
            vec![fn_ty(vec![TypeInfo::Array(Box::new(blank_obj())), blank_obj()], TypeInfo::Void)],
            struct_ty("ResizeObserver", vec![
                field("observe", fn_ty(vec![TypeInfo::Node], TypeInfo::Void)),
                field("unobserve", fn_ty(vec![TypeInfo::Node], TypeInfo::Void)),
                field("disconnect", fn_ty(vec![], TypeInfo::Void)),
            ]),
        ));

        // ── Timer functions ──────────────────────────────────────────
        scope.insert("Timer".into(), fn_ty(
            vec![fn_ty(vec![], TypeInfo::Void), TypeInfo::I32],
            struct_ty("Timer", vec![
                field("clear", fn_ty(vec![], TypeInfo::Void)),
                field("refresh", fn_ty(vec![], TypeInfo::Void)),
            ]),
        ));
        scope.insert("setTimeout".into(), fn_ty(
            vec![fn_ty(vec![], TypeInfo::Void), TypeInfo::I32],
            TypeInfo::I32,
        ));
        scope.insert("setInterval".into(), fn_ty(
            vec![fn_ty(vec![], TypeInfo::Void), TypeInfo::I32],
            TypeInfo::I32,
        ));
        scope.insert("clearTimeout".into(), fn_ty(
            vec![TypeInfo::I32], TypeInfo::Void,
        ));
        scope.insert("clearInterval".into(), fn_ty(
            vec![TypeInfo::I32], TypeInfo::Void,
        ));
        scope.insert("requestAnimationFrame".into(), fn_ty(
            vec![fn_ty(vec![TypeInfo::F64], TypeInfo::Void)],
            TypeInfo::I32,
        ));
        scope.insert("cancelAnimationFrame".into(), fn_ty(
            vec![TypeInfo::I32], TypeInfo::Void,
        ));
        scope.insert("requestIdleCallback".into(), fn_ty(
            vec![fn_ty(vec![blank_obj()], TypeInfo::Void)],
            TypeInfo::I32,
        ));

        // ── Parsing / encoding ───────────────────────────────────────
        scope.insert("parseInt".into(), fn_ty(
            vec![TypeInfo::String, opt(TypeInfo::I32)], TypeInfo::I32,
        ));
        scope.insert("parseFloat".into(), fn_ty(
            vec![TypeInfo::String], TypeInfo::F64,
        ));
        scope.insert("isNaN".into(), fn_ty(
            vec![TypeInfo::F64], TypeInfo::Bool,
        ));
        scope.insert("isFinite".into(), fn_ty(
            vec![TypeInfo::F64], TypeInfo::Bool,
        ));
        scope.insert("decodeURI".into(), fn_ty(
            vec![TypeInfo::String], TypeInfo::String,
        ));
        scope.insert("encodeURI".into(), fn_ty(
            vec![TypeInfo::String], TypeInfo::String,
        ));
        scope.insert("decodeURIComponent".into(), fn_ty(
            vec![TypeInfo::String], TypeInfo::String,
        ));
        scope.insert("encodeURIComponent".into(), fn_ty(
            vec![TypeInfo::String], TypeInfo::String,
        ));

        // ── Rakit runtime ────────────────────────────────────────────
        scope.insert("gunakanFetch".into(), fn_ty(
            vec![TypeInfo::String],
            TypeInfo::Array(Box::new(TypeInfo::Infer)),
        ));
        scope.insert("gunakanKonteks".into(), fn_ty(
            vec![TypeInfo::Infer], TypeInfo::Infer,
        ));

        // ── JSX / render ─────────────────────────────────────────────
        scope.insert("h".into(), fn_ty(
            vec![TypeInfo::String, blank_obj(),
                 TypeInfo::Array(Box::new(TypeInfo::Node))],
            TypeInfo::Node,
        ));
        scope.insert("cetak".into(), fn_ty(
            vec![TypeInfo::String], TypeInfo::Void,
        ));
        scope.insert("render".into(), fn_ty(
            vec![TypeInfo::Node, TypeInfo::Infer], TypeInfo::Void,
        ));
        scope.insert("tampilkan".into(), fn_ty(
            vec![TypeInfo::Node], TypeInfo::Void,
        ));

        // ── JSON utilities (legacy) ──────────────────────────────────
        scope.insert("parseJSON".into(), fn_ty(
            vec![TypeInfo::String], TypeInfo::Infer,
        ));
        scope.insert("stringifyJSON".into(), fn_ty(
            vec![TypeInfo::Infer], TypeInfo::String,
        ));

        // ── Context hook ─────────────────────────────────────────────
        scope.insert("konteks".into(), fn_ty(
            vec![TypeInfo::Infer], TypeInfo::Infer,
        ));

        // ── Time utilities ───────────────────────────────────────────
        scope.insert("tunda".into(), fn_ty(
            vec![TypeInfo::I32], TypeInfo::Void,
        ));
        scope.insert("sekarang".into(), fn_ty(
            vec![], TypeInfo::I64,
        ));
        scope.insert("waktu".into(), blank_obj());

        // ── Result type ─────────────────────────────────────────────
        scope.insert("Hasil".into(), blank_obj());

        // ── Other hooks ─────────────────────────────────────────────
        scope.insert("acu".into(), fn_ty(
            vec![TypeInfo::Infer], TypeInfo::Infer,
        ));
        scope.insert("panggil".into(), fn_ty(
            vec![TypeInfo::Infer], TypeInfo::Infer,
        ));
        scope.insert("pengedger".into(), fn_ty(
            vec![TypeInfo::Infer], TypeInfo::Infer,
        ));
        scope.insert("berhenti".into(), fn_ty(
            vec![], TypeInfo::Void,
        ));
    }

    pub fn register_top_level(&mut self, item: &HirItem) {
        match item {
            HirItem::Function(f) => {
                let fn_ty = FnType::new(
                    f.params.iter().map(|p| p.ty.clone()).collect(),
                    f.return_ty.clone(),
                );
                self.fn_sigs.insert(f.name.clone(), fn_ty.clone());
                self.scope[0].bindings.insert(f.name.clone(), TypeInfo::Fn(fn_ty));
            }
            HirItem::Struct(s) => {
                let st = StructType {
                    name: s.name.clone(),
                    fields: s.fields.iter().map(|f| FieldType {
                        name: f.name.clone(),
                        ty: f.ty.clone(),
                    }).collect(),
                    generics: s.generics.iter().map(|g| TypeInfo::Generic(g.clone())).collect(),
                };
                self.struct_defs.insert(s.name.clone(), st.clone());
                self.scope[0].bindings.insert(s.name.clone(), TypeInfo::Struct(st));
            }
            HirItem::Enum(e) => {
                let et = EnumType {
                    name: e.name.clone(),
                    variants: e.variants.clone(),
                };
                self.enum_defs.insert(e.name.clone(), et.clone());
                self.scope[0].bindings.insert(e.name.clone(), TypeInfo::Enum(et));
            }
            HirItem::TypeAlias(t) => {
                self.scope[0].bindings.insert(t.name.clone(), t.ty.clone());
            }
            _ => {}
        }
    }

    pub fn check_program(&mut self, program: &mut HirProgram) -> bool {
        let items_copy = program.items.clone();
        for item in &items_copy {
            self.register_top_level(item);
        }
        for item in &mut program.items {
            self.check_item(item);
        }
        self.errors == 0
    }

    pub fn check_item(&mut self, item: &mut HirItem) {
        match item {
            HirItem::Function(f) => {
                self.enter_scope();
                for param in &f.params {
                    self.scope_last().bindings.insert(param.name.clone(), param.ty.clone());
                }
                for stmt in &mut f.body.stmts {
                    self.check_stmt(stmt);
                }
                self.exit_scope();
            }
            HirItem::Component(c) => {
                self.enter_scope();
                self.scope_last().bindings.insert(c.props_param.name.clone(), c.props_param.ty.clone());
                for hc in &c.hook_calls {
                    if let HookKind::State { ref state_var, ref setter_var, ref ty, .. } = hc.kind {
                        self.scope_last().bindings.insert(state_var.clone(), ty.clone());
                        self.scope_last().bindings.insert(
                            setter_var.clone(),
                            TypeInfo::Fn(FnType::new(vec![ty.clone()], TypeInfo::Void)),
                        );
                    }
                }
                for stmt in &mut c.body_stmts {
                    self.check_stmt(stmt);
                }
                self.exit_scope();
            }
            _ => {}
        }
    }

    pub fn check_stmt(&mut self, stmt: &mut HirStmt) {
        match stmt {
            HirStmt::Let(l) => {
                let val_ty = self.infer_expr_mut(&mut l.value);
                if l.ty != TypeInfo::Infer && !self.unify(&l.ty, &val_ty) {
                    self.error(format!(
                        "Tipe tidak cocok: diharapkan {:?}, ditemukan {:?}",
                        l.ty, val_ty
                    ));
                }
                self.scope_last().bindings.insert(l.name.clone(), if l.ty != TypeInfo::Infer { l.ty.clone() } else { val_ty });
            }
            HirStmt::Expr(e) => { self.infer_expr_mut(e); }
            HirStmt::If(i) => {
                let cond_ty = self.infer_expr_mut(&mut i.condition);
                if cond_ty != TypeInfo::Bool && !cond_ty.is_error() {
                    self.error("Kondisi 'jika' harus bertipe Bool");
                }
                self.enter_scope();
                for s in &mut i.then_block.stmts { self.check_stmt(s); }
                self.exit_scope();
                if let Some(ref mut else_block) = i.else_block {
                    self.enter_scope();
                    for s in &mut else_block.stmts { self.check_stmt(s); }
                    self.exit_scope();
                }
            }
            HirStmt::While(w) => {
                let cond_ty = self.infer_expr_mut(&mut w.condition);
                if cond_ty != TypeInfo::Bool && !cond_ty.is_error() {
                    self.error("Kondisi 'ulang' harus bertipe Bool");
                }
                self.enter_scope();
                for s in &mut w.body.stmts { self.check_stmt(s); }
                self.exit_scope();
            }
            HirStmt::Return(Some(e)) => { self.infer_expr_mut(e); }
            HirStmt::Return(None) => {}
            HirStmt::Block(b) => {
                self.enter_scope();
                for s in &mut b.stmts { self.check_stmt(s); }
                self.exit_scope();
            }
            HirStmt::Match(m) => {
                self.infer_expr_mut(&mut m.expr);
                for arm in &mut m.arms {
                    self.check_pattern(&arm.pattern);
                    self.infer_expr_mut(&mut arm.body);
                }
            }
            HirStmt::Break | HirStmt::Continue => {}
            HirStmt::Try(t) => {
                self.enter_scope();
                for s in &mut t.try_block.stmts { self.check_stmt(s); }
                self.exit_scope();
                self.enter_scope();
                self.scope_last().bindings.insert(t.catch_var.clone(), TypeInfo::Infer);
                for s in &mut t.catch_block.stmts { self.check_stmt(s); }
                self.exit_scope();
            }
            HirStmt::Throw(e) => { self.infer_expr_mut(e); }
        }
    }

    pub fn infer_expr(&self, expr: &HirExpr) -> TypeInfo {
        match expr {
            HirExpr::Number(_, ty) => ty.clone(),
            HirExpr::String(_, ty) => ty.clone(),
            HirExpr::Bool(_, ty) => ty.clone(),
            HirExpr::Null(ty) => ty.clone(),
            HirExpr::Ident(name, ty) => {
                if ty.is_infer() {
                    self.lookup(name)
                } else {
                    ty.clone()
                }
            }
            HirExpr::Binary(b) => b.ty.clone(),
            HirExpr::Unary(u) => u.ty.clone(),
            HirExpr::Assign(a) => a.ty.clone(),
            HirExpr::Ternary(t) => t.ty.clone(),
            HirExpr::Call(c) => c.ty.clone(),
            HirExpr::Member(m) => m.ty.clone(),
            HirExpr::Index(i) => i.ty.clone(),
            HirExpr::Array(_, ty) => ty.clone(),
            HirExpr::StructInit(s) => s.ty.clone(),
            HirExpr::JsxElement(e) => e.ty.clone(),
            HirExpr::HookState(h) => h.ty.clone(),
            HirExpr::HookEffect(_) => TypeInfo::Void,
            HirExpr::HookMemo(h) => h.ty.clone(),
            HirExpr::Block(b) => {
                b.stmts.last().map(|s| match s {
                    HirStmt::Expr(e) => self.infer_expr(e),
                    _ => TypeInfo::Void,
                }).unwrap_or(TypeInfo::Void)
            }
        }
    }

    pub fn infer_expr_mut(&mut self, expr: &mut HirExpr) -> TypeInfo {
        let ty = match expr {
            HirExpr::Number(_, ref mut ty) => { *ty = TypeInfo::F64; TypeInfo::F64 }
            HirExpr::String(_, ref mut ty) => { *ty = TypeInfo::String; TypeInfo::String }
            HirExpr::Bool(_, ref mut ty) => { *ty = TypeInfo::Bool; TypeInfo::Bool }
            HirExpr::Null(ref mut ty) => {
                if *ty == TypeInfo::Infer {
                    *ty = TypeInfo::Optional(Box::new(TypeInfo::Infer));
                }
                ty.clone()
            }
            HirExpr::Ident(name, ref mut ty) => {
                let resolved = self.lookup(name);
                *ty = resolved.clone();
                resolved
            }
            HirExpr::Binary(ref mut b) => {
                let lhs_ty = self.infer_expr_mut(&mut b.lhs);
                let rhs_ty = self.infer_expr_mut(&mut b.rhs);
                let result = self.check_binary_op(b.op.clone(), &lhs_ty, &rhs_ty);
                b.ty = result.clone();
                result
            }
            HirExpr::Unary(ref mut u) => {
                let inner = self.infer_expr_mut(&mut u.expr);
                let result = match u.op {
                    HirUnaryOp::Neg if inner.is_numeric() => inner,
                    HirUnaryOp::Neg => { self.error("Operator tidak cocok dengan tipe"); TypeInfo::Error }
                    HirUnaryOp::Not => TypeInfo::Bool,
                };
                u.ty = result.clone();
                result
            }
            HirExpr::Call(ref mut c) => {
                let callee_ty = self.infer_expr_mut(&mut c.callee);
                let mut arg_tys = Vec::new();
                for arg in &mut c.args {
                    arg_tys.push(self.infer_expr_mut(arg));
                }
                let result = self.check_call(&callee_ty, &arg_tys);
                c.ty = result.clone();
                result
            }
            HirExpr::Ternary(ref mut t) => {
                let cond_ty = self.infer_expr_mut(&mut t.condition);
                if cond_ty != TypeInfo::Bool && !cond_ty.is_error() {
                    self.error("Kondisi ternary harus bertipe Bool");
                }
                let then_ty = self.infer_expr_mut(&mut t.then_expr);
                let else_ty = self.infer_expr_mut(&mut t.else_expr);
                let result = if self.unify(&then_ty, &else_ty) { then_ty } else { else_ty };
                t.ty = result.clone();
                result
            }
            HirExpr::Assign(ref mut a) => {
                let _target_ty = self.infer_expr_mut(&mut a.target);
                let val_ty = self.infer_expr_mut(&mut a.value);
                a.ty = TypeInfo::Void;
                val_ty
            }
            HirExpr::Member(ref mut m) => {
                let obj_ty = self.infer_expr_mut(&mut m.object);
                let result = self.check_member(&obj_ty, &m.field);
                m.ty = result.clone();
                result
            }
            HirExpr::Index(ref mut i) => {
                let obj_ty = self.infer_expr_mut(&mut i.object);
                let _idx_ty = self.infer_expr_mut(&mut i.index);
                let result = match obj_ty {
                    TypeInfo::Array(ref inner) => *inner.clone(),
                    _ => { self.error("Hanya array yang bisa di-index"); TypeInfo::Error }
                };
                i.ty = result.clone();
                result
            }
            HirExpr::Array(ref mut items, ref mut ty) => {
                if items.is_empty() {
                    *ty = TypeInfo::Array(Box::new(TypeInfo::Infer));
                    ty.clone()
                } else {
                    let item_ty = self.infer_expr_mut(&mut items[0]);
                    for item in &mut items[1..] {
                        self.infer_expr_mut(item);
                    }
                    *ty = TypeInfo::Array(Box::new(item_ty.clone()));
                    ty.clone()
                }
            }
            HirExpr::StructInit(ref mut s) => {
                if s.name == "Attrs" {
                    for field in &mut s.fields {
                        self.infer_expr_mut(&mut field.value);
                    }
                    s.ty = TypeInfo::Struct(StructType {
                        name: "Attrs".into(),
                        fields: s.fields.iter().map(|f| FieldType {
                            name: f.name.clone(),
                            ty: TypeInfo::Infer,
                        }).collect(),
                        generics: Vec::new(),
                    });
                    return s.ty.clone();
                }
                let st_opt = self.struct_defs.get(&s.name).cloned();
                if let Some(st) = st_opt {
                    for field in &mut s.fields {
                        self.infer_expr_mut(&mut field.value);
                    }
                    s.ty = TypeInfo::Struct(st);
                    s.ty.clone()
                } else {
                    self.error(format!("Struktur '{}' tidak dikenal", s.name));
                    TypeInfo::Error
                }
            }
            HirExpr::JsxElement(ref mut e) => {
                for (_, ref mut attr_expr) in &mut e.attrs {
                    self.infer_expr_mut(attr_expr);
                }
                for child in &mut e.children {
                    self.infer_expr_mut(child);
                }
                e.ty = TypeInfo::Node;
                TypeInfo::Node
            }
            HirExpr::HookState(ref mut h) => {
                let init_ty = self.infer_expr_mut(&mut *h.initial);
                h.ty = init_ty.clone();
                init_ty
            }
            HirExpr::HookEffect(ref mut h) => {
                self.infer_expr_mut(&mut h.callback);
                for dep in &mut h.deps {
                    self.infer_expr_mut(dep);
                }
                TypeInfo::Void
            }
            HirExpr::HookMemo(ref mut h) => {
                let result_ty = self.infer_expr_mut(&mut h.callback);
                h.ty = result_ty.clone();
                for dep in &mut h.deps {
                    self.infer_expr_mut(dep);
                }
                result_ty
            }
            HirExpr::Block(ref mut b) => {
                self.enter_scope();
                for s in &mut b.stmts {
                    self.check_stmt(s);
                }
                let result = b.stmts.last().map(|s| match s {
                    HirStmt::Expr(e) => self.infer_expr(e),
                    _ => TypeInfo::Void,
                }).unwrap_or(TypeInfo::Void);
                self.exit_scope();
                result
            }
        };
        ty
    }

    fn check_binary_op(&self, op: HirBinaryOp, lhs: &TypeInfo, rhs: &TypeInfo) -> TypeInfo {
        match op {
            HirBinaryOp::Add | HirBinaryOp::Sub | HirBinaryOp::Mul
            | HirBinaryOp::Div | HirBinaryOp::Mod => {
                if !lhs.is_numeric() || !rhs.is_numeric() {
                    TypeInfo::Error
                } else {
                    type_widen(lhs, rhs)
                }
            }
            HirBinaryOp::And | HirBinaryOp::Or => {
                if *lhs == TypeInfo::Bool && *rhs == TypeInfo::Bool {
                    TypeInfo::Bool
                } else {
                    TypeInfo::Error
                }
            }
            HirBinaryOp::Eq | HirBinaryOp::Ne
            | HirBinaryOp::Lt | HirBinaryOp::Gt
            | HirBinaryOp::Le | HirBinaryOp::Ge => {
                TypeInfo::Bool
            }
            HirBinaryOp::Concat => TypeInfo::String,
            HirBinaryOp::NullCoalescing => {
                if let TypeInfo::Optional(inner) = lhs {
                    *inner.clone()
                } else {
                    lhs.clone()
                }
            }
        }
    }

    fn check_call(&self, callee_ty: &TypeInfo, _arg_tys: &[TypeInfo]) -> TypeInfo {
        // Built‑in constructors: String, Number, Boolean, BigInt, Array, Object, etc.
        if let Some(ret) = self.try_constructor_call(callee_ty) {
            return ret;
        }
        match callee_ty {
            TypeInfo::Fn(fn_ty) => *fn_ty.ret.clone(),
            _ => TypeInfo::Error,
        }
    }

    /// Recognise built‑in constructors used as functions: `String(42)`, `Array(5)`, etc.
    fn try_constructor_call(&self, callee_ty: &TypeInfo) -> Option<TypeInfo> {
        match callee_ty {
            TypeInfo::String => Some(TypeInfo::String),
            TypeInfo::F64 => Some(TypeInfo::F64),       // Number()
            TypeInfo::Bool => Some(TypeInfo::Bool),     // Boolean()
            TypeInfo::I64 => Some(TypeInfo::I64),       // BigInt()
            TypeInfo::Array(_) => Some(callee_ty.clone()),
            TypeInfo::Struct(st) if st.name == "Object" => Some(self.blank_obj()),
            TypeInfo::Struct(st) if st.name == "Date" => Some(TypeInfo::Struct(st.clone())),
            TypeInfo::Struct(st) if st.name == "Error" => Some(TypeInfo::Struct(st.clone())),
            TypeInfo::Struct(st) if st.name == "Promise" => Some(TypeInfo::Struct(st.clone())),
            TypeInfo::Struct(st) if st.name == "Map" => Some(self.blank_obj()),
            TypeInfo::Struct(st) if st.name == "Set" => Some(self.blank_obj()),
            TypeInfo::Struct(st) if st.name == "RegExp" => Some(TypeInfo::String),
            _ => None,
        }
    }

    fn blank_obj(&self) -> TypeInfo {
        TypeInfo::Struct(StructType {
            name: String::new(), fields: vec![], generics: vec![],
        })
    }

    fn check_member(&self, obj_ty: &TypeInfo, field: &str) -> TypeInfo {
        match obj_ty {
            TypeInfo::String => self.string_member(field),
            TypeInfo::F64 | TypeInfo::F32 | TypeInfo::I32 | TypeInfo::I64
                | TypeInfo::U32 | TypeInfo::U64 => self.number_member(field),
            TypeInfo::Bool => self.boolean_member(field),
            TypeInfo::Array(inner) => self.array_member(field, inner),
            TypeInfo::Struct(st) => {
                st.fields.iter().find(|f| f.name == field)
                    .map(|f| f.ty.clone())
                    .unwrap_or(TypeInfo::Error)
            }
            _ => TypeInfo::Error,
        }
    }

    /// Built‑in String instance methods (English + Indonesian aliases)
    fn string_member(&self, field: &str) -> TypeInfo {
        let fn_ty = |params: Vec<TypeInfo>, ret: TypeInfo| TypeInfo::Fn(FnType::new(params, ret));
        let arr_str = || TypeInfo::Array(Box::new(TypeInfo::String));
        let opt = |t: TypeInfo| TypeInfo::Optional(Box::new(t));
        match field {
            "length" | "panjang" => TypeInfo::I32,
            "split" | "pisah" => fn_ty(vec![TypeInfo::String], arr_str()),
            "potong" | "slice" => fn_ty(vec![TypeInfo::I32, TypeInfo::I32], TypeInfo::String),
            "substring" | "substr" => fn_ty(vec![TypeInfo::I32, TypeInfo::I32], TypeInfo::String),
            "charAt" | "karakterDi" => fn_ty(vec![TypeInfo::I32], TypeInfo::String),
            "charCodeAt" | "kodeKarakterDi" => fn_ty(vec![TypeInfo::I32], TypeInfo::I32),
            "charPointAt" => fn_ty(vec![TypeInfo::I32], TypeInfo::I32),
            "replace" | "ganti" => fn_ty(vec![TypeInfo::String, TypeInfo::String], TypeInfo::String),
            "replaceAll" => fn_ty(vec![TypeInfo::String, TypeInfo::String], TypeInfo::String),
            "startsWith" | "mulaiDengan" => fn_ty(vec![TypeInfo::String], TypeInfo::Bool),
            "endsWith" | "akhirDengan" => fn_ty(vec![TypeInfo::String], TypeInfo::Bool),
            "includes" | "mengandung" => fn_ty(vec![TypeInfo::String], TypeInfo::Bool),
            "indexOf" | "indexDari" => fn_ty(vec![TypeInfo::String], TypeInfo::I32),
            "lastIndexOf" => fn_ty(vec![TypeInfo::String], TypeInfo::I32),
            "trim" | "potongSpasi" | "trimStart" | "trimEnd" => fn_ty(vec![], TypeInfo::String),
            "toUpperCase" | "keHurufBesar" => fn_ty(vec![], TypeInfo::String),
            "toLowerCase" | "keHurufKecil" => fn_ty(vec![], TypeInfo::String),
            "concat" | "gabung" => fn_ty(vec![TypeInfo::String], TypeInfo::String),
            "repeat" | "ulang" => fn_ty(vec![TypeInfo::I32], TypeInfo::String),
            "padStart" | "padEnd" => fn_ty(vec![TypeInfo::I32, TypeInfo::String], TypeInfo::String),
            "match" | "cocok" => fn_ty(vec![TypeInfo::String], arr_str()),
            "matchAll" => fn_ty(vec![TypeInfo::String], arr_str()),
            "search" | "cari" => fn_ty(vec![TypeInfo::String], TypeInfo::I32),
            "toString" => fn_ty(vec![], TypeInfo::String),
            "toLocaleLowerCase" | "toLocaleUpperCase" => fn_ty(vec![], TypeInfo::String),
            "localeCompare" => fn_ty(vec![TypeInfo::String], TypeInfo::I32),
            "normalize" => fn_ty(vec![], TypeInfo::String),
            "codePointAt" => fn_ty(vec![TypeInfo::I32], TypeInfo::I32),
            "at" => fn_ty(vec![TypeInfo::I32], opt(TypeInfo::String)),
            _ => TypeInfo::Error,
        }
    }

    /// Built‑in Number / BigInt instance methods
    fn number_member(&self, field: &str) -> TypeInfo {
        let fn_ty = |params: Vec<TypeInfo>, ret: TypeInfo| TypeInfo::Fn(FnType::new(params, ret));
        let opt = |t: TypeInfo| TypeInfo::Optional(Box::new(t));
        match field {
            "toString" => fn_ty(vec![opt(TypeInfo::I32)], TypeInfo::String),
            "toFixed" | "keTetap" => fn_ty(vec![TypeInfo::I32], TypeInfo::String),
            "toExponential" => fn_ty(vec![TypeInfo::I32], TypeInfo::String),
            "toPrecision" | "kePresisi" => fn_ty(vec![TypeInfo::I32], TypeInfo::String),
            "toLocaleString" => fn_ty(vec![opt(TypeInfo::String)], TypeInfo::String),
            "valueOf" => fn_ty(vec![], TypeInfo::F64),
            _ => TypeInfo::Error,
        }
    }

    /// Built‑in Boolean instance methods
    fn boolean_member(&self, field: &str) -> TypeInfo {
        let fn_ty = |params: Vec<TypeInfo>, ret: TypeInfo| TypeInfo::Fn(FnType::new(params, ret));
        match field {
            "toString" => fn_ty(vec![], TypeInfo::String),
            "valueOf" => fn_ty(vec![], TypeInfo::Bool),
            _ => TypeInfo::Error,
        }
    }

    /// Built‑in Array instance methods (English + Indonesian aliases)
    fn array_member(&self, field: &str, inner: &TypeInfo) -> TypeInfo {
        let fn_ty = |params: Vec<TypeInfo>, ret: TypeInfo| TypeInfo::Fn(FnType::new(params, ret));
        let inner = inner.clone();
        let arr_of = |el: TypeInfo| TypeInfo::Array(Box::new(el));
        let opt = |t: TypeInfo| TypeInfo::Optional(Box::new(t));
        match field {
            "length" | "panjang" => TypeInfo::I32,
            "push" | "dorong" => fn_ty(vec![inner.clone()], TypeInfo::I32),
            "pop" | "keluarkan" => fn_ty(vec![], opt(inner.clone())),
            "shift" | "geserKiri" => fn_ty(vec![], opt(inner.clone())),
            "unshift" | "geserKanan" => fn_ty(vec![inner.clone()], TypeInfo::I32),
            "map" | "petakan" => fn_ty(
                vec![fn_ty(vec![inner.clone(), TypeInfo::I32], TypeInfo::Infer)],
                arr_of(TypeInfo::Infer),
            ),
            "filter" | "saring" => fn_ty(
                vec![fn_ty(vec![inner.clone(), TypeInfo::I32], TypeInfo::Bool)],
                arr_of(inner.clone()),
            ),
            "reduce" | "reduksi" => fn_ty(
                vec![fn_ty(vec![TypeInfo::Infer, inner.clone(), TypeInfo::I32], TypeInfo::Infer),
                     TypeInfo::Infer],
                TypeInfo::Infer,
            ),
            "reduceRight" => fn_ty(
                vec![fn_ty(vec![TypeInfo::Infer, inner.clone(), TypeInfo::I32], TypeInfo::Infer),
                     TypeInfo::Infer],
                TypeInfo::Infer,
            ),
            "find" | "temukan" => fn_ty(
                vec![fn_ty(vec![inner.clone(), TypeInfo::I32], TypeInfo::Bool)],
                opt(inner.clone()),
            ),
            "findIndex" | "cariIndeks" => fn_ty(
                vec![fn_ty(vec![inner.clone(), TypeInfo::I32], TypeInfo::Bool)],
                TypeInfo::I32,
            ),
            "findLast" | "temukanTerakhir" => fn_ty(
                vec![fn_ty(vec![inner.clone(), TypeInfo::I32], TypeInfo::Bool)],
                opt(inner.clone()),
            ),
            "findLastIndex" => fn_ty(
                vec![fn_ty(vec![inner.clone(), TypeInfo::I32], TypeInfo::Bool)],
                TypeInfo::I32,
            ),
            "some" | "beberapa" => fn_ty(
                vec![fn_ty(vec![inner.clone(), TypeInfo::I32], TypeInfo::Bool)],
                TypeInfo::Bool,
            ),
            "every" => fn_ty(
                vec![fn_ty(vec![inner.clone(), TypeInfo::I32], TypeInfo::Bool)],
                TypeInfo::Bool,
            ),
            "includes" | "mengandung" => fn_ty(vec![inner.clone()], TypeInfo::Bool),
            "indexOf" | "indexDari" => fn_ty(vec![inner.clone()], TypeInfo::I32),
            "lastIndexOf" => fn_ty(vec![inner.clone()], TypeInfo::I32),
            "join" | "gabung" => fn_ty(vec![TypeInfo::String], TypeInfo::String),
            "concat" => fn_ty(vec![arr_of(inner.clone())], arr_of(inner.clone())),
            "potong" | "slice" => fn_ty(vec![TypeInfo::I32, opt(TypeInfo::I32)], arr_of(inner.clone())),
            "splice" | "sambung" => fn_ty(
                vec![TypeInfo::I32, TypeInfo::I32, inner.clone()],
                arr_of(inner.clone()),
            ),
            "reverse" | "balik" => fn_ty(vec![], arr_of(inner.clone())),
            "sort" | "urutkan" => fn_ty(
                vec![opt(fn_ty(vec![inner.clone(), inner.clone()], TypeInfo::I32))],
                arr_of(inner.clone()),
            ),
            "forEach" | "setiap" => fn_ty(
                vec![fn_ty(vec![inner.clone(), TypeInfo::I32], TypeInfo::Void)],
                TypeInfo::Void,
            ),
            "flat" | "rata" => fn_ty(vec![opt(TypeInfo::I32)], arr_of(TypeInfo::Infer)),
            "flatMap" => fn_ty(
                vec![fn_ty(vec![inner.clone(), TypeInfo::I32], arr_of(TypeInfo::Infer))],
                arr_of(TypeInfo::Infer),
            ),
            "fill" | "isi" => fn_ty(
                vec![inner.clone(), opt(TypeInfo::I32), opt(TypeInfo::I32)],
                arr_of(inner.clone()),
            ),
            "copyWithin" => fn_ty(
                vec![TypeInfo::I32, TypeInfo::I32, opt(TypeInfo::I32)],
                arr_of(inner.clone()),
            ),
            "keys" => fn_ty(vec![], arr_of(TypeInfo::I32)),
            "values" => fn_ty(vec![], arr_of(inner.clone())),
            "entries" => fn_ty(vec![], arr_of(arr_of(TypeInfo::Infer))),
            "toString" => fn_ty(vec![], TypeInfo::String),
            "toLocaleString" => fn_ty(vec![], TypeInfo::String),
            "at" => fn_ty(vec![TypeInfo::I32], opt(inner.clone())),
            // Spread-like: for `...arr` in function calls
            _ => TypeInfo::Error,
        }
    }

    fn check_pattern(&self, pattern: &HirPattern) {
        match pattern {
            HirPattern::Wildcard => {}
            HirPattern::Literal(_) => {}
            HirPattern::Ident(_) => {}
            HirPattern::Struct { name, fields } => {
                if let Some(st) = self.struct_defs.get(name) {
                    for (fname, _) in fields {
                        if !st.fields.iter().any(|f| f.name == *fname) {
                        }
                    }
                }
            }
            HirPattern::Enum { name, variant, fields: _ } => {
                if let Some(et) = self.enum_defs.get(name) {
                    if !et.variants.iter().any(|v| v.name == *variant) {
                    }
                }
            }
        }
    }

    fn lookup(&self, name: &str) -> TypeInfo {
        for scope in self.scope.iter().rev() {
            if let Some(ty) = scope.bindings.get(name) {
                return ty.clone();
            }
        }
        TypeInfo::Error
    }

    fn unify(&self, expected: &TypeInfo, actual: &TypeInfo) -> bool {
        match (expected, actual) {
            (a, b) if a == b => true,
            (TypeInfo::Infer, _) | (_, TypeInfo::Infer) => true,
            (TypeInfo::I32, TypeInfo::I64) => true,
            (TypeInfo::I64, TypeInfo::I32) => true,
            (TypeInfo::F32, TypeInfo::F64) => true,
            (TypeInfo::F64, TypeInfo::F32) => true,
            (TypeInfo::Optional(a), _b) if **a == TypeInfo::Infer => true,
            (TypeInfo::Optional(a), b) if a.as_ref() == b => true,
            (a, TypeInfo::Optional(b)) if a == b.as_ref() => true,
            (TypeInfo::Error, _) | (_, TypeInfo::Error) => true,
            _ => false,
        }
    }

    fn enter_scope(&mut self) {
        let depth = self.scope.last().map(|s| s.depth + 1).unwrap_or(0);
        self.scope.push(Scope { bindings: HashMap::new(), depth });
    }

    fn exit_scope(&mut self) {
        self.scope.pop();
    }

    fn scope_last(&mut self) -> &mut Scope {
        self.scope.last_mut().unwrap()
    }

    fn error(&mut self, msg: impl Into<String>) {
        self.errors += 1;
        self.diagnostics.push(Diagnostic::error(msg));
    }
}
