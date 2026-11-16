use std::collections::HashMap;

use std::time::{SystemTime, UNIX_EPOCH};

use crate::{ast, context, lexer};

pub fn is_truthy(v: &context::Value) -> bool {
    !matches!(v, context::Value::Nil | context::Value::Boolean(false))
}

pub struct Interpreter<'a> {
    pub context: context::Context<'a>,
    pub statement_exec_count: usize,
    pub working_set_size_after_last_gc: usize,
}

impl Default for Interpreter<'_> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a> Interpreter<'a> {
    pub fn new() -> Self {
        let mut context = context::Context::new();

        context.push_new_environment_auto();

        context.insert_symbol(
            "clock",
            context::Value::Function(context::Function {
                context_function_slot_idx: 0,
                closure_id: 0,
            }),
        );

        Self {
            context,
            statement_exec_count: 0,
            working_set_size_after_last_gc: 0,
        }
    }

    fn runtime_error(message: &str) -> ! {
        eprintln!("Runtime error:, '{message}'");
        std::process::exit(1);
    }

    fn evaluate_literal(expr: &ast::expr::Literal<'a>) -> context::NodeResult {
        let ast::expr::Literal { token } = expr;

        match token.kind {
            lexer::TokenKind::Nil => {
                context::NodeResult::new(context::Value::Nil, context::Control::Continue)
            }
            lexer::TokenKind::False => {
                context::NodeResult::new(context::Value::Boolean(false), context::Control::Continue)
            }
            lexer::TokenKind::True => {
                context::NodeResult::new(context::Value::Boolean(true), context::Control::Continue)
            }
            lexer::TokenKind::Number => context::NodeResult::new(
                context::Value::Number(token.lexeme.parse().unwrap()),
                context::Control::Continue,
            ),
            lexer::TokenKind::String => context::NodeResult::new(
                context::Value::String(token.lexeme[1..token.lexeme.len() - 1].to_string()),
                context::Control::Continue,
            ),
            _ => panic!(),
        }
    }

    fn evaluate_variable(&mut self, expr: &ast::expr::Variable<'a>) -> context::NodeResult {
        let ast::expr::Variable {
            identifier: id,
            lookup_hop_count,
            env_slot_idx,
        } = expr;
        let mut lookup_hop_count_mut_copy = *lookup_hop_count;
        let mut env_slot_idx_mut_copy = *env_slot_idx;
        context::NodeResult::new(
            self.context.get_symbol_value(
                id.lexeme,
                &mut lookup_hop_count_mut_copy,
                &mut env_slot_idx_mut_copy,
            ),
            context::Control::Continue,
        )
    }

    fn evaluate_this(&mut self) -> context::NodeResult {
        context::NodeResult::new(
            self.context.get_symbol_value("this", &mut None, &mut None),
            context::Control::Continue,
        )
    }

    fn make_bound_method(
        &mut self,
        context_function_slot_idx: usize,
        closure_id: usize,
        instance: &context::Value,
    ) -> context::Value {
        self.context.push_new_environment(Some(closure_id));
        self.context.insert_symbol("this", instance.clone());
        self.context.pop_environment();
        context::Value::Function(context::Function {
            context_function_slot_idx,
            closure_id: self.context.next_env_id - 1,
        })
    }

