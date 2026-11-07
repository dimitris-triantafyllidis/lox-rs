use std::{fs, io, process::ExitCode};

use lox_rs::{lexer, parser, interpreter, semantic_pass};

fn main() -> ExitCode {

    println!();
    println!("===== Benchmark: 'binary_trees.lox' =====");
    run_file("tests/benchmark/binary_trees.lox");
    println!();

    println!("===== Benchmark: 'equality.lox' =====");
    run_file("tests/benchmark/equality.lox");
    println!();

    println!("===== Benchmark: 'fib.lox' =====");
    run_file("tests/benchmark/fib.lox");
    println!();

    println!("===== Benchmark: 'instantiation.lox' =====");
    run_file("tests/benchmark/instantiation.lox");
    println!();

    println!("===== Benchmark: 'invocation.lox' =====");
    run_file("tests/benchmark/invocation.lox");
    println!();

    println!("===== Benchmark: 'method_call.lox' =====");
    run_file("tests/benchmark/method_call.lox");
    println!();

    println!("===== Benchmark: 'properties.lox' =====");
    run_file("tests/benchmark/properties.lox");
    println!();

    println!("===== Benchmark: 'string_equality.lox' =====");
    run_file("tests/benchmark/string_equality.lox");
    println!();

    println!("===== Benchmark: 'trees.lox' =====");
    run_file("tests/benchmark/trees.lox");
    println!();

    println!("===== Benchmark: 'zoo_batch.lox' =====");
    run_file("tests/benchmark/zoo_batch.lox");
    println!();

    println!("===== Benchmark: 'zoo.lox' =====");
    run_file( "tests/benchmark/zoo.lox" );
    println!();

    ExitCode::from(0)
}

fn run_file(file_path: &str) {

    match fs::read_to_string(file_path) {
        io::Result::Ok(s) => {
            let tokens = lexer::scan(&s);
            let mut syntax_tree = parser::parse(&tokens);
            let mut semantic_pass = semantic_pass::SemanticPass::new();
            semantic_pass.visit(&mut syntax_tree);
            let defs = semantic_pass.defs.clone();
            let mut interpreter = interpreter::Interpreter::new(semantic_pass.context);
            interpreter.execute(&syntax_tree, &defs);
        },
        io::Result::Err(e) => {
            eprintln!("io error: {e}");
        }
    }

}
