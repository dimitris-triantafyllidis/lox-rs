use crate::lexer::*;
use crate::parser::*;
use crate::interpreter::*;
use crate::semantic_pass::*;
use crate::vm::Bytecode;

#[derive(Debug, Clone, PartialEq)]
pub struct Compiler {
    pub ast: Vec<statement::Statement>,
    pub bytecode: Bytecode
}

impl Compiler {

    pub fn new(ast: Vec<statement::Statement>) -> Self {
        Compiler {
            ast: ast.clone(),
            bytecode: Bytecode::new()
        }
    }

    pub fn emit_byte(self: &mut Self, byte: u8) {
        self.bytecode.code.push(byte);
    }

    pub fn emit_return(self: &mut Self) {
        self.emit_byte(crate::OP_RETURN);
    }

    pub fn emit_bytes(self: &mut Self, byte1: u8, byte2: u8) {
        self.emit_byte(byte1);
        self.emit_byte(byte2);
    }

    pub fn emit_constant(self: &mut Self, value: &Value) {
        let constant_idx = self.bytecode.write_constant(value.clone()) as u8;
        self.emit_bytes(crate::OP_CONSTANT, constant_idx);
    }

    pub fn visit_literal(self: &mut Self, expr: &mut expression::Literal) {

        let expression::Literal { token } = expr;

        match token.kind {
            TokenKind::Nil => {
                self.emit_byte(crate::OP_NIL);
            },
            TokenKind::False => {
                self.emit_byte(crate::OP_FALSE);
            },
            TokenKind::True  => {
                self.emit_byte(crate::OP_TRUE);
            },
            TokenKind::Number => {
                self.emit_constant(&Value::Number(token.lexeme.parse().unwrap()));
            },
            TokenKind::String => {
                self.emit_constant(&Value::String(token.lexeme[1..token.lexeme.len() - 1].to_string()));
            },
            _ => panic!()
        }
    }

    pub fn visit_variable(self: &mut Self, expr: &mut expression::Variable) {

    }

    pub fn visit_this(self: &Self) {

    }

    pub fn visit_super(self: &Self, expr: &mut expression::Super) {

    }

    pub fn visit_unary(self: &mut Self, expr: &mut expression::UnaryOperation) {

        let expression::UnaryOperation { operator: op, right: rhs } = expr;

        self.visit_expression(rhs);

        match op {
            Token { kind: TokenKind::Minus, .. } => {
                self.emit_byte(crate::OP_NEGATE)
            },
            Token { kind: TokenKind::Bang, .. } => {
                panic!();
            },
            _ => {
                panic!();
            }
        }
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
    }

    pub fn visit_binary(self: &mut Self, expr: &mut expression::BinaryOperation) {

        let expression::BinaryOperation { operator: op, left: lhs, right: rhs } = expr;

        self.visit_expression(lhs);
        self.visit_expression(rhs);

        match op.kind {
            TokenKind::Plus => {
                self.emit_byte(crate::OP_ADD);
            },
            TokenKind::Minus => {
                self.emit_byte(crate::OP_SUBTRACT);
            },
            TokenKind::Star => {
                self.emit_byte(crate::OP_MULTIPLY);
            },
            TokenKind::Slash => {
                self.emit_byte(crate::OP_DIVIDE);
            },
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

    pub fn visit_statement(self: &mut Self, statement: &mut statement::Statement) {

        match statement {
            statement::Statement::Expression(stmt)          => self.visit_expression_statement(stmt),
            statement::Statement::Print(stmt)               => self.visit_print_statement(stmt),
            statement::Statement::Return(stmt)              => self.visit_return_statement(stmt),
            statement::Statement::While(stmt)               => self.visit_while_statement(stmt),
            statement::Statement::For(stmt)                 => self.visit_for_statement(stmt),
            statement::Statement::VariableDeclaration(stmt) => self.visit_variable_declaration_statement(stmt),
            statement::Statement::FunctionDeclaration(stmt) => self.visit_function_declaration_statement(stmt),
            statement::Statement::ClassDeclaration(stmt)    => self.visit_class_declaration_statement(stmt),
            statement::Statement::Block(stmt)               => self.visit_block_statement(stmt),
            statement::Statement::If(stmt)                  => self.visit_if_statement(stmt),
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

        let statement::Return { expr } = stmt;

        match expr {
            Some ( statement::Expression { expr } ) => {
                self.visit_expression(expr);
            },
            None => { }
        }

    }

    pub fn visit_while_statement(self: &mut Self, stmt: &mut statement::While) {
        let statement::While { condition: statement::Expression { expr }, body } = stmt;
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

    }

    pub fn visit_if_statement(self: &mut Self, stmt: &mut statement::If) {

        let statement::If { condition: statement::Expression { expr }, then_statement, else_statement } = stmt;
        self.visit_expression(expr);
        self.visit_statement(then_statement);
        if let Some(statement) = else_statement {
            self.visit_statement(statement);
        }

    }

    pub fn visit_function_declaration_statement(self: &mut Self, stmt: &mut statement::FunctionDeclaration) {

        let statement::FunctionDeclaration { id, pars, body } = stmt;

        if let statement::Statement::Block ( statement::Block { statements } ) = body.as_mut() {
            for ref mut statement in statements {
                self.visit_statement(statement);
            }
        }
        else {
            panic!("Expected block statement");
        }
    }


    pub fn visit_class_declaration_statement(self: &mut Self, stmt: &mut statement::ClassDeclaration) {

        let statement::ClassDeclaration { id, method_decls, super_class } = stmt;

        if let Some ( statement::Expression { expr } ) = super_class {
            if let expression::Expression::Variable(expr) = expr {
                self.visit_variable(expr);
            }
            else {
                panic!("Expected variable expression");
            }
        }

        for method_decl in method_decls {
            let statement::FunctionDeclaration { id, .. } = method_decl;
            self.visit_function_declaration_statement(method_decl);
        }

    }

    pub fn visit_variable_declaration_statement(self: &mut Self, stmt: &mut statement::VariableDeclaration) {

        let statement::VariableDeclaration { id, init } = stmt;

        if let Some ( statement::Expression { expr } ) = init {
            self.visit_expression(expr);
        }

    }

    pub fn visit_block_statement(self: &mut Self, stmt: &mut statement::Block) {

        let statement::Block { statements } = stmt;
        for statement in statements {
            self.visit_statement(statement);
        }
    }

    pub fn compile(self: &mut Self) {

        for ref mut statement in self.ast.clone() {
            self.visit_statement(statement);
        }

        self.emit_return();

        return;
    }

}
