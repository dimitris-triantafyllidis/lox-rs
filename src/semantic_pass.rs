use crate::lexer::*;
use crate::parser::statement::Expression;
use crate::parser::*;
use crate::context::*;
use crate::interpreter::*;

use crate::parser::expression::*;
use crate::parser::statement::*;

#[derive(Debug, Clone, PartialEq)]
pub enum FunctionContext {
    Function,
    Method,
    Initializer
}

#[derive(Debug, Clone, PartialEq)]
pub enum ClassContext {
    Class
}

pub struct SemanticPass {
    pub context: Context,
    pub function_context_stack: Vec<FunctionContext>,
    pub class_context_stack: Vec<ClassContext>
}

impl SemanticPass {

    pub fn new() -> Self {

        let mut context = Context::new();
        context.push_new_environment_auto();

        let function_context_stack = Vec::<FunctionContext>::new();
        let class_context_stack = Vec::<ClassContext>::new();

        Self {
            context,
            function_context_stack,
            class_context_stack
        }
    }

    pub fn visit_literal(self: &mut Self, _: &mut expression::Literal) { }

    pub fn visit_variable(self: &mut Self, expr: &mut expression::Variable) {

        let Variable { identifier: id, lookup_hop_count: hop_count } = expr;
        if !hop_count.is_none() {
            panic!();
        }
        self.context.get_symbol_value(&id.lexeme, hop_count);
    }

    pub fn visit_this(self: &Self) {
        if self.class_context_stack.is_empty() {
            panic!("Unexpected 'this' outside a class context");
        }
        else {
            self.context.get_symbol_value(&"this".to_string(), &mut None);
        }
    }

    pub fn visit_super(self: &Self, expr: &mut expression::Super) {

        let expression::Super { lookup_hop_count: hop_count, .. } = expr;
        if !hop_count.is_none() {
            panic!();
        }
        self.context.get_symbol_value(&"super".to_string(), &mut None);
    }

    pub fn visit_unary(self: &mut Self, expr: &mut expression::UnaryOperation) {

        let expression::UnaryOperation { operator: _, right: rhs } = expr;
        self.visit_expression(rhs);

    }

    pub fn visit_parentheses(self: &mut Self, expr: &mut expression::Parentheses) {

        let expression::Parentheses { expression: e } = expr;
        self.visit_expression(e);

    }

    pub fn visit_get(self: &mut Self, expr: &mut expression::Get) {

        let expression::Get { instance, .. } = expr;
        self.visit_expression(instance);
    }

    pub fn visit_set(self: &mut Self, expr: &mut expression::Set) {

        let expression::Set { instance, value, .. } = expr;
        self.visit_expression(instance);
        self.visit_expression(value);
    }

    pub fn visit_assignment(self: &mut Self, expr: &mut expression::Assignment) {

        let expression::Assignment { left: lhs, expression: rhs, lookup_hop_count: hop_count } = expr;
        self.visit_expression(rhs);
        self.context.set_symbol_value(&lhs.lexeme, Value::Nil, hop_count);
    }

    pub fn visit_binary(self: &mut Self, expr: &mut expression::BinaryOperation) {

        let expression::BinaryOperation { operator: op, left: lhs, right: rhs } = expr;

        self.visit_expression(lhs);
        self.visit_expression(rhs);

        match op.kind {
            TokenKind::Plus         => { },
            TokenKind::Minus        => { },
            TokenKind::Star         => { },
            TokenKind::Slash        => { },
            TokenKind::EqualEqual   => { },
            TokenKind::BangEqual    => { },
            TokenKind::Less         => { },
            TokenKind::LessEqual    => { },
            TokenKind::Greater      => { },
            TokenKind::GreaterEqual => { },
            _ => panic!()
        }

    }

    pub fn visit_logical_or(self: &mut Self, expr: &mut expression::LogicalOr) {

        let expression::LogicalOr { left: lhs, right: rhs } = expr;
        self.visit_expression(lhs);
        self.visit_expression(rhs);

    }

