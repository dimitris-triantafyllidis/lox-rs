use std::collections::HashMap;

use std::time::{SystemTime, UNIX_EPOCH};

use crate::lexer::*;
use crate::parser::*;
use crate::context::*;

use expression::*;
use statement::*;

#[derive(Debug, Clone, PartialEq)]
pub struct Instance<'a> {
    pub class: Value,
    pub properties: HashMap<&'a str, Value>,
    pub reachable: bool
}

#[derive(Debug, Clone, PartialEq)]
pub enum NativeFunction {
    None,
    Clock
}

#[derive(Debug, Clone, PartialEq)]
pub struct Function<'a> {
    pub pars: Vec<&'a str>,
    pub body: Statement<'a>,
    pub is_native: bool,
    pub is_initializer: bool,
    pub is_method: bool,
    pub native_function: NativeFunction,
    pub name: String
}

#[derive(Debug, Clone, PartialEq)]
pub struct Class<'a> {
    pub methods: HashMap<&'a str, usize>,
    pub super_class: Option<usize>,
    pub name: String
}

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Nil,
    Boolean (bool),
    Number (f64),
    String (String),
    Function {
        context_function_slot_idx: usize,
        closure_id: usize
    },
    Class {
        context_class_slot_idx: usize,
        closure_id: usize,
        super_class: Option<Box<Value>>
    },
    Instance (usize)
}

#[derive(Debug, Clone, PartialEq)]
pub enum Control {
    Continue,
    FunctionReturn
}

#[derive(Debug, Clone, PartialEq)]
pub struct NodeResult {
    pub value: Value,
    pub control: Control
}

impl<'a> NodeResult {
    pub fn new(value: Value, control: Control) -> Self {
        return Self {
            value,
            control
        }
    }
}

pub fn is_truthy(v: &Value) -> bool {
    match v {
        Value::Nil => false,
        Value::Boolean(false) => false,
        _ => true
    }
}
pub struct Interpreter<'a> {
    pub context: Context<'a>,
    pub statement_exec_count: usize,
    pub working_set_size: usize
}

impl<'a> Interpreter<'a> {

    pub fn new(_ctx: Context<'a>) -> Self {

        let mut context = Context::new();

        context.push_new_environment_auto();

        context.insert_symbol (
            "clock",
            Value::Function {
                context_function_slot_idx: 0,
                closure_id: 0
            }
        );

        Self {
            context,
            statement_exec_count: 0,
            working_set_size: 0
        }
    }

    pub fn runtime_error(&self, message: &str) -> ! {
        eprintln!("Runtime error:, '{}'", message);
        std::process::exit(1);
    }

    pub fn evaluate_literal(&mut self, expr: &expression::Literal<'a>, _defs: &'a Definitions) -> NodeResult {

        let Literal { token } = expr;

        match token.kind {
            TokenKind::Nil => {
                return NodeResult::new(Value::Nil, Control::Continue);
            },
            TokenKind::False => {
                return NodeResult::new(Value::Boolean(false), Control::Continue);
            },
            TokenKind::True  => {
                return NodeResult::new(Value::Boolean(true), Control::Continue);
            },
            TokenKind::Number => {
                return NodeResult::new (
                    Value::Number (
                        token.lexeme.parse().unwrap()
                    ),
                    Control::Continue
                )
            },
            TokenKind::String => {
                return NodeResult::new (
                    Value::String (
                        token.lexeme[1..token.lexeme.len() - 1].to_string()
                    ),
                    Control::Continue
                )
            },
            _ => panic!()
        }
    }

    pub fn evaluate_variable(&mut self, expr: &expression::Variable<'a>, _defs: &'a Definitions) -> NodeResult {

        let Variable { identifier: id, lookup_hop_count, env_slot_idx } = expr;
        let mut lookup_hop_count_mut_copy = *lookup_hop_count;
        let mut env_slot_idx_mut_copy = *env_slot_idx;
        NodeResult::new (
            self.context.get_symbol_value(&id.lexeme, &mut lookup_hop_count_mut_copy, &mut env_slot_idx_mut_copy),
            Control::Continue
        )
    }

    pub fn evaluate_this(&mut self, _defs: &'a Definitions) -> NodeResult {

        NodeResult::new (
            self.context.get_symbol_value("this", &mut None, &mut None),
            Control::Continue
        )

    }

