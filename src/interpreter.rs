use std::collections::HashMap;

use std::time::{SystemTime, UNIX_EPOCH};

use crate::lexer::*;
use crate::parser::*;
use crate::context::*;

use expression::*;
use statement::*;

#[derive(Debug, Clone, PartialEq)]
pub struct Instance<'a> {
    pub class: Box<Value<'a>>,
    pub properties: HashMap<&'a str, Value<'a>>,
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
    pub closure_id: usize,
    pub is_native: bool,
    pub native_function: NativeFunction
}

#[derive(Debug, Clone, PartialEq)]
pub struct Class<'a> {
    pub methods: HashMap<&'a str, Value<'a>>,
    pub super_class: Option<Box<Value<'a>>>,
    pub name: String
}

#[derive(Debug, Clone, PartialEq)]
pub enum Value<'a> {
    Nil,
    Boolean (bool),
    Number (f64),
    String (String),
    Function (Function<'a>),
    Class (usize),
    Instance (usize)
}

#[derive(Debug, Clone, PartialEq)]
pub enum Control {
    Continue,
    FunctionReturn
}

#[derive(Debug, Clone, PartialEq)]
pub struct NodeResult<'a> {
    pub value: Value<'a>,
    pub control: Control
}

impl<'a> NodeResult<'a> {
    pub fn new(value: Value<'a>, control: Control) -> Self {
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

    pub fn new() -> Self {

        let mut context = Context::new();
        context.push_new_environment_auto();

        context.insert_symbol (
            "clock",
            Value::Function (
                Function {
                    pars: Vec::new(),
                    body: Statement::Block (
                        Block {
                            statements: Vec::new()
                        }
                    ),
                    closure_id: 0,
                    is_native: true,
                    native_function: NativeFunction::Clock
                }
            )
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

    pub fn evaluate_literal(&mut self, expr: &expression::Literal<'a>) -> NodeResult<'a> {

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

    pub fn evaluate_variable(&mut self, expr: &expression::Variable<'a>) -> NodeResult<'a> {

        let Variable { identifier: id, lookup_hop_count, env_slot_idx } = expr;
        let mut lookup_hop_count_mut_copy = *lookup_hop_count;
        let mut env_slot_idx_mut_copy = *env_slot_idx;
        NodeResult::new (
            self.context.get_symbol_value(&id.lexeme, &mut lookup_hop_count_mut_copy, &mut env_slot_idx_mut_copy),
            Control::Continue
        )
    }

    pub fn evaluate_this(&mut self) -> NodeResult<'a> {

        NodeResult::new (
            self.context.get_symbol_value("this", &mut None, &mut None),
            Control::Continue
        )

    }

    pub fn evaluate_super(&mut self, expr: &expression::Super<'a>) -> NodeResult<'a> {

        let Super { property, lookup_hop_count, env_slot_idx } = expr;

        let mut lookup_hop_count_mut_copy = *lookup_hop_count;
        let mut env_slot_idx_mut_copy = *env_slot_idx;
        let instance_value = self.context.get_symbol_value("this", &mut None, &mut None);
        let mut value: Value;

        let mut method_lookup_class = Some(Box::new(self.context.get_symbol_value("super", &mut lookup_hop_count_mut_copy, &mut env_slot_idx_mut_copy)));
        while let Some(box_value) = method_lookup_class {
            if let Value::Class ( class_id ) = box_value.as_ref() {
                let Class { methods, super_class, .. } = &self.context.classes[*class_id];
                if methods.contains_key(&property.lexeme) {
                    value = methods.get(&property.lexeme).unwrap().clone();
                    if let Value::Function ( Function { ref mut closure_id, .. } ) = value {
                        self.context.push_new_environment(Some(*closure_id));
                        self.context.insert_symbol("this", instance_value);
                        *closure_id = self.context.next_env_id - 1;
                        self.context.pop_environment();
                        return NodeResult { value, control: Control::Continue };
                    }
                    else {
                        panic!("Expected function value");
                    }
                }
                else {
                    method_lookup_class = super_class.clone();
                    continue;
                }
            }
            else {
                panic!("Expected class value");
            }
        }
        panic!("Undefined property '{}'.", property.lexeme);
    }

    pub fn evaluate_set(&mut self, expr: &expression::Set<'a>) -> NodeResult<'a> {

        let Set { instance, property, value } = expr;

        let instance = self.evaluate_expression(instance.as_ref()).value;
        let value = self.evaluate_expression(value).value;

        if let Value::Instance ( instance_id ) = instance {
            let instance_value = self.context.instances.get_mut(&instance_id).unwrap();
            instance_value.properties.insert(property.lexeme, value.clone());
            return NodeResult::new (
                value,
                Control::Continue
            );
        }
        else {
            panic!("Only instances have fields.");
        }

    }

    pub fn evaluate_get_internal(&mut self, instance_value: &Value<'a>, property: &Token<'a>) -> Option<Value<'a>> {

        if let Value::Instance ( instance_id ) = instance_value {
            let instance = self.context.instances.get(instance_id).unwrap();
            let mut value: Value;

            if instance.properties.contains_key(&property.lexeme) {
                value = instance.properties.get(&property.lexeme).unwrap().clone();
                return Some(value);
            }
            else {
                let mut method_lookup_class = Some(instance.class.clone());
                while let Some(box_value) = method_lookup_class {
                    if let Value::Class ( class_id ) = box_value.as_ref() {
                        let Class { methods, super_class, .. } = &self.context.classes[*class_id];
                        if methods.contains_key(&property.lexeme) {
                            value = methods.get(&property.lexeme).unwrap().clone();
                            if let Value::Function ( Function { ref mut closure_id, .. } ) = value {
                                self.context.push_new_environment(Some(*closure_id));
                                self.context.insert_symbol("this", instance_value.clone());
                                *closure_id = self.context.next_env_id - 1;
                                self.context.pop_environment();
                                return Some(value);
                            }
                            else {
                                panic!("Expected function value");
                            }
                        }
                        else {
                            method_lookup_class = super_class.clone();
                            continue;
                        }
                    }
                    else {
                        panic!("Expected class value");
                    }
                }
                return None;
            }
        }
        else {
            panic!("Only instances have properties.");
        }
    }

    pub fn evaluate_get(&mut self, expr: &expression::Get<'a>) -> NodeResult<'a> {

        let Get { instance, property } = expr;
        let instance_value = self.evaluate_expression(instance.as_ref()).value;

        if let Some(value) = self.evaluate_get_internal(&instance_value, property) {
            return NodeResult::new (
                value,
                Control::Continue
            );
        }
        else {
            self.runtime_error(&std::format!("Undefined property '{}'.", property.lexeme));
        }
    }

    pub fn evaluate_unary(&mut self, expr: &expression::UnaryOperation<'a>) -> NodeResult<'a> {

        let expression::UnaryOperation { operator: op, right: rhs } = expr;
        let rhs_result = self.evaluate_expression(rhs);

        if op.kind == TokenKind::Minus {
            if let Value::Number (v) = rhs_result.value {
                return NodeResult::new (
                    Value::Number (-v),
                    Control::Continue
                )
            }
            else {
                self.runtime_error("Operand must be a number.");
            }
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

    pub fn evaluate_parentheses(&mut self, expr: &expression::Parentheses<'a>) -> NodeResult<'a> {

        let expression::Parentheses { expression: e } = expr;
        let expression_result = self.evaluate_expression(e);
        return NodeResult::new (
            expression_result.value,
            Control::Continue
        )

    }

    pub fn evaluate_assignment(&mut self, expr: &expression::Assignment<'a>) -> NodeResult<'a> {

        let expression::Assignment { left: lhs, expression: rhs, lookup_hop_count, env_slot_idx } = expr;
        let rhs_result = self.evaluate_expression(rhs);
        let mut lookup_hop_count_mut_copy = *lookup_hop_count;
        let mut env_slot_idx_mut_copy = *env_slot_idx;
        self.context.set_symbol_value(&lhs.lexeme, rhs_result.value.clone(), &mut lookup_hop_count_mut_copy, &mut env_slot_idx_mut_copy);
        return NodeResult::new (
            rhs_result.value,
            Control::Continue
        )

    }

    pub fn op_add(&self, vl: Value<'a>, vr: Value<'a>) -> Value<'a> {
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

    pub fn op_sub(&self, vl: Value<'a>, vr: Value<'a>) -> Value<'a> {
        if let ( Value::Number (vl), Value::Number (vr) ) = ( &vl, &vr ) {
            return Value::Number(vl - vr);
        }
        else {
            self.runtime_error("Operands must be numbers.");
        }
    }

    pub fn op_mul(&self, vl: Value<'a>, vr: Value<'a>) -> Value<'a> {
        if let ( Value::Number (vl), Value::Number (vr) ) = ( &vl, &vr ) {
            return Value::Number(vl * vr);
        }
        else {
            self.runtime_error("Operands must be numbers.");
        }
    }

    pub fn op_div(&self, vl: Value<'a>, vr: Value<'a>) -> Value<'a> {
        if let ( Value::Number (vl), Value::Number (vr) ) = ( &vl, &vr ) {
            return Value::Number(vl / vr);
        }
        else {
            self.runtime_error("Operands must be numbers.");
        }
    }

    pub fn op_equal(&self, vl: Value<'a>, vr: Value<'a>) -> Value<'a> {
        return Value::Boolean(vl == vr);
    }

    pub fn op_not_equal(&self, vl: Value<'a>, vr: Value<'a>) -> Value<'a> {
        return Value::Boolean(vl != vr);
    }

    pub fn op_less(&self, vl: Value<'a>, vr: Value<'a>) -> Value<'a> {
        if let ( Value::Number (vl), Value::Number (vr) ) = ( &vl, &vr ) {
            return Value::Boolean(vl < vr);
        }
        else {
            self.runtime_error("Operands must be numbers.");
        }
    }

    pub fn op_less_equal(&self, vl: Value<'a>, vr: Value<'a>) -> Value<'a> {
        if let ( Value::Number (vl), Value::Number (vr) ) = ( &vl, &vr ) {
            return Value::Boolean(vl <= vr);
        }
        else {
            self.runtime_error("Operands must be numbers.");
        }
    }

    pub fn op_greater(&self, vl: Value<'a>, vr: Value<'a>) -> Value<'a> {
        if let ( Value::Number (vl), Value::Number (vr) ) = ( &vl, &vr ) {
            return Value::Boolean(vl > vr);
        }
        else {
            self.runtime_error("Operands must be numbers.");
        }
    }

    pub fn op_greater_equal(&self, vl: Value<'a>, vr: Value<'a>) -> Value<'a> {
        if let ( Value::Number (vl), Value::Number (vr) ) = ( &vl, &vr ) {
            return Value::Boolean(vl >= vr);
        }
        else {
            self.runtime_error("Operands must be numbers.");
        }
    }

    pub fn evaluate_binary(&mut self, expr: &expression::BinaryOperation<'a>) -> NodeResult<'a> {

        let expression::BinaryOperation { operator: op, left: lhs, right: rhs } = expr;

        let lhs_value = self.evaluate_expression(lhs).value;
        let rhs_value = self.evaluate_expression(rhs).value;

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

    pub fn evaluate_logical_or(&mut self, expr: &expression::LogicalOr<'a>) -> NodeResult<'a> {

        let LogicalOr { left: lhs, right: rhs } = expr;

        let lhs_result = self.evaluate_expression(lhs);

        if is_truthy(&lhs_result.value) {
            return NodeResult::new ( lhs_result.value, Control::Continue );
        }
        else {
            let rhs_result = self.evaluate_expression(rhs);
            return NodeResult::new ( rhs_result.value, Control::Continue );
        }

    }

    pub fn evaluate_logical_and(&mut self, expr: &expression::LogicalAnd<'a>) -> NodeResult<'a> {

        let LogicalAnd { left: lhs, right: rhs } = expr;

        let lhs_result = self.evaluate_expression(lhs);

        if !is_truthy(&lhs_result.value) {
            return NodeResult::new ( lhs_result.value, Control::Continue );
        }
        else {
            let rhs_result = self.evaluate_expression(rhs);
            return NodeResult::new ( rhs_result.value, Control::Continue );
        }

    }

    pub fn evaluate_function_call(&mut self, callee: &Value<'a>, arguments: &[Value<'a>]) -> Value<'a> {

        if let Value::Function ( Function { pars, body, closure_id, is_native, native_function } ) = callee {

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

                if let Statement::Block ( Block { statements } ) = &body {
                    for ref statement in statements {
                        let statement_result = self.execute_statement(statement);
                        if statement_result.control == Control::FunctionReturn {
                            self.context.pop_environment();
                            return statement_result.value;
                        }
                    }
                }
                else {
                    panic!("Expected block statement");
                }

                self.context.pop_environment();
                return Value::Nil;
            }
            else {
                panic!("Expected {} arguments but got {}.", pars.len(), arguments.len());
            }
        }
        else {
            panic!("Expected function value");
        }
    }

    pub fn evaluate_call(&mut self, expr: &expression::Call<'a>) -> NodeResult<'a> {

        let Call { callee, arguments } = &expr;

        let callee = self.evaluate_expression(callee).value;

        if let Value::Function ( Function { pars, .. } ) = &callee {
            if pars.len() == arguments.len() {

                let mut argument_values = Vec::<Value>::new();

                for i in 0..pars.len() {
                    argument_values.push(self.evaluate_expression(&arguments[i]).value);
                }

                return NodeResult::new (
                    self.evaluate_function_call(&callee, &argument_values),
                    Control::Continue
                );
            }
            else {
                panic!("Expected {} arguments but got {}.", pars.len(), arguments.len());
            }
        }
        else if let Value::Class ( .. ) = &callee {

            let new_instance =
                Value::Instance (
                    self.context.insert_instance (
                        Instance {
                            class:      Box::new(callee.clone()),
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
                &init_token
            ) {
                if let Value::Function ( Function { ref pars, .. } ) = init_bound_function {
                    if pars.len() == arguments.len() {

                        let mut argument_values = Vec::<Value>::new();

                        for i in 0..pars.len() {
                            argument_values.push(self.evaluate_expression(&arguments[i]).value);
                        }

                        self.evaluate_function_call(&init_bound_function, &argument_values);
                    }
                    else {
                        panic!("Expected {} arguments but got {}.", pars.len(), arguments.len());
                    }
                }
            }
            else {
                if arguments.len() != 0 {
                    panic!("Expected 0 arguments but got {}.", arguments.len());
                }
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

    pub fn evaluate_expression(&mut self, expr: &expression::Expression<'a>) -> NodeResult<'a> {

        match expr {
            expression::Expression::Literal         (expr) => return self.evaluate_literal(expr),
            expression::Expression::UnaryOperation  (expr) => return self.evaluate_unary(expr),
            expression::Expression::BinaryOperation (expr) => return self.evaluate_binary(expr),
            expression::Expression::Parentheses     (expr) => return self.evaluate_parentheses(expr),
            expression::Expression::Variable        (expr) => return self.evaluate_variable(expr),
            expression::Expression::This                   => return self.evaluate_this(),
            expression::Expression::Assignment      (expr) => return self.evaluate_assignment(expr),
            expression::Expression::LogicalOr       (expr) => return self.evaluate_logical_or(expr),
            expression::Expression::LogicalAnd      (expr) => return self.evaluate_logical_and(expr),
            expression::Expression::Call            (expr) => return self.evaluate_call(expr),
            expression::Expression::Get             (expr) => return self.evaluate_get(expr),
            expression::Expression::Set             (expr) => return self.evaluate_set(expr),
            expression::Expression::Super           (expr) => return self.evaluate_super(expr),
        }

    }

    pub fn execute_statement(&mut self, statement: &Statement<'a>) -> NodeResult<'a> {

        self.statement_exec_count += 1;

        let working_set_size =
            self.context.environments.len() +
            self.context.instances.len();

        if working_set_size > self.working_set_size * 3 / 2 && working_set_size > 256 {
            self.context.gc_collect();
            self.working_set_size = working_set_size;
        }

        match statement {
            Statement::Expression (stmt)          => return self.execute_expression_statement(stmt),
            Statement::Print (stmt)               => return self.execute_print_statement(stmt),
            Statement::Return (stmt)              => return self.execute_return_statement(stmt),
            Statement::While (stmt)               => return self.execute_while_statement(stmt),
            Statement::For (stmt)                 => return self.execute_for_statement(stmt),
            Statement::VariableDeclaration (stmt) => return self.execute_variable_declaration_statement(stmt),
            Statement::FunctionDeclaration (stmt) => return self.execute_function_declaration_statement(stmt),
            Statement::ClassDeclaration (stmt)    => return self.execute_class_declaration_statement(stmt),
            Statement::Block (stmt)               => return self.execute_block_statement(stmt),
            Statement::If (stmt)                  => return self.execute_if_statement(stmt),
        }

    }

    pub fn execute_expression_statement(&mut self, stmt: &statement::Expression<'a>) -> NodeResult<'a> {

        let statement::Expression { expr } = stmt;
        let expr_result = self.evaluate_expression(&expr);
        return NodeResult::new ( Value::Nil, expr_result.control );

    }

    pub fn execute_print_statement(&mut self, stmt: &statement::Print<'a>) -> NodeResult<'a> {

        let Print { expr } = stmt;

        let expr_result = self.evaluate_expression(expr);

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
            Value::Function{..} => {
                println!("<fn>");
            },
            Value::Class ( class_id ) => {
                let Class { name, ..} = &self.context.classes[class_id];
                println!("{}", name);
            },
            Value::Instance ( instance_id ) => {
                let instance = self.context.instances.get(&instance_id).unwrap();
                let Instance { class, .. } = instance;
                let Value::Class ( class_id ) = class.as_ref() else { panic!() };
                let Class { name, .. } = &self.context.classes[*class_id];
                println!("{} instance", name)
            }
        }

        return NodeResult::new ( Value::Nil, expr_result.control );
    }

    pub fn execute_return_statement(&mut self, stmt: &statement::Return<'a>) -> NodeResult<'a> {

        let statement::Return { expr } = stmt;

        match expr {
            Some ( statement::Expression { expr } ) => {
                let expr_result = self.evaluate_expression(expr);
                return NodeResult::new ( expr_result.value, Control::FunctionReturn );
            },
            None => {
                return NodeResult::new ( Value::Nil, Control::FunctionReturn );
            }
        }
    }

    pub fn execute_while_statement(&mut self, stmt: &statement::While<'a>) -> NodeResult<'a> {

        let While { condition: statement::Expression { expr }, body } = stmt;

        loop {
            let expr_value = self.evaluate_expression(expr).value;
            if is_truthy(&expr_value) {
                let statement_result = self.execute_statement(body);
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

    pub fn execute_for_statement(&mut self, stmt: &statement::For<'a>) -> NodeResult<'a> {

        let For {
            init: initializer_statement,
            cond: condition_expression,
            incr: increment_expression,
            body: body_statement
        } = stmt;

        self.context.push_new_environment_auto();

        if let Some(statement) = initializer_statement {
            self.execute_statement(statement.as_ref());
        }

        loop {
            if let Some ( statement::Expression { expr } ) = condition_expression {
                let expr_value = self.evaluate_expression(expr).value;
                if !is_truthy(&expr_value) {
                    break;
                }
            }

            let body_statement_result = self.execute_statement(body_statement);
            if body_statement_result.control == Control::FunctionReturn {
                self.context.pop_environment();
                return body_statement_result;
            }

            if let Some ( statement::Expression { expr } ) = increment_expression {
                self.evaluate_expression(expr);
            }
        }

        self.context.pop_environment();
        return NodeResult::new ( Value::Nil, Control::Continue );

    }

    pub fn execute_if_statement(&mut self, stmt: &statement::If<'a>) -> NodeResult<'a> {

        let If { condition: statement::Expression { expr }, then_statement, else_statement } = stmt;
        let condition_expression_value = self.evaluate_expression(expr).value;
        if is_truthy(&condition_expression_value) {
            return self.execute_statement(then_statement);
        }
        else if let Some(statement) = else_statement {
            return self.execute_statement(statement);
        }
        else {
            return NodeResult::new ( Value::Nil, Control::Continue );
        }
    }

    pub fn execute_function_declaration_statement(&mut self, stmt: &statement::FunctionDeclaration<'a>) -> NodeResult<'a> {

        let FunctionDeclaration { id, pars, body } = stmt;

        let mut par_lexemes = Vec::<&'a str>::new();

        for par in pars {
            par_lexemes.push(par.lexeme);
        }

        self.context.insert_symbol (
            &id.lexeme,
            Value::Function (
                Function {
                    pars: par_lexemes,
                    body: *body.clone(),
                    closure_id: self.context.get_current_environment_id(),
                    is_native: false,
                    native_function: NativeFunction::None,
                }
            )
        );

        return NodeResult::new ( Value::Nil, Control::Continue );
    }

    pub fn execute_class_declaration_statement(&mut self, stmt: &statement::ClassDeclaration<'a>) -> NodeResult<'a> {

        let ClassDeclaration { id, method_decls, super_class } = stmt;

        let super_class_result: Option<Box<Value>>;

        if let Some ( expr ) = super_class {
            let super_class_value: Value;
            if let expression::Expression::Variable(expr) = expr {
                super_class_value = self.evaluate_variable(expr).value;
            }
            else {
                panic!("Expected variable expression");
            }
            if let Value::Class {..} = super_class_value {
                super_class_result = Some(Box::new(super_class_value.clone()));
                self.context.push_new_environment_auto();
                self.context.insert_symbol("super", super_class_value);
            }
            else {
                panic!("Superclass must be a class.");
            }
        }
        else {
            super_class_result = None;
        }

        let mut class_method_map = HashMap::<Token, Value>::new();

        for method_decl in method_decls {
            let FunctionDeclaration { id, pars, body } = method_decl;

            let mut par_lexemes = Vec::<&'a str>::new();

            for par in pars {
                par_lexemes.push(par.lexeme);
            }

            class_method_map.insert (
                id.clone(),
                Value::Function (
                    Function {
                        pars: par_lexemes,
                        body: *body.clone(),
                        closure_id: self.context.get_current_environment_id(),
                        is_native: false,
                        native_function: NativeFunction::Clock
                    }
                )
            );
        }

        if let Some(..) = super_class {
            self.context.pop_environment();
        }

        let mut class_method_lexeme_map = HashMap::<&'a str, Value>::new();

        for (method_token, value) in class_method_map {
            class_method_lexeme_map.insert(method_token.lexeme, value);
        }

        let class_id =
            self.context.insert_class (
                Class {
                    methods: class_method_lexeme_map,
                    super_class: super_class_result,
                    name: id.lexeme.to_string()
                }
            );

        self.context.insert_symbol (
            &id.lexeme,
            Value::Class (
                class_id
            )
        );

        return NodeResult::new ( Value::Nil, Control::Continue );

    }

    pub fn execute_variable_declaration_statement(&mut self, stmt: &statement::VariableDeclaration<'a>) -> NodeResult<'a> {

        let VariableDeclaration { id, init } = stmt;

        let expr_value = match init {
            None => Value::Nil,
            Some ( statement::Expression { expr } ) => self.evaluate_expression(expr).value
        };

        self.context.insert_symbol(&id.lexeme, expr_value);

        return NodeResult::new ( Value::Nil, Control::Continue );

    }

    pub fn execute_block_statement(&mut self, stmt: &statement::Block<'a>) -> NodeResult<'a> {

        let Block { statements } = stmt;

        self.context.push_new_environment_auto();
        for ref statement in statements {
            let statement_result = self.execute_statement(statement);
            if statement_result.control == Control::FunctionReturn {
                self.context.pop_environment();
                return statement_result;
            }
        }
        self.context.pop_environment();

        return NodeResult::new ( Value::Nil, Control::Continue );

    }

    pub fn execute(&mut self, statements: &[Statement<'a>]) {
        for ref statement in statements {
            self.execute_statement(&statement);
        }
        return;
    }

}