    fn evaluate_super(
        &mut self,
        expr: &ast::expr::Super<'a>,
        defs: &'a context::Definitions,
    ) -> context::NodeResult {
        let ast::expr::Super {
            property,
            lookup_hop_count,
            env_slot_idx,
        } = expr;

        let mut lookup_hop_count_mut_copy = *lookup_hop_count;
        let mut env_slot_idx_mut_copy = *env_slot_idx;
        let instance_value = self.context.get_symbol_value("this", &mut None, &mut None);

        let mut method_lookup_class = Some(self.context.get_symbol_value(
            "super",
            &mut lookup_hop_count_mut_copy,
            &mut env_slot_idx_mut_copy,
        ));
        while let Some(maybe_value) = method_lookup_class {
            let context::Value::Class(context::Class {
                context_class_slot_idx,
                closure_id,
                super_class,
            }) = maybe_value
            else {
                panic!("Expected class value");
            };
            let context::ClassDefinition { ref methods, .. } = defs.classes[context_class_slot_idx];
            let Some(context_function_slot_idx) = methods.get(&property.lexeme) else {
                method_lookup_class = super_class.map(|boxed| *boxed);
                continue;
            };
            return context::NodeResult {
                value: self.make_bound_method(
                    *context_function_slot_idx,
                    closure_id,
                    &instance_value,
                ),
                control: context::Control::Continue,
            };
        }
        panic!("Undefined property '{}'.", property.lexeme);
    }

    fn evaluate_set(
        &mut self,
        expr: &ast::expr::Set<'a>,
        defs: &'a context::Definitions,
    ) -> context::NodeResult {
        let ast::expr::Set {
            instance,
            property,
            value,
        } = expr;

        let instance = self.evaluate_expression(instance.as_ref(), defs).value;
        let value = self.evaluate_expression(value, defs).value;

        let context::Value::Instance(instance_id) = instance else {
            panic!("Only instances have fields.");
        };
        let instance_value = self.context.instances.get_mut(&instance_id).unwrap();
        instance_value
            .properties
            .insert(property.lexeme, value.clone());
        context::NodeResult::new(value, context::Control::Continue)
    }

    fn evaluate_get_internal(
        &mut self,
        instance_value: &context::Value,
        property: &lexer::Token<'a>,
        defs: &'a context::Definitions,
    ) -> Option<context::Value> {
        let context::Value::Instance(instance_id) = instance_value else {
            panic!("Only instances have properties.");
        };
        let instance = self.context.instances.get(instance_id).unwrap();

        if let Some(value) = instance.properties.get(&property.lexeme) {
            Some(value.clone())
        } else {
            let mut method_lookup_class = Some(instance.class.clone());
            while let Some(maybe_value) = method_lookup_class {
                let context::Value::Class(context::Class {
                    context_class_slot_idx,
                    closure_id,
                    super_class,
                }) = maybe_value
                else {
                    panic!("Expected class value");
                };
                let context::ClassDefinition { methods, .. } =
                    &defs.classes[context_class_slot_idx];
                let Some(context_function_slot_idx) = methods.get(&property.lexeme) else {
                    method_lookup_class = super_class.map(|boxed| *boxed);
                    continue;
                };
                return Some(self.make_bound_method(
                    *context_function_slot_idx,
                    closure_id,
                    instance_value,
                ));
            }
            None
        }
    }

    fn evaluate_get(
        &mut self,
        expr: &ast::expr::Get<'a>,
        defs: &'a context::Definitions,
    ) -> context::NodeResult {
        let ast::expr::Get { instance, property } = expr;
        let instance_value = self.evaluate_expression(instance.as_ref(), defs).value;

        let Some(value) = self.evaluate_get_internal(&instance_value, property, defs) else {
            Interpreter::runtime_error(&std::format!("Undefined property '{}'.", property.lexeme));
        };
        context::NodeResult::new(value, context::Control::Continue)
    }

    fn evaluate_unary(
        &mut self,
        expr: &ast::expr::UnaryOperation<'a>,
        defs: &'a context::Definitions,
    ) -> context::NodeResult {
        let ast::expr::UnaryOperation {
            operator: op,
            right: rhs,
        } = expr;
        let rhs_result = self.evaluate_expression(rhs, defs);

        if op.kind == lexer::TokenKind::Minus {
            let context::Value::Number(v) = rhs_result.value else {
                Interpreter::runtime_error("Operand must be a number.");
            };
            context::NodeResult::new(context::Value::Number(-v), context::Control::Continue)
        } else if op.kind == lexer::TokenKind::Bang {
            context::NodeResult::new(
                context::Value::Boolean(!is_truthy(&rhs_result.value)),
                context::Control::Continue,
            )
        } else {
            panic!();
        }
    }