    pub fn evaluate_super(&mut self, expr: &expression::Super<'a>, defs: &'a Definitions) -> NodeResult {

        let Super { property, lookup_hop_count, env_slot_idx } = expr;

        let mut lookup_hop_count_mut_copy = *lookup_hop_count;
        let mut env_slot_idx_mut_copy = *env_slot_idx;
        let instance_value = self.context.get_symbol_value("this", &mut None, &mut None);
        let value: Value;

        let mut method_lookup_class = Some(Box::new(self.context.get_symbol_value("super", &mut lookup_hop_count_mut_copy, &mut env_slot_idx_mut_copy)));
        while let Some(maybe_value) = method_lookup_class {
            let Value::Class { context_class_slot_idx, closure_id, super_class } = maybe_value.as_ref() else {
                panic!("Expected class value");
            };
            let Class { ref methods, .. } = defs.classes[*context_class_slot_idx];
            if methods.contains_key(&property.lexeme) {
                let context_function_slot_idx = methods.get(&property.lexeme).unwrap().clone();
                let Function { .. } = defs.functions[context_function_slot_idx];
                self.context.push_new_environment(Some(*closure_id));
                self.context.insert_symbol("this", instance_value.clone());
                value = Value::Function {
                    context_function_slot_idx,
                    closure_id: self.context.next_env_id - 1
                };
                self.context.pop_environment();
                return NodeResult { value, control: Control::Continue };
            }
            else {
                method_lookup_class = super_class.clone();
                continue;
            }
        }
        panic!("Undefined property '{}'.", property.lexeme);
    }

    pub fn evaluate_set(&mut self, expr: &expression::Set<'a>, defs: &'a Definitions) -> NodeResult {

        let Set { instance, property, value } = expr;

        let instance = self.evaluate_expression(instance.as_ref(), defs).value;
        let value = self.evaluate_expression(value, defs).value;

        let Value::Instance ( instance_id ) = instance else {  panic!("Only instances have fields."); };
        let instance_value = self.context.instances.get_mut(&instance_id).unwrap();
        instance_value.properties.insert(property.lexeme, value.clone());
        return NodeResult::new (
            value,
            Control::Continue
        );
    }

    pub fn evaluate_get_internal(&mut self, instance_value: &Value, property: &Token<'a>, defs: &'a Definitions) -> Option<Value> {

        let Value::Instance ( instance_id ) = instance_value else {
            panic!("Only instances have properties.");
        };
        let instance = self.context.instances.get(instance_id).unwrap();
        let value: Value;

        if instance.properties.contains_key(&property.lexeme) {
            value = instance.properties.get(&property.lexeme).unwrap().clone();
            return Some(value);
        }
        else {
            let mut method_lookup_class = Some(Box::new(instance.class.clone()));
            while let Some(maybe_value) = method_lookup_class {
                let Value::Class { context_class_slot_idx, closure_id, super_class } = maybe_value.as_ref() else {
                    panic!("Expected class value");
                };
                let Class { methods, .. } = &defs.classes[*context_class_slot_idx];
                if methods.contains_key(&property.lexeme) {
                    let context_function_slot_idx = methods.get(&property.lexeme).unwrap().clone();
                    let Function { .. } = defs.functions[context_function_slot_idx];
                    self.context.push_new_environment(Some(*closure_id));
                    self.context.insert_symbol("this", instance_value.clone());
                    value = Value::Function {
                        context_function_slot_idx,
                        closure_id: self.context.next_env_id - 1
                    };
                    self.context.pop_environment();
                    return Some(value);

                }
                else {
                    method_lookup_class = super_class.clone();
                    continue;
                }
            }
            return None;
        }
    }

    pub fn evaluate_get(&mut self, expr: &expression::Get<'a>, defs: &'a Definitions) -> NodeResult {

        let Get { instance, property } = expr;
        let instance_value = self.evaluate_expression(instance.as_ref(), defs).value;

        let Some(value) = self.evaluate_get_internal(&instance_value, property, defs) else {
            self.runtime_error(&std::format!("Undefined property '{}'.", property.lexeme));
        };
        return NodeResult::new (
            value,
            Control::Continue
        );
    }

