use crate::lexer::*;

pub mod expression {

    use crate::lexer::*;

    #[derive(Debug, Clone, PartialEq)]
    pub struct Literal<'a> {
        pub token: Token<'a>
    }

    #[derive(Debug, Clone, PartialEq)]
    pub struct Variable<'a> {
        pub identifier: Token<'a>,
        pub lookup_hop_count: Option<usize>,
        pub env_slot_idx: Option<usize>
    }

    #[derive(Debug, Clone, PartialEq)]
    pub struct UnaryOperation<'a> {
        pub operator: Token<'a>,
        pub right: Box<Expression<'a>>
    }

    #[derive(Debug, Clone, PartialEq)]
    pub struct BinaryOperation<'a> {
        pub operator: Token<'a>,
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
        pub left: Token<'a>,
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
        pub property: Token<'a>,
        pub value: Box<Expression<'a>>
    }

    #[derive(Debug, Clone, PartialEq)]
    pub struct Get<'a> {
        pub instance: Box<Expression<'a>>,
        pub property: Token<'a>
    }

    #[derive(Debug, Clone, PartialEq)]
    pub struct Super<'a> {
        pub property: Token<'a>,
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

use expression::*;

pub mod statement {

    use crate::{lexer::*, parser::statement};

    #[derive(Debug, PartialEq, Clone)]
    pub struct Expression<'a> {
        pub expr: super::expression::Expression<'a>
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
        pub expr: super::expression::Expression<'a>
    }

    #[derive(Debug, PartialEq, Clone)]
    pub struct Return<'a> {
        pub expr: Option<Expression<'a>>
    }

    #[derive(Debug, PartialEq, Clone)]
    pub struct FunctionDeclaration<'a> {
        pub id: Token<'a>,
        pub pars: Vec<Token<'a>>,
        pub body: Box<Statement<'a>>,
        pub is_initializer: bool,
        pub is_method: bool,
        pub context_function_slot_idx: Option<usize>
    }

    #[derive(Debug, PartialEq, Clone)]
    pub struct ClassDeclaration<'a> {
        pub id: Token<'a>,
        pub method_decls: Vec<statement::FunctionDeclaration<'a>>,
        pub super_class: Option<super::expression::Expression<'a>>
    }

    #[derive(Debug, PartialEq, Clone)]
    pub struct VariableDeclaration<'a> {
        pub id: Token<'a>,
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

use statement::*;

pub fn syntax_error<'a>(tokens: &[Token<'a>], cursor: usize, message: &str) -> ! {
    eprintln!("Syntax error in line {}, '{}': {}", tokens[cursor].line_number, tokens[cursor].lexeme, message);
    std::process::exit(1);
}

pub fn parse<'a>(tokens: &[Token<'a>]) -> Vec<Statement<'a>> {

    let mut cursor: usize = 0;
    let mut stmt: Statement;

    let mut statements = Vec::<Statement>::new();

    loop {

        if tokens[cursor].kind == TokenKind::EOF {
            break;
        }

        (stmt, cursor) = parse_declaration(tokens, cursor);
        statements.push(stmt);

    }

    return statements;
}

pub fn parse_declaration<'a>(tokens: &[Token<'a>], cursor: usize) -> (Statement<'a>, usize) {

    if tokens[cursor].kind == TokenKind::Var {
        return parse_variable_declaration(tokens, cursor + 1);
    }
    if tokens[cursor].kind == TokenKind::Fun {
        return parse_function_declaration(tokens, cursor + 1);
    }
    if tokens[cursor].kind == TokenKind::Class {
        return parse_class_declaration(tokens, cursor + 1);
    }
    else {
        return parse_statement(tokens, cursor);
    }
}