    fn evaluate_parentheses(
        &mut self,
        expr: &ast::expr::Parentheses<'a>,
        defs: &'a context::Definitions,
    ) -> context::NodeResult {
        let ast::expr::Parentheses { expression: e } = expr;
        let expression_result = self.evaluate_expression(e, defs);
        context::NodeResult::new(expression_result.value, context::Control::Continue)
    }

    fn evaluate_assignment(
        &mut self,
        expr: &ast::expr::Assignment<'a>,
        defs: &'a context::Definitions,
    ) -> context::NodeResult {
        let ast::expr::Assignment {
            left: lhs,
            expression: rhs,
            lookup_hop_count,
            env_slot_idx,
        } = expr;
        let rhs_result = self.evaluate_expression(rhs, defs);
        let mut lookup_hop_count_mut_copy = *lookup_hop_count;
        let mut env_slot_idx_mut_copy = *env_slot_idx;
        self.context.set_symbol_value(
            lhs.lexeme,
            rhs_result.value.clone(),
            &mut lookup_hop_count_mut_copy,
            &mut env_slot_idx_mut_copy,
        );
        context::NodeResult::new(rhs_result.value, context::Control::Continue)
    }

    fn op_add(vl: &context::Value, vr: &context::Value) -> context::Value {
        if let (context::Value::Number(vl), context::Value::Number(vr)) = (&vl, &vr) {
            context::Value::Number(vl + vr)
        } else if let (context::Value::String(vl), context::Value::String(vr)) = (&vl, &vr) {
            context::Value::String(vl.clone() + vr)
        } else {
            Interpreter::runtime_error("Operands must be two numbers or two strings.");
        }
    }

    fn op_sub(vl: &context::Value, vr: &context::Value) -> context::Value {
        let (context::Value::Number(vl), context::Value::Number(vr)) = (&vl, &vr) else {
            Interpreter::runtime_error("Operands must be numbers.");
        };
        context::Value::Number(vl - vr)
    }

    fn op_mul(vl: &context::Value, vr: &context::Value) -> context::Value {
        let (context::Value::Number(vl), context::Value::Number(vr)) = (&vl, &vr) else {
            Interpreter::runtime_error("Operands must be numbers.");
        };
        context::Value::Number(vl * vr)
    }

    fn op_div(vl: &context::Value, vr: &context::Value) -> context::Value {
        let (context::Value::Number(vl), context::Value::Number(vr)) = (&vl, &vr) else {
            Interpreter::runtime_error("Operands must be numbers.");
        };
        context::Value::Number(vl / vr)
    }

    fn op_equal(vl: &context::Value, vr: &context::Value) -> context::Value {
        context::Value::Boolean(vl == vr)
    }

    fn op_not_equal(vl: &context::Value, vr: &context::Value) -> context::Value {
        context::Value::Boolean(vl != vr)
    }

    fn op_less(vl: &context::Value, vr: &context::Value) -> context::Value {
        let (context::Value::Number(vl), context::Value::Number(vr)) = (&vl, &vr) else {
            Interpreter::runtime_error("Operands must be numbers.");
        };
        context::Value::Boolean(vl < vr)
    }

    fn op_less_equal(vl: &context::Value, vr: &context::Value) -> context::Value {
        let (context::Value::Number(vl), context::Value::Number(vr)) = (&vl, &vr) else {
            Interpreter::runtime_error("Operands must be numbers.");
        };
        context::Value::Boolean(vl <= vr)
    }

    fn op_greater(vl: &context::Value, vr: &context::Value) -> context::Value {
        let (context::Value::Number(vl), context::Value::Number(vr)) = (&vl, &vr) else {
            Interpreter::runtime_error("Operands must be numbers.");
        };
        context::Value::Boolean(vl > vr)
    }

