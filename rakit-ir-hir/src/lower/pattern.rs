use rakit_ir_ast as ast;
use crate::hir::*;
use crate::ty::TypeInfo;

/// Lower AST pattern to HIR pattern.
pub fn lower_pattern(pattern: &ast::Pattern) -> HirPattern {
    match pattern {
        ast::Pattern::Wildcard => HirPattern::Wildcard,
        ast::Pattern::Literal(lit) => HirPattern::Literal(match lit {
            ast::Literal::Number(n) => HirLiteral::Number(*n),
            ast::Literal::String(s) => HirLiteral::String(s.clone()),
            ast::Literal::Bool(b) => HirLiteral::Bool(*b),
            ast::Literal::Null => HirLiteral::Null,
            ast::Literal::Char(c) => HirLiteral::String(c.to_string()),
        }),
        ast::Pattern::Ident(name) => HirPattern::Ident(name.clone()),
        ast::Pattern::Struct { name, fields } => {
            HirPattern::Struct {
                name: name.clone(),
                fields: fields.iter().map(|(n, p)| (n.clone(), lower_pattern(p))).collect(),
            }
        }
        ast::Pattern::Enum { name, variant, fields } => {
            HirPattern::Enum {
                name: name.clone(),
                variant: variant.clone(),
                fields: fields.iter().map(lower_pattern).collect(),
            }
        }
    }
}

/// Desugar `cocok` pattern matching into nested `jika` statements.
/// This transforms:
///   cocok expr {
///     Pola1 => body1,
///     Pola2 => body2,
///   }
/// into nested if-else chains.
pub fn desugar_match(expr: &HirExpr, arms: &[HirMatchArm]) -> HirExpr {
    if arms.is_empty() {
        return HirExpr::Null(TypeInfo::Infer);
    }

    let mut result = build_arm_expr(expr, &arms[0]);
    for arm in &arms[1..] {
        result = build_else_arm(expr, arm, result);
    }
    result
}

fn build_arm_expr(scrutinee: &HirExpr, arm: &HirMatchArm) -> HirExpr {
    let cond = pattern_to_condition(scrutinee, &arm.pattern);
    HirExpr::Ternary(HirTernary {
        condition: Box::new(cond),
        then_expr: Box::new(arm.body.clone()),
        else_expr: Box::new(HirExpr::Null(TypeInfo::Infer)),
        ty: TypeInfo::Infer,
    })
}

fn build_else_arm(scrutinee: &HirExpr, arm: &HirMatchArm, prev: HirExpr) -> HirExpr {
    let cond = pattern_to_condition(scrutinee, &arm.pattern);
    HirExpr::Ternary(HirTernary {
        condition: Box::new(cond),
        then_expr: Box::new(arm.body.clone()),
        else_expr: Box::new(prev),
        ty: TypeInfo::Infer,
    })
}

fn pattern_to_condition(scrutinee: &HirExpr, pattern: &HirPattern) -> HirExpr {
    match pattern {
        HirPattern::Wildcard => HirExpr::Bool(true, TypeInfo::Bool),
        HirPattern::Literal(lit) => {
            let lit_expr = match lit {
                HirLiteral::Number(n) => HirExpr::Number(*n, TypeInfo::F64),
                HirLiteral::String(s) => HirExpr::String(s.clone(), TypeInfo::String),
                HirLiteral::Bool(b) => HirExpr::Bool(*b, TypeInfo::Bool),
                HirLiteral::Null => HirExpr::Null(TypeInfo::Infer),
            };
            HirExpr::Binary(HirBinary {
                op: HirBinaryOp::Eq,
                lhs: Box::new(scrutinee.clone()),
                rhs: Box::new(lit_expr),
                ty: TypeInfo::Bool,
            })
        }
        HirPattern::Ident(_) => HirExpr::Bool(true, TypeInfo::Bool),
        HirPattern::Struct { name: _, fields: _ } => {
            HirExpr::Bool(true, TypeInfo::Bool)
        }
        HirPattern::Enum { name: _, variant: _, fields: _ } => {
            HirExpr::Bool(true, TypeInfo::Bool)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wildcard_pattern() {
        let p = ast::Pattern::Wildcard;
        let hir = lower_pattern(&p);
        assert!(matches!(hir, HirPattern::Wildcard));
    }

    #[test]
    fn test_ident_pattern() {
        let p = ast::Pattern::Ident("x".into());
        let hir = lower_pattern(&p);
        assert!(matches!(hir, HirPattern::Ident(name) if name == "x"));
    }
}