pub fn parse_function_declaration<'a>(tokens: &[Token<'a>], cursor: usize) -> (Statement<'a>, usize) {

    let mut cursor = cursor;

    if tokens[cursor].kind != TokenKind::Identifier {
        syntax_error(tokens, cursor, &"Expect variable name.");
    }

    let identifier = tokens[cursor].clone();
    cursor += 1;

    if tokens[cursor].kind != TokenKind::LeftParenthesis {
        syntax_error(tokens, cursor, &"Expected '('");
    }

    let mut parameters = Vec::<Token>::new();
    cursor += 1;

    loop {
        if tokens[cursor].kind == TokenKind::RightParenthesis {
            cursor += 1;
            break;
        }
        else if parameters.len() < 255 {
            if tokens[cursor].kind == TokenKind::Identifier {
                parameters.push(tokens[cursor].clone());
                cursor += 1;
            }
            else {
                syntax_error(tokens, cursor, &"Expect variable name.");
            }
            if tokens[cursor].kind == TokenKind::Comma {
                cursor += 1;
            }
            else if tokens[cursor].kind == TokenKind::RightParenthesis {
                cursor += 1;
                break;
            }
            else {
                syntax_error(tokens, cursor, &"Expect ')' after parameters.");
            }
        }
        else {
            syntax_error(tokens, cursor, &"Can't have more than 255 parameters.");
        }
    }

    if tokens[cursor].kind != TokenKind::LeftBrace {
        syntax_error(tokens, cursor, &"Expect '{' before function body.");
    }
    cursor += 1;
    let (body, cursor) = parse_block(tokens, cursor);
    return (
        Statement::FunctionDeclaration (
            FunctionDeclaration {
                id: identifier,
                pars: parameters,
                body: Box::new(body),
                is_initializer: false,
                is_method: false,
                context_function_slot_idx: None
            }
        ),
        cursor
    )
}

pub fn parse_class_declaration<'a>(tokens: &[Token<'a>], cursor: usize) -> (Statement<'a>, usize) {

    let mut cursor = cursor;

    if tokens[cursor].kind != TokenKind::Identifier {
        syntax_error(tokens, cursor, &"Expected identifier");
    }

    let identifier = tokens[cursor].clone();
    cursor += 1;

    let super_class: Option<expression::Expression>;

    if tokens[cursor].kind == TokenKind::Less {
        cursor += 1;
        if tokens[cursor].kind != TokenKind::Identifier {
            syntax_error(tokens, cursor, &"Expect superclass name.");
        }
        if tokens[cursor] == identifier {
            syntax_error(tokens, cursor, &"A class can't inherit from itself.");
        }
        super_class = Some (
            expression::Expression::Variable (
                Variable {
                    identifier: tokens[cursor].clone(),
                    lookup_hop_count: None,
                    env_slot_idx: None
                }
            )
        );
        cursor += 1;
    }
    else {
        super_class = None;
    }

    if tokens[cursor].kind != TokenKind::LeftBrace {
        syntax_error(tokens, cursor, &"Expected '{{'");
    }

    let mut methods = Vec::<statement::FunctionDeclaration>::new();
    cursor += 1;

    loop {
        if tokens[cursor].kind == TokenKind::RightBrace {
            return (
                Statement::ClassDeclaration (
                    ClassDeclaration {
                        id: identifier,
                        method_decls: methods,
                        super_class: super_class
                    }
                ),
                cursor + 1
            );
        }
        else {
            let method: Statement;
            (method, cursor) = parse_function_declaration(tokens, cursor);

            if let Statement::FunctionDeclaration (m) = method {
                methods.push(m);
            }
        }
    }
}

pub fn parse_variable_declaration<'a>(tokens: &[Token<'a>], cursor: usize) -> (Statement<'a>, usize) {

    let mut cursor = cursor;
    let id: Token;
    let expr: expression::Expression;

    if tokens[cursor].kind != TokenKind::Identifier {
        syntax_error(tokens, cursor, &"Expect variable name.");
    }
    id = tokens[cursor].clone();
    cursor += 1;
    if tokens[cursor].kind == TokenKind::Equal {
        (expr, cursor) = parse_expression(tokens, cursor + 1);
        if tokens[cursor].kind != TokenKind::Semicolon {
            syntax_error(tokens, cursor, &"Expected ';'");
        }
        return (
            Statement::VariableDeclaration ( VariableDeclaration { id: id, init: Some(statement::Expression { expr }) } ),
            cursor + 1
        );
    }
    else {
        if tokens[cursor].kind != TokenKind::Semicolon {
            syntax_error(tokens, cursor, &"Expected ';'");
        }
        return (
            Statement::VariableDeclaration ( VariableDeclaration { id: id, init: None } ),
            cursor + 1
        );
    }
}

