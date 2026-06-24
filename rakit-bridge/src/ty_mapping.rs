use rakit_ir_hir::ty::*;
use crate::brak_types::BrakTy;

/// Map Rakit TypeInfo to Brak IR type.
pub fn convert_type(rakit_ty: &TypeInfo) -> BrakTy {
    match rakit_ty {
        TypeInfo::I32 => BrakTy::Int(32),
        TypeInfo::I64 => BrakTy::Int(64),
        TypeInfo::U32 => BrakTy::UInt(32),
        TypeInfo::U64 => BrakTy::UInt(64),
        TypeInfo::F32 => BrakTy::Float(32),
        TypeInfo::F64 => BrakTy::Float(64),
        TypeInfo::Bool => BrakTy::Bool,
        TypeInfo::String => BrakTy::Pointer(Box::new(BrakTy::U8)),
        TypeInfo::Char => BrakTy::UInt(32),
        TypeInfo::Void => BrakTy::Void,
        TypeInfo::Node => BrakTy::Pointer(Box::new(BrakTy::Void)),
        TypeInfo::Array(inner) => BrakTy::Array(Box::new(convert_type(inner))),
        TypeInfo::Optional(inner) => BrakTy::Optional(Box::new(convert_type(inner))),
        TypeInfo::Fn(fn_ty) => {
            let params: Vec<BrakTy> = fn_ty.params.iter().map(convert_type).collect();
            BrakTy::Fn(params, Box::new(convert_type(&fn_ty.ret)))
        }
        TypeInfo::Struct(struct_ty) => {
            let fields: Vec<(String, BrakTy)> = struct_ty.fields.iter()
                .map(|f| (f.name.clone(), convert_type(&f.ty)))
                .collect();
            BrakTy::Struct(fields)
        }
        TypeInfo::Enum(enum_ty) => {
            let variants: Vec<String> = enum_ty.variants.iter()
                .map(|v| v.name.clone())
                .collect();
            BrakTy::Enum(variants)
        }
        TypeInfo::Tuple(tys) => {
            BrakTy::Struct(tys.iter().enumerate()
                .map(|(i, t)| (format!("_{}", i), convert_type(t)))
                .collect())
        }
        TypeInfo::Generic(name) => BrakTy::Named(name.clone()),
        TypeInfo::TypeParam(_) | TypeInfo::Infer => BrakTy::Named("_".into()),
        TypeInfo::Error => BrakTy::Named("Error".into()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_primitive_mapping() {
        assert_eq!(convert_type(&TypeInfo::I32), BrakTy::Int(32));
        assert_eq!(convert_type(&TypeInfo::Bool), BrakTy::Bool);
        assert_eq!(convert_type(&TypeInfo::Void), BrakTy::Void);
    }

    #[test]
    fn test_array_mapping() {
        let ty = TypeInfo::Array(Box::new(TypeInfo::I32));
        assert_eq!(
            convert_type(&ty),
            BrakTy::Array(Box::new(BrakTy::Int(32)))
        );
    }

    #[test]
    fn test_fn_mapping() {
        let ty = TypeInfo::Fn(FnType::new(
            vec![TypeInfo::I32, TypeInfo::String],
            TypeInfo::Bool,
        ));
        let brak = convert_type(&ty);
        assert!(matches!(brak, BrakTy::Fn(..)));
    }
}
