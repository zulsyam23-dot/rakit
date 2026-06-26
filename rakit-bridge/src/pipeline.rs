use rakit_ir_hir::hir::HirProgram;
use brak_ir_lir::lower::LirLower;
use brak_codegen_wasm::WasmBackend;
use brak_codegen_c::CBackend;
use brak_codegen_obj::ObjBackend;
use brak_codegen_traits::CodegenBackend;
use brak_link_traits::{LinkerBackend, ObjectFile};
use brak_link_native::NativeLinker;
use rakit_core::{Result, Diagnostic};
use crate::hir_to_mir::RakitToBrakBridge;

fn bridge_err(e: impl std::fmt::Display) -> Vec<Diagnostic> {
    vec![Diagnostic::error(format!("Bridge error: {}", e))]
}

fn codegen_err(e: impl std::fmt::Display) -> Vec<Diagnostic> {
    vec![Diagnostic::error(format!("Codegen error: {}", e))]
}

fn linker_err(e: impl std::fmt::Display) -> Vec<Diagnostic> {
    vec![Diagnostic::error(format!("Linker error: {}", e))]
}

pub struct RakitCompiler;

impl RakitCompiler {
    pub fn new() -> Self {
        RakitCompiler
    }

    /// Full pipeline: Rakit HIR → Brak MIR → Brak LIR → WASM
    pub fn compile_to_wasm(&self, hir: &HirProgram) -> Result<String> {
        let mut bridge = RakitToBrakBridge::new();
        let mir = bridge.convert_program(hir).map_err(bridge_err)?;

        let mut lower = LirLower::new();
        let lir = lower.lower(mir);

        let backend = WasmBackend;
        let wasm_bytes = backend.emit(&lir).map_err(codegen_err)?;
        let wasm_text = String::from_utf8_lossy(&wasm_bytes).to_string();

        Ok(wasm_text)
    }

    /// Full pipeline: Rakit HIR → Brak MIR → Brak LIR → C code
    pub fn compile_to_c(&self, hir: &HirProgram) -> Result<String> {
        let mut bridge = RakitToBrakBridge::new();
        let mir = bridge.convert_program(hir).map_err(bridge_err)?;

        let mut lower = LirLower::new();
        let lir = lower.lower(mir);

        let backend = CBackend;
        let c_bytes = backend.emit(&lir).map_err(codegen_err)?;
        let c_text = String::from_utf8_lossy(&c_bytes).to_string();

        Ok(c_text)
    }

    /// Pipeline stages for debugging
    pub fn dump_mir(&self, hir: &HirProgram) -> Result<String> {
        let mut bridge = RakitToBrakBridge::new();
        let mir = bridge.convert_program(hir).map_err(bridge_err)?;
        Ok(format!("{:#?}", mir))
    }

    pub fn dump_lir(&self, hir: &HirProgram) -> Result<String> {
        let mut bridge = RakitToBrakBridge::new();
        let mir = bridge.convert_program(hir).map_err(bridge_err)?;
        let mut lower = LirLower::new();
        let lir = lower.lower(mir);
        Ok(format!("{:#?}", lir))
    }

    /// Full pipeline: Rakit HIR → Brak MIR → Brak LIR → Native object → PE/ELF executable
    pub fn compile_to_native(&self, hir: &HirProgram, entry: &str) -> Result<Vec<u8>> {
        let mut bridge = RakitToBrakBridge::new();
        let mir = bridge.convert_program(hir).map_err(bridge_err)?;

        let mut lower = LirLower::new();
        let lir = lower.lower(mir);

        let obj_backend = ObjBackend::default();
        let obj_bytes = obj_backend.emit(&lir).map_err(codegen_err)?;

        let linker = NativeLinker;
        let obj_file = ObjectFile {
            name: format!("{}.o", entry),
            data: obj_bytes,
        };
        let output = linker.link(&[obj_file], entry, 0x400000).map_err(linker_err)?;

        Ok(output.data)
    }
}