    pub fn evaluate_unary(&mut self, expr: &expression::UnaryOperation<'a>, defs: &'a Definitions) -> NodeResult {

        let expression::UnaryOperation { operator: op, right: rhs } = expr;
        let rhs_result = self.evaluate_expression(rhs, defs);

        if op.kind == TokenKind::Minus {
            let Value::Number (v) = rhs_result.value else {
                self.runtime_error("Operand must be a number.");
            };
            return NodeResult::new (
                Value::Number (-v),
                Control::Continue
            )
        }
        else if op.kind == TokenKind::Bang {
            return NodeResult::new (
                Value::Boolean (!is_truthy(&rhs_result.value)),
                Control::Continue
            )
        }
        else {
            panic!();
        }
    }

    pub fn evaluate_parentheses(&mut self, expr: &expression::Parentheses<'a>, defs: &'a Definitions) -> NodeResult {

        let expression::Parentheses { expression: e } = expr;
        let expression_result = self.evaluate_expression(e, defs);
        return NodeResult::new (
            expression_result.value,
            Control::Continue
        )
    }

    pub fn evaluate_assignment(&mut self, expr: &expression::Assignment<'a>, defs: &'a Definitions) -> NodeResult {

        let expression::Assignment { left: lhs, expression: rhs, lookup_hop_count, env_slot_idx } = expr;
        let rhs_result = self.evaluate_expression(rhs, defs);
        let mut lookup_hop_count_mut_copy = *lookup_hop_count;
        let mut env_slot_idx_mut_copy = *env_slot_idx;
        self.context.set_symbol_value(&lhs.lexeme, rhs_result.value.clone(), &mut lookup_hop_count_mut_copy, &mut env_slot_idx_mut_copy);
        return NodeResult::new (
            rhs_result.value,
            Control::Continue
        )
    }

    pub fn op_add(&self, vl: Value, vr: Value) -> Value {
        if let ( Value::Number (vl), Value::Number (vr) ) = ( &vl, &vr ) {
            return Value::Number(vl + vr);
        }
        else if let ( Value::String (vl), Value::String (vr) ) = ( &vl, &vr ) {
            return Value::String(vl.clone() + vr);
        }
        else {
            self.runtime_error("Operands must be two numbers or two strings.");
        }
    }

    pub fn op_sub(&self, vl: Value, vr: Value) -> Value {
        let ( Value::Number (vl), Value::Number (vr) ) = ( &vl, &vr ) else {
            self.runtime_error("Operands must be numbers.");
        };
        return Value::Number(vl - vr);
    }

    pub fn op_mul(&self, vl: Value, vr: Value) -> Value {
        let ( Value::Number (vl), Value::Number (vr) ) = ( &vl, &vr ) else {
            self.runtime_error("Operands must be numbers.");
        };
        return Value::Number(vl * vr);
    }

    pub fn op_div(&self, vl: Value, vr: Value) -> Value {
        let ( Value::Number (vl), Value::Number (vr) ) = ( &vl, &vr ) else {
            self.runtime_error("Operands must be numbers.");
        };
        return Value::Number(vl / vr);
    }

    pub fn op_equal(&self, vl: Value, vr: Value) -> Value {
        return Value::Boolean(vl == vr);
    }

    pub fn op_not_equal(&self, vl: Value, vr: Value) -> Value {
        return Value::Boolean(vl != vr);
    }

    pub fn op_less(&self, vl: Value, vr: Value) -> Value {
        let ( Value::Number (vl), Value::Number (vr) ) = ( &vl, &vr ) else {
            self.runtime_error("Operands must be numbers.");
        };
        return Value::Boolean(vl < vr);
    }

    pub fn op_less_equal(&self, vl: Value, vr: Value) -> Value {
        let ( Value::Number (vl), Value::Number (vr) ) = ( &vl, &vr ) else {
            self.runtime_error("Operands must be numbers.");
        };
        return Value::Boolean(vl <= vr);
    }

    pub fn op_greater(&self, vl: Value, vr: Value) -> Value {
        let ( Value::Number (vl), Value::Number (vr) ) = ( &vl, &vr ) else {
            self.runtime_error("Operands must be numbers.");
        };
        return Value::Boolean(vl > vr);
    }

