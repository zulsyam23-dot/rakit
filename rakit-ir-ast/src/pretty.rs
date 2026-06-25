use crate::ast::*;
use crate::ty::*;
use std::fmt::Write;

/// Pretty printer untuk AST Rakit — mencetak struktur pohon dengan indentasi.
pub struct AstPrettyPrinter {
    indent: usize,
    output: String,
}

impl AstPrettyPrinter {
    pub fn new() -> Self {
        AstPrettyPrinter {
            indent: 0,
            output: String::new(),
        }
    }

    pub fn print(&mut self, program: &Program) -> String {
        self.output.clear();
        self.output.push_str("Program {\n");
        self.indent += 1;
        for item in &program.items {
            self.print_indent();
            self.print_item(item);
            self.output.push('\n');
        }
        self.indent -= 1;
        self.output.push_str("}\n");
        self.output.clone()
    }

    fn print_indent(&mut self) {
        write!(self.output, "{}", "  ".repeat(self.indent)).unwrap();
    }

    fn print_item(&mut self, item: &Item) {
        match item {
            Item::Function(f) => {
                write!(self.output, "Function \"{}\"", f.name).unwrap();
                if !f.params.is_empty() {
                    write!(self.output, "(").unwrap();
                    for (i, p) in f.params.iter().enumerate() {
                        if i > 0 { write!(self.output, ", ").unwrap(); }
                        write!(self.output, "{}: ", p.name).unwrap();
                        self.print_type(&p.ty);
                    }
                    write!(self.output, ")").unwrap();
                }
                if let Some(ty) = &f.return_ty {
                    write!(self.output, " -> ").unwrap();
                    self.print_type(ty);
                }
                self.output.push_str(" {\n");
                self.indent += 1;
                for stmt in &f.body.stmts {
                    self.print_indent();
                    self.print_stmt(stmt);
                    self.output.push('\n');
                }
                self.indent -= 1;
                self.print_indent();
                self.output.push('}');
            },
            Item::Component(c) => {
                write!(self.output, "Component \"{}\"", c.name).unwrap();
                if !c.props.is_empty() {
                    write!(self.output, "(").unwrap();
                    for (i, p) in c.props.iter().enumerate() {
                        if i > 0 { write!(self.output, ", ").unwrap(); }
                        write!(self.output, "{}: ", p.name).unwrap();
                        self.print_type(&p.ty);
                    }
                    write!(self.output, ")").unwrap();
                }
                self.output.push_str(" {\n");
                self.indent += 1;
                for stmt in &c.body.statements {
                    self.print_indent();
                    self.print_stmt(stmt);
                    self.output.push('\n');
                }
                self.print_indent();
                self.output.push_str("render: ");
                self.print_expr(&c.body.render);
                self.output.push('\n');
                self.indent -= 1;
                self.print_indent();
                self.output.push('}');
            },
            Item::Struct(s) => {
                write!(self.output, "Struct \"{}\"", s.name).unwrap();
                if !s.generics.is_empty() {
                    write!(self.output, "<{}>", s.generics.join(", ")).unwrap();
                }
                self.output.push_str(" { ");
                for (i, f) in s.fields.iter().enumerate() {
                    if i > 0 { write!(self.output, ", ").unwrap(); }
                    write!(self.output, "{}: ", f.name).unwrap();
                    self.print_type(&f.ty);
                }
                self.output.push_str(" }");
            },
            Item::Enum(e) => {
                write!(self.output, "Enum \"{}\"", e.name).unwrap();
                self.output.push_str(" { ");
                for (i, v) in e.variants.iter().enumerate() {
                    if i > 0 { write!(self.output, ", ").unwrap(); }
                    write!(self.output, "{}", v.name).unwrap();
                }
                self.output.push_str(" }");
            },
            Item::TypeAlias(t) => {
                write!(self.output, "TypeAlias \"{}\" -> ", t.name).unwrap();
                self.print_type(&t.ty);
            },
            Item::Import(i) => {
                write!(self.output, "Import from \"{}\"", i.module).unwrap();
                if !i.names.is_empty() {
                    write!(self.output, " -> {{{}}}", i.names.join(", ")).unwrap();
                }
            },
            Item::Export(e) => {
                write!(self.output, "Export \"{}\"", e.item_name).unwrap();
            },
        }
    }

