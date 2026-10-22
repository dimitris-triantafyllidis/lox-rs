use std::{env, fs, io, process::ExitCode};

mod lexer;
mod parser;
mod interpreter;
mod context;
mod semantic_pass;

use crate::lexer::*;
use crate::parser::*;
use crate::interpreter::*;
use crate::semantic_pass::*;

fn main() -> ExitCode {

    println! ();
    println! ("===== Benchmark: 'binary_trees.lox' =====");
    run_file ( &"test/benchmark/binary_trees.lox"    .to_string() );
    println! ();

    println! ("===== Benchmark: 'equality.lox' =====");
    run_file ( &"test/benchmark/equality.lox"        .to_string() );
    println! ();

    println! ("===== Benchmark: 'fib.lox' =====");
    run_file ( &"test/benchmark/fib.lox"             .to_string() );
    println! ();

    println! ("===== Benchmark: 'instantiation.lox' =====");
    run_file ( &"test/benchmark/instantiation.lox"   .to_string() );
    println! ();

    println! ("===== Benchmark: 'invocation.lox' =====");
    run_file ( &"test/benchmark/invocation.lox"      .to_string() );
    println! ();

    println! ("===== Benchmark: 'method_call.lox' =====");
    run_file ( &"test/benchmark/method_call.lox"     .to_string() );
    println! ();

    println! ("===== Benchmark: 'properties.lox' =====");
    run_file ( &"test/benchmark/properties.lox"      .to_string() );
    println! ();

    println! ("===== Benchmark: 'string_equality.lox' =====");
    run_file ( &"test/benchmark/string_equality.lox" .to_string() );
    println! ();

    println! ("===== Benchmark: 'trees.lox' =====");
    run_file ( &"test/benchmark/trees.lox"           .to_string() );
    println! ();

    println! ("===== Benchmark: 'zoo_batch.lox' =====");
    run_file ( &"test/benchmark/zoo_batch.lox"       .to_string() );
    println! ();

    println! ("===== Benchmark: 'zoo.lox' =====");
    run_file ( &"test/benchmark/zoo.lox"             .to_string() );
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
            let mut interpreter = Interpreter::new();
            interpreter.execute(&syntax_tree);
        },
        io::Result::Err(e) => {
            eprintln!("io error: {}", e);
        }
    }

}