    pub fn op_greater_equal(&self, vl: Value, vr: Value) -> Value {
        let ( Value::Number (vl), Value::Number (vr) ) = ( &vl, &vr ) else {
            self.runtime_error("Operands must be numbers.");
        };
        return Value::Boolean(vl >= vr);
    }

    pub fn evaluate_binary(&mut self, expr: &expression::BinaryOperation<'a>, defs: &'a Definitions) -> NodeResult {

        let expression::BinaryOperation { operator: op, left: lhs, right: rhs } = expr;

        let lhs_value = self.evaluate_expression(lhs, defs).value;
        let rhs_value = self.evaluate_expression(rhs, defs).value;

        match op.kind {
            TokenKind::Plus         => return NodeResult::new ( self.op_add           ( lhs_value, rhs_value ), Control::Continue ),
            TokenKind::Minus        => return NodeResult::new ( self.op_sub           ( lhs_value, rhs_value ), Control::Continue ),
            TokenKind::Star         => return NodeResult::new ( self.op_mul           ( lhs_value, rhs_value ), Control::Continue ),
            TokenKind::Slash        => return NodeResult::new ( self.op_div           ( lhs_value, rhs_value ), Control::Continue ),
            TokenKind::Less         => return NodeResult::new ( self.op_less          ( lhs_value, rhs_value ), Control::Continue ),
            TokenKind::LessEqual    => return NodeResult::new ( self.op_less_equal    ( lhs_value, rhs_value ), Control::Continue ),
            TokenKind::Greater      => return NodeResult::new ( self.op_greater       ( lhs_value, rhs_value ), Control::Continue ),
            TokenKind::GreaterEqual => return NodeResult::new ( self.op_greater_equal ( lhs_value, rhs_value ), Control::Continue ),
            TokenKind::EqualEqual   => return NodeResult::new ( self.op_equal         ( lhs_value, rhs_value ), Control::Continue ),
            TokenKind::BangEqual    => return NodeResult::new ( self.op_not_equal     ( lhs_value, rhs_value ), Control::Continue ),
            _ => {}
        }

        panic!();
    }

    pub fn evaluate_logical_or(&mut self, expr: &expression::LogicalOr<'a>, defs: &'a Definitions) -> NodeResult {

        let LogicalOr { left: lhs, right: rhs } = expr;

        let lhs_result = self.evaluate_expression(lhs, defs);

        if is_truthy(&lhs_result.value) {
            return NodeResult::new ( lhs_result.value, Control::Continue );
        }
        else {
            let rhs_result = self.evaluate_expression(rhs, defs);
            return NodeResult::new ( rhs_result.value, Control::Continue );
        }

    }

    pub fn evaluate_logical_and(&mut self, expr: &expression::LogicalAnd<'a>, defs: &'a Definitions) -> NodeResult {

        let LogicalAnd { left: lhs, right: rhs } = expr;

        let lhs_result = self.evaluate_expression(lhs, defs);

        if !is_truthy(&lhs_result.value) {
            return NodeResult::new ( lhs_result.value, Control::Continue );
        }
        else {
            let rhs_result = self.evaluate_expression(rhs, defs);
            return NodeResult::new ( rhs_result.value, Control::Continue );
        }

    }

