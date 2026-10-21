use std::{
    fs,
    path::Path,
    process::Command,
};

fn parse_expected_output(source: &str) -> Vec<String> {
    source
        .lines()
        .filter_map(|line| {
            line.split_once("// expect:")
                .map(|(_, expected)| expected.trim().to_string())
        })
        .collect()
}

fn run_lox_test(path: &Path) {
    let source = fs::read_to_string(path)
        .expect("Failed to read Lox test");

    let expected_lines = parse_expected_output(&source);

    let output = Command::new(env!("CARGO_BIN_EXE_lox"))
        .arg(path)
        .output()
        .expect("Failed to execute interpreter");

    let actual = String::from_utf8(output.stdout)
        .expect("stdout was not valid UTF-8");

    let expected = if expected_lines.is_empty() {
        String::new()
    } else {
        format!("{}\n", expected_lines.join("\n"))
    };

    assert_eq!(
        actual,
        expected,
        "Output mismatch in {}",
        path.display()
    );
}

#[test]
fn test_assignment_associativity() {
    run_lox_test(Path::new("test/assignment/associativity.lox"));
}

#[test]
fn test_assignment_global() {
    run_lox_test(Path::new("test/assignment/global.lox"));
}

#[test]
fn test_assignment_local() {
    run_lox_test(Path::new("test/assignment/local.lox"));
}

#[test]
fn test_assignment_syntax() {
    run_lox_test(Path::new("test/assignment/syntax.lox"));
}

#[test]
fn test_assignment_to_this() {
    run_lox_test(Path::new("test/assignment/to_this.lox"));
}

#[test]
fn test_block_empty() {
    run_lox_test(Path::new("test/block/empty.lox"));
}

#[test]
fn test_block_scope() {
    run_lox_test(Path::new("test/block/scope.lox"));
}

#[test]
fn test_bool_empty() {
    run_lox_test(Path::new("test/bool/equality.lox"));
}

#[test]
fn test_bool_scope() {
    run_lox_test(Path::new("test/bool/not.lox"));
}

#[test]
fn test_call_bool() {
    run_lox_test(Path::new("test/call/bool.lox"));
}

#[test]
fn test_call_nil() {
    run_lox_test(Path::new("test/call/nil.lox"));
}

#[test]
fn test_call_num() {
    run_lox_test(Path::new("test/call/num.lox"));
}

#[test]
fn test_call_object() {
    run_lox_test(Path::new("test/call/object.lox"));
}

#[test]
fn test_call_string() {
    run_lox_test(Path::new("test/call/string.lox"));
}

#[test]
fn test_class_inherited_method() {
    run_lox_test(Path::new("test/class/inherited_method.lox"));
}