pub fn parse_statement<'a>(tokens: &[Token<'a>], cursor: usize) -> (Statement<'a>, usize) {

    if tokens[cursor].kind == TokenKind::Print {
        return parse_print_statement(tokens, cursor + 1);
    }
    if tokens[cursor].kind == TokenKind::Return {
        return parse_return_statement(tokens, cursor + 1);
    }
    else if tokens[cursor].kind == TokenKind::While {
        return parse_while_statement(tokens, cursor + 1);
    }
    else if tokens[cursor].kind == TokenKind::For {
        return parse_for_statement(tokens, cursor + 1);
    }
    else if tokens[cursor].kind == TokenKind::If {
        return parse_if_statement(tokens, cursor + 1);
    }
    else if tokens[cursor].kind == TokenKind::LeftBrace {
        return parse_block(tokens, cursor + 1);
    }
    else {
        return parse_expression_statement(tokens, cursor);
    }
}

pub fn parse_while_statement<'a>(tokens: &[Token<'a>], cursor: usize) -> (Statement<'a>, usize) {

    let mut cursor = cursor;

    if tokens[cursor].kind != TokenKind::LeftParenthesis {
        syntax_error(tokens, cursor, &"Expected '('");
    }

    cursor += 1;

    let (condition_expression, mut cursor) = parse_expression(tokens, cursor);

    if tokens[cursor].kind != TokenKind::RightParenthesis {
        syntax_error(tokens, cursor, &"Expected ')'");
    }

    cursor += 1;

    let (body_statement, cursor) = parse_statement(tokens, cursor);

    return (
        Statement::While (
            While {
                condition: statement::Expression { expr: condition_expression },
                body: Box::new(body_statement)
            },
        ),
        cursor
    );
}

pub fn parse_for_statement<'a>(tokens: &[Token<'a>], cursor: usize) -> (Statement<'a>, usize) {

    let mut cursor = cursor;

    let initializer_statement: Option<Box<Statement>>;
    let condition_expression: Option<statement::Expression>;
    let increment_expression: Option<statement::Expression>;
    let body_statement: Box<Statement>;

    if tokens[cursor].kind != TokenKind::LeftParenthesis {
        syntax_error(tokens, cursor, &"Expected '('");
    }

    cursor += 1;

    if tokens[cursor].kind == TokenKind::Semicolon {
        initializer_statement = None;
        cursor += 1;
    }
    else {
        let is: Statement;
        (is, cursor) = parse_declaration(tokens, cursor);
        initializer_statement = Some(Box::new(is));
    }

    if tokens[cursor].kind == TokenKind::Semicolon {
        condition_expression = None;
    }
    else {
        let ce: expression::Expression;
        (ce, cursor) = parse_expression(tokens, cursor);
        condition_expression = Some ( statement::Expression { expr: ce } );
    }

    cursor += 1;

    if tokens[cursor].kind == TokenKind::RightParenthesis {
        increment_expression = None;
    }
    else {
        let ie: expression::Expression;
        (ie, cursor) = parse_expression(tokens, cursor);
        increment_expression = Some ( statement::Expression { expr: ie } );
    }

    cursor += 1;

    let bs: Statement;
    (bs, cursor) = parse_statement(tokens, cursor);
    body_statement = Box::new(bs);

    return (
        Statement::For (
            For {
                init: initializer_statement,
                cond: condition_expression,
                incr: increment_expression,
                body: body_statement
            }
        ),
        cursor
    );

}

pub fn parse_expression_statement<'a>(tokens: &[Token<'a>], cursor: usize) -> (Statement<'a>, usize) {

    let (expr, cursor) = parse_expression(tokens, cursor);

    if tokens[cursor].kind != TokenKind::Semicolon {
        syntax_error(tokens, cursor, &"Expected ';'");
    }
    return (
        Statement::Expression ( statement::Expression { expr } ),
        cursor + 1
    );
}

pub fn parse_block<'a>(tokens: &[Token<'a>], cursor: usize) -> (Statement<'a>, usize) {

    let mut statements = Vec::<Statement>::new();
    let mut cursor = cursor;

    loop {

        if tokens[cursor].kind == TokenKind::RightBrace {
            cursor += 1;
            break;
        }

        if tokens[cursor].kind == TokenKind::EOF {
            syntax_error(tokens, cursor, &"Expected '}}'");
        }

        let statement: Statement;

        (statement, cursor) = parse_declaration(tokens, cursor);

        statements.push(statement);

    }

    return (
        Statement::Block ( Block { statements } ),
        cursor
    )

}