    fn op_greater_equal(vl: &context::Value, vr: &context::Value) -> context::Value {
        let (context::Value::Number(vl), context::Value::Number(vr)) = (&vl, &vr) else {
            Interpreter::runtime_error("Operands must be numbers.");
        };
        context::Value::Boolean(vl >= vr)
    }

    fn evaluate_binary(
        &mut self,
        expr: &ast::expr::BinaryOperation<'a>,
        defs: &'a context::Definitions,
    ) -> context::NodeResult {
        let ast::expr::BinaryOperation {
            operator: op,
            left: lhs,
            right: rhs,
        } = expr;

        let lhs_value = self.evaluate_expression(lhs, defs).value;
        let rhs_value = self.evaluate_expression(rhs, defs).value;

        match op.kind {
            lexer::TokenKind::Plus => {
                return context::NodeResult::new(
                    Interpreter::op_add(&lhs_value, &rhs_value),
                    context::Control::Continue,
                );
            }
            lexer::TokenKind::Minus => {
                return context::NodeResult::new(
                    Interpreter::op_sub(&lhs_value, &rhs_value),
                    context::Control::Continue,
                );
            }
            lexer::TokenKind::Star => {
                return context::NodeResult::new(
                    Interpreter::op_mul(&lhs_value, &rhs_value),
                    context::Control::Continue,
                );
            }
            lexer::TokenKind::Slash => {
                return context::NodeResult::new(
                    Interpreter::op_div(&lhs_value, &rhs_value),
                    context::Control::Continue,
                );
            }
            lexer::TokenKind::Less => {
                return context::NodeResult::new(
                    Interpreter::op_less(&lhs_value, &rhs_value),
                    context::Control::Continue,
                );
            }
            lexer::TokenKind::LessEqual => {
                return context::NodeResult::new(
                    Interpreter::op_less_equal(&lhs_value, &rhs_value),
                    context::Control::Continue,
                );
            }
            lexer::TokenKind::Greater => {
                return context::NodeResult::new(
                    Interpreter::op_greater(&lhs_value, &rhs_value),
                    context::Control::Continue,
                );
            }
            lexer::TokenKind::GreaterEqual => {
                return context::NodeResult::new(
                    Interpreter::op_greater_equal(&lhs_value, &rhs_value),
                    context::Control::Continue,
                );
            }
            lexer::TokenKind::EqualEqual => {
                return context::NodeResult::new(
                    Interpreter::op_equal(&lhs_value, &rhs_value),
                    context::Control::Continue,
                );
            }
            lexer::TokenKind::BangEqual => {
                return context::NodeResult::new(
                    Interpreter::op_not_equal(&lhs_value, &rhs_value),
                    context::Control::Continue,
                );
            }
            _ => {}
        }

        panic!();
    }

    fn evaluate_logical_or(
        &mut self,
        expr: &ast::expr::LogicalOr<'a>,
        defs: &'a context::Definitions,
    ) -> context::NodeResult {
        let ast::expr::LogicalOr {
            left: lhs,
            right: rhs,
        } = expr;

        let lhs_result = self.evaluate_expression(lhs, defs);

        if is_truthy(&lhs_result.value) {
            context::NodeResult::new(lhs_result.value, context::Control::Continue)
        } else {
            let rhs_result = self.evaluate_expression(rhs, defs);
            context::NodeResult::new(rhs_result.value, context::Control::Continue)
        }
    }

    fn evaluate_logical_and(
        &mut self,
        expr: &ast::expr::LogicalAnd<'a>,
        defs: &'a context::Definitions,
    ) -> context::NodeResult {
        let ast::expr::LogicalAnd {
            left: lhs,
            right: rhs,
        } = expr;

        let lhs_result = self.evaluate_expression(lhs, defs);

        if is_truthy(&lhs_result.value) {
            let rhs_result = self.evaluate_expression(rhs, defs);
            context::NodeResult::new(rhs_result.value, context::Control::Continue)
        } else {
            context::NodeResult::new(lhs_result.value, context::Control::Continue)
        }
    }

