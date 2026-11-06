use std::{fs, io, process::ExitCode};

use lox_rs::lexer::*;
use lox_rs::parser::*;
use lox_rs::interpreter::*;
use lox_rs::semantic_pass::*;

fn main() -> ExitCode {

    println! ();
    println! ("===== Benchmark: 'binary_trees.lox' =====");
    run_file ( &"tests/benchmark/binary_trees.lox"    .to_string() );
    println! ();

    println! ("===== Benchmark: 'equality.lox' =====");
    run_file ( &"tests/benchmark/equality.lox"        .to_string() );
    println! ();

    println! ("===== Benchmark: 'fib.lox' =====");
    run_file ( &"tests/benchmark/fib.lox"             .to_string() );
    println! ();

    println! ("===== Benchmark: 'instantiation.lox' =====");
    run_file ( &"tests/benchmark/instantiation.lox"   .to_string() );
    println! ();

    println! ("===== Benchmark: 'invocation.lox' =====");
    run_file ( &"tests/benchmark/invocation.lox"      .to_string() );
    println! ();

    println! ("===== Benchmark: 'method_call.lox' =====");
    run_file ( &"tests/benchmark/method_call.lox"     .to_string() );
    println! ();

    println! ("===== Benchmark: 'properties.lox' =====");
    run_file ( &"tests/benchmark/properties.lox"      .to_string() );
    println! ();

    println! ("===== Benchmark: 'string_equality.lox' =====");
    run_file ( &"tests/benchmark/string_equality.lox" .to_string() );
    println! ();

    println! ("===== Benchmark: 'trees.lox' =====");
    run_file ( &"tests/benchmark/trees.lox"           .to_string() );
    println! ();

    println! ("===== Benchmark: 'zoo_batch.lox' =====");
    run_file ( &"tests/benchmark/zoo_batch.lox"       .to_string() );
    println! ();

    println! ("===== Benchmark: 'zoo.lox' =====");
    run_file ( &"tests/benchmark/zoo.lox"             .to_string() );
    println! ();

    return ExitCode::from(0);
}

fn run_file(file_path: &String) {

    match fs::read_to_string(file_path) {
        io::Result::Ok(s) => {
            let tokens = lexer_scan(&s);
            let mut syntax_tree = parse(&tokens);
            let mut semantic_pass = SemanticPass::new();
            semantic_pass.visit(&mut syntax_tree);
            let defs = semantic_pass.defs.clone();
            let mut interpreter = Interpreter::new(semantic_pass.context);
            interpreter.execute(&syntax_tree, &defs);
        },
        io::Result::Err(e) => {
            eprintln!("io error: {}", e);
        }
    }

}
