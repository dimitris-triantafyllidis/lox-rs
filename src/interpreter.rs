use std::collections::HashMap;
use std::hash::Hash;

use crate::lexer::*;
use crate::parser::*;
use crate::context::*;

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Nil,
    Boolean (bool),
    Number (f64),
    String (String),
    Function (Vec<Token>, Statement, usize),
    Class (HashMap<Token, Value>),
    Instance (Box<Value>, HashMap<Token, Value>)
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

impl NodeResult {
    pub fn new(value: Value, control: Control) -> Self {
        return Self {
            value,
            control
        }
    }
    pub fn default() -> Self {
        return Self {
            value: Value::Nil,
            control: Control::Continue
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
pub struct Interpreter {
    pub context: Context
}

impl Interpreter {

    pub fn new() -> Self {

        let mut context = Context::new();
        context.push_new_environment_auto();

        Self {
            context
        }
    }

    pub fn evaluate_literal(self: &mut Self, expr: &Expression) -> NodeResult {

        match expr {
            Expression::Literal{token} if token.kind == TokenKind::Nil => {
                return NodeResult::new(Value::Nil, Control::Continue);
            },
            Expression::Literal{token} if token.kind == TokenKind::False => {
                return NodeResult::new(Value::Boolean(false), Control::Continue);
            },
            Expression::Literal{token} if token.kind == TokenKind::True  => {
                return NodeResult::new(Value::Boolean(true), Control::Continue);
            },
            Expression::Literal{token} if token.kind == TokenKind::Number => {
                return NodeResult::new (
                    Value::Number (
                        token.lexeme.parse().unwrap()
                    ),
                    Control::Continue
                )
            },
            Expression::Literal{token} if token.kind == TokenKind::String => {
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

    pub fn evaluate_variable(self: &mut Self, expr: &Expression) -> NodeResult {

        match expr {
            Expression::Variable { identifier: id, lookup_hop_count: hop_count } => {
                let mut hop_count_mut_copy = *hop_count;
                NodeResult::new (
                    self.context.get_symbol_value(&id.lexeme, &mut hop_count_mut_copy).clone(),
                    Control::Continue
                )
            },
            _ => {
                panic!()
            }
        }
    }

    pub fn evaluate_unary(self: &mut Self, expr: &Expression) -> NodeResult {

        if let Expression::UnaryOperation { operator: op, right: rhs } = expr {
            let rhs_result = self.evaluate_expression(rhs);
            match rhs_result.value {
                Value::Number (v) if op.kind == TokenKind::Minus => {
                    return NodeResult::new (
                        Value::Number (-v),
                        Control::Continue
                    )
                }
                _ if op.kind == TokenKind::Bang => {
                    return NodeResult::new (
                        Value::Boolean (!is_truthy(&rhs_result.value)),
                        Control::Continue
                    )
                }
                _ => panic!()
            }
        }

        panic!()

    }

    pub fn evaluate_parentheses(self: &mut Self, expr: &Expression) -> NodeResult {

        if let Expression::Parentheses { expression: e } = expr {
            let expression_result = self.evaluate_expression(e);
            return NodeResult::new (
                expression_result.value,
                Control::Continue
            )
        }

        panic!()

    }

    pub fn evaluate_assignment(self: &mut Self, expr: &Expression) -> NodeResult {

        match expr {
            Expression::Assignment {
                left: lhs,
                expression: rhs,
                lookup_hop_count: hop_count
            } => {
                let rhs_result = self.evaluate_expression(rhs);
                let mut hop_count_mut_copy = *hop_count;
                self.context.set_symbol_value(&lhs.lexeme, rhs_result.value.clone(), &mut hop_count_mut_copy);
                return NodeResult::new (
                    rhs_result.value,
                    Control::Continue
                )
            },
            _ => {
                panic!()
            }
        }

    }

    pub fn evaluate_binary(self: &mut Self, expr: &Expression) -> NodeResult {

        if let Expression::BinaryOperation { operator: op, left: lhs, right: rhs } = expr {

            let lhs_value = self.evaluate_expression(lhs).value;
            let rhs_value = self.evaluate_expression(rhs).value;

            if let ( Value::Number (vl), Value::Number (vr) ) = ( lhs_value.clone(), rhs_value.clone() ) {
                match op.kind {
                    TokenKind::Plus         => { return NodeResult::new ( Value::Number ( vl + vr),   Control::Continue ) },
                    TokenKind::Minus        => { return NodeResult::new ( Value::Number ( vl - vr),   Control::Continue ) },
                    TokenKind::Star         => { return NodeResult::new ( Value::Number ( vl * vr),   Control::Continue ) },
                    TokenKind::Slash        => { return NodeResult::new ( Value::Number ( vl / vr),   Control::Continue ) },
                    TokenKind::EqualEqual   => { return NodeResult::new ( Value::Boolean ( vl == vr), Control::Continue ) },
                    TokenKind::BangEqual    => { return NodeResult::new ( Value::Boolean ( vl != vr), Control::Continue ) },
                    TokenKind::Less         => { return NodeResult::new ( Value::Boolean ( vl <  vr), Control::Continue ) },
                    TokenKind::LessEqual    => { return NodeResult::new ( Value::Boolean ( vl <= vr), Control::Continue ) },
                    TokenKind::Greater      => { return NodeResult::new ( Value::Boolean ( vl >  vr), Control::Continue ) },
                    TokenKind::GreaterEqual => { return NodeResult::new ( Value::Boolean ( vl >= vr), Control::Continue ) },
                    _ => panic!()
                }
            }

            if let ( Value::String (vl), Value::String(vr) ) = ( lhs_value.clone(), rhs_value.clone() ) {
                match op.kind {
                    TokenKind::Plus         => { return NodeResult::new ( Value::String  (vl + &vr.clone()), Control::Continue ) },
                    TokenKind::EqualEqual   => { return NodeResult::new ( Value::Boolean (vl == vr),         Control::Continue ) },
                    TokenKind::BangEqual    => { return NodeResult::new ( Value::Boolean (vl != vr),         Control::Continue ) },
                    TokenKind::Less         => { return NodeResult::new ( Value::Boolean (vl <  vr),         Control::Continue ) },
                    TokenKind::LessEqual    => { return NodeResult::new ( Value::Boolean (vl <= vr),         Control::Continue ) },
                    TokenKind::Greater      => { return NodeResult::new ( Value::Boolean (vl >  vr),         Control::Continue ) },
                    TokenKind::GreaterEqual => { return NodeResult::new ( Value::Boolean (vl >= vr),         Control::Continue ) },
                    _ => panic!()
                }
            }

            if op.kind == TokenKind::EqualEqual {
                return NodeResult::new ( Value::Boolean(lhs_value == rhs_value), Control::Continue );
            }
            else if op.kind == TokenKind::BangEqual {
                return NodeResult::new ( Value::Boolean(lhs_value != rhs_value), Control::Continue );
            }
        }

        panic!()

    }

    pub fn evaluate_logical_or(self: &mut Self, expr: &Expression) -> NodeResult {

        if let Expression::LogicalOr { left: lhs, right: rhs } = expr {

            let lhs_result = self.evaluate_expression(lhs);

            if is_truthy(&lhs_result.value) {
                return NodeResult::new ( lhs_result.value, Control::Continue );
            }
            else {
                let rhs_result = self.evaluate_expression(rhs);
                return NodeResult::new ( rhs_result.value, Control::Continue );
            }
        }

        panic!()

    }

    pub fn evaluate_logical_and(self: &mut Self, expr: &Expression) -> NodeResult {

        if let Expression::LogicalAnd { left: lhs, right: rhs } = expr {

            let lhs_result = self.evaluate_expression(lhs);

            if !is_truthy(&lhs_result.value) {
                return NodeResult::new ( lhs_result.value, Control::Continue );
            }
            else {
                let rhs_result = self.evaluate_expression(rhs);
                return NodeResult::new ( rhs_result.value, Control::Continue );
            }
        }

        panic!()

    }

    pub fn evaluate_call(self: &mut Self, expr: &Expression) -> NodeResult {

        if let Expression::Call { callee, arguments } = expr {

            let callee = self.evaluate_expression(callee).value;

            if let Value::Function(parameters, body, closure_id) = callee {
                if parameters.len() == arguments.len() {

                    let mut argument_values = Vec::<Value>::new();

                    for i in 0..parameters.len() {
                        argument_values.push(self.evaluate_expression(&arguments[i]).value.clone());
                    }

                    self.context.push_new_environment(Some(closure_id));

                    for i in 0..parameters.len() {
                        self.context.insert_symbol (
                            &parameters[i].lexeme,
                            argument_values[i].clone()
                        );
                    }

                    if let Statement::Block(statements) = body {
                        for statement in statements {
                            let statement_result = self.execute_statement(&statement);
                            if statement_result.control == Control::FunctionReturn {
                                self.context.pop_environment();
                                return NodeResult::new (
                                    statement_result.value,
                                    Control::Continue
                                );
                            }
                        }
                    }
                    else {
                        panic!("Expected block statement");
                    }

                    self.context.pop_environment();
                    return NodeResult::new ( Value::Nil, Control::Continue );
                }
                else {
                    panic!("Wrong number of arguments");
                }
            }
            else if let Value::Class(..) = callee {
                if arguments.len() == 0 {
                    return NodeResult::new (
                        Value::Instance (
                            Box::new(callee),
                            HashMap::<Token, Value>::new()
                        ),
                        Control::Continue
                    );
                }
                else {
                    panic!("Class calls cannot take arguments yet");
                }
            }
            else {
                panic!("Expected function value or class value");
            }
        }
        else {
            panic!("Expected call expression")
        }

    }

    pub fn evaluate_expression(self: &mut Self, expr: &Expression) -> NodeResult {

        match expr {
            Expression::Literal         { .. } => return self.evaluate_literal(expr),
            Expression::UnaryOperation  { .. } => return self.evaluate_unary(expr),
            Expression::BinaryOperation { .. } => return self.evaluate_binary(expr),
            Expression::Parentheses     { .. } => return self.evaluate_parentheses(expr),
            Expression::Variable        { .. } => return self.evaluate_variable(expr),
            Expression::Assignment      { .. } => return self.evaluate_assignment(expr),
            Expression::LogicalOr       { .. } => return self.evaluate_logical_or(expr),
            Expression::LogicalAnd      { .. } => return self.evaluate_logical_and(expr),
            Expression::Call            { .. } => return self.evaluate_call(expr),
            _ => panic!()
        }

    }

    pub fn execute_statement(self: &mut Self, statement: &Statement) -> NodeResult {

        match statement {
            Statement::Expression          (..) => return self.execute_expression_statement(statement),
            Statement::Print               (..) => return self.execute_print_statement(statement),
            Statement::Return              (..) => return self.execute_return_statement(statement),
            Statement::While               (..) => return self.execute_while_statement(statement),
            Statement::For                 (..) => return self.execute_for_statement(statement),
            Statement::VariableDeclaration (..) => return self.execute_variable_declaration_statement(statement),
            Statement::FunctionDeclaration (..) => return self.execute_function_declaration_statement(statement),
            Statement::ClassDeclaration    (..) => return self.execute_class_declaration_statement(statement),
            Statement::Block               (..) => return self.execute_block_statement(statement),
            Statement::If                  (..) => return self.execute_if_statement(statement),
            _ => panic!()
        }

    }

    pub fn execute_expression_statement(self: &mut Self, statement: &Statement) -> NodeResult {

        if let Statement::Expression(expr) = statement {
            let expr_result = self.evaluate_expression(&expr);
            return NodeResult::new ( Value::Nil, expr_result.control );
        }

        panic!();

    }

    pub fn execute_print_statement(self: &mut Self, statement: &Statement) -> NodeResult {

        if let Statement::Print(expr) = statement {

            let expr_result = self.evaluate_expression(&expr);

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
                    println!("\"{s}\"");
                },
                Value::Function(..) => {
                    println!("<fn>");
                }
                _ => {
                    panic!();
                }
            }

            return NodeResult::new ( Value::Nil, expr_result.control );
        }

        panic!();

    }

    pub fn execute_return_statement(self: &mut Self, statement: &Statement) -> NodeResult {

        if let Statement::Return(Some(expr)) = statement {
            let expr_result = self.evaluate_expression(&expr);
            return NodeResult::new ( expr_result.value, Control::FunctionReturn );
        }
        else if let Statement::Return(None) = statement {
            return NodeResult::new ( Value::Nil, Control::FunctionReturn );
        }

        panic!();

    }

    pub fn execute_while_statement(self: &mut Self, statement: &Statement) -> NodeResult {

        if let Statement::While(condition_expression, body_statement) = statement {

            loop {
                let expr_value = self.evaluate_expression(&condition_expression).value;
                if is_truthy(&expr_value) {
                    let statement_result = self.execute_statement(body_statement);
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

        panic!();

    }

    pub fn execute_for_statement(self: &mut Self, statement: &Statement) -> NodeResult {

        if let Statement::For (
            initializer_statement,
            condition_expression,
            increment_expression,
            body_statement,
        ) = statement {

            self.context.push_new_environment_auto();

            if let Some(statement) = initializer_statement {
                self.execute_statement(statement.as_ref());
            }

            loop {
                if let Some(ce) = condition_expression {
                    let expr_value = self.evaluate_expression(ce).value;
                    if is_truthy(&expr_value) {
                        let statement_result = self.execute_statement(body_statement);
                        if statement_result.control == Control::FunctionReturn {
                            self.context.pop_environment();
                            return statement_result;
                        }
                    }
                    else {
                        break;
                    }
                }

                if let Some(ie) = increment_expression {
                    self.evaluate_expression(ie);
                }
            }

            self.context.pop_environment();
            return NodeResult::new ( Value::Nil, Control::Continue );
        }

        panic!();

    }

    pub fn execute_if_statement(self: &mut Self, statement: &Statement) -> NodeResult {

        if let Statement::If(condition_expression, then_statement, else_statement) = statement {
            let condition_expression_value = self.evaluate_expression(condition_expression).value;
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

        panic!();

    }

    pub fn execute_function_declaration_statement(self: &mut Self, statement: &Statement) -> NodeResult {

        if let Statement::FunctionDeclaration(id, params, body) = statement {

            self.context.insert_symbol (
                &id.lexeme,
                Value::Function (
                    params.clone(),
                    *body.clone(),
                    self.context.get_current_environment_id()
                )
            );

            return NodeResult::new ( Value::Nil, Control::Continue );
        }

        panic!();

    }

    pub fn execute_class_declaration_statement(self: &mut Self, statement: &Statement) -> NodeResult {

        if let Statement::ClassDeclaration(id, statements) = statement {

            let mut class_method_map = HashMap::<Token, Value>::new();

            for statement in statements {
                if let Statement::FunctionDeclaration(id, params, body) = statement {
                    class_method_map.insert (
                        id.clone(),
                        Value::Function (
                            params.clone(),
                            *body.clone(),
                            self.context.get_current_environment_id()
                        )
                    );
                }
            }

            self.context.insert_symbol (
                &id.lexeme,
                Value::Class (
                    class_method_map
                )
            );

            return NodeResult::new ( Value::Nil, Control::Continue );
        }

        panic!();

    }

    pub fn execute_variable_declaration_statement(self: &mut Self, statement: &Statement) -> NodeResult {

        if let Statement::VariableDeclaration(id, expr) = statement {

            let expr_value = match expr {
                None => Value::Nil,
                Some(expr) => self.evaluate_expression(&expr).value
            };

            self.context.insert_symbol(&id.lexeme, expr_value.clone());

            return NodeResult::new ( Value::Nil, Control::Continue );
        }

        panic!();

    }

    pub fn execute_block_statement(self: &mut Self, statement: &Statement) -> NodeResult {

        if let Statement::Block(statements) = statement {
            self.context.push_new_environment_auto();
            for statement in statements {
                let statement_result = self.execute_statement(&statement);
                if statement_result.control == Control::FunctionReturn {
                    self.context.pop_environment();
                    return statement_result;
                }
            }
            self.context.pop_environment();
            return NodeResult::new ( Value::Nil, Control::Continue );
        }

        panic!();
    }

    pub fn execute(self: &mut Self, statements: &Vec<Statement>) {
        for statement in statements {
            self.execute_statement(statement);
        }
        return;
    }

}