    fn evaluate_function_call(
        &mut self,
        callee: &context::Function,
        arguments: &[context::Value],
        defs: &'a context::Definitions,
    ) -> context::Value {
        let context::Function {
            context_function_slot_idx,
            closure_id,
        } = callee;
        let context::FunctionDefinition {
            pars,
            body,
            is_native,
            native_function,
            is_initializer,
            ..
        } = &defs.functions[*context_function_slot_idx];

        if pars.len() == arguments.len() {
            self.context.push_new_environment(Some(*closure_id));

            if *is_native {
                match native_function {
                    context::NativeFunction::Clock => {
                        self.context.pop_environment();
                        let s = SystemTime::now()
                            .duration_since(UNIX_EPOCH)
                            .unwrap()
                            .as_secs_f64();
                        return context::Value::Number(s);
                    }
                    context::NativeFunction::None => {
                        panic!()
                    }
                }
            }

            for i in 0..pars.len() {
                self.context.insert_symbol(pars[i], arguments[i].clone());
            }

            let ast::stmt::Statement::Block(ast::stmt::Block { statements }) = &body else {
                panic!("Expected block statement");
            };
            for statement in statements {
                let statement_result = self.execute_statement(statement, defs);
                if statement_result.control == context::Control::FunctionReturn {
                    if *is_initializer {
                        let return_value =
                            self.context
                                .get_symbol_value("this", &mut Some(1), &mut Some(0));
                        self.context.pop_environment();
                        return return_value;
                    }
                    self.context.pop_environment();
                    return statement_result.value;
                }
            }

            if *is_initializer {
                let return_value =
                    self.context
                        .get_symbol_value("this", &mut Some(1), &mut Some(0));
                self.context.pop_environment();
                return_value
            } else {
                self.context.pop_environment();
                context::Value::Nil
            }
        } else {
            panic!(
                "Expected {} arguments but got {}.",
                pars.len(),
                arguments.len()
            );
        }
    }

    fn evaluate_call(
        &mut self,
        expr: &ast::expr::Call<'a>,
        defs: &'a context::Definitions,
    ) -> context::NodeResult {
        let ast::expr::Call { callee, arguments } = &expr;

        let callee = self.evaluate_expression(callee, defs).value;

        if let context::Value::Function(function) = &callee {
            let mut argument_values = Vec::<context::Value>::new();

            for arg in arguments {
                argument_values.push(self.evaluate_expression(arg, defs).value);
            }

            context::NodeResult::new(
                self.evaluate_function_call(function, &argument_values, defs),
                context::Control::Continue,
            )
        } else if let context::Value::Class { .. } = &callee {
            let new_instance =
                context::Value::Instance(self.context.insert_instance(context::Instance {
                    class: callee,
                    properties: HashMap::<&'a str, context::Value>::new(),
                    reachable: false,
                }));

            let init_token = lexer::Token {
                kind: lexer::TokenKind::Identifier,
                lexeme: "init",
                line_number: 0,
            };

            if let Some(init_bound_function) =
                self.evaluate_get_internal(&new_instance, &init_token, defs)
            {
                if let context::Value::Function(function) = init_bound_function {
                    let mut argument_values = Vec::<context::Value>::new();

                    for arg in arguments {
                        argument_values.push(self.evaluate_expression(arg, defs).value);
                    }

                    self.evaluate_function_call(&function, &argument_values, defs);
                }
            } else if !arguments.is_empty() {
                panic!("Expected 0 arguments but got {}.", arguments.len());
            }

            context::NodeResult::new(new_instance, context::Control::Continue)
        } else {
            panic!("Can only call functions and classes.");
        }
    }

