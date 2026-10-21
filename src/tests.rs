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

#[test]
fn test_closure_assign_to_closure() {
    run_lox_test(Path::new("test/closure/assign_to_closure.lox"));
}

#[test]
fn test_closure_assign_to_shadowed_later() {
    run_lox_test(Path::new("test/closure/assign_to_shadowed_later.lox"));
}

#[test]
fn test_closure_close_over_function_parameter() {
    run_lox_test(Path::new("test/closure/close_over_function_parameter.lox"));
}

#[test]
fn test_closure_close_over_later_variable() {
    run_lox_test(Path::new("test/closure/close_over_later_variable.lox"));
}

#[test]
fn test_closure_close_over_method_parameter() {
    run_lox_test(Path::new("test/closure/close_over_method_parameter.lox"));
}

#[test]
fn test_closure_closed_closure_in_function() {
    run_lox_test(Path::new("test/closure/closed_closure_in_function.lox"));
}

#[test]
fn test_closure_nested_closure() {
    run_lox_test(Path::new("test/closure/nested_closure.lox"));
}

#[test]
fn test_closure_open_closure_in_function() {
    run_lox_test(Path::new("test/closure/open_closure_in_function.lox"));
}

#[test]
fn test_closure_reference_closure_multiple_times() {
    run_lox_test(Path::new("test/closure/reference_closure_multiple_times.lox"));
}

#[test]
fn test_closure_reuse_closure_slot() {
    run_lox_test(Path::new("test/closure/reuse_closure_slot.lox"));
}

#[test]
fn test_closure_shadow_closure_with_local() {
    run_lox_test(Path::new("test/closure/shadow_closure_with_local.lox"));
}

#[test]
fn test_closure_unused_closure() {
    run_lox_test(Path::new("test/closure/unused_closure.lox"));
}

#[test]
fn test_closure_unused_later_closure() {
    run_lox_test(Path::new("test/closure/unused_later_closure.lox"));
}

#[test]
fn test_comments_line_at_eof() {
    run_lox_test(Path::new("test/comments/line_at_eof.lox"));
}

#[test]
fn test_comments_only_line_comment_and_line() {
    run_lox_test(Path::new("test/comments/only_line_comment_and_line.lox"));
}

#[test]
fn test_comments_only_line_comment() {
    run_lox_test(Path::new("test/comments/only_line_comment.lox"));
}

#[test]
fn test_comments_unicode() {
    run_lox_test(Path::new("test/comments/unicode.lox"));
}