pub fn parse_print_statement<'a>(tokens: &[Token<'a>], cursor: usize) -> (Statement<'a>, usize) {

    let (expr, cursor) = parse_expression(tokens, cursor);

    if tokens[cursor].kind != TokenKind::Semicolon {
        syntax_error(tokens, cursor, &"Expected ';'");
    }
    return (
        Statement::Print ( Print { expr } ),
        cursor + 1
    );
}

pub fn parse_return_statement<'a>(tokens: &[Token<'a>], cursor: usize) -> (Statement<'a>, usize) {

    if tokens[cursor].kind == TokenKind::Semicolon {
        return (
            Statement::Return ( Return { expr: None } ),
            cursor + 1
        );
    }

    let (expr, cursor) = parse_expression(tokens, cursor);

    if tokens[cursor].kind != TokenKind::Semicolon {
        syntax_error(tokens, cursor, &"Expected ';'");
    }
    return (
        Statement::Return ( Return { expr: Some(statement::Expression { expr }) } ),
        cursor + 1
    );
}

pub fn parse_if_statement<'a>(tokens: &[Token<'a>], cursor: usize) -> (Statement<'a>, usize) {

    let mut cursor = cursor;

    if tokens[cursor].kind != TokenKind::LeftParenthesis {
        syntax_error(tokens, cursor, &"Expected '('");
    }

    cursor += 1;

    let (condition_expression, mut cursor) = parse_expression(tokens, cursor);

    if tokens[cursor].kind != TokenKind::RightParenthesis {
        syntax_error(tokens, cursor, &"Expected ')'");
    }

    cursor += 1;

    let (then_statement, mut cursor) = parse_statement(tokens, cursor);

    if tokens[cursor].kind == TokenKind::Else {

        cursor += 1;

        let (else_statement, cursor) = parse_statement(tokens, cursor);

        return (
            Statement::If (
                If {
                    condition: statement::Expression { expr: condition_expression },
                    then_statement: Box::new(then_statement),
                    else_statement: Some(Box::new(else_statement))
                }
            ),
            cursor
        );
    }
    else {
        return (
            Statement::If (
                If {
                    condition: statement::Expression { expr: condition_expression },
                    then_statement: Box::new(then_statement),
                    else_statement: None
                }
            ),
            cursor
        );
    }

}

pub fn parse_expression<'a>(tokens: &[Token<'a>], cursor: usize) -> ( expression::Expression<'a>, usize ) {
    return parse_assignment(tokens, cursor);
}

pub fn parse_assignment<'a>(tokens: &[Token<'a>], cursor: usize) -> ( expression::Expression<'a>, usize ) {

    let (expr, mut cursor) = parse_logic_or(tokens, cursor);

    if tokens[cursor].kind != TokenKind::Equal {
        return (expr, cursor);
    }

    let cursor_equals = cursor;
    cursor += 1;
    let value_expr: expression::Expression;

    (value_expr, cursor) = parse_assignment(tokens, cursor);

    match expr {
        expression::Expression::Variable ( Variable { identifier: t, lookup_hop_count, env_slot_idx } )=> {
            return (
                expression::Expression::Assignment (
                    Assignment {
                        left: t,
                        lookup_hop_count,
                        env_slot_idx,
                        expression: Box::<expression::Expression>::new(value_expr)
                    }
                ),
                cursor
            );
        },
        expression::Expression::Get ( Get { instance, property } ) => {
            return (
                expression::Expression::Set (
                    Set {
                        instance: instance,
                        property: property,
                        value: Box::new(value_expr)
                    }
                ),
                cursor
            );
        }
        _ => {
            syntax_error(tokens, cursor_equals, &"Invalid assignment target.");
        }
    }
}

pub fn parse_logic_or<'a>(tokens: &[Token<'a>], cursor: usize) -> ( expression::Expression<'a>, usize ) {

    let (mut expr, mut cursor) = parse_logic_and(tokens, cursor);

    loop {
        if tokens[cursor].kind != TokenKind::Or {
            break;
        }
        let left = Box::<expression::Expression>::new(expr);
        cursor += 1;
        let (right_expr, new_cursor) = parse_logic_and(tokens, cursor);
        let right = Box::<expression::Expression>::new(right_expr);
        expr = expression::Expression::LogicalOr ( LogicalOr { left, right } );
        cursor = new_cursor;
    }

    return (expr, cursor);

}

