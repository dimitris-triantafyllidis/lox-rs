use std::collections::HashMap;

use std::time::{SystemTime, UNIX_EPOCH};

use crate::{ast, context, lexer};

#[derive(Debug, Clone, PartialEq)]
pub struct Instance<'a> {
    pub class: Value,
    pub properties: HashMap<&'a str, Value>,
    pub reachable: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub enum NativeFunction {
    None,
    Clock,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FunctionDefinition<'a> {
    pub pars: Vec<&'a str>,
    pub body: ast::stmt::Statement<'a>,
    pub is_native: bool,
    pub is_initializer: bool,
    pub is_method: bool,
    pub native_function: NativeFunction,
    pub name: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ClassDefinition<'a> {
    pub methods: HashMap<&'a str, usize>,
    pub super_class: Option<usize>,
    pub name: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Function {
    pub context_function_slot_idx: usize,
    pub closure_id: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Class {
    pub context_class_slot_idx: usize,
    pub closure_id: usize,
    pub super_class: Option<Box<Value>>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Nil,
    Boolean(bool),
    Number(f64),
    String(String),
    Function(Function),
    Class(Class),
    Instance(usize),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Control {
    Continue,
    FunctionReturn,
}

#[derive(Debug, Clone, PartialEq)]
pub struct NodeResult {
    pub value: Value,
    pub control: Control,
}

impl NodeResult {
    pub fn new(value: Value, control: Control) -> Self {
        Self { value, control }
    }
}

pub fn is_truthy(v: &Value) -> bool {
    !matches!(v, Value::Nil | Value::Boolean(false))
}

pub struct Interpreter<'a> {
    pub context: context::Context<'a>,
    pub statement_exec_count: usize,
    pub working_set_size_after_last_gc: usize,
}

impl<'a> Interpreter<'a> {
    pub fn new(_ctx: context::Context<'a>) -> Self {
        let mut context = context::Context::new();

        context.push_new_environment_auto();

        context.insert_symbol(
            "clock",
            Value::Function(Function {
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

    fn runtime_error(&self, message: &str) -> ! {
        eprintln!("Runtime error:, '{message}'");
        std::process::exit(1);
    }

    fn evaluate_literal(
        &mut self,
        expr: &ast::expr::Literal<'a>,
        _defs: &'a context::Definitions,
    ) -> NodeResult {
        let ast::expr::Literal { token } = expr;

        match token.kind {
            lexer::TokenKind::Nil => NodeResult::new(Value::Nil, Control::Continue),
            lexer::TokenKind::False => NodeResult::new(Value::Boolean(false), Control::Continue),
            lexer::TokenKind::True => NodeResult::new(Value::Boolean(true), Control::Continue),
            lexer::TokenKind::Number => NodeResult::new(
                Value::Number(token.lexeme.parse().unwrap()),
                Control::Continue,
            ),
            lexer::TokenKind::String => NodeResult::new(
                Value::String(token.lexeme[1..token.lexeme.len() - 1].to_string()),
                Control::Continue,
            ),
            _ => panic!(),
        }
    }

    fn evaluate_variable(
        &mut self,
        expr: &ast::expr::Variable<'a>,
        _defs: &'a context::Definitions,
    ) -> NodeResult {
        let ast::expr::Variable {
            identifier: id,
            lookup_hop_count,
            env_slot_idx,
        } = expr;
        let mut lookup_hop_count_mut_copy = *lookup_hop_count;
        let mut env_slot_idx_mut_copy = *env_slot_idx;
        NodeResult::new(
            self.context.get_symbol_value(
                id.lexeme,
                &mut lookup_hop_count_mut_copy,
                &mut env_slot_idx_mut_copy,
            ),
            Control::Continue,
        )
    }

    fn evaluate_this(&mut self, _defs: &'a context::Definitions) -> NodeResult {
        NodeResult::new(
            self.context.get_symbol_value("this", &mut None, &mut None),
            Control::Continue,
        )
    }

    fn make_bound_method(
        &mut self,
        context_function_slot_idx: usize,
        closure_id: usize,
        instance: &Value,
    ) -> Value {
        self.context.push_new_environment(Some(closure_id));
        self.context.insert_symbol("this", instance.clone());
        self.context.pop_environment();
        Value::Function(Function {
            context_function_slot_idx,
            closure_id: self.context.next_env_id - 1,
        })
    }

    fn evaluate_super(
        &mut self,
        expr: &ast::expr::Super<'a>,
        defs: &'a context::Definitions,
    ) -> NodeResult {
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
            let Value::Class(Class {
                context_class_slot_idx,
                closure_id,
                super_class,
            }) = maybe_value
            else {
                panic!("Expected class value");
            };
            let ClassDefinition { ref methods, .. } = defs.classes[context_class_slot_idx];
            let Some(context_function_slot_idx) = methods.get(&property.lexeme) else {
                method_lookup_class = super_class.map(|boxed| *boxed);
                continue;
            };
            return NodeResult {
                value: self.make_bound_method(
                    *context_function_slot_idx,
                    closure_id,
                    &instance_value,
                ),
                control: Control::Continue,
            };
        }
        panic!("Undefined property '{}'.", property.lexeme);
    }

    fn evaluate_set(
        &mut self,
        expr: &ast::expr::Set<'a>,
        defs: &'a context::Definitions,
    ) -> NodeResult {
        let ast::expr::Set {
            instance,
            property,
            value,
        } = expr;

        let instance = self.evaluate_expression(instance.as_ref(), defs).value;
        let value = self.evaluate_expression(value, defs).value;

        let Value::Instance(instance_id) = instance else {
            panic!("Only instances have fields.");
        };
        let instance_value = self.context.instances.get_mut(&instance_id).unwrap();
        instance_value
            .properties
            .insert(property.lexeme, value.clone());
        NodeResult::new(value, Control::Continue)
    }

    fn evaluate_get_internal(
        &mut self,
        instance_value: &Value,
        property: &lexer::Token<'a>,
        defs: &'a context::Definitions,
    ) -> Option<Value> {
        let Value::Instance(instance_id) = instance_value else {
            panic!("Only instances have properties.");
        };
        let instance = self.context.instances.get(instance_id).unwrap();

        if let Some(value) = instance.properties.get(&property.lexeme) {
            Some(value.clone())
        } else {
            let mut method_lookup_class = Some(instance.class.clone());
            while let Some(maybe_value) = method_lookup_class {
                let Value::Class(Class {
                    context_class_slot_idx,
                    closure_id,
                    super_class,
                }) = maybe_value
                else {
                    panic!("Expected class value");
                };
                let ClassDefinition { methods, .. } = &defs.classes[context_class_slot_idx];
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
    ) -> NodeResult {
        let ast::expr::Get { instance, property } = expr;
        let instance_value = self.evaluate_expression(instance.as_ref(), defs).value;

        let Some(value) = self.evaluate_get_internal(&instance_value, property, defs) else {
            self.runtime_error(&std::format!("Undefined property '{}'.", property.lexeme));
        };
        NodeResult::new(value, Control::Continue)
    }

    fn evaluate_unary(
        &mut self,
        expr: &ast::expr::UnaryOperation<'a>,
        defs: &'a context::Definitions,
    ) -> NodeResult {
        let ast::expr::UnaryOperation {
            operator: op,
            right: rhs,
        } = expr;
        let rhs_result = self.evaluate_expression(rhs, defs);

        if op.kind == lexer::TokenKind::Minus {
            let Value::Number(v) = rhs_result.value else {
                self.runtime_error("Operand must be a number.");
            };
            NodeResult::new(Value::Number(-v), Control::Continue)
        } else if op.kind == lexer::TokenKind::Bang {
            NodeResult::new(
                Value::Boolean(!is_truthy(&rhs_result.value)),
                Control::Continue,
            )
        } else {
            panic!();
        }
    }

    fn evaluate_parentheses(
        &mut self,
        expr: &ast::expr::Parentheses<'a>,
        defs: &'a context::Definitions,
    ) -> NodeResult {
        let ast::expr::Parentheses { expression: e } = expr;
        let expression_result = self.evaluate_expression(e, defs);
        NodeResult::new(expression_result.value, Control::Continue)
    }

    fn evaluate_assignment(
        &mut self,
        expr: &ast::expr::Assignment<'a>,
        defs: &'a context::Definitions,
    ) -> NodeResult {
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
        NodeResult::new(rhs_result.value, Control::Continue)
    }

    fn op_add(&self, vl: &Value, vr: &Value) -> Value {
        if let (Value::Number(vl), Value::Number(vr)) = (&vl, &vr) {
            Value::Number(vl + vr)
        } else if let (Value::String(vl), Value::String(vr)) = (&vl, &vr) {
            Value::String(vl.clone() + vr)
        } else {
            self.runtime_error("Operands must be two numbers or two strings.");
        }
    }

    fn op_sub(&self, vl: &Value, vr: &Value) -> Value {
        let (Value::Number(vl), Value::Number(vr)) = (&vl, &vr) else {
            self.runtime_error("Operands must be numbers.");
        };
        Value::Number(vl - vr)
    }

    fn op_mul(&self, vl: &Value, vr: &Value) -> Value {
        let (Value::Number(vl), Value::Number(vr)) = (&vl, &vr) else {
            self.runtime_error("Operands must be numbers.");
        };
        Value::Number(vl * vr)
    }

    fn op_div(&self, vl: &Value, vr: &Value) -> Value {
        let (Value::Number(vl), Value::Number(vr)) = (&vl, &vr) else {
            self.runtime_error("Operands must be numbers.");
        };
        Value::Number(vl / vr)
    }

    fn op_equal(&self, vl: &Value, vr: &Value) -> Value {
        Value::Boolean(vl == vr)
    }

    fn op_not_equal(&self, vl: &Value, vr: &Value) -> Value {
        Value::Boolean(vl != vr)
    }

    fn op_less(&self, vl: &Value, vr: &Value) -> Value {
        let (Value::Number(vl), Value::Number(vr)) = (&vl, &vr) else {
            self.runtime_error("Operands must be numbers.");
        };
        Value::Boolean(vl < vr)
    }

    fn op_less_equal(&self, vl: &Value, vr: &Value) -> Value {
        let (Value::Number(vl), Value::Number(vr)) = (&vl, &vr) else {
            self.runtime_error("Operands must be numbers.");
        };
        Value::Boolean(vl <= vr)
    }

    fn op_greater(&self, vl: &Value, vr: &Value) -> Value {
        let (Value::Number(vl), Value::Number(vr)) = (&vl, &vr) else {
            self.runtime_error("Operands must be numbers.");
        };
        Value::Boolean(vl > vr)
    }

    fn op_greater_equal(&self, vl: &Value, vr: &Value) -> Value {
        let (Value::Number(vl), Value::Number(vr)) = (&vl, &vr) else {
            self.runtime_error("Operands must be numbers.");
        };
        Value::Boolean(vl >= vr)
    }

    fn evaluate_binary(
        &mut self,
        expr: &ast::expr::BinaryOperation<'a>,
        defs: &'a context::Definitions,
    ) -> NodeResult {
        let ast::expr::BinaryOperation {
            operator: op,
            left: lhs,
            right: rhs,
        } = expr;

        let lhs_value = self.evaluate_expression(lhs, defs).value;
        let rhs_value = self.evaluate_expression(rhs, defs).value;

        match op.kind {
            lexer::TokenKind::Plus => {
                return NodeResult::new(self.op_add(&lhs_value, &rhs_value), Control::Continue);
            }
            lexer::TokenKind::Minus => {
                return NodeResult::new(self.op_sub(&lhs_value, &rhs_value), Control::Continue);
            }
            lexer::TokenKind::Star => {
                return NodeResult::new(self.op_mul(&lhs_value, &rhs_value), Control::Continue);
            }
            lexer::TokenKind::Slash => {
                return NodeResult::new(self.op_div(&lhs_value, &rhs_value), Control::Continue);
            }
            lexer::TokenKind::Less => {
                return NodeResult::new(self.op_less(&lhs_value, &rhs_value), Control::Continue);
            }
            lexer::TokenKind::LessEqual => {
                return NodeResult::new(
                    self.op_less_equal(&lhs_value, &rhs_value),
                    Control::Continue,
                );
            }
            lexer::TokenKind::Greater => {
                return NodeResult::new(self.op_greater(&lhs_value, &rhs_value), Control::Continue);
            }
            lexer::TokenKind::GreaterEqual => {
                return NodeResult::new(
                    self.op_greater_equal(&lhs_value, &rhs_value),
                    Control::Continue,
                );
            }
            lexer::TokenKind::EqualEqual => {
                return NodeResult::new(self.op_equal(&lhs_value, &rhs_value), Control::Continue);
            }
            lexer::TokenKind::BangEqual => {
                return NodeResult::new(
                    self.op_not_equal(&lhs_value, &rhs_value),
                    Control::Continue,
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
    ) -> NodeResult {
        let ast::expr::LogicalOr {
            left: lhs,
            right: rhs,
        } = expr;

        let lhs_result = self.evaluate_expression(lhs, defs);

        if is_truthy(&lhs_result.value) {
            NodeResult::new(lhs_result.value, Control::Continue)
        } else {
            let rhs_result = self.evaluate_expression(rhs, defs);
            NodeResult::new(rhs_result.value, Control::Continue)
        }
    }

    fn evaluate_logical_and(
        &mut self,
        expr: &ast::expr::LogicalAnd<'a>,
        defs: &'a context::Definitions,
    ) -> NodeResult {
        let ast::expr::LogicalAnd {
            left: lhs,
            right: rhs,
        } = expr;

        let lhs_result = self.evaluate_expression(lhs, defs);

        if is_truthy(&lhs_result.value) {
            let rhs_result = self.evaluate_expression(rhs, defs);
            NodeResult::new(rhs_result.value, Control::Continue)
        } else {
            NodeResult::new(lhs_result.value, Control::Continue)
        }
    }

    fn evaluate_function_call(
        &mut self,
        callee: &Function,
        arguments: &[Value],
        defs: &'a context::Definitions,
    ) -> Value {
        let Function {
            context_function_slot_idx,
            closure_id,
        } = callee;
        let FunctionDefinition {
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
                    NativeFunction::Clock => {
                        self.context.pop_environment();
                        let ms = SystemTime::now()
                            .duration_since(UNIX_EPOCH)
                            .unwrap()
                            .as_millis();
                        return Value::Number((ms as f64) / 1000.0);
                    }
                    NativeFunction::None => {
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
                if statement_result.control == Control::FunctionReturn {
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
                Value::Nil
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
    ) -> NodeResult {
        let ast::expr::Call { callee, arguments } = &expr;

        let callee = self.evaluate_expression(callee, defs).value;

        if let Value::Function(function) = &callee {
            let mut argument_values = Vec::<Value>::new();

            for arg in arguments {
                argument_values.push(self.evaluate_expression(arg, defs).value);
            }

            NodeResult::new(
                self.evaluate_function_call(function, &argument_values, defs),
                Control::Continue,
            )
        } else if let Value::Class { .. } = &callee {
            let new_instance = Value::Instance(self.context.insert_instance(Instance {
                class: callee,
                properties: HashMap::<&'a str, Value>::new(),
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
                if let Value::Function(function) = init_bound_function {
                    let mut argument_values = Vec::<Value>::new();

                    for arg in arguments {
                        argument_values.push(self.evaluate_expression(arg, defs).value);
                    }

                    self.evaluate_function_call(&function, &argument_values, defs);
                }
            } else if !arguments.is_empty() {
                panic!("Expected 0 arguments but got {}.", arguments.len());
            }

            NodeResult::new(new_instance, Control::Continue)
        } else {
            panic!("Can only call functions and classes.");
        }
    }

    fn evaluate_expression(
        &mut self,
        expr: &ast::expr::Expression<'a>,
        defs: &'a context::Definitions,
    ) -> NodeResult {
        match expr {
            ast::expr::Expression::Literal(expr) => self.evaluate_literal(expr, defs),
            ast::expr::Expression::UnaryOperation(expr) => self.evaluate_unary(expr, defs),
            ast::expr::Expression::BinaryOperation(expr) => self.evaluate_binary(expr, defs),
            ast::expr::Expression::Parentheses(expr) => self.evaluate_parentheses(expr, defs),
            ast::expr::Expression::Variable(expr) => self.evaluate_variable(expr, defs),
            ast::expr::Expression::This => self.evaluate_this(defs),
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
    ) -> NodeResult {
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
                //println!();
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
                self.execute_class_declaration_statement(stmt, defs)
            }
            ast::stmt::Statement::Block(stmt) => self.execute_block_statement(stmt, defs),
            ast::stmt::Statement::If(stmt) => self.execute_if_statement(stmt, defs),
        }
    }

    fn execute_expression_statement(
        &mut self,
        stmt: &ast::stmt::Expression<'a>,
        defs: &'a context::Definitions,
    ) -> NodeResult {
        let ast::stmt::Expression { expr } = stmt;
        let expr_result = self.evaluate_expression(expr, defs);
        NodeResult::new(Value::Nil, expr_result.control)
    }

    fn execute_print_statement(
        &mut self,
        stmt: &ast::stmt::Print<'a>,
        defs: &'a context::Definitions,
    ) -> NodeResult {
        let ast::stmt::Print { expr } = stmt;

        let expr_result = self.evaluate_expression(expr, defs);

        match expr_result.value {
            Value::Boolean(b) => {
                println!("{b}");
            }
            Value::Number(n) => {
                println!("{n}");
            }
            Value::Nil => {
                println!("nil");
            }
            Value::String(s) => {
                println!("{s}");
            }
            Value::Function(Function {
                context_function_slot_idx,
                ..
            }) => {
                let FunctionDefinition {
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
            Value::Class(Class {
                context_class_slot_idx,
                ..
            }) => {
                let ClassDefinition { name, .. } = &defs.classes[context_class_slot_idx];
                println!("{name}");
            }
            Value::Instance(instance_id) => {
                let instance = self.context.instances.get(&instance_id).unwrap();
                let Instance { class, .. } = instance;
                let Value::Class(Class {
                    context_class_slot_idx,
                    ..
                }) = class
                else {
                    panic!()
                };
                let ClassDefinition { name, .. } = &defs.classes[*context_class_slot_idx];
                println!("{name} instance");
            }
        }

        NodeResult::new(Value::Nil, expr_result.control)
    }

    fn execute_return_statement(
        &mut self,
        stmt: &ast::stmt::Return<'a>,
        defs: &'a context::Definitions,
    ) -> NodeResult {
        let ast::stmt::Return { expr } = stmt;

        match expr {
            Some(ast::stmt::Expression { expr }) => {
                let expr_result = self.evaluate_expression(expr, defs);
                NodeResult::new(expr_result.value, Control::FunctionReturn)
            }
            None => NodeResult::new(Value::Nil, Control::FunctionReturn),
        }
    }

    fn execute_while_statement(
        &mut self,
        stmt: &ast::stmt::While<'a>,
        defs: &'a context::Definitions,
    ) -> NodeResult {
        let ast::stmt::While {
            condition: ast::stmt::Expression { expr },
            body,
        } = stmt;

        loop {
            let expr_value = self.evaluate_expression(expr, defs).value;
            if is_truthy(&expr_value) {
                let statement_result = self.execute_statement(body, defs);
                if statement_result.control == Control::FunctionReturn {
                    return statement_result;
                }
            } else {
                break;
            }
        }

        NodeResult::new(Value::Nil, Control::Continue)
    }

    fn execute_for_statement(
        &mut self,
        stmt: &ast::stmt::For<'a>,
        defs: &'a context::Definitions,
    ) -> NodeResult {
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
            if body_statement_result.control == Control::FunctionReturn {
                self.context.pop_environment();
                return body_statement_result;
            }

            if let Some(ast::stmt::Expression { expr }) = increment_expression {
                self.evaluate_expression(expr, defs);
            }
        }

        self.context.pop_environment();
        NodeResult::new(Value::Nil, Control::Continue)
    }

    fn execute_if_statement(
        &mut self,
        stmt: &ast::stmt::If<'a>,
        defs: &'a context::Definitions,
    ) -> NodeResult {
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
            NodeResult::new(Value::Nil, Control::Continue)
        }
    }

    fn execute_function_declaration_statement(
        &mut self,
        stmt: &ast::stmt::FunctionDeclaration<'a>,
        _defs: &'a context::Definitions,
    ) -> NodeResult {
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
            Value::Function(Function {
                context_function_slot_idx: context_function_slot_idx.unwrap(),
                closure_id: self.context.get_current_environment_id(),
            }),
        );

        NodeResult::new(Value::Nil, Control::Continue)
    }

    fn execute_class_declaration_statement(
        &mut self,
        stmt: &ast::stmt::ClassDeclaration<'a>,
        defs: &'a context::Definitions,
    ) -> NodeResult {
        let ast::stmt::ClassDeclaration {
            id,
            super_class: super_class_expr,
            context_class_slot_idx,
            ..
        } = stmt;

        let super_class_value: Option<Box<Value>>;

        if let Some(expr) = super_class_expr {
            let ast::expr::Expression::Variable(expr) = expr else {
                panic!("Expected variable expression");
            };

            super_class_value = Some(Box::new(self.evaluate_variable(expr, defs).value));

            let Value::Class { .. } = super_class_value.as_ref().unwrap().as_ref() else {
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
            Value::Class(Class {
                context_class_slot_idx: context_class_slot_idx.unwrap(),
                closure_id: class_closure_id,
                super_class: super_class_value,
            }),
        );

        NodeResult::new(Value::Nil, Control::Continue)
    }

    fn execute_variable_declaration_statement(
        &mut self,
        stmt: &ast::stmt::VariableDeclaration<'a>,
        defs: &'a context::Definitions,
    ) -> NodeResult {
        let ast::stmt::VariableDeclaration { id, init } = stmt;

        let expr_value = match init {
            None => Value::Nil,
            Some(ast::stmt::Expression { expr }) => self.evaluate_expression(expr, defs).value,
        };

        self.context.insert_symbol(id.lexeme, expr_value);

        NodeResult::new(Value::Nil, Control::Continue)
    }

    fn execute_block_statement(
        &mut self,
        stmt: &ast::stmt::Block<'a>,
        defs: &'a context::Definitions,
    ) -> NodeResult {
        let ast::stmt::Block { statements } = stmt;

        self.context.push_new_environment_auto();
        for statement in statements {
            let statement_result = self.execute_statement(statement, defs);
            if statement_result.control == Control::FunctionReturn {
                self.context.pop_environment();
                return statement_result;
            }
        }
        self.context.pop_environment();

        NodeResult::new(Value::Nil, Control::Continue)
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