    pub fn evaluate_function_call(&mut self, callee: &Value, arguments: &[Value], defs: &'a Definitions) -> Value {

        let Value::Function { context_function_slot_idx, closure_id } = callee else {
            panic!("Expected function value");
        };
        let Function { pars, body, is_native, native_function, is_initializer, .. } = &defs.functions[*context_function_slot_idx];
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
                    },
                    NativeFunction::None => {
                        panic!()
                    }
                }
            }

            for i in 0..pars.len() {
                self.context.insert_symbol (
                    &pars[i],
                    arguments[i].clone()
                );
            }

            let Statement::Block ( Block { statements } ) = &body else {
                panic!("Expected block statement");
            };
            for ref statement in statements {
                let statement_result = self.execute_statement(statement, defs);
                if statement_result.control == Control::FunctionReturn {
                    if *is_initializer {
                        let return_value = self.context.get_symbol_value("this", &mut Some(1), &mut Some(0));
                        self.context.pop_environment();
                        return return_value;
                    }
                    else {
                        self.context.pop_environment();
                        return statement_result.value;
                    }
                }
            }

            if *is_initializer {
                let return_value = self.context.get_symbol_value("this", &mut Some(1), &mut Some(0));
                self.context.pop_environment();
                return return_value;
            }
            else {
                self.context.pop_environment();
                return Value::Nil;
            }
        }
        else {
            panic!("Expected {} arguments but got {}.", pars.len(), arguments.len());
        }
    }

    pub fn evaluate_call(&mut self, expr: &expression::Call<'a>, defs: &'a Definitions) -> NodeResult {

        let Call { callee, arguments } = &expr;

        let callee = self.evaluate_expression(callee, defs).value;

        if let Value::Function { context_function_slot_idx, .. } = &callee {
            let Function { ref pars, .. } = defs.functions[*context_function_slot_idx];
            if pars.len() != arguments.len() {
                panic!("Expected {} arguments but got {}.", pars.len(), arguments.len());
            }

            let mut argument_values = Vec::<Value>::new();

            for i in 0..pars.len() {
                argument_values.push(self.evaluate_expression(&arguments[i], defs).value);
            }

            return NodeResult::new (
                self.evaluate_function_call(&callee, &argument_values, defs),
                Control::Continue
            );
        }
        else if let Value::Class { .. } = &callee {

            let new_instance =
                Value::Instance (
                    self.context.insert_instance (
                        Instance {
                            class:      callee,
                            properties: HashMap::<&'a str, Value>::new(),
                            reachable:  false
                        }
                    )
                );

            let init_token =
                Token {
                    kind:TokenKind::Identifier,
                    lexeme: "init",
                    line_number: 0
                };

            if let Some(init_bound_function) = self.evaluate_get_internal (
                &new_instance,
                &init_token,
                defs
            ) {
                if let Value::Function { context_function_slot_idx, .. } = init_bound_function {

                    let Function { ref pars, .. } = defs.functions[context_function_slot_idx];

                    if pars.len() != arguments.len() {
                        panic!("Expected {} arguments but got {}.", pars.len(), arguments.len());
                    };

                    let mut argument_values = Vec::<Value>::new();

                    for i in 0..pars.len() {
                        argument_values.push(self.evaluate_expression(&arguments[i], defs).value);
                    }

                    self.evaluate_function_call(&init_bound_function, &argument_values, defs);
                }
            }
            else if arguments.len() != 0 {
                panic!("Expected 0 arguments but got {}.", arguments.len());
            }

            return NodeResult::new (
                new_instance,
                Control::Continue
            )
        }
        else {
            panic!("Can only call functions and classes.");
        }

    }

    pub fn evaluate_expression(&mut self, expr: &expression::Expression<'a>, defs: &'a Definitions) -> NodeResult {

        match expr {
            expression::Expression::Literal         (expr) => return self.evaluate_literal(expr, defs),
            expression::Expression::UnaryOperation  (expr) => return self.evaluate_unary(expr, defs),
            expression::Expression::BinaryOperation (expr) => return self.evaluate_binary(expr, defs),
            expression::Expression::Parentheses     (expr) => return self.evaluate_parentheses(expr, defs),
            expression::Expression::Variable        (expr) => return self.evaluate_variable(expr, defs),
            expression::Expression::This                   => return self.evaluate_this(defs),
            expression::Expression::Assignment      (expr) => return self.evaluate_assignment(expr, defs),
            expression::Expression::LogicalOr       (expr) => return self.evaluate_logical_or(expr, defs),
            expression::Expression::LogicalAnd      (expr) => return self.evaluate_logical_and(expr, defs),
            expression::Expression::Call            (expr) => return self.evaluate_call(expr, defs),
            expression::Expression::Get             (expr) => return self.evaluate_get(expr, defs),
            expression::Expression::Set             (expr) => return self.evaluate_set(expr, defs),
            expression::Expression::Super           (expr) => return self.evaluate_super(expr, defs),
        }

    }

    pub fn execute_statement(&mut self, statement: &Statement<'a>, defs: &'a Definitions) -> NodeResult {

        self.statement_exec_count += 1;

        let working_set_size =
            self.context.environments.len() +
            self.context.instances.len();

        if working_set_size > self.working_set_size * 3 / 2 && working_set_size > 256 {
            // println!("gc started with #envs: {}, #instances: {}", self.context.environments.len(), self.context.instances.len());
            self.context.gc_collect(defs);
            self.working_set_size = working_set_size;
            // println!("Class slot count: {}", self.context.classes.len());
            // println!("gc finished with #envs: {}, #instances: {}", self.context.environments.len(), self.context.instances.len());
            // println!();
        }

        match statement {
            Statement::Expression (stmt)          => return self.execute_expression_statement(stmt, defs),
            Statement::Print (stmt)               => return self.execute_print_statement(stmt, defs),
            Statement::Return (stmt)              => return self.execute_return_statement(stmt, defs),
            Statement::While (stmt)               => return self.execute_while_statement(stmt, defs),
            Statement::For (stmt)                 => return self.execute_for_statement(stmt, defs),
            Statement::VariableDeclaration (stmt) => return self.execute_variable_declaration_statement(stmt, defs),
            Statement::FunctionDeclaration (stmt) => return self.execute_function_declaration_statement(stmt, defs),
            Statement::ClassDeclaration (stmt)    => return self.execute_class_declaration_statement(stmt, defs),
            Statement::Block (stmt)               => return self.execute_block_statement(stmt, defs),
            Statement::If (stmt)                  => return self.execute_if_statement(stmt, defs),
        }

    }

    pub fn execute_expression_statement(&mut self, stmt: &statement::Expression<'a>, defs: &'a Definitions) -> NodeResult {

        let statement::Expression { expr } = stmt;
        let expr_result = self.evaluate_expression(&expr, defs);
        return NodeResult::new ( Value::Nil, expr_result.control );

    }

    pub fn execute_print_statement(&mut self, stmt: &statement::Print<'a>, defs: &'a Definitions) -> NodeResult {

        let Print { expr } = stmt;

        let expr_result = self.evaluate_expression(expr, defs);

        match expr_result.value {
            Value::Boolean(b) => {
                println!("{b}");
            },
            Value::Number(n) => {
                println!("{n}");
            },
            Value::Nil => {
                println!("nil");
            },
            Value::String(s) => {
                println!("{s}");
            },
            Value::Function { context_function_slot_idx, .. } => {
                let Function { is_native, is_method, name, .. } = &defs.functions[context_function_slot_idx];
                if *is_method {
                    println!("<fn method>")
                }
                else if *is_native {
                    println!("<native fn>");
                }
                else {
                    println!("<fn {name}>");
                }
            },
            Value::Class { context_class_slot_idx, .. } => {
                let Class { name, ..} = &defs.classes[context_class_slot_idx];
                println!("{}", name);
            },
            Value::Instance ( instance_id ) => {
                let instance = self.context.instances.get(&instance_id).unwrap();
                let Instance { class, .. } = instance;
                let Value::Class { context_class_slot_idx, .. } = class else { panic!() };
                let Class { name, .. } = &defs.classes[*context_class_slot_idx];
                println!("{} instance", name)
            }
        }

        return NodeResult::new ( Value::Nil, expr_result.control );
    }

    pub fn execute_return_statement(&mut self, stmt: &statement::Return<'a>, defs: &'a Definitions) -> NodeResult {

        let statement::Return { expr } = stmt;

        match expr {
            Some ( statement::Expression { expr } ) => {
                let expr_result = self.evaluate_expression(expr, defs);
                return NodeResult::new ( expr_result.value, Control::FunctionReturn );
            },
            None => {
                return NodeResult::new ( Value::Nil, Control::FunctionReturn );
            }
        }
    }

    pub fn execute_while_statement(&mut self, stmt: &statement::While<'a>, defs: &'a Definitions) -> NodeResult {

        let While { condition: statement::Expression { expr }, body } = stmt;

        loop {
            let expr_value = self.evaluate_expression(expr, defs).value;
            if is_truthy(&expr_value) {
                let statement_result = self.execute_statement(body, defs);
                if statement_result.control == Control::FunctionReturn {
                    return statement_result;
                }
            }
            else {
                break;
            }
        }

        return NodeResult::new ( Value::Nil, Control::Continue );
    }

    pub fn execute_for_statement(&mut self, stmt: &statement::For<'a>, defs: &'a Definitions) -> NodeResult {

        let For {
            init: initializer_statement,
            cond: condition_expression,
            incr: increment_expression,
            body: body_statement
        } = stmt;

        self.context.push_new_environment_auto();

        if let Some(statement) = initializer_statement {
            self.execute_statement(statement.as_ref(), defs);
        }

        loop {
            if let Some ( statement::Expression { expr } ) = condition_expression {
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

            if let Some ( statement::Expression { expr } ) = increment_expression {
                self.evaluate_expression(expr, defs);
            }
        }

        self.context.pop_environment();
        return NodeResult::new ( Value::Nil, Control::Continue );

    }

    pub fn execute_if_statement(&mut self, stmt: &statement::If<'a>, defs: &'a Definitions) -> NodeResult {

        let If { condition: statement::Expression { expr }, then_statement, else_statement } = stmt;
        let condition_expression_value = self.evaluate_expression(expr, defs).value;
        if is_truthy(&condition_expression_value) {
            return self.execute_statement(then_statement, defs);
        }
        else if let Some(statement) = else_statement {
            return self.execute_statement(statement, defs);
        }
        else {
            return NodeResult::new ( Value::Nil, Control::Continue );
        }
    }

    pub fn execute_function_declaration_statement(&mut self, stmt: &statement::FunctionDeclaration<'a>, _defs: &'a Definitions) -> NodeResult {

        let FunctionDeclaration { id, context_function_slot_idx, .. } = stmt;

        if context_function_slot_idx.is_none() {
            panic!("Internal interpreter error: {}", id.lexeme);
        }

        self.context.insert_symbol (
            &id.lexeme,
            Value::Function {
                context_function_slot_idx: context_function_slot_idx.unwrap(),
                closure_id: self.context.get_current_environment_id(),
            }
        );

        return NodeResult::new ( Value::Nil, Control::Continue );
    }

    pub fn execute_class_declaration_statement(&mut self, stmt: &statement::ClassDeclaration<'a>, defs: &'a Definitions) -> NodeResult {

        let ClassDeclaration { id, super_class: super_class_expr, context_class_slot_idx, .. } = stmt;

        let super_class_value: Option<Box<Value>>;

        if let Some ( expr ) = super_class_expr {

            let expression::Expression::Variable(expr) = expr else {
                panic!("Expected variable expression");
            };

            super_class_value = Some(Box::new(self.evaluate_variable(expr, defs).value));

            let Value::Class { .. } = super_class_value.as_ref().unwrap().as_ref() else {
                panic!("Superclass must be a class.");
            };
            self.context.push_new_environment_auto();
            self.context.insert_symbol("super", super_class_value.as_ref().unwrap().as_ref().clone());
        }
        else {
            super_class_value = None;
        }

        let class_closure_id = self.context.get_current_environment_id();

        if let Some(..) = super_class_expr {
            self.context.pop_environment();
        }

        self.context.insert_symbol (
            &id.lexeme,
            Value::Class {
                context_class_slot_idx: context_class_slot_idx.unwrap(),
                closure_id: class_closure_id,
                super_class: super_class_value
            }
        );

        return NodeResult::new ( Value::Nil, Control::Continue );

    }

    pub fn execute_variable_declaration_statement(&mut self, stmt: &statement::VariableDeclaration<'a>, defs: &'a Definitions) -> NodeResult {

        let VariableDeclaration { id, init } = stmt;

        let expr_value = match init {
            None => Value::Nil,
            Some ( statement::Expression { expr } ) => self.evaluate_expression(expr, defs).value
        };

        self.context.insert_symbol(&id.lexeme, expr_value);

        return NodeResult::new ( Value::Nil, Control::Continue );

    }

    pub fn execute_block_statement(&mut self, stmt: &statement::Block<'a>, defs: &'a Definitions) -> NodeResult {

        let Block { statements } = stmt;

        self.context.push_new_environment_auto();
        for ref statement in statements {
            let statement_result = self.execute_statement(statement, defs);
            if statement_result.control == Control::FunctionReturn {
                self.context.pop_environment();
                return statement_result;
            }
        }
        self.context.pop_environment();

        return NodeResult::new ( Value::Nil, Control::Continue );

    }

    pub fn execute(&mut self, statements: &[Statement<'a>], defs: &'a Definitions) {
        for ref statement in statements {
            self.execute_statement(&statement, defs);
        }
        return;
    }

}
