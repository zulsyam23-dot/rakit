use std::fmt::Write;
use crate::hir::*;

pub struct HirPrettyPrinter {
    indent: usize,
    output: String,
}

impl HirPrettyPrinter {
    pub fn new() -> Self {
        HirPrettyPrinter { indent: 0, output: String::new() }
    }

    pub fn print_program(&mut self, program: &HirProgram) -> String {
        self.output.clear();
        self.output.push_str("HirProgram {\n");
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

    fn print_item(&mut self, item: &HirItem) {
        match item {
            HirItem::Function(f) => {
                write!(self.output, "Function \"{}\"", f.name).unwrap();
                if !f.params.is_empty() {
                    write!(self.output, "(").unwrap();
                    for (i, p) in f.params.iter().enumerate() {
                        if i > 0 { write!(self.output, ", ").unwrap(); }
                        write!(self.output, "{}: {:?}", p.name, p.ty).unwrap();
                    }
                    write!(self.output, ")").unwrap();
                }
                write!(self.output, " -> {:?}", f.return_ty).unwrap();
                self.print_block(&f.body);
            }
            HirItem::Component(c) => {
                write!(self.output, "Component \"{}\"", c.name).unwrap();
                write!(self.output, "(props: {:?})", c.props_param.ty).unwrap();
                if !c.hook_calls.is_empty() {
                    write!(self.output, " hooks: [").unwrap();
                    for (i, hc) in c.hook_calls.iter().enumerate() {
                        if i > 0 { write!(self.output, ", ").unwrap(); }
                        match &hc.kind {
                            HookKind::State { state_var, setter_var, ty, .. } => {
                                write!(self.output, "HookState({}, {}): {:?}", state_var, setter_var, ty).unwrap();
                            }
                            HookKind::Effect { .. } => {
                                write!(self.output, "HookEffect").unwrap();
                            }
                            HookKind::Memo { result_var, ty, .. } => {
                                write!(self.output, "HookMemo({}): {:?}", result_var, ty).unwrap();
                            }
                        }
                    }
                    write!(self.output, "]").unwrap();
                }
                self.output.push_str(" {\n");
                self.indent += 1;
                for stmt in &c.body_stmts {
                    self.print_indent();
                    self.print_stmt(stmt);
                    self.output.push('\n');
                }
                self.print_indent();
                write!(self.output, "render: ").unwrap();
                self.print_expr(&c.render);
                self.output.push('\n');
                self.indent -= 1;
                self.print_indent();
                self.output.push('}');
            }
            HirItem::Struct(s) => {
                write!(self.output, "Struct \"{}\"", s.name).unwrap();
                if !s.generics.is_empty() {
                    write!(self.output, "<{}>", s.generics.join(", ")).unwrap();
                }
                self.output.push_str(" { ");
                for (i, f) in s.fields.iter().enumerate() {
                    if i > 0 { write!(self.output, ", ").unwrap(); }
                    write!(self.output, "{}: {:?}", f.name, f.ty).unwrap();
                }
                write!(self.output, " }}").unwrap();
            }
            HirItem::Enum(e) => {
                write!(self.output, "Enum \"{}\"", e.name).unwrap();
                self.output.push_str(" { ");
                for (i, v) in e.variants.iter().enumerate() {
                    if i > 0 { write!(self.output, ", ").unwrap(); }
                    write!(self.output, "{}", v.name).unwrap();
                }
                write!(self.output, " }}").unwrap();
            }
            HirItem::TypeAlias(t) => {
                write!(self.output, "TypeAlias \"{}\" -> {:?}", t.name, t.ty).unwrap();
            }
            HirItem::Import(i) => {
                write!(self.output, "Import from \"{}\"", i.module).unwrap();
                if !i.names.is_empty() {
                    write!(self.output, " -> {{{}}}", i.names.join(", ")).unwrap();
                }
            }
            HirItem::Export(e) => {
                write!(self.output, "Export \"{}\"", e.item_name).unwrap();
            }
        }
    }

    fn print_block(&mut self, block: &HirBlock) {
        if block.stmts.is_empty() {
            self.output.push_str(" {}");
            return;
        }
        self.output.push_str(" {\n");
        self.indent += 1;
        for stmt in &block.stmts {
            self.print_indent();
            self.print_stmt(stmt);
            self.output.push('\n');
        }
        self.indent -= 1;
        self.print_indent();
        self.output.push('}');
    }

    fn print_stmt(&mut self, stmt: &HirStmt) {
        match stmt {
            HirStmt::Let(l) => {
                if l.mutable { write!(self.output, "mut ").unwrap(); }
                write!(self.output, "{}: {:?} = ", l.name, l.ty).unwrap();
                self.print_expr(&l.value);
            }
            HirStmt::Expr(e) => self.print_expr(e),
            HirStmt::If(i) => {
                write!(self.output, "if ").unwrap();
                self.print_expr(&i.condition);
                self.print_block(&i.then_block);
                if let Some(ref else_block) = i.else_block {
                    write!(self.output, " else ").unwrap();
                    self.print_block(else_block);
                }
            }
            HirStmt::While(w) => {
                write!(self.output, "while ").unwrap();
                self.print_expr(&w.condition);
                self.print_block(&w.body);
            }
            HirStmt::Match(m) => {
                write!(self.output, "match ").unwrap();
                self.print_expr(&m.expr);
                self.output.push_str(" { ... }");
            }
            HirStmt::Return(Some(e)) => {
                write!(self.output, "return ").unwrap();
                self.print_expr(e);
            }
            HirStmt::Return(None) => write!(self.output, "return").unwrap(),
            HirStmt::Block(b) => self.print_block(b),
            HirStmt::Break => write!(self.output, "break").unwrap(),
            HirStmt::Continue => write!(self.output, "continue").unwrap(),
            HirStmt::Try(t) => {
                write!(self.output, "try ").unwrap();
                self.print_block(&t.try_block);
                write!(self.output, " catch({}) ", t.catch_var).unwrap();
                self.print_block(&t.catch_block);
            }
            HirStmt::Throw(e) => {
                write!(self.output, "throw ").unwrap();
                self.print_expr(e);
            }
        }
    }

    fn print_expr(&mut self, expr: &HirExpr) {
        match expr {
            HirExpr::Number(n, ty) => write!(self.output, "{}: {:?}", n, ty).unwrap(),
            HirExpr::String(s, _) => write!(self.output, "\"{}\"", s).unwrap(),
            HirExpr::Bool(b, _) => write!(self.output, "{}", if *b { "true" } else { "false" }).unwrap(),
            HirExpr::Null(_) => write!(self.output, "null").unwrap(),
            HirExpr::Ident(name, ty) => write!(self.output, "{}: {:?}", name, ty).unwrap(),
            HirExpr::Binary(b) => {
                write!(self.output, "(").unwrap();
                self.print_expr(&b.lhs);
                write!(self.output, " {:?} ", b.op).unwrap();
                self.print_expr(&b.rhs);
                write!(self.output, ")").unwrap();
            }
            HirExpr::Unary(u) => {
                write!(self.output, "{:?}", u.op).unwrap();
                self.print_expr(&u.expr);
            }
            HirExpr::Assign(a) => {
                self.print_expr(&a.target);
                write!(self.output, " = ").unwrap();
                self.print_expr(&a.value);
            }
            HirExpr::Ternary(t) => {
                self.print_expr(&t.condition);
                write!(self.output, " ? ").unwrap();
                self.print_expr(&t.then_expr);
                write!(self.output, " : ").unwrap();
                self.print_expr(&t.else_expr);
            }
            HirExpr::Call(c) => {
                self.print_expr(&c.callee);
                write!(self.output, "(").unwrap();
                for (i, arg) in c.args.iter().enumerate() {
                    if i > 0 { write!(self.output, ", ").unwrap(); }
                    self.print_expr(arg);
                }
                write!(self.output, ")").unwrap();
            }
            HirExpr::Member(m) => {
                self.print_expr(&m.object);
                write!(self.output, ".{}", m.field).unwrap();
            }
            HirExpr::Index(i) => {
                self.print_expr(&i.object);
                write!(self.output, "[").unwrap();
                self.print_expr(&i.index);
                write!(self.output, "]").unwrap();
            }
            HirExpr::Array(items, _) => {
                write!(self.output, "[").unwrap();
                for (i, item) in items.iter().enumerate() {
                    if i > 0 { write!(self.output, ", ").unwrap(); }
                    self.print_expr(item);
                }
                write!(self.output, "]").unwrap();
            }
            HirExpr::StructInit(s) => {
                write!(self.output, "{} {{ ", s.name).unwrap();
                for (i, f) in s.fields.iter().enumerate() {
                    if i > 0 { write!(self.output, ", ").unwrap(); }
                    write!(self.output, "{}: ", f.name).unwrap();
                    self.print_expr(&f.value);
                }
                write!(self.output, " }}").unwrap();
            }
            HirExpr::JsxElement(e) => {
                write!(self.output, "h(\"{}\", {{", e.tag).unwrap();
                for (i, (name, val)) in e.attrs.iter().enumerate() {
                    if i > 0 { write!(self.output, ", ").unwrap(); }
                    write!(self.output, "{}: ", name).unwrap();
                    self.print_expr(val);
                }
                write!(self.output, "}}, [").unwrap();
                for (i, child) in e.children.iter().enumerate() {
                    if i > 0 { write!(self.output, ", ").unwrap(); }
                    self.print_expr(child);
                }
                write!(self.output, "])").unwrap();
            }
            HirExpr::HookState(h) => {
                write!(self.output, "HookState({}, {}, {:?})", h.state_var, h.setter_var, h.ty).unwrap();
            }
            HirExpr::HookEffect(_) => {
                write!(self.output, "HookEffect(...)").unwrap();
            }
            HirExpr::HookMemo(h) => {
                write!(self.output, "HookMemo({}, {:?})", h.result_var, h.ty).unwrap();
            }
            HirExpr::Block(b) => {
                self.print_block(b);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ty::TypeInfo;

    #[test]
    fn test_pretty_print_function() {
        let program = HirProgram {
            items: vec![
                HirItem::Function(HirFunction {
                    name: "main".into(),
                    params: vec![],
                    return_ty: TypeInfo::I32,
                    body: HirBlock {
                        stmts: vec![
                            HirStmt::Expr(HirExpr::Number(42.0, TypeInfo::I32)),
                        ],
                    },
                    type_params: vec![],
                }),
            ],
        };
        let mut printer = HirPrettyPrinter::new();
        let output = printer.print_program(&program);
        assert!(output.contains("Function \"main\""));
        assert!(output.contains("42: I32"));
    }
}
