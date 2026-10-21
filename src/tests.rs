

use std::{
    fs,
    path::Path,
    process::Command,
};

struct CompileError {
    token: String,
    message: String,
}

struct Expectations {
    output: Vec<String>,
    runtime_error: Option<String>,
    compile_error: Option<CompileError>,
}

fn parse_expectations(source: &str) -> Expectations {
    let mut output = Vec::new();
    let mut runtime_error = None;
    let mut compile_error = None;

    for line in source.lines() {
        if let Some((_, expected)) =
            line.split_once("// expect runtime error:")
        {
            runtime_error = Some(expected.trim().to_string());
        } else if let Some((_, annotation)) =
            line.split_once("// ")
        {
            let annotation = annotation
                .strip_prefix("[line ")
                .and_then(|s| s.split_once("] "))
                .map_or(annotation, |(_, rest)| rest);

            if let Some(error) = annotation.strip_prefix("Error at ") {
                if let Some((token, message)) = error.split_once(": ") {
                    compile_error = Some(CompileError {
                        token: token.trim().trim_matches('\'').to_string(),
                        message: message.trim().to_string(),
                    });
                }
            } else if let Some((_, expected)) =
                line.split_once("// expect:")
            {
                output.push(expected.trim().to_string());
            }
        }
    }

    Expectations {
        output,
        runtime_error,
        compile_error,
    }
}

fn run_lox_test(path: &Path) {
    let source = fs::read_to_string(path)
        .expect("Failed to read Lox test");

    let expectations = parse_expectations(&source);

    let output = Command::new(env!("CARGO_BIN_EXE_lox"))
        .arg(path)
        .output()
        .expect("Failed to execute interpreter");

    let stdout = String::from_utf8(output.stdout)
        .expect("stdout was not valid UTF-8");

    let stderr = String::from_utf8(output.stderr)
        .expect("stderr was not valid UTF-8");

    if let Some(expected_error) = expectations.runtime_error {
        assert!(
            !output.status.success(),
            "Expected runtime error '{}' in {}, but interpreter exited successfully.\nstdout:\n{}",
            expected_error,
            path.display(),
            stdout
        );

        assert!(
            stderr.contains(&expected_error),
            "Expected error message '{}' in {}, but got stderr:\n{}",
            expected_error,
            path.display(),
            stderr
        );
    } else if let Some(CompileError { token, message }) =
        expectations.compile_error
    {
        assert!(
            !output.status.success(),
            "Expected compile error in {}, but interpreter exited successfully.\nstdout:\n{}",
            path.display(),
            stdout
        );

        assert!(
            stderr.contains(&token) && stderr.contains(&message),
            "Expected compile error at token '{}' with message '{}' in {}, but got stderr:\n{}",
            token,
            message,
            path.display(),
            stderr
        );
    } else {
        assert!(
            output.status.success(),
            "Interpreter failed on {}.\nstdout:\n{}\nstderr:\n{}",
            path.display(),
            stdout,
            stderr
        );

        let expected = if expectations.output.is_empty() {
            String::new()
        } else {
            format!("{}\n", expectations.output.join("\n"))
        };

        assert_eq!(
            stdout,
            expected,
            "Output mismatch in {}",
            path.display()
        );
    }
}

// assignment

#[test]
fn test_assignment_associativity() {
    run_lox_test(Path::new("test/assignment/associativity.lox"));
}

#[test]
fn test_assignment_global() {
    run_lox_test(Path::new("test/assignment/global.lox"));
}

#[test]
fn test_assignment_grouping() {
    run_lox_test(Path::new("test/assignment/grouping.lox"));
}

#[test]
fn test_assignment_infix_operator() {
    run_lox_test(Path::new("test/assignment/infix_operator.lox"));
}

#[test]
fn test_assignment_local() {
    run_lox_test(Path::new("test/assignment/local.lox"));
}

