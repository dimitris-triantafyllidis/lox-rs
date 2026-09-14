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

    pub fn evaluate_literal(self: &mut Self, expr: &Expression) -> Value {

        match expr {
            Expression::Literal{token} if token.kind == TokenKind::Nil   => { return Value::Nil             },
            Expression::Literal{token} if token.kind == TokenKind::False => { return Value::Boolean (false) },
            Expression::Literal{token} if token.kind == TokenKind::True  => { return Value::Boolean (true)  },
            Expression::Literal{token} if token.kind == TokenKind::Number => {
                return Value::Number (
                    token.lexeme.parse().unwrap()
                )
            },
            Expression::Literal{token} if token.kind == TokenKind::String => {
                return Value::String (
                    token.lexeme
                        [1..token.lexeme.len() - 1].to_string()
                )
            },
            _ => panic!()
        }

    }

    pub fn evaluate_variable(self: &mut Self, expr: &Expression) -> Value {

        match expr {
            Expression::Variable { identifier: id } => {
                self.context.get_symbol_value(&id.lexeme).clone()
            },
            _ => {
                panic!()
            }
        }

    }

    pub fn evaluate_unary(self: &mut Self, expr: &Expression) -> Value {

        if let Expression::UnaryOperation { operator: op, right: rhs } = expr {
            let rhs_value = self.evaluate_expression(rhs);
            match rhs_value {
                Value::Number (v) if op.kind == TokenKind::Minus => {
                    return Value::Number (-v)
                }
                _ if op.kind == TokenKind::Bang => {
                    return Value::Boolean (!is_truthy(&rhs_value))
                }
                _ => panic!()
            }
        }

        panic!()

    }

    pub fn evaluate_parentheses(self: &mut Self, expr: &Expression) -> Value {

        if let Expression::Parentheses { expression: e } = expr {
            return self.evaluate_expression(e);
        }

        panic!()

    }

    pub fn evaluate_assignment(self: &mut Self, expr: &Expression) -> Value {

        match expr {
            Expression::Assignment {
                left: lhs,
                expression: rhs
            } => {
                    let rhs_value = self.evaluate_expression(rhs);
                    self.context.set_symbol_value(&lhs.lexeme, rhs_value.clone());
                    return rhs_value;
            },
            _ => {
                panic!()
            }
        }

    }

    pub fn evaluate_binary(self: &mut Self, expr: &Expression) -> Value {

        if let Expression::BinaryOperation { operator: op, left: lhs, right: rhs } = expr {

            let lhs_value = self.evaluate_expression(lhs);
            let rhs_value = self.evaluate_expression(rhs);

            if let ( Value::Number (vl), Value::Number (vr) ) = ( lhs_value.clone(), rhs_value.clone() ) {
                match op.kind {
                    TokenKind::Plus  => { return Value::Number ( vl + vr) },
                    TokenKind::Minus => { return Value::Number ( vl - vr) },
                    TokenKind::Star  => { return Value::Number ( vl * vr) },
                    TokenKind::Slash => { return Value::Number ( vl / vr) },
                    TokenKind::EqualEqual   => { return Value::Boolean ( vl == vr) },
                    TokenKind::BangEqual    => { return Value::Boolean ( vl != vr) },
                    TokenKind::Less         => { return Value::Boolean ( vl <  vr) },
                    TokenKind::LessEqual    => { return Value::Boolean ( vl <= vr) },
                    TokenKind::Greater      => { return Value::Boolean ( vl >  vr) },
                    TokenKind::GreaterEqual => { return Value::Boolean ( vl >= vr) },
                    _ => panic!()
                }
            }

            if let ( Value::String (vl), Value::String(vr) ) = ( lhs_value.clone(), rhs_value.clone() ) {
                match op.kind {
                    TokenKind::Plus  => { return Value::String(vl + &vr.clone() ) },
                    TokenKind::EqualEqual   => { return Value::Boolean (vl == vr) },
                    TokenKind::BangEqual    => { return Value::Boolean (vl != vr) },
                    TokenKind::Less         => { return Value::Boolean (vl <  vr) },
                    TokenKind::LessEqual    => { return Value::Boolean (vl <= vr) },
                    TokenKind::Greater      => { return Value::Boolean (vl >  vr) },
                    TokenKind::GreaterEqual => { return Value::Boolean (vl >= vr) },
                    _ => panic!()
                }
            }

            if op.kind == TokenKind::EqualEqual {
                return Value::Boolean(lhs_value == rhs_value);
            }
            else if op.kind == TokenKind::BangEqual {
                return Value::Boolean(lhs_value != rhs_value);
            }
        }

        panic!()

    }

    pub fn evaluate_logical_or(self: &mut Self, expr: &Expression) -> Value {

        if let Expression::LogicalOr { left: lhs, right: rhs } = expr {

            let lhs_value = self.evaluate_expression(lhs);

            if is_truthy(&lhs_value) {
                return lhs_value;
            }
            else {
                return self.evaluate_expression(rhs);
            }
        }

        panic!()

    }

    pub fn evaluate_logical_and(self: &mut Self, expr: &Expression) -> Value {

        if let Expression::LogicalAnd { left: lhs, right: rhs } = expr {

            let lhs_value = self.evaluate_expression(lhs);

            if !is_truthy(&lhs_value) {
                return lhs_value;
            }
            else {
                return self.evaluate_expression(rhs);
            }
        }

        panic!()

    }

    pub fn evaluate_function_call(self: &mut Self, expr: &Expression) -> Value {

        if let Expression::Call { callee, arguments } = expr {

            let callee = self.evaluate_expression(callee);

            if let Value::Function(parameters, body, closure_id) = callee {
                if parameters.len() == arguments.len() {

                    let mut argument_values = Vec::<Value>::new();

                    for i in 0..parameters.len() {
                        argument_values.push(self.evaluate_expression(&arguments[i]).clone());
                    }

                    self.context.push_new_environment(Some(closure_id));

                    for i in 0..parameters.len() {
                        self.context.insert_symbol (
                            &parameters[i].lexeme,
                            argument_values[i].clone()
                        );
                    }

                    if let Statement::Block(statements) = body {
                        self.execute(&statements);
                    }
                    else {
                        panic!("Expected block statement");
                    }

                    self.context.pop_environment();
                    return Value::Nil;
                }
                else {
                    panic!("Wrong number of arguments");
                }
            }
            else {
                panic!("Expected function value");
            }
        }
        else {
            panic!("Expected call expression")
        }

    }

    pub fn evaluate_expression(self: &mut Self, expr: &Expression) -> Value {

        return match expr {
            Expression::Literal { .. } => self.evaluate_literal(expr),
            Expression::UnaryOperation { .. }  => self.evaluate_unary(expr),
            Expression::BinaryOperation { .. } => self.evaluate_binary(expr),
            Expression::Parentheses { .. } => self.evaluate_parentheses(expr),
            Expression::Variable { .. } => self.evaluate_variable(expr),
            Expression::Assignment { .. } => self.evaluate_assignment(expr),
            Expression::LogicalOr { .. } => self.evaluate_logical_or(expr),
            Expression::LogicalAnd { .. } => self.evaluate_logical_and(expr),
            Expression::Call { .. } => self.evaluate_function_call(expr),
            _ => panic!()
        }

    }

    pub fn execute_statement(self: &mut Self, statement: &Statement) {

        match statement {
            Statement::Expression(..) => self.execute_expression_statement(statement),
            Statement::Print(..) => self.execute_print_statement(statement),
            Statement::While(..) => self.execute_while_statement(statement),
            Statement::For(..) => self.execute_for_statement(statement),
            Statement::VariableDeclaration(..) => self.execute_variable_declaration_statement(statement),
            Statement::FunctionDeclaration(..) => self.execute_function_declaration_statement(statement),
            Statement::Block(..) => self.execute_block_statement(statement),
            Statement::If(..) => self.execute_if_statement(statement),
            _ => panic!()
        }

    }

    pub fn execute_expression_statement(self: &mut Self, statement: &Statement) {

        if let Statement::Expression(expr) = statement {
            self.evaluate_expression(&expr);
            return;
        }

        panic!();

    }

    pub fn execute_print_statement(self: &mut Self, statement: &Statement) {

        if let Statement::Print(expr) = statement {

            let expr_value = self.evaluate_expression(&expr);

            match expr_value {
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
                _ => {
                    panic!();
                }
            }

            return;
        }

        panic!();

    }

    pub fn execute_while_statement(self: &mut Self, statement: &Statement) {

        if let Statement::While(condition_expression, body_statement) = statement {

            loop {
                let expr_value = self.evaluate_expression(&condition_expression);
                if is_truthy(&expr_value) {
                    self.execute_statement(body_statement);
                }
                else {
                    break;
                }
            }

            return;
        }

        panic!();

    }

    pub fn execute_for_statement(self: &mut Self, statement: &Statement) {

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
                    let expr_value = self.evaluate_expression(ce);
                    if is_truthy(&expr_value) {
                        self.execute_statement(body_statement);
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

            return;
        }

        panic!();

    }

    pub fn execute_if_statement(self: &mut Self, statement: &Statement) {

        if let Statement::If(condition_expression, then_statement, else_statement) = statement {
            let condition_expression_value = self.evaluate_expression(condition_expression);
            if is_truthy(&condition_expression_value) {
                self.execute_statement(then_statement);
                return;
            }
            else if let Some(statement) = else_statement {
                self.execute_statement(statement);
                return;
            }
            else {
                return;
            }
        }

        panic!();

    }

    pub fn execute_function_declaration_statement(self: &mut Self, statement: &Statement) {

        if let Statement::FunctionDeclaration(id, params, body) = statement {

            self.context.insert_symbol (
                &id.lexeme,
                Value::Function (
                    params.clone(),
                    *body.clone(),
                    self.context.get_current_environment_id()
                )
            );

            return;
        }

        panic!();

    }

    pub fn execute_variable_declaration_statement(self: &mut Self, statement: &Statement) {

        if let Statement::VariableDeclaration(id, expr) = statement {

            let expr_value = match expr {
                None => Value::Nil,
                Some(expr) => self.evaluate_expression(&expr)
            };

            self.context.insert_symbol(&id.lexeme, expr_value.clone());

            return;
        }

        panic!();

    }

    pub fn execute_block_statement(self: &mut Self, statement: &Statement) {

        if let Statement::Block(statements) = statement {
            self.context.push_new_environment_auto();
            for statement in statements {
                self.execute_statement(statement);
            }
            self.context.pop_environment();
            return;
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
