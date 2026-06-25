use rakit_ir_hir::hir::HirProgram;
use brak_ir_mir::mir::MirProgram;
use brak_ir_lir::lir::LirProgram;
use brak_ir_lir::lower::LirLower;
use brak_codegen_wasm::WasmBackend;
use brak_codegen_c::CBackend;
use brak_codegen_traits::CodegenBackend;
use brak_core::Result;
use crate::hir_to_mir::RakitToBrakBridge;
use crate::error::BridgeError;

pub struct RakitCompiler;

impl RakitCompiler {
    pub fn new() -> Self {
        RakitCompiler
    }

    /// Full pipeline: Rakit HIR → Brak MIR → Brak LIR → WASM
    pub fn compile_to_wasm(&self, hir: &HirProgram) -> Result<String> {
        // Step 1: Rakit HIR → Brak MIR
        let mut bridge = RakitToBrakBridge::new();
        let mir = bridge.convert_program(hir)
            .map_err(|e| -> Box<dyn std::error::Error> { Box::new(e) })?;

        // Step 2: Brak MIR → Brak LIR
        let mut lower = LirLower::new();
        let lir = lower.lower(mir);

        // Step 3: Brak LIR → WASM text
        let backend = WasmBackend;
        let wasm_bytes = backend.emit(&lir)?;
        let wasm_text = String::from_utf8_lossy(&wasm_bytes).to_string();

        Ok(wasm_text)
    }

    /// Full pipeline: Rakit HIR → Brak MIR → Brak LIR → C code
    pub fn compile_to_c(&self, hir: &HirProgram) -> Result<String> {
        // Step 1: Rakit HIR → Brak MIR
        let mut bridge = RakitToBrakBridge::new();
        let mir = bridge.convert_program(hir)
            .map_err(|e| -> Box<dyn std::error::Error> { Box::new(e) })?;

        // Step 2: Brak MIR → Brak LIR
        let mut lower = LirLower::new();
        let lir = lower.lower(mir);

        // Step 3: Brak LIR → C code
        let backend = CBackend;
        let c_bytes = backend.emit(&lir)?;
        let c_text = String::from_utf8_lossy(&c_bytes).to_string();

        Ok(c_text)
    }

    /// Pipeline stages for debugging
    pub fn dump_mir(&self, hir: &HirProgram) -> Result<String> {
        let mut bridge = RakitToBrakBridge::new();
        let mir = bridge.convert_program(hir)
            .map_err(|e| -> Box<dyn std::error::Error> { Box::new(e) })?;
        Ok(format!("{:#?}", mir))
    }

    pub fn dump_lir(&self, hir: &HirProgram) -> Result<String> {
        let mut bridge = RakitToBrakBridge::new();
        let mir = bridge.convert_program(hir)
            .map_err(|e| -> Box<dyn std::error::Error> { Box::new(e) })?;
        let mut lower = LirLower::new();
        let lir = lower.lower(mir);
        Ok(format!("{:#?}", lir))
    }
}
