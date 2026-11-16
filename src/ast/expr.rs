use crate::lexer;

#[derive(Debug, Clone, PartialEq)]
pub struct Literal<'a> {
    pub token: lexer::Token<'a>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Variable<'a> {
    pub identifier: lexer::Token<'a>,
    pub lookup_hop_count: Option<usize>,
    pub env_slot_idx: Option<usize>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct UnaryOperation<'a> {
    pub operator: lexer::Token<'a>,
    pub right: Box<Expression<'a>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct BinaryOperation<'a> {
    pub operator: lexer::Token<'a>,
    pub left: Box<Expression<'a>>,
    pub right: Box<Expression<'a>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LogicalOr<'a> {
    pub left: Box<Expression<'a>>,
    pub right: Box<Expression<'a>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LogicalAnd<'a> {
    pub left: Box<Expression<'a>>,
    pub right: Box<Expression<'a>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Parentheses<'a> {
    pub expression: Box<Expression<'a>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Assignment<'a> {
    pub left: lexer::Token<'a>,
    pub lookup_hop_count: Option<usize>,
    pub env_slot_idx: Option<usize>,
    pub expression: Box<Expression<'a>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Call<'a> {
    pub callee: Box<Expression<'a>>,
    pub arguments: Vec<Expression<'a>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Set<'a> {
    pub instance: Box<Expression<'a>>,
    pub property: lexer::Token<'a>,
    pub value: Box<Expression<'a>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Get<'a> {
    pub instance: Box<Expression<'a>>,
    pub property: lexer::Token<'a>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Super<'a> {
    pub property: lexer::Token<'a>,
    pub lookup_hop_count: Option<usize>,
    pub env_slot_idx: Option<usize>,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Expression<'a> {
    Literal(Literal<'a>),
    Variable(Variable<'a>),
    UnaryOperation(UnaryOperation<'a>),
    BinaryOperation(BinaryOperation<'a>),
    LogicalOr(LogicalOr<'a>),
    LogicalAnd(LogicalAnd<'a>),
    Parentheses(Parentheses<'a>),
    Assignment(Assignment<'a>),
    Call(Call<'a>),
    Set(Set<'a>),
    Get(Get<'a>),
    This,
    Super(Super<'a>),
}