pub fn parse_logic_and<'a>(tokens: &[Token<'a>], cursor: usize) -> ( expression::Expression<'a>, usize ) {

    let (mut expr, mut cursor) = parse_equality(tokens, cursor);

    loop {
        if tokens[cursor].kind != TokenKind::And {
            break;
        }
        let left = Box::<expression::Expression>::new(expr);
        cursor += 1;
        let (right_expr, new_cursor) = parse_equality(tokens, cursor);
        let right = Box::<expression::Expression>::new(right_expr);
        expr = expression::Expression::LogicalAnd ( LogicalAnd { left, right } );
        cursor = new_cursor;
    }

    return (expr, cursor);

}

pub fn parse_equality<'a>(tokens: &[Token<'a>], cursor: usize) -> ( expression::Expression<'a>, usize ) {

    let (mut expr, mut cursor) = parse_comparison(tokens, cursor);

    loop {
        if
            tokens[cursor].kind != TokenKind::EqualEqual &&
            tokens[cursor].kind != TokenKind::BangEqual
        {
            break;
        }
        let left = Box::<expression::Expression>::new(expr);
        let operator = tokens[cursor].clone();
        cursor += 1;
        let (right_expr, new_cursor) = parse_comparison(tokens, cursor);
        let right = Box::<expression::Expression>::new(right_expr);
        expr = expression::Expression::BinaryOperation ( BinaryOperation { left, operator, right } );
        cursor = new_cursor;
    }

    return (expr, cursor);

}

pub fn parse_comparison<'a>(tokens: &[Token<'a>], cursor: usize) -> ( expression::Expression<'a>, usize ) {

    let (mut expr, mut cursor) = parse_term(tokens, cursor);

    loop {

        if
            tokens[cursor].kind != TokenKind::Greater      &&
            tokens[cursor].kind != TokenKind::GreaterEqual &&
            tokens[cursor].kind != TokenKind::Less         &&
            tokens[cursor].kind != TokenKind::LessEqual
        {
            break;
        }
        let left = Box::<expression::Expression>::new(expr);
        let operator = tokens[cursor].clone();
        cursor += 1;
        let (right_expr, new_cursor) = parse_term(tokens, cursor);
        let right = Box::<expression::Expression>::new(right_expr);
        expr = expression::Expression::BinaryOperation ( BinaryOperation { left, operator, right } );
        cursor = new_cursor;
    }

    return (expr, cursor);

}

pub fn parse_term<'a>(tokens: &[Token<'a>], cursor: usize) -> ( expression::Expression<'a>, usize ) {

    let (mut expr, mut cursor) = parse_factor(tokens, cursor);

    loop {

        if
            tokens[cursor].kind != TokenKind::Plus  &&
            tokens[cursor].kind != TokenKind::Minus
        {
            break;
        }
        let left = Box::<expression::Expression>::new(expr);
        let operator = tokens[cursor].clone();
        cursor += 1;
        let (right_expr, new_cursor) = parse_factor(tokens, cursor);
        let right = Box::<expression::Expression>::new(right_expr);
        expr = expression::Expression::BinaryOperation ( BinaryOperation { left, operator, right } );
        cursor = new_cursor;
    }

    return (expr, cursor);
}

pub fn parse_factor<'a>(tokens: &[Token<'a>], cursor: usize) -> ( expression::Expression<'a>, usize ) {

    let (mut expr, mut cursor) = parse_unary(tokens, cursor);

    loop {

        if
            tokens[cursor].kind != TokenKind::Star  &&
            tokens[cursor].kind != TokenKind::Slash
        {
            break;
        }
        let left = Box::<expression::Expression>::new(expr);
        let operator = tokens[cursor].clone();
        cursor += 1;
        let (right_expr, new_cursor) = parse_unary(tokens, cursor);
        let right = Box::<expression::Expression>::new(right_expr);
        expr = expression::Expression::BinaryOperation ( BinaryOperation { left, operator, right } );
        cursor = new_cursor;
    }

    return (expr, cursor);

}

pub fn parse_unary<'a>(tokens: &[Token<'a>], cursor: usize) -> ( expression::Expression<'a>, usize ) {

    let mut cursor = cursor;

    if
        tokens[cursor].kind == TokenKind::Bang  ||
        tokens[cursor].kind == TokenKind::Minus
    {
        let operator = tokens[cursor].clone();

        cursor += 1;

        let (right_expr, new_cursor) = parse_unary(tokens, cursor);
        let right = Box::<expression::Expression>::new(right_expr);

        cursor = new_cursor;

        return (expression::Expression::UnaryOperation ( UnaryOperation { operator, right } ), cursor);
    }

    return parse_call(tokens, cursor);

}