    pub fn visit_logical_and(self: &mut Self, expr: &mut expression::LogicalAnd) {

        let expression::LogicalAnd { left: lhs, right: rhs } = expr;
        self.visit_expression(lhs);
        self.visit_expression(rhs);

    }

    pub fn visit_call(self: &mut Self, expr: &mut expression::Call) {

        let expression::Call { callee, arguments } = expr;
        self.visit_expression(callee);
        for argument in arguments {
            self.visit_expression(argument);
        }
    }

    pub fn visit_expression(self: &mut Self, expr: &mut expression::Expression) {

        match expr {
            expression::Expression::Literal         (expr) => self.visit_literal(expr),
            expression::Expression::UnaryOperation  (expr) => self.visit_unary(expr),
            expression::Expression::BinaryOperation (expr) => self.visit_binary(expr),
            expression::Expression::Parentheses     (expr) => self.visit_parentheses(expr),
            expression::Expression::Variable        (expr) => self.visit_variable(expr),
            expression::Expression::This                   => self.visit_this(),
            expression::Expression::Assignment      (expr) => self.visit_assignment(expr),
            expression::Expression::LogicalOr       (expr) => self.visit_logical_or(expr),
            expression::Expression::LogicalAnd      (expr) => self.visit_logical_and(expr),
            expression::Expression::Call            (expr) => self.visit_call(expr),
            expression::Expression::Get             (expr) => self.visit_get(expr),
            expression::Expression::Set             (expr) => self.visit_set(expr),
            expression::Expression::Super           (expr) => self.visit_super(expr),
        }

    }

    pub fn visit_statement(self: &mut Self, statement: &mut Statement) {

        match statement {
            Statement::Expression(stmt)          => self.visit_expression_statement(stmt),
            Statement::Print(stmt)               => self.visit_print_statement(stmt),
            Statement::Return(stmt)              => self.visit_return_statement(stmt),
            Statement::While(stmt)               => self.visit_while_statement(stmt),
            Statement::For(stmt)                 => self.visit_for_statement(stmt),
            Statement::VariableDeclaration(stmt) => self.visit_variable_declaration_statement(stmt),
            Statement::FunctionDeclaration(stmt) => {
                self.function_context_stack.push(FunctionContext::Function);
                self.visit_function_declaration_statement(stmt);
                self.function_context_stack.pop();
            },
            Statement::ClassDeclaration(stmt)   => self.visit_class_declaration_statement(stmt),
            Statement::Block(stmt)              => self.visit_block_statement(stmt),
            Statement::If(stmt)                 => self.visit_if_statement(stmt),
        }

    }

    pub fn visit_expression_statement(self: &mut Self, stmt: &mut statement::Expression) {
        let statement::Expression { expr } = stmt;
        self.visit_expression(expr);
    }

    pub fn visit_print_statement(self: &mut Self, stmt: &mut statement::Print) {
        let statement::Print { expr } = stmt;
        self.visit_expression(expr);
    }

    pub fn visit_return_statement(self: &mut Self, stmt: &mut statement::Return) {

        if self.function_context_stack.is_empty() {
            panic!("Unexpected 'return' outside a function context")
        }

        let statement::Return { expr } = stmt;

        match expr {
            Some ( statement::Expression { expr } ) => {
                self.visit_expression(expr);
            },
            None => { }
        }

    }

    pub fn visit_while_statement(self: &mut Self, stmt: &mut statement::While) {
        let While { condition: statement::Expression { expr }, body } = stmt;
        self.visit_expression(expr);
        self.visit_statement(body);
    }