#[test]
fn test_assignment_prefix_operator() {
    run_lox_test(Path::new("test/assignment/prefix_operator.lox"));
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
fn test_assignment_undefined() {
    run_lox_test(Path::new("test/assignment/undefined.lox"));
}

// benchmark

#[test]
#[ignore]
fn test_benchmark_binary_trees() {
    run_lox_test(Path::new("test/benchmark/binary_trees.lox"));
}

#[test]
#[ignore]
fn test_benchmark_equality() {
    run_lox_test(Path::new("test/benchmark/equality.lox"));
}

#[test]
#[ignore]
fn test_benchmark_fib() {
    run_lox_test(Path::new("test/benchmark/fib.lox"));
}

#[test]
#[ignore]
fn test_benchmark_instantiation() {
    run_lox_test(Path::new("test/benchmark/instantiation.lox"));
}

#[test]
#[ignore]
fn test_benchmark_invocation() {
    run_lox_test(Path::new("test/benchmark/invocation.lox"));
}

#[test]
#[ignore]
fn test_benchmark_method_call() {
    run_lox_test(Path::new("test/benchmark/method_call.lox"));
}

#[test]
#[ignore]
fn test_benchmark_properties() {
    run_lox_test(Path::new("test/benchmark/properties.lox"));
}

#[test]
#[ignore]
fn test_benchmark_string_equality() {
    run_lox_test(Path::new("test/benchmark/string_equality.lox"));
}

#[test]
#[ignore]
fn test_benchmark_trees() {
    run_lox_test(Path::new("test/benchmark/trees.lox"));
}

#[test]
#[ignore]
fn test_benchmark_zoo_batch() {
    run_lox_test(Path::new("test/benchmark/zoo_batch.lox"));
}

#[test]
#[ignore]
fn test_benchmark_zoo() {
    run_lox_test(Path::new("test/benchmark/zoo.lox"));
}

// block

#[test]
fn test_block_empty() {
    run_lox_test(Path::new("test/block/empty.lox"));
}

#[test]
fn test_block_scope() {
    run_lox_test(Path::new("test/block/scope.lox"));
}

// bool

#[test]
fn test_bool_empty() {
    run_lox_test(Path::new("test/bool/equality.lox"));
}

#[test]
fn test_bool_scope() {
    run_lox_test(Path::new("test/bool/not.lox"));
}

// call

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

// class

#[test]
fn test_class_empty() {
    run_lox_test(Path::new("test/class/empty.lox"));
}

#[test]
fn test_class_inherit_self() {
    run_lox_test(Path::new("test/class/inherit_self.lox"));
}

#[test]
fn test_class_inherited_method() {
    run_lox_test(Path::new("test/class/inherited_method.lox"));
}

#[test]
fn test_class_local_inherit_other() {
    run_lox_test(Path::new("test/class/local_inherit_other.lox"));
}

#[test]
fn test_class_local_inherit_self() {
    run_lox_test(Path::new("test/class/local_inherit_self.lox"));
}

#[test]
fn test_class_local_reference_self() {
    run_lox_test(Path::new("test/class/local_reference_self.lox"));
}

#[test]
fn test_class_reference_self() {
    run_lox_test(Path::new("test/class/reference_self.lox"));
}

// closure

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

// comments

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

// constructor

#[test]
fn test_constructor_arguments() {
    run_lox_test(Path::new("test/constructor/arguments.lox"));
}

#[test]
fn test_constructor_call_init_early_exit() {
    run_lox_test(Path::new("test/constructor/call_init_early_exit.lox"));
}

#[test]
fn test_constructor_call_init_explicitly() {
    run_lox_test(Path::new("test/constructor/call_init_explicitly.lox"));
}

#[test]
fn test_constructor_default_arguments() {
    run_lox_test(Path::new("test/constructor/default_arguments.lox"));
}

#[test]
fn test_constructor_default() {
    run_lox_test(Path::new("test/constructor/default.lox"));
}

#[test]
fn test_constructor_early_return() {
    run_lox_test(Path::new("test/constructor/early_return.lox"));
}

#[test]
fn test_constructor_extra_arguments() {
    run_lox_test(Path::new("test/constructor/extra_arguments.lox"));
}

#[test]
fn test_constructor_init_not_method() {
    run_lox_test(Path::new("test/constructor/init_not_method.lox"));
}

#[test]
fn test_constructor_missing_arguments() {
    run_lox_test(Path::new("test/constructor/missing_arguments.lox"));
}

#[test]
fn test_constructor_return_in_nested_function() {
    run_lox_test(Path::new("test/constructor/return_in_nested_function.lox"));
}

#[test]
fn test_constructor_return_value() {
    run_lox_test(Path::new("test/constructor/return_value.lox"));
}

// while

#[test]
fn test_while_class_in_body() {
    run_lox_test(Path::new("test/while/class_in_body.lox"));
}

#[test]
fn test_while_closure_in_body() {
    run_lox_test(Path::new("test/while/closure_in_body.lox"));
}

#[test]
fn test_while_fun_in_body() {
    run_lox_test(Path::new("test/while/fun_in_body.lox"));
}

#[test]
fn test_while_return_closure() {
    run_lox_test(Path::new("test/while/return_closure.lox"));
}

#[test]
fn test_while_return_inside() {
    run_lox_test(Path::new("test/while/return_inside.lox"));
}

#[test]
fn test_while_syntax() {
    run_lox_test(Path::new("test/while/syntax.lox"));
}

#[test]
fn test_while_var_in_body() {
    run_lox_test(Path::new("test/while/var_in_body.lox"));
}

// expressions

#[test]
#[ignore]
fn test_expressions_evaluate() {
    run_lox_test(Path::new("test/expressions/evaluate.lox"));
}

#[test]
#[ignore]
fn test_expressions_parse() {
    run_lox_test(Path::new("test/expressions/parse.lox"));
}

// field

#[test]
fn test_field_call_function_field() {
    run_lox_test(Path::new("test/field/call_function_field.lox"));
}

#[test]
fn test_field_call_nonfunction_field() {
    run_lox_test(Path::new("test/field/call_nonfunction_field.lox"));
}

#[test]
fn test_field_get_and_set_method() {
    run_lox_test(Path::new("test/field/get_and_set_method.lox"));
}

#[test]
fn test_field_get_on_bool() {
    run_lox_test(Path::new("test/field/get_on_bool.lox"));
}

#[test]
fn test_field_get_on_class() {
    run_lox_test(Path::new("test/field/get_on_class.lox"));
}

#[test]
fn test_field_get_on_function() {
    run_lox_test(Path::new("test/field/get_on_function.lox"));
}

#[test]
fn test_field_get_on_nil() {
    run_lox_test(Path::new("test/field/get_on_nil.lox"));
}

#[test]
fn test_field_get_on_num() {
    run_lox_test(Path::new("test/field/get_on_num.lox"));
}

#[test]
fn test_field_get_on_string() {
    run_lox_test(Path::new("test/field/get_on_string.lox"));
}

#[test]
fn test_field_many() {
    run_lox_test(Path::new("test/field/many.lox"));
}

#[test]
fn test_field_method_binds_this() {
    run_lox_test(Path::new("test/field/method_binds_this.lox"));
}

#[test]
fn test_field_method() {
    run_lox_test(Path::new("test/field/method.lox"));
}

#[test]
fn test_field_on_instance() {
    run_lox_test(Path::new("test/field/on_instance.lox"));
}

#[test]
fn test_field_set_evaluation_order() {
    run_lox_test(Path::new("test/field/set_evaluation_order.lox"));
}

#[test]
fn test_field_set_on_bool() {
    run_lox_test(Path::new("test/field/set_on_bool.lox"));
}

#[test]
fn test_field_set_on_class() {
    run_lox_test(Path::new("test/field/set_on_class.lox"));
}

#[test]
fn test_field_set_on_function() {
    run_lox_test(Path::new("test/field/set_on_function.lox"));
}

#[test]
fn test_field_set_on_nil() {
    run_lox_test(Path::new("test/field/set_on_nil.lox"));
}

#[test]
fn test_field_set_on_num() {
    run_lox_test(Path::new("test/field/set_on_num.lox"));
}

#[test]
fn test_field_set_on_string() {
    run_lox_test(Path::new("test/field/set_on_string.lox"));
}

#[test]
fn test_field_undefined() {
    run_lox_test(Path::new("test/field/undefined.lox"));
}

// for

#[test]
fn test_for_class_in_body() {
    run_lox_test(Path::new("test/for/class_in_body.lox"));
}

#[test]
fn test_for_closure_in_body() {
    run_lox_test(Path::new("test/for/closure_in_body.lox"));
}

#[test]
fn test_for_fun_in_body() {
    run_lox_test(Path::new("test/for/fun_in_body.lox"));
}

#[test]
fn test_for_return_closure() {
    run_lox_test(Path::new("test/for/return_closure.lox"));
}

#[test]
fn test_for_return_inside() {
    run_lox_test(Path::new("test/for/return_inside.lox"));
}

#[test]
fn test_for_scope() {
    run_lox_test(Path::new("test/for/scope.lox"));
}

#[test]
fn test_for_statement_condition() {
    run_lox_test(Path::new("test/for/statement_condition.lox"));
}

#[test]
fn test_for_statement_increment() {
    run_lox_test(Path::new("test/for/statement_increment.lox"));
}

#[test]
fn test_for_statement_initializer() {
    run_lox_test(Path::new("test/for/statement_initializer.lox"));
}

#[test]
fn test_for_syntax() {
    run_lox_test(Path::new("test/for/syntax.lox"));
}

#[test]
fn test_for_var_in_body() {
    run_lox_test(Path::new("test/for/var_in_body.lox"));
}

// function

#[test]
fn test_function_body_must_be_block() {
    run_lox_test(Path::new("test/function/body_must_be_block.lox"));
}

#[test]
fn test_function_empty_body() {
    run_lox_test(Path::new("test/function/empty_body.lox"));
}

#[test]
fn test_function_extra_arguments() {
    run_lox_test(Path::new("test/function/extra_arguments.lox"));
}

#[test]
fn test_function_local_mutual_recursion() {
    run_lox_test(Path::new("test/function/local_mutual_recursion.lox"));
}

#[test]
fn test_function_local_recursion() {
    run_lox_test(Path::new("test/function/local_recursion.lox"));
}

#[test]
fn test_function_missing_arguments() {
    run_lox_test(Path::new("test/function/missing_arguments.lox"));
}

#[test]
fn test_function_missing_comma_in_parameters() {
    run_lox_test(Path::new("test/function/missing_comma_in_parameters.lox"));
}

#[test]
fn test_function_mutual_recursion() {
    run_lox_test(Path::new("test/function/mutual_recursion.lox"));
}

#[test]
fn test_function_nested_call_with_arguments() {
    run_lox_test(Path::new("test/function/nested_call_with_arguments.lox"));
}

#[test]
fn test_function_parameters() {
    run_lox_test(Path::new("test/function/parameters.lox"));
}

#[test]
fn test_function_print() {
    run_lox_test(Path::new("test/function/print.lox"));
}

#[test]
fn test_function_recursion() {
    run_lox_test(Path::new("test/function/recursion.lox"));
}

#[test]
fn test_function_too_many_arguments() {
    run_lox_test(Path::new("test/function/too_many_arguments.lox"));
}

#[test]
fn test_function_too_many_parameters() {
    run_lox_test(Path::new("test/function/too_many_parameters.lox"));
}

// if

#[test]
fn test_if_class_in_else() {
    run_lox_test(Path::new("test/if/class_in_else.lox"));
}

#[test]
fn test_if_class_in_then() {
    run_lox_test(Path::new("test/if/class_in_then.lox"));
}

#[test]
fn test_if_dangling_else() {
    run_lox_test(Path::new("test/if/dangling_else.lox"));
}

#[test]
fn test_if_else() {
    run_lox_test(Path::new("test/if/else.lox"));
}

#[test]
fn test_if_fun_in_else() {
    run_lox_test(Path::new("test/if/fun_in_else.lox"));
}

#[test]
fn test_if_fun_in_then() {
    run_lox_test(Path::new("test/if/fun_in_then.lox"));
}

#[test]
fn test_if_if() {
    run_lox_test(Path::new("test/if/if.lox"));
}

#[test]
fn test_if_truth() {
    run_lox_test(Path::new("test/if/truth.lox"));
}

#[test]
fn test_if_var_in_else() {
    run_lox_test(Path::new("test/if/var_in_else.lox"));
}

#[test]
fn test_if_var_in_then() {
    run_lox_test(Path::new("test/if/var_in_then.lox"));
}

// inheritance

#[test]
fn test_inheritance_constructor() {
    run_lox_test(Path::new("test/inheritance/constructor.lox"));
}

#[test]
fn test_inheritance_inherit_from_function() {
    run_lox_test(Path::new("test/inheritance/inherit_from_function.lox"));
}

#[test]
fn test_inheritance_inherit_from_nil() {
    run_lox_test(Path::new("test/inheritance/inherit_from_nil.lox"));
}

#[test]
fn test_inheritance_inherit_from_number() {
    run_lox_test(Path::new("test/inheritance/inherit_from_number.lox"));
}

#[test]
fn test_inheritance_inherit_methods() {
    run_lox_test(Path::new("test/inheritance/inherit_methods.lox"));
}

#[test]
fn test_inheritance_parenthesized_superclass() {
    run_lox_test(Path::new("test/inheritance/parenthesized_superclass.lox"));
}

#[test]
fn test_inheritance_set_fields_from_base_class() {
    run_lox_test(Path::new("test/inheritance/set_fields_from_base_class.lox"));
}

// limit

#[test]
#[ignore]
fn test_limit_loop_too_large() {
    run_lox_test(Path::new("test/limit/loop_too_large.lox"));
}

#[test]
#[ignore]
fn test_limit_no_reuse_constants() {
    run_lox_test(Path::new("test/limit/no_reuse_constants.lox"));
}

#[test]
#[ignore]
fn test_limit_stack_overflow() {
    run_lox_test(Path::new("test/limit/stack_overflow.lox"));
}

#[test]
#[ignore]
fn test_limit_too_many_constants() {
    run_lox_test(Path::new("test/limit/too_many_constants.lox"));
}

#[test]
#[ignore]
fn test_limit_too_many_locals() {
    run_lox_test(Path::new("test/limit/too_many_locals.lox"));
}

#[test]
#[ignore]
fn test_limit_too_many_upvalues() {
    run_lox_test(Path::new("test/limit/too_many_upvalues.lox"));
}

// method

#[test]
fn test_method_arity() {
    run_lox_test(Path::new("test/method/arity.lox"));
}

#[test]
fn test_method_empty_block() {
    run_lox_test(Path::new("test/method/empty_block.lox"));
}

#[test]
fn test_method_extra_arguments() {
    run_lox_test(Path::new("test/method/extra_arguments.lox"));
}

#[test]
fn test_method_missing_arguments() {
    run_lox_test(Path::new("test/method/missing_arguments.lox"));
}

#[test]
fn test_method_not_found() {
    run_lox_test(Path::new("test/method/not_found.lox"));
}

#[test]
fn test_method_print_bound_method() {
    run_lox_test(Path::new("test/method/print_bound_method.lox"));
}

#[test]
fn test_method_refer_to_name() {
    run_lox_test(Path::new("test/method/refer_to_name.lox"));
}

#[test]
fn test_method_too_many_arguments() {
    run_lox_test(Path::new("test/method/too_many_arguments.lox"));
}

#[test]
fn test_method_too_many_parameters() {
    run_lox_test(Path::new("test/method/too_many_parameters.lox"));
}

// variable

#[test]
fn test_variable_collide_with_parameter() {
    run_lox_test(Path::new("test/variable/collide_with_parameter.lox"));
}

#[test]
fn test_variable_duplicate_local() {
    run_lox_test(Path::new("test/variable/duplicate_local.lox"));
}

#[test]
fn test_variable_duplicate_parameter() {
    run_lox_test(Path::new("test/variable/duplicate_parameter.lox"));
}

#[test]
fn test_variable_early_bound() {
    run_lox_test(Path::new("test/variable/early_bound.lox"));
}

#[test]
fn test_variable_in_middle_of_block() {
    run_lox_test(Path::new("test/variable/in_middle_of_block.lox"));
}

#[test]
fn test_variable_in_nested_block() {
    run_lox_test(Path::new("test/variable/in_nested_block.lox"));
}

#[test]
fn test_variable_local_from_method() {
    run_lox_test(Path::new("test/variable/local_from_method.lox"));
}

#[test]
fn test_variable_redeclare_global() {
    run_lox_test(Path::new("test/variable/redeclare_global.lox"));
}

#[test]
fn test_variable_redefine_global() {
    run_lox_test(Path::new("test/variable/redefine_global.lox"));
}

#[test]
fn test_variable_scope_reuse_in_different_blocks() {
    run_lox_test(Path::new("test/variable/scope_reuse_in_different_blocks.lox"));
}

#[test]
fn test_variable_shadow_and_local() {
    run_lox_test(Path::new("test/variable/shadow_and_local.lox"));
}

#[test]
fn test_variable_shadow_global() {
    run_lox_test(Path::new("test/variable/shadow_global.lox"));
}

#[test]
fn test_variable_shadow_local() {
    run_lox_test(Path::new("test/variable/shadow_local.lox"));
}

#[test]
fn test_variable_undefined_global() {
    run_lox_test(Path::new("test/variable/undefined_global.lox"));
}

#[test]
fn test_variable_undefined_local() {
    run_lox_test(Path::new("test/variable/undefined_local.lox"));
}

#[test]
fn test_variable_uninitialized() {
    run_lox_test(Path::new("test/variable/uninitialized.lox"));
}

#[test]
fn test_variable_unreached_undefined() {
    run_lox_test(Path::new("test/variable/unreached_undefined.lox"));
}

#[test]
fn test_variable_use_false_as_var() {
    run_lox_test(Path::new("test/variable/use_false_as_var.lox"));
}

#[test]
fn test_variable_use_global_in_initializer() {
    run_lox_test(Path::new("test/variable/use_global_in_initializer.lox"));
}

#[test]
fn test_variable_use_local_in_initializer() {
    run_lox_test(Path::new("test/variable/use_local_in_initializer.lox"));
}

#[test]
fn test_variable_use_nil_as_var() {
    run_lox_test(Path::new("test/variable/use_nil_as_var.lox"));
}

#[test]
fn test_variable_use_this_as_var() {
    run_lox_test(Path::new("test/variable/use_this_as_var.lox"));
}

// regression

#[test]
fn test_regression_40() {
    run_lox_test(Path::new("test/regression/40.lox"));
}

#[test]
fn test_regression_394() {
    run_lox_test(Path::new("test/regression/394.lox"));
}

// string

#[test]
fn test_string_error_after_multiline() {
    run_lox_test(Path::new("test/string/error_after_multiline.lox"));
}

#[test]
fn test_string_literals() {
    run_lox_test(Path::new("test/string/literals.lox"));
}

#[test]
fn test_string_multiline() {
    run_lox_test(Path::new("test/string/multiline.lox"));
}

#[test]
fn test_string_unterminated() {
    run_lox_test(Path::new("test/string/unterminated.lox"));
}

// this

#[test]
fn test_this_closure() {
    run_lox_test(Path::new("test/this/closure.lox"));
}

#[test]
fn test_this_nested_class() {
    run_lox_test(Path::new("test/this/nested_class.lox"));
}

#[test]
fn test_this_nested_closure() {
    run_lox_test(Path::new("test/this/nested_closure.lox"));
}

#[test]
fn test_this_this_at_top_level() {
    run_lox_test(Path::new("test/this/this_at_top_level.lox"));
}

#[test]
fn test_this_this_in_method() {
    run_lox_test(Path::new("test/this/this_in_method.lox"));
}

#[test]
fn test_this_this_in_top_level_function() {
    run_lox_test(Path::new("test/this/this_in_top_level_function.lox"));
}

// super

#[test]
fn test_super_bound_method() {
    run_lox_test(Path::new("test/super/bound_method.lox"));
}

#[test]
fn test_super_call_other_method() {
    run_lox_test(Path::new("test/super/call_other_method.lox"));
}

#[test]
fn test_super_call_same_method() {
    run_lox_test(Path::new("test/super/call_same_method.lox"));
}

#[test]
fn test_super_closure() {
    run_lox_test(Path::new("test/super/closure.lox"));
}

#[test]
fn test_super_constructor() {
    run_lox_test(Path::new("test/super/constructor.lox"));
}

#[test]
fn test_super_extra_arguments() {
    run_lox_test(Path::new("test/super/extra_arguments.lox"));
}

#[test]
fn test_super_indirectly_inherited() {
    run_lox_test(Path::new("test/super/indirectly_inherited.lox"));
}

#[test]
fn test_super_missing_arguments() {
    run_lox_test(Path::new("test/super/missing_arguments.lox"));
}

#[test]
fn test_super_no_superclass_bind() {
    run_lox_test(Path::new("test/super/no_superclass_bind.lox"));
}

#[test]
fn test_super_no_superclass_call() {
    run_lox_test(Path::new("test/super/no_superclass_call.lox"));
}

#[test]
fn test_super_no_superclass_method() {
    run_lox_test(Path::new("test/super/no_superclass_method.lox"));
}

#[test]
fn test_super_parenthesized() {
    run_lox_test(Path::new("test/super/parenthesized.lox"));
}

#[test]
fn test_super_reassign_superclass() {
    run_lox_test(Path::new("test/super/reassign_superclass.lox"));
}

#[test]
fn test_super_super_at_top_level() {
    run_lox_test(Path::new("test/super/super_at_top_level.lox"));
}

#[test]
fn test_super_super_in_closure_in_inherited_method() {
    run_lox_test(Path::new("test/super/super_in_closure_in_inherited_method.lox"));
}

#[test]
fn test_super_super_in_inherited_method() {
    run_lox_test(Path::new("test/super/super_in_inherited_method.lox"));
}

#[test]
fn test_super_super_in_top_level_function() {
    run_lox_test(Path::new("test/super/super_in_top_level_function.lox"));
}

#[test]
fn test_super_super_without_dot() {
    run_lox_test(Path::new("test/super/super_without_dot.lox"));
}

#[test]
fn test_super_super_without_name() {
    run_lox_test(Path::new("test/super/super_without_name.lox"));
}

#[test]
fn test_super_this_in_superclass_method() {
    run_lox_test(Path::new("test/super/this_in_superclass_method.lox"));
}

// return

#[test]
fn test_return_after_else() {
    run_lox_test(Path::new("test/return/after_else.lox"));
}

#[test]
fn test_return_after_if() {
    run_lox_test(Path::new("test/return/after_if.lox"));
}

#[test]
fn test_return_after_while() {
    run_lox_test(Path::new("test/return/after_while.lox"));
}

#[test]
fn test_return_at_top_level() {
    run_lox_test(Path::new("test/return/at_top_level.lox"));
}

#[test]
fn test_return_in_function() {
    run_lox_test(Path::new("test/return/in_function.lox"));
}

#[test]
fn test_return_in_method() {
    run_lox_test(Path::new("test/return/in_method.lox"));
}

#[test]
fn test_return_return_nil_if_no_value() {
    run_lox_test(Path::new("test/return/return_nil_if_no_value.lox"));
}

// operator

#[test]
fn test_operator_add_bool_nil() {
    run_lox_test(Path::new("test/operator/add_bool_nil.lox"));
}

#[test]
fn test_operator_add_bool_num() {
    run_lox_test(Path::new("test/operator/add_bool_num.lox"));
}

#[test]
fn test_operator_add_bool_string() {
    run_lox_test(Path::new("test/operator/add_bool_string.lox"));
}

#[test]
fn test_operator_add_nil_nil() {
    run_lox_test(Path::new("test/operator/add_nil_nil.lox"));
}

#[test]
fn test_operator_add_num_nil() {
    run_lox_test(Path::new("test/operator/add_num_nil.lox"));
}

#[test]
fn test_operator_add_string_nil() {
    run_lox_test(Path::new("test/operator/add_string_nil.lox"));
}

#[test]
fn test_operator_add() {
    run_lox_test(Path::new("test/operator/add.lox"));
}

#[test]
fn test_operator_comparison() {
    run_lox_test(Path::new("test/operator/comparison.lox"));
}

#[test]
fn test_operator_divide_nonnum_num() {
    run_lox_test(Path::new("test/operator/divide_nonnum_num.lox"));
}

#[test]
fn test_operator_divide_num_nonnum() {
    run_lox_test(Path::new("test/operator/divide_num_nonnum.lox"));
}

#[test]
fn test_operator_divide() {
    run_lox_test(Path::new("test/operator/divide.lox"));
}

#[test]
fn test_operator_equals_class() {
    run_lox_test(Path::new("test/operator/equals_class.lox"));
}

#[test]
fn test_operator_equals_method() {
    run_lox_test(Path::new("test/operator/equals_method.lox"));
}

#[test]
fn test_operator_equals() {
    run_lox_test(Path::new("test/operator/equals.lox"));
}

#[test]
fn test_operator_greater_nonnum_num() {
    run_lox_test(Path::new("test/operator/greater_nonnum_num.lox"));
}

#[test]
fn test_operator_greater_num_nonnum() {
    run_lox_test(Path::new("test/operator/greater_num_nonnum.lox"));
}

#[test]
fn test_operator_greater_or_equal_nonnum_num() {
    run_lox_test(Path::new("test/operator/greater_or_equal_nonnum_num.lox"));
}

#[test]
fn test_operator_greater_or_equal_num_nonnum() {
    run_lox_test(Path::new("test/operator/greater_or_equal_num_nonnum.lox"));
}

#[test]
fn test_operator_less_nonnum_num() {
    run_lox_test(Path::new("test/operator/less_nonnum_num.lox"));
}

#[test]
fn test_operator_less_num_nonnum() {
    run_lox_test(Path::new("test/operator/less_num_nonnum.lox"));
}

#[test]
fn test_operator_less_or_equal_nonnum_num() {
    run_lox_test(Path::new("test/operator/less_or_equal_nonnum_num.lox"));
}

#[test]
fn test_operator_less_or_equal_num_nonnum() {
    run_lox_test(Path::new("test/operator/less_or_equal_num_nonnum.lox"));
}

#[test]
fn test_operator_multiply_nonnum_num() {
    run_lox_test(Path::new("test/operator/multiply_nonnum_num.lox"));
}

#[test]
fn test_operator_multiply_num_nonnum() {
    run_lox_test(Path::new("test/operator/multiply_num_nonnum.lox"));
}

#[test]
fn test_operator_multiply() {
    run_lox_test(Path::new("test/operator/multiply.lox"));
}

#[test]
fn test_operator_negate_nonnum() {
    run_lox_test(Path::new("test/operator/negate_nonnum.lox"));
}

#[test]
fn test_operator_negate() {
    run_lox_test(Path::new("test/operator/negate.lox"));
}

#[test]
fn test_operator_not_class() {
    run_lox_test(Path::new("test/operator/not_class.lox"));
}

#[test]
fn test_operator_not_equals() {
    run_lox_test(Path::new("test/operator/not_equals.lox"));
}

#[test]
fn test_operator_not() {
    run_lox_test(Path::new("test/operator/not.lox"));
}

#[test]
fn test_operator_subtract_nonnum_num() {
    run_lox_test(Path::new("test/operator/subtract_nonnum_num.lox"));
}

#[test]
fn test_operator_subtract_num_nonnum() {
    run_lox_test(Path::new("test/operator/subtract_num_nonnum.lox"));
}

#[test]
fn test_operator_subtract() {
    run_lox_test(Path::new("test/operator/subtract.lox"));
}

// number

#[test]
fn test_number_decimal_point_at_eof() {
    run_lox_test(Path::new("test/number/decimal_point_at_eof.lox"));
}

#[test]
fn test_number_leading_dot() {
    run_lox_test(Path::new("test/number/leading_dot.lox"));
}

#[test]
fn test_number_literals() {
    run_lox_test(Path::new("test/number/literals.lox"));
}

#[test]
fn test_number_nan_equality() {
    run_lox_test(Path::new("test/number/nan_equality.lox"));
}

#[test]
fn test_number_trailing_dot() {
    run_lox_test(Path::new("test/number/trailing_dot.lox"));
}

// nil

#[test]
fn test_nil_literal() {
    run_lox_test(Path::new("test/nil/literal.lox"));
}

// print

#[test]
fn test_print_missing_argument() {
    run_lox_test(Path::new("test/print/missing_argument.lox"));
}

// scanning

#[test]
#[ignore]
fn test_scanning_identifiers() {
    run_lox_test(Path::new("test/scanning/identifiers.lox"));
}

#[test]
#[ignore]
fn test_scanning_keywords() {
    run_lox_test(Path::new("test/scanning/keywords.lox"));
}

#[test]
#[ignore]
fn test_scanning_numbers() {
    run_lox_test(Path::new("test/scanning/numbers.lox"));
}

#[test]
#[ignore]
fn test_scanning_punctuators() {
    run_lox_test(Path::new("test/scanning/punctuators.lox"));
}

#[test]
#[ignore]
fn test_scanning_strings() {
    run_lox_test(Path::new("test/scanning/strings.lox"));
}

#[test]
#[ignore]
fn test_scanning_whitespace() {
    run_lox_test(Path::new("test/scanning/whitespace.lox"));
}

// logical_operator

#[test]
fn test_logical_operator_and_truth() {
    run_lox_test(Path::new("test/logical_operator/and_truth.lox"));
}

#[test]
fn test_logical_operator_and() {
    run_lox_test(Path::new("test/logical_operator/and.lox"));
}

#[test]
fn test_logical_operator_or_truth() {
    run_lox_test(Path::new("test/logical_operator/or_truth.lox"));
}

#[test]
fn test_logical_operator_or() {
    run_lox_test(Path::new("test/logical_operator/or.lox"));
}

// other

#[test]
fn test_empty_file() {
    run_lox_test(Path::new("test/empty_file.lox"));
}

#[test]
fn test_precedence() {
    run_lox_test(Path::new("test/precedence.lox"));
}

#[test]
fn test_unexpected_character() {
    run_lox_test(Path::new("test/unexpected_character.lox"));
}