pub fn parse_call<'a>(tokens: &[Token<'a>], cursor: usize) -> ( expression::Expression<'a>, usize ) {

    let (mut expr, mut cursor) = parse_primary(tokens, cursor);

    loop {

        if tokens[cursor].kind == TokenKind::LeftParenthesis {

            cursor += 1;
            let mut arguments = Vec::<expression::Expression>::new();

            loop {
                if tokens[cursor].kind == TokenKind::RightParenthesis {
                    cursor += 1;
                    expr = expression::Expression::Call (
                        Call {
                            callee: Box::new(expr),
                            arguments
                        }
                    );
                    break;
                }
                else if arguments.len() < 255 {
                    let argument: expression::Expression;
                    (argument, cursor) = parse_expression(tokens, cursor);
                    arguments.push(argument);
                    if tokens[cursor].kind == TokenKind::Comma {
                        cursor += 1;
                    }
                    else if tokens[cursor].kind == TokenKind::RightParenthesis {
                        cursor += 1;
                        expr = expression::Expression::Call (
                            Call {
                                callee: Box::new(expr),
                                arguments
                            }
                        );
                        break;
                    }
                    else {
                        syntax_error(tokens, cursor, &"Expected ')' or ','");
                    }
                }
                else {
                    syntax_error(tokens, cursor, &"Can't have more than 255 arguments.");
                }
            }
        }
        else if tokens[cursor].kind == TokenKind::Dot {
            cursor += 1;
            if tokens[cursor].kind == TokenKind::Identifier {
                let identifier = tokens[cursor].clone();
                cursor += 1;
                expr = expression::Expression::Get (
                    Get {
                        instance: Box::new(expr),
                        property: identifier
                    }
                )
            }
            else {
                syntax_error(tokens, cursor, &"Expect property name after '.'.");
            }
        }
        else {
            break;
        }
    }

    return (expr, cursor);
}

pub fn parse_primary<'a>(tokens: &[Token<'a>], mut cursor: usize) -> ( expression::Expression<'a>, usize ) {

    if
        tokens[cursor].kind == TokenKind::Nil    ||
        tokens[cursor].kind == TokenKind::False  ||
        tokens[cursor].kind == TokenKind::True   ||
        tokens[cursor].kind == TokenKind::Number ||
        tokens[cursor].kind == TokenKind::String
    {
        return (
            expression::Expression::Literal (
                Literal {
                    token: tokens[cursor].clone()
                }
            ),
            cursor + 1
        );
    }

    if tokens[cursor].kind == TokenKind::This {
        return (
            expression::Expression::This,
            cursor + 1
        );
    }

    if tokens[cursor].kind == TokenKind::Super {

        cursor += 1;

        if tokens[cursor].kind == TokenKind::Dot {
            cursor += 1;
        }
        else {
            syntax_error(tokens, cursor, &"Expect '.' after 'super'.");
        }

        if tokens[cursor].kind == TokenKind::Identifier {
            return (
                expression::Expression::Super (
                    Super {
                        property: tokens[cursor].clone(),
                        lookup_hop_count: None,
                        env_slot_idx: None
                    }
                ),
                cursor + 1
            );
        }
        else {
            syntax_error(tokens, cursor, &"Expect superclass method name.");
        }

    }

    if tokens[cursor].kind == TokenKind::Identifier {
        return (
            expression::Expression::Variable (
                Variable {
                    identifier: tokens[cursor].clone(),
                    lookup_hop_count: None,
                    env_slot_idx: None
                }
            ),
            cursor + 1
        );
    }

    if tokens[cursor].kind == TokenKind::LeftParenthesis {
        cursor += 1;

        let (expr, new_cursor) = parse_expression(tokens, cursor);
        cursor = new_cursor;

        if tokens[cursor].kind != TokenKind::RightParenthesis {
            syntax_error(tokens, cursor, &"Expected ')'");
        }

        cursor += 1;

        return ( expression::Expression::Parentheses ( Parentheses { expression: Box::<expression::Expression>::new(expr) } ), cursor);
    }

    syntax_error(tokens, cursor, &"Expect expression.");
}
