pub mod expr {

    use crate::lexer as lexer;

    #[derive(Debug, Clone, PartialEq)]
    pub struct Literal<'a> {
        pub token: lexer::Token<'a>
    }

    #[derive(Debug, Clone, PartialEq)]
    pub struct Variable<'a> {
        pub identifier: lexer::Token<'a>,
        pub lookup_hop_count: Option<usize>,
        pub env_slot_idx: Option<usize>
    }

    #[derive(Debug, Clone, PartialEq)]
    pub struct UnaryOperation<'a> {
        pub operator: lexer::Token<'a>,
        pub right: Box<Expression<'a>>
    }

    #[derive(Debug, Clone, PartialEq)]
    pub struct BinaryOperation<'a> {
        pub operator: lexer::Token<'a>,
        pub left: Box<Expression<'a>>,
        pub right: Box<Expression<'a>>
    }

    #[derive(Debug, Clone, PartialEq)]
    pub struct LogicalOr<'a> {
        pub left: Box<Expression<'a>>,
        pub right: Box<Expression<'a>>
    }

    #[derive(Debug, Clone, PartialEq)]
    pub struct LogicalAnd<'a> {
        pub left: Box<Expression<'a>>,
        pub right: Box<Expression<'a>>
    }

    #[derive(Debug, Clone, PartialEq)]
    pub struct Parentheses<'a> {
        pub expression: Box<Expression<'a>>
    }

    #[derive(Debug, Clone, PartialEq)]
    pub struct Assignment<'a> {
        pub left: lexer::Token<'a>,
        pub lookup_hop_count: Option<usize>,
        pub env_slot_idx: Option<usize>,
        pub expression: Box<Expression<'a>>
    }

    #[derive(Debug, Clone, PartialEq)]
    pub struct Call<'a> {
        pub callee: Box<Expression<'a>>,
        pub arguments: Vec<Expression<'a>>
    }

    #[derive(Debug, Clone, PartialEq)]
    pub struct Set<'a> {
        pub instance: Box<Expression<'a>>,
        pub property: lexer::Token<'a>,
        pub value: Box<Expression<'a>>
    }

    #[derive(Debug, Clone, PartialEq)]
    pub struct Get<'a> {
        pub instance: Box<Expression<'a>>,
        pub property: lexer::Token<'a>
    }

    #[derive(Debug, Clone, PartialEq)]
    pub struct Super<'a> {
        pub property: lexer::Token<'a>,
        pub lookup_hop_count: Option<usize>,
        pub env_slot_idx: Option<usize>
    }

    #[derive(Debug, PartialEq, Clone)]
    pub enum Expression<'a> {
        Literal (Literal<'a>),
        Variable (Variable<'a>),
        UnaryOperation (UnaryOperation<'a>),
        BinaryOperation (BinaryOperation<'a>),
        LogicalOr (LogicalOr<'a>),
        LogicalAnd (LogicalAnd<'a>),
        Parentheses (Parentheses<'a>),
        Assignment (Assignment<'a>),
        Call (Call<'a>),
        Set (Set<'a>),
        Get (Get<'a>),
        This,
        Super (Super<'a>)
    }
}

pub mod stmt {

    use crate::lexer as lexer;
    use crate::ast as ast;

    #[derive(Debug, PartialEq, Clone)]
    pub struct Expression<'a> {
        pub expr: ast::expr::Expression<'a>
    }

    #[derive(Debug, PartialEq, Clone)]
    pub struct If<'a> {
        pub condition: Expression<'a>,
        pub then_statement: Box<Statement<'a>>,
        pub else_statement: Option<Box<Statement<'a>>>
    }

    #[derive(Debug, PartialEq, Clone)]
    pub struct While<'a> {
        pub condition: Expression<'a>,
        pub body: Box<Statement<'a>>
    }

    #[derive(Debug, PartialEq, Clone)]
    pub struct For<'a> {
        pub init: Option<Box<Statement<'a>>>,
        pub cond: Option<Expression<'a>>,
        pub incr: Option<Expression<'a>>,
        pub body: Box<Statement<'a>>
    }

    #[derive(Debug, PartialEq, Clone)]
    pub struct Print<'a> {
        pub expr: ast::expr::Expression<'a>
    }

    #[derive(Debug, PartialEq, Clone)]
    pub struct Return<'a> {
        pub expr: Option<Expression<'a>>
    }

    #[derive(Debug, PartialEq, Clone)]
    pub struct FunctionDeclaration<'a> {
        pub id: lexer::Token<'a>,
        pub pars: Vec<lexer::Token<'a>>,
        pub body: Box<Statement<'a>>,
        pub is_initializer: bool,
        pub is_method: bool,
        pub context_function_slot_idx: Option<usize>
    }

    #[derive(Debug, PartialEq, Clone)]
    pub struct ClassDeclaration<'a> {
        pub id: lexer::Token<'a>,
        pub method_decls: Vec<FunctionDeclaration<'a>>,
        pub super_class: Option<ast::expr::Expression<'a>>,
        pub context_class_slot_idx: Option<usize>
    }

    #[derive(Debug, PartialEq, Clone)]
    pub struct VariableDeclaration<'a> {
        pub id: lexer::Token<'a>,
        pub init: Option<Expression<'a>>
    }

    #[derive(Debug, PartialEq, Clone)]
    pub struct Block<'a> {
        pub statements: Vec<Statement<'a>>
    }

    #[derive(Debug, PartialEq, Clone)]
    pub enum Statement<'a> {
        Expression (Expression<'a>),
        If (If<'a>),
        While (While<'a>),
        For (For<'a>),
        Print (Print<'a>),
        Return (Return<'a>),
        FunctionDeclaration (FunctionDeclaration<'a>),
        ClassDeclaration (ClassDeclaration<'a>),
        VariableDeclaration (VariableDeclaration<'a>),
        Block (Block<'a>)
    }
}