    fn print_type(&mut self, ty: &Type) {
        match ty {
            Type::Named(name) => write!(self.output, "{}", name).unwrap(),
            Type::Generic(g) => {
                write!(self.output, "{}<", g.name).unwrap();
                for (i, p) in g.params.iter().enumerate() {
                    if i > 0 { write!(self.output, ", ").unwrap(); }
                    self.print_type(p);
                }
                write!(self.output, ">").unwrap();
            }
            Type::Struct(fields) => {
                write!(self.output, "{{ ").unwrap();
                for (i, f) in fields.iter().enumerate() {
                    if i > 0 { write!(self.output, ", ").unwrap(); }
                    write!(self.output, "{}: ", f.name).unwrap();
                    self.print_type(&f.ty);
                }
                write!(self.output, " }}").unwrap();
            }
            Type::Tuple(tys) => {
                write!(self.output, "(").unwrap();
                for (i, t) in tys.iter().enumerate() {
                    if i > 0 { write!(self.output, ", ").unwrap(); }
                    self.print_type(t);
                }
                write!(self.output, ")").unwrap();
            }
            Type::Array(inner) => {
                write!(self.output, "Array<").unwrap();
                self.print_type(inner);
                write!(self.output, ">").unwrap();
            }
            Type::Fn(params, ret) => {
                write!(self.output, "(").unwrap();
                for (i, p) in params.iter().enumerate() {
                    if i > 0 { write!(self.output, ", ").unwrap(); }
                    self.print_type(p);
                }
                write!(self.output, ") -> ").unwrap();
                self.print_type(ret);
            }
            Type::Optional(inner) => {
                self.print_type(inner);
                write!(self.output, "?").unwrap();
            }
            Type::Infer => write!(self.output, "_").unwrap(),
            Type::Union(variants) => {
                for (i, v) in variants.iter().enumerate() {
                    if i > 0 { write!(self.output, " | ").unwrap(); }
                    self.print_type(v);
                }
            },
        }
    }