    fn evaluate_expression(
        &mut self,
        expr: &ast::expr::Expression<'a>,
        defs: &'a context::Definitions,
    ) -> context::NodeResult {
        match expr {
            ast::expr::Expression::Literal(expr) => Interpreter::evaluate_literal(expr),
            ast::expr::Expression::UnaryOperation(expr) => self.evaluate_unary(expr, defs),
            ast::expr::Expression::BinaryOperation(expr) => self.evaluate_binary(expr, defs),
            ast::expr::Expression::Parentheses(expr) => self.evaluate_parentheses(expr, defs),
            ast::expr::Expression::Variable(expr) => self.evaluate_variable(expr),
            ast::expr::Expression::This => self.evaluate_this(),
            ast::expr::Expression::Assignment(expr) => self.evaluate_assignment(expr, defs),
            ast::expr::Expression::LogicalOr(expr) => self.evaluate_logical_or(expr, defs),
            ast::expr::Expression::LogicalAnd(expr) => self.evaluate_logical_and(expr, defs),
            ast::expr::Expression::Call(expr) => self.evaluate_call(expr, defs),
            ast::expr::Expression::Get(expr) => self.evaluate_get(expr, defs),
            ast::expr::Expression::Set(expr) => self.evaluate_set(expr, defs),
            ast::expr::Expression::Super(expr) => self.evaluate_super(expr, defs),
        }
    }

    fn execute_statement(
        &mut self,
        statement: &ast::stmt::Statement<'a>,
        defs: &'a context::Definitions,
    ) -> context::NodeResult {
        self.statement_exec_count += 1;

        if self.statement_exec_count.is_multiple_of(16384) {
            let working_set_size = self.context.environments.len() + self.context.instances.len();

            if working_set_size > self.working_set_size_after_last_gc * 3 / 2
                && working_set_size > 256
            {
                // println!("gc started with #envs: {}, #instances: {} at {}", self.context.environments.len(), self.context.instances.len(), self.statement_exec_count);
                self.context.gc_collect(defs);
                self.working_set_size_after_last_gc =
                    self.context.environments.len() + self.context.instances.len();
                // println!("Class slot count: {}", self.context.classes.len());
                // println!("gc finished with #envs: {}, #instances: {}", self.context.environments.len(), self.context.instances.len());
                // println!();
            }
        }

        match statement {
            ast::stmt::Statement::Expression(stmt) => self.execute_expression_statement(stmt, defs),
            ast::stmt::Statement::Print(stmt) => self.execute_print_statement(stmt, defs),
            ast::stmt::Statement::Return(stmt) => self.execute_return_statement(stmt, defs),
            ast::stmt::Statement::While(stmt) => self.execute_while_statement(stmt, defs),
            ast::stmt::Statement::For(stmt) => self.execute_for_statement(stmt, defs),
            ast::stmt::Statement::VariableDeclaration(stmt) => {
                self.execute_variable_declaration_statement(stmt, defs)
            }
            ast::stmt::Statement::FunctionDeclaration(stmt) => {
                self.execute_function_declaration_statement(stmt, defs)
            }
            ast::stmt::Statement::ClassDeclaration(stmt) => {
                self.execute_class_declaration_statement(stmt)
            }
            ast::stmt::Statement::Block(stmt) => self.execute_block_statement(stmt, defs),
            ast::stmt::Statement::If(stmt) => self.execute_if_statement(stmt, defs),
        }
    }

    fn execute_expression_statement(
        &mut self,
        stmt: &ast::stmt::Expression<'a>,
        defs: &'a context::Definitions,
    ) -> context::NodeResult {
        let ast::stmt::Expression { expr } = stmt;
        let expr_result = self.evaluate_expression(expr, defs);
        context::NodeResult::new(context::Value::Nil, expr_result.control)
    }

