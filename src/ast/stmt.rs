use crate::{ast, lexer};

#[derive(Debug, PartialEq, Clone)]
pub struct Expression<'a> {
    pub expr: ast::expr::Expression<'a>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct If<'a> {
    pub condition: Expression<'a>,
    pub then_statement: Box<Statement<'a>>,
    pub else_statement: Option<Box<Statement<'a>>>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct While<'a> {
    pub condition: Expression<'a>,
    pub body: Box<Statement<'a>>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct For<'a> {
    pub init: Option<Box<Statement<'a>>>,
    pub cond: Option<Expression<'a>>,
    pub incr: Option<Expression<'a>>,
    pub body: Box<Statement<'a>>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Print<'a> {
    pub expr: ast::expr::Expression<'a>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Return<'a> {
    pub expr: Option<Expression<'a>>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct FunctionDeclaration<'a> {
    pub id: lexer::Token<'a>,
    pub pars: Vec<lexer::Token<'a>>,
    pub body: Box<Statement<'a>>,
    pub is_initializer: bool,
    pub is_method: bool,
    pub context_function_slot_idx: Option<usize>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct ClassDeclaration<'a> {
    pub id: lexer::Token<'a>,
    pub method_decls: Vec<FunctionDeclaration<'a>>,
    pub super_class: Option<ast::expr::Expression<'a>>,
    pub context_class_slot_idx: Option<usize>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct VariableDeclaration<'a> {
    pub id: lexer::Token<'a>,
    pub init: Option<Expression<'a>>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Block<'a> {
    pub statements: Vec<Statement<'a>>,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Statement<'a> {
    Expression(Expression<'a>),
    If(If<'a>),
    While(While<'a>),
    For(For<'a>),
    Print(Print<'a>),
    Return(Return<'a>),
    FunctionDeclaration(FunctionDeclaration<'a>),
    ClassDeclaration(ClassDeclaration<'a>),
    VariableDeclaration(VariableDeclaration<'a>),
    Block(Block<'a>),
}
