use std::time::Instant;
use rakit_frontend::{Lexer, Parser};
use rakit_ir_hir::hir::HirProgram;
use rakit_ir_hir::lower::Lower;

pub fn bench_parse_simple_program() -> usize {
    let source = "fungsi main() -> I32 {\n    0\n}";

    let start = Instant::now();
    let mut lexer = Lexer::new(source);
    let tokens = lexer.tokenize();
    let mut parser = Parser::new(tokens, source);
    let _program = parser.parse_program();
    start.elapsed().as_micros() as usize
}

pub fn bench_full_compile_small() -> usize {
    let source = "\
fungsi tambah(a: I32, b: I32) -> I32 {
    a + b
}

komponen Tombol(props: { teks: String, onClick: () -> Void }) {
    tampilkan {
        <button onClick={props.onClick}>
            {props.teks}
        </button>
    }
}

fungsi main() -> I32 {
    tambah(1, 2)
    0
}";

    let start = Instant::now();

    let mut lexer = Lexer::new(source);
    let tokens = lexer.tokenize();
    let mut parser = Parser::new(tokens, source);
    let ast = parser.parse_program();

    let mut lower = Lower::new();
    let _hir = lower.lower_program(&ast);

    start.elapsed().as_micros() as usize
}

pub fn run_all_compile_benches() {
    let us = bench_parse_simple_program();
    println!("parse simple program: {} us", us);

    let us = bench_full_compile_small();
    println!("full compile small: {} us", us);
}
