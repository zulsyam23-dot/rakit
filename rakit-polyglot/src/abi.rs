use rakit_ir_hir::ty::{TypeInfo, FnType};

#[derive(Debug, Clone)]
pub enum AbiType {
    Integer,
    Float,
    Xmm,
    Pointer,
}

#[derive(Debug, Clone)]
pub struct NormalizedParam {
    pub original_type: TypeInfo,
    pub abi_type: AbiType,
    pub reg: Option<String>,
}

#[derive(Debug, Clone)]
pub struct NormalizedFn {
    pub params: Vec<NormalizedParam>,
    pub return_abi: AbiType,
    pub caller_saves: Vec<&'static str>,
}

/// Normalisasi calling convention untuk cross-language calls.
pub struct RakitAbi;

impl RakitAbi {
    pub const SYSV_ABI: &'static str = "sysv";
    pub const MS_ABI: &'static str = "ms";

    pub fn for_target(target: &str) -> &'static str {
        match target {
            "win32" | "windows" => Self::MS_ABI,
            "linux" | "macos" | "wasm" => Self::SYSV_ABI,
            _ => Self::SYSV_ABI,
        }
    }

    pub fn normalize_signature(sig: &FnType, abi: &str) -> NormalizedFn {
        let mut normalized_params = Vec::new();

        for param in &sig.params {
            match param {
                TypeInfo::F32 | TypeInfo::F64 if abi == Self::SYSV_ABI => {
                    normalized_params.push(NormalizedParam {
                        original_type: param.clone(),
                        abi_type: AbiType::Xmm,
                        reg: None,
                    });
                }
                TypeInfo::Struct(s) if size_of_struct(s) > 16 => {
                    normalized_params.push(NormalizedParam {
                        original_type: param.clone(),
                        abi_type: AbiType::Pointer,
                        reg: None,
                    });
                }
                _ => {
                    normalized_params.push(NormalizedParam {
                        original_type: param.clone(),
                        abi_type: AbiType::Integer,
                        reg: None,
                    });
                }
            }
        }

        let return_abi = match sig.ret.as_ref() {
            TypeInfo::F32 | TypeInfo::F64 if abi == Self::SYSV_ABI => AbiType::Xmm,
            TypeInfo::Struct(s) if size_of_struct(s) > 16 => AbiType::Pointer,
            _ => AbiType::Integer,
        };

        NormalizedFn {
            params: normalized_params,
            return_abi,
            caller_saves: vec!["RAX", "RCX", "RDX", "RSI", "RDI", "R8", "R9", "R10", "R11"],
        }
    }
}

fn size_of_struct(s: &rakit_ir_hir::ty::StructType) -> usize {
    let mut size = 0;
    for field in &s.fields {
        size += type_size(&field.ty);
    }
    size
}

fn type_size(ty: &TypeInfo) -> usize {
    match ty {
        TypeInfo::I32 | TypeInfo::U32 | TypeInfo::F32 => 4,
        TypeInfo::I64 | TypeInfo::U64 | TypeInfo::F64 => 8,
        TypeInfo::Bool => 1,
        TypeInfo::String | TypeInfo::Node => 8,
        TypeInfo::Void => 0,
        TypeInfo::Array(inner) => 8 + type_size(inner) * 4,
        TypeInfo::Optional(inner) => type_size(inner),
        TypeInfo::Struct(s) => s.fields.iter().map(|f| type_size(&f.ty)).sum(),
        _ => 8,
    }
}