    fn execute_print_statement(
        &mut self,
        stmt: &ast::stmt::Print<'a>,
        defs: &'a context::Definitions,
    ) -> context::NodeResult {
        let ast::stmt::Print { expr } = stmt;

        let expr_result = self.evaluate_expression(expr, defs);

        match expr_result.value {
            context::Value::Boolean(b) => {
                println!("{b}");
            }
            context::Value::Number(n) => {
                println!("{n}");
            }
            context::Value::Nil => {
                println!("nil");
            }
            context::Value::String(s) => {
                println!("{s}");
            }
            context::Value::Function(context::Function {
                context_function_slot_idx,
                ..
            }) => {
                let context::FunctionDefinition {
                    is_native,
                    is_method,
                    name,
                    ..
                } = &defs.functions[context_function_slot_idx];
                if *is_method {
                    println!("<fn method>");
                } else if *is_native {
                    println!("<native fn>");
                } else {
                    println!("<fn {name}>");
                }
            }
            context::Value::Class(context::Class {
                context_class_slot_idx,
                ..
            }) => {
                let context::ClassDefinition { name, .. } = &defs.classes[context_class_slot_idx];
                println!("{name}");
            }
            context::Value::Instance(instance_id) => {
                let instance = self.context.instances.get(&instance_id).unwrap();
                let context::Instance { class, .. } = instance;
                let context::Value::Class(context::Class {
                    context_class_slot_idx,
                    ..
                }) = class
                else {
                    panic!()
                };
                let context::ClassDefinition { name, .. } = &defs.classes[*context_class_slot_idx];
                println!("{name} instance");
            }
        }

        context::NodeResult::new(context::Value::Nil, expr_result.control)
    }

    fn execute_return_statement(
        &mut self,
        stmt: &ast::stmt::Return<'a>,
        defs: &'a context::Definitions,
    ) -> context::NodeResult {
        let ast::stmt::Return { expr } = stmt;

        match expr {
            Some(ast::stmt::Expression { expr }) => {
                let expr_result = self.evaluate_expression(expr, defs);
                context::NodeResult::new(expr_result.value, context::Control::FunctionReturn)
            }
            None => context::NodeResult::new(context::Value::Nil, context::Control::FunctionReturn),
        }
    }

    fn execute_while_statement(
        &mut self,
        stmt: &ast::stmt::While<'a>,
        defs: &'a context::Definitions,
    ) -> context::NodeResult {
        let ast::stmt::While {
            condition: ast::stmt::Expression { expr },
            body,
        } = stmt;

        loop {
            let expr_value = self.evaluate_expression(expr, defs).value;
            if is_truthy(&expr_value) {
                let statement_result = self.execute_statement(body, defs);
                if statement_result.control == context::Control::FunctionReturn {
                    return statement_result;
                }
            } else {
                break;
            }
        }

        context::NodeResult::new(context::Value::Nil, context::Control::Continue)
    }

    fn execute_for_statement(
        &mut self,
        stmt: &ast::stmt::For<'a>,
        defs: &'a context::Definitions,
    ) -> context::NodeResult {
        let ast::stmt::For {
            init: initializer_statement,
            cond: condition_expression,
            incr: increment_expression,
            body: body_statement,
        } = stmt;

        self.context.push_new_environment_auto();

        if let Some(statement) = initializer_statement {
            self.execute_statement(statement.as_ref(), defs);
        }

        loop {
            if let Some(ast::stmt::Expression { expr }) = condition_expression {
                let expr_value = self.evaluate_expression(expr, defs).value;
                if !is_truthy(&expr_value) {
                    break;
                }
            }

            let body_statement_result = self.execute_statement(body_statement, defs);
            if body_statement_result.control == context::Control::FunctionReturn {
                self.context.pop_environment();
                return body_statement_result;
            }

            if let Some(ast::stmt::Expression { expr }) = increment_expression {
                self.evaluate_expression(expr, defs);
            }
        }

        self.context.pop_environment();
        context::NodeResult::new(context::Value::Nil, context::Control::Continue)
    }

