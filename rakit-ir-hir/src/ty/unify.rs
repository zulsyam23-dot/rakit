use super::*;

/// Type unification: checks if two types are compatible with optional widening.
pub fn unify_types(expected: &TypeInfo, actual: &TypeInfo) -> bool {
    match (expected, actual) {
        (a, b) if a == b => true,
        (TypeInfo::Infer, _) | (_, TypeInfo::Infer) => true,
        (TypeInfo::Error, _) | (_, TypeInfo::Error) => true,
        (TypeInfo::I32, TypeInfo::I64) => true,
        (TypeInfo::I64, TypeInfo::I32) => true,
        (TypeInfo::U32, TypeInfo::U64) => true,
        (TypeInfo::U64, TypeInfo::U32) => true,
        (TypeInfo::F32, TypeInfo::F64) => true,
        (TypeInfo::F64, TypeInfo::F32) => true,
        (TypeInfo::Optional(a), _b) if **a == TypeInfo::Infer => true,
        (TypeInfo::Optional(a), b) if a.as_ref() == b => true,
        (a, TypeInfo::Optional(b)) if a == b.as_ref() => true,
        (TypeInfo::Array(a), TypeInfo::Array(b)) => unify_types(a, b),
        (TypeInfo::Fn(a), TypeInfo::Fn(b)) => {
            a.params.len() == b.params.len()
                && a.params.iter().zip(&b.params).all(|(x, y)| unify_types(x, y))
                && unify_types(&a.ret, &b.ret)
        }
        (TypeInfo::Struct(a), TypeInfo::Struct(b)) => a.name == b.name,
        (TypeInfo::Enum(a), TypeInfo::Enum(b)) => a.name == b.name,
        (TypeInfo::Tuple(a), TypeInfo::Tuple(b)) => {
            a.len() == b.len() && a.iter().zip(b).all(|(x, y)| unify_types(x, y))
        }
        (TypeInfo::Generic(_), _) | (_, TypeInfo::Generic(_)) => true,
        (TypeInfo::TypeParam(_), _) | (_, TypeInfo::TypeParam(_)) => true,
        _ => false,
    }
}

/// Widen numeric types to a common type.
pub fn widen_numeric(a: &TypeInfo, b: &TypeInfo) -> TypeInfo {
    use TypeInfo::*;
    if a == b { return a.clone(); }
    match (a, b) {
        (F64, _) | (_, F64) => F64,
        (F32, _) | (_, F32) => F32,
        (I64, _) | (_, I64) => I64,
        (U64, _) | (_, U64) => U64,
        (I32, U32) | (U32, I32) => I64,
        _ => F64,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unify_identical() {
        assert!(unify_types(&TypeInfo::I32, &TypeInfo::I32));
    }

    #[test]
    fn test_unify_widening() {
        assert!(unify_types(&TypeInfo::I32, &TypeInfo::I64));
        assert!(unify_types(&TypeInfo::F32, &TypeInfo::F64));
    }

    #[test]
    fn test_unify_with_infer() {
        assert!(unify_types(&TypeInfo::Infer, &TypeInfo::I32));
        assert!(unify_types(&TypeInfo::String, &TypeInfo::Infer));
    }

    #[test]
    fn test_unify_optional() {
        assert!(unify_types(&TypeInfo::Optional(Box::new(TypeInfo::String)), &TypeInfo::String));
    }

    #[test]
    fn test_unify_structural() {
        let a = TypeInfo::Array(Box::new(TypeInfo::I32));
        let b = TypeInfo::Array(Box::new(TypeInfo::I32));
        assert!(unify_types(&a, &b));
    }

    #[test]
    fn test_unify_mismatch() {
        assert!(!unify_types(&TypeInfo::I32, &TypeInfo::String));
        assert!(!unify_types(&TypeInfo::Bool, &TypeInfo::I64));
    }
}