    fn print_stmt(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::Let(let_def) => {
                if let_def.mutable {
                    write!(self.output, "ubah ").unwrap();
                } else {
                    write!(self.output, "konstan ").unwrap();
                }
                write!(self.output, "{}", let_def.name).unwrap();
                if let Some(ty) = &let_def.ty {
                    write!(self.output, ": ").unwrap();
                    self.print_type(ty);
                }
                write!(self.output, " = ").unwrap();
                self.print_expr(&let_def.value);
            },
            Stmt::Expr(expr, _) => {
                self.print_expr(expr);
            },
            Stmt::If(if_stmt) => {
                write!(self.output, "jika ").unwrap();
                self.print_expr(&if_stmt.condition);
                self.output.push_str(" { ... }");
                if if_stmt.else_block.is_some() {
                    self.output.push_str(" lain { ... }");
                }
            },
            Stmt::While(w) => {
                write!(self.output, "ulang ").unwrap();
                self.print_expr(&w.condition);
                self.output.push_str(" { ... }");
            },
            Stmt::Match(m) => {
                write!(self.output, "cocok ").unwrap();
                self.print_expr(&m.expr);
                self.output.push_str(" { ... }");
            },
            Stmt::Return(Some(expr), _) => {
                write!(self.output, "berhenti ").unwrap();
                self.print_expr(expr);
            },
            Stmt::Return(None, _) => write!(self.output, "berhenti").unwrap(),
            Stmt::Block(block) => {
                self.output.push_str("{\n");
                self.indent += 1;
                for s in &block.stmts {
                    self.print_indent();
                    self.print_stmt(s);
                    self.output.push('\n');
                }
                self.indent -= 1;
                self.print_indent();
                self.output.push('}');
            },
            Stmt::Break(_) => write!(self.output, "berhenti (break)").unwrap(),
            Stmt::Continue(_) => write!(self.output, "lanjut").unwrap(),
            Stmt::Try(t) => {
                write!(self.output, "coba {{ ... }} tangkap({}) {{ ... }}", t.catch_var).unwrap();
            },
            Stmt::Throw(expr, _) => {
                write!(self.output, "lempar ").unwrap();
                self.print_expr(expr);
            },
            Stmt::HookState(hs) => {
                write!(self.output, "keadaan({}, {})", hs.state_var, hs.setter_var).unwrap();
                if let Some(ty) = &hs.ty {
                    write!(self.output, ": ").unwrap();
                    self.print_type(ty);
                }
                write!(self.output, " = ").unwrap();
                self.print_expr(&hs.value);
            },
            Stmt::FnDef(f) => {
                write!(self.output, "fungsi {}(", f.name).unwrap();
                for (i, p) in f.params.iter().enumerate() {
                    if i > 0 { write!(self.output, ", ").unwrap(); }
                    write!(self.output, "{}: ", p.name).unwrap();
                    self.print_type(&p.ty);
                }
                write!(self.output, ")").unwrap();
                if let Some(rt) = &f.return_ty {
                    write!(self.output, " -> ").unwrap();
                    self.print_type(rt);
                }
                write!(self.output, " {{ ... }}").unwrap();
            },
        }
    }

    fn print_expr(&mut self, expr: &Expr) {
        match expr {
            Expr::Literal(lit) => self.print_literal(lit),
            Expr::Ident(name) => write!(self.output, "{}", name).unwrap(),
            Expr::Binary(op, lhs, rhs) => {
                write!(self.output, "(").unwrap();
                self.print_expr(lhs);
                write!(self.output, " {} ", match op {
                    BinaryOp::Add => "+",
                    BinaryOp::Sub => "-",
                    BinaryOp::Mul => "*",
                    BinaryOp::Div => "/",
                    BinaryOp::Mod => "%",
                    BinaryOp::And => "&&",
                    BinaryOp::Or => "||",
                    BinaryOp::Eq => "==",
                    BinaryOp::Ne => "!=",
                    BinaryOp::Lt => "<",
                    BinaryOp::Gt => ">",
                    BinaryOp::Le => "<=",
                    BinaryOp::Ge => ">=",
                    BinaryOp::Concat => "++",
                    BinaryOp::NullCoalescing => "??",
                }).unwrap();
                self.print_expr(rhs);
                write!(self.output, ")").unwrap();
            },
            Expr::Unary(op, expr) => {
                let op_str = match op {
                    UnaryOp::Neg => "-",
                    UnaryOp::Not => "!",
                };
                write!(self.output, "{}", op_str).unwrap();
                self.print_expr(expr);
            },
            Expr::Call(callee, args) => {
                self.print_expr(callee);
                write!(self.output, "(").unwrap();
                for (i, arg) in args.iter().enumerate() {
                    if i > 0 { write!(self.output, ", ").unwrap(); }
                    self.print_expr(arg);
                }
                write!(self.output, ")").unwrap();
            },
            Expr::Member(obj, field) => {
                self.print_expr(obj);
                write!(self.output, ".{}", field).unwrap();
            },
            Expr::JsxElement(elem) => {
                write!(self.output, "<{}", elem.tag).unwrap();
                for attr in &elem.attrs {
                    match attr {
                        JsxAttr::Literal { name, value, .. } => {
                            write!(self.output, " {}=\"{}\"", name, value).unwrap();
                        }
                        JsxAttr::Expr { name, value, .. } => {
                            write!(self.output, " {}=", name).unwrap();
                            self.print_expr(value);
                        }
                        JsxAttr::Spread(_, _) => {
                            write!(self.output, " ...").unwrap();
                        }
                    }
                }
                if elem.children.is_empty() {
                    write!(self.output, " />").unwrap();
                } else {
                    write!(self.output, ">").unwrap();
                    for child in &elem.children {
                        match child {
                            JsxChild::Text(t) => write!(self.output, "\"{}\"", t).unwrap(),
                            JsxChild::Expr(e) => { self.print_expr(e); },
                            JsxChild::Element(e) => {
                                self.print_expr(&Expr::JsxElement(e.clone()));
                            }
                            JsxChild::Fragment(f) => {
                                self.print_expr(&Expr::JsxFragment(f.clone()));
                            }
                        }
                    }
                    write!(self.output, "</{}>", elem.tag).unwrap();
                }
            },
            Expr::JsxFragment(frag) => {
                write!(self.output, "<>").unwrap();
                for child in &frag.children {
                    match child {
                        JsxChild::Text(t) => write!(self.output, "\"{}\"", t).unwrap(),
                        JsxChild::Expr(e) => { self.print_expr(e); },
                        JsxChild::Element(e) => {
                            self.print_expr(&Expr::JsxElement(e.clone()));
                        }
                        JsxChild::Fragment(f) => {
                            self.print_expr(&Expr::JsxFragment(f.clone()));
                        }
                    }
                }
                write!(self.output, "</>").unwrap();
            },
            Expr::BlockExpr(block) => {
                self.output.push_str("{\n");
                self.indent += 1;
                for s in &block.stmts {
                    self.print_indent();
                    self.print_stmt(s);
                    self.output.push('\n');
                }
                self.indent -= 1;
                self.print_indent();
                self.output.push('}');
            },
            Expr::Ternary(cond, then_expr, else_expr) => {
                self.print_expr(cond);
                write!(self.output, " ? ").unwrap();
                self.print_expr(then_expr);
                write!(self.output, " : ").unwrap();
                self.print_expr(else_expr);
            },
            Expr::Array(items) => {
                write!(self.output, "[").unwrap();
                for (i, item) in items.iter().enumerate() {
                    if i > 0 { write!(self.output, ", ").unwrap(); }
                    self.print_expr(item);
                }
                write!(self.output, "]").unwrap();
            },
            Expr::StructInit(name, fields) => {
                write!(self.output, "{} {{ ", name).unwrap();
                for (i, f) in fields.iter().enumerate() {
                    if i > 0 { write!(self.output, ", ").unwrap(); }
                    write!(self.output, "{}: ", f.name).unwrap();
                    self.print_expr(&f.value);
                }
                write!(self.output, " }}").unwrap();
            },
            Expr::Assign { target, value } => {
                self.print_expr(target);
                write!(self.output, " = ").unwrap();
                self.print_expr(value);
            },
            Expr::Index(obj, index) => {
                self.print_expr(obj);
                write!(self.output, "[").unwrap();
                self.print_expr(index);
                write!(self.output, "]").unwrap();
            },
            Expr::Object(fields) => {
                write!(self.output, "{{ ").unwrap();
                for (i, f) in fields.iter().enumerate() {
                    if i > 0 { write!(self.output, ", ").unwrap(); }
                    if f.spread {
                        write!(self.output, "...").unwrap();
                        self.print_expr(&f.value);
                    } else {
                        write!(self.output, "{}: ", f.name).unwrap();
                        self.print_expr(&f.value);
                    }
                }
                write!(self.output, " }}").unwrap();
            },
            Expr::Spread(expr) => {
                write!(self.output, "...").unwrap();
                self.print_expr(expr);
            },
            Expr::ArrowFn(arrow) => {
                write!(self.output, "(").unwrap();
                for (i, p) in arrow.params.iter().enumerate() {
                    if i > 0 { write!(self.output, ", ").unwrap(); }
                    write!(self.output, "{}", p.name).unwrap();
                }
                write!(self.output, ") => ").unwrap();
                self.print_expr(&arrow.body);
            },
        }
    }

    fn print_literal(&mut self, lit: &Literal) {
        match lit {
            Literal::Number(n) => {
                if *n == (*n as i64) as f64 {
                    write!(self.output, "{}", *n as i64).unwrap();
                } else {
                    write!(self.output, "{}", n).unwrap();
                }
            },
            Literal::String(s) => write!(self.output, "\"{}\"", s).unwrap(),
            Literal::Bool(b) => write!(self.output, "{}", if *b { "benar" } else { "salah" }).unwrap(),
            Literal::Null => write!(self.output, "batal").unwrap(),
            Literal::Char(c) => write!(self.output, "'{}'", c).unwrap(),
        }
    }
}