    pub fn visit_for_statement(self: &mut Self, stmt: &mut statement::For) {

        let statement::For {
            init: initializer_statement,
            cond: condition_expression,
            incr: increment_expression,
            body: body_statement,
        } = stmt;

        self.context.push_new_environment_auto();

        if let Some ( statement ) = initializer_statement {
            self.visit_statement(statement);
        }

        if let Some ( statement::Expression { expr } ) = condition_expression {
            self.visit_expression(expr);
        }

        self.visit_statement(body_statement);

        if let Some ( statement::Expression { expr } ) = increment_expression {
            self.visit_expression(expr);
        }

        self.context.pop_environment();
    }

    pub fn visit_if_statement(self: &mut Self, stmt: &mut statement::If) {

        let If { condition: statement::Expression { expr }, then_statement, else_statement } = stmt;
        self.visit_expression(expr);
        self.visit_statement(then_statement);
        if let Some(statement) = else_statement {
            self.visit_statement(statement);
        }

    }

    pub fn visit_function_declaration_statement(self: &mut Self, stmt: &mut statement::FunctionDeclaration) {

        let FunctionDeclaration { id, pars, body } = stmt;

        let function_context = self.function_context_stack.last().cloned().unwrap();
        let mut closure_id = self.context.get_current_environment_id();

        if function_context == FunctionContext::Method || function_context == FunctionContext::Initializer {
            self.context.push_new_environment(Some(closure_id));
            self.context.insert_symbol(&"this".to_string(), Value::Nil);
            closure_id = self.context.next_id - 1;
        }

        self.context.insert_symbol (
            &id.lexeme,
            Value::Function (
                Function {
                    pars: pars.clone(),
                    body: *body.clone(),
                    closure_id
                }
            )
        );

        self.context.push_new_environment_auto();

        for i in 0..pars.len() {
            self.context.insert_symbol (
                &pars[i].lexeme,
                Value::Nil
            );
        }

        if let Statement::Block ( Block { statements } ) = body.as_mut() {
            for ref mut statement in statements {
                self.visit_statement(statement);
            }

            if function_context == FunctionContext::Method || function_context == FunctionContext::Initializer {
                self.context.pop_environment();
            }

        }
        else {
            panic!("Expected block statement");
        }

        self.context.pop_environment();
    }


    pub fn visit_class_declaration_statement(self: &mut Self, stmt: &mut statement::ClassDeclaration) {

        let ClassDeclaration { id, method_decls, super_class } = stmt;

        if let Some ( statement::Expression { expr } ) = super_class {
            if let expression::Expression::Variable(expr) = expr {
                self.visit_variable(expr);
            }
            else {
                panic!("Expected variable expression");
            }
        }

        self.class_context_stack.push(ClassContext::Class);

        self.context.insert_symbol (
            &id.lexeme,
            Value::Nil
        );

        if let Some(..) = super_class {
            self.context.push_new_environment_auto();
            self.context.insert_symbol(&"super".to_string(), Value::Nil);
        }

        for method_decl in method_decls {
            let FunctionDeclaration { id, .. } = method_decl;
            if id.lexeme == "init" {
                self.function_context_stack.push(FunctionContext::Initializer);
            }
            else {
                self.function_context_stack.push(FunctionContext::Method);
            }
            self.visit_function_declaration_statement(method_decl);
            self.function_context_stack.pop();
        }

        if let Some(..) = super_class {
            self.context.pop_environment();
        }

        self.class_context_stack.pop();

    }

    pub fn visit_variable_declaration_statement(self: &mut Self, stmt: &mut statement::VariableDeclaration) {

        let VariableDeclaration { id, init } = stmt;

        if let Some ( statement::Expression { expr } ) = init {
            self.visit_expression(expr);
        }

        self.context.insert_symbol(&id.lexeme, Value::Nil);
    }

    pub fn visit_block_statement(self: &mut Self, stmt: &mut statement::Block) {

        let Block { statements } = stmt;
        self.context.push_new_environment_auto();
        for statement in statements {
            self.visit_statement(statement);
        }
        self.context.pop_environment();
    }

    pub fn visit(self: &mut Self, statements: &mut Vec<Statement>) {
        for statement in statements {
            self.visit_statement(statement);
        }
        return;
    }

}