    fn execute_if_statement(
        &mut self,
        stmt: &ast::stmt::If<'a>,
        defs: &'a context::Definitions,
    ) -> context::NodeResult {
        let ast::stmt::If {
            condition: ast::stmt::Expression { expr },
            then_statement,
            else_statement,
        } = stmt;
        let condition_expression_value = self.evaluate_expression(expr, defs).value;
        if is_truthy(&condition_expression_value) {
            self.execute_statement(then_statement, defs)
        } else if let Some(statement) = else_statement {
            self.execute_statement(statement, defs)
        } else {
            context::NodeResult::new(context::Value::Nil, context::Control::Continue)
        }
    }

    fn execute_function_declaration_statement(
        &mut self,
        stmt: &ast::stmt::FunctionDeclaration<'a>,
        _defs: &'a context::Definitions,
    ) -> context::NodeResult {
        let ast::stmt::FunctionDeclaration {
            id,
            context_function_slot_idx,
            ..
        } = stmt;

        assert!(
            !context_function_slot_idx.is_none(),
            "Internal interpreter error: {}",
            id.lexeme
        );

        self.context.insert_symbol(
            id.lexeme,
            context::Value::Function(context::Function {
                context_function_slot_idx: context_function_slot_idx.unwrap(),
                closure_id: self.context.get_current_environment_id(),
            }),
        );

        context::NodeResult::new(context::Value::Nil, context::Control::Continue)
    }

    fn execute_class_declaration_statement(
        &mut self,
        stmt: &ast::stmt::ClassDeclaration<'a>,
    ) -> context::NodeResult {
        let ast::stmt::ClassDeclaration {
            id,
            super_class: super_class_expr,
            context_class_slot_idx,
            ..
        } = stmt;

        let super_class_value: Option<Box<context::Value>>;

        if let Some(expr) = super_class_expr {
            let ast::expr::Expression::Variable(expr) = expr else {
                panic!("Expected variable expression");
            };

            super_class_value = Some(Box::new(self.evaluate_variable(expr).value));

            let context::Value::Class { .. } = super_class_value.as_ref().unwrap().as_ref() else {
                panic!("Superclass must be a class.");
            };
            self.context.push_new_environment_auto();
            self.context.insert_symbol(
                "super",
                super_class_value.as_ref().unwrap().as_ref().clone(),
            );
        } else {
            super_class_value = None;
        }

        let class_closure_id = self.context.get_current_environment_id();

        if super_class_expr.is_some() {
            self.context.pop_environment();
        }

        self.context.insert_symbol(
            id.lexeme,
            context::Value::Class(context::Class {
                context_class_slot_idx: context_class_slot_idx.unwrap(),
                closure_id: class_closure_id,
                super_class: super_class_value,
            }),
        );

        context::NodeResult::new(context::Value::Nil, context::Control::Continue)
    }

    fn execute_variable_declaration_statement(
        &mut self,
        stmt: &ast::stmt::VariableDeclaration<'a>,
        defs: &'a context::Definitions,
    ) -> context::NodeResult {
        let ast::stmt::VariableDeclaration { id, init } = stmt;

        let expr_value = match init {
            None => context::Value::Nil,
            Some(ast::stmt::Expression { expr }) => self.evaluate_expression(expr, defs).value,
        };

        self.context.insert_symbol(id.lexeme, expr_value);

        context::NodeResult::new(context::Value::Nil, context::Control::Continue)
    }

    fn execute_block_statement(
        &mut self,
        stmt: &ast::stmt::Block<'a>,
        defs: &'a context::Definitions,
    ) -> context::NodeResult {
        let ast::stmt::Block { statements } = stmt;

        self.context.push_new_environment_auto();
        for statement in statements {
            let statement_result = self.execute_statement(statement, defs);
            if statement_result.control == context::Control::FunctionReturn {
                self.context.pop_environment();
                return statement_result;
            }
        }
        self.context.pop_environment();

        context::NodeResult::new(context::Value::Nil, context::Control::Continue)
    }

    pub fn execute(
        &mut self,
        statements: &[ast::stmt::Statement<'a>],
        defs: &'a context::Definitions,
    ) {
        for statement in statements {
            self.execute_statement(statement, defs);
        }
    }
}
