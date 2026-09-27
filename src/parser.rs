use crate::lexer::*;

pub mod expression {

    use crate::lexer::*;

    #[derive(Debug, Clone, PartialEq)]
    pub struct Literal {
        pub token: Token
    }

    #[derive(Debug, Clone, PartialEq)]
    pub struct Variable {
        pub identifier: Token,
        pub lookup_hop_count: Option<usize>
    }

    #[derive(Debug, Clone, PartialEq)]
    pub struct UnaryOperation {
        pub operator: Token,
        pub right: Box<Expression>
    }

    #[derive(Debug, Clone, PartialEq)]
    pub struct BinaryOperation {
        pub operator: Token,
        pub left: Box<Expression>,
        pub right: Box<Expression>
    }

    #[derive(Debug, Clone, PartialEq)]
    pub struct LogicalOr {
        pub left: Box<Expression>,
        pub right: Box<Expression>
    }

    #[derive(Debug, Clone, PartialEq)]
    pub struct LogicalAnd {
        pub left: Box<Expression>,
        pub right: Box<Expression>
    }

    #[derive(Debug, Clone, PartialEq)]
    pub struct Parentheses {
        pub expression: Box<Expression>
    }

    #[derive(Debug, Clone, PartialEq)]
    pub struct Assignment {
        pub left: Token,
        pub lookup_hop_count: Option<usize>,
        pub expression: Box<Expression>
    }

    #[derive(Debug, Clone, PartialEq)]
    pub struct Call {
        pub callee: Box<Expression>,
        pub arguments: Vec<Expression>
    }

    #[derive(Debug, Clone, PartialEq)]
    pub struct Set {
        pub instance: Box<Expression>,
        pub property: Token,
        pub value: Box<Expression>
    }

    #[derive(Debug, Clone, PartialEq)]
    pub struct Get {
        pub instance: Box<Expression>,
        pub property: Token
    }

    #[derive(Debug, Clone, PartialEq)]
    pub struct Super {
        pub property: Token,
        pub lookup_hop_count: Option<usize>
    }

    #[derive(Debug, PartialEq, Clone)]
    pub enum Expression {
        Literal (Literal),
        Variable (Variable),
        UnaryOperation (UnaryOperation),
        BinaryOperation (BinaryOperation),
        LogicalOr (LogicalOr),
        LogicalAnd (LogicalAnd),
        Parentheses (Parentheses),
        Assignment (Assignment),
        Call (Call),
        Set (Set),
        Get (Get),
        This,
        Super (Super)
    }

}

use expression::*;

pub mod statement {

    use crate::{lexer::*, parser::statement};

    #[derive(Debug, PartialEq, Clone)]
    pub struct Expression {
        pub expr: super::expression::Expression
    }

    #[derive(Debug, PartialEq, Clone)]
    pub struct If {
        pub condition: Expression,
        pub then_statement: Box<Statement>,
        pub else_statement: Option<Box<Statement>>
    }

    #[derive(Debug, PartialEq, Clone)]
    pub struct While {
        pub condition: Expression,
        pub body: Box<Statement>
    }

    #[derive(Debug, PartialEq, Clone)]
    pub struct For {
        pub init: Option<Box<Statement>>,
        pub cond: Option<Expression>,
        pub incr: Option<Expression>,
        pub body: Box<Statement>
    }

    #[derive(Debug, PartialEq, Clone)]
    pub struct Print {
        pub expr: super::expression::Expression
    }

    #[derive(Debug, PartialEq, Clone)]
    pub struct Return {
        pub expr: Option<Expression>
    }

    #[derive(Debug, PartialEq, Clone)]
    pub struct FunctionDeclaration {
        pub id: Token,
        pub pars: Vec<Token>,
        pub body: Box<Statement>
    }

    #[derive(Debug, PartialEq, Clone)]
    pub struct ClassDeclaration {
        pub id: Token,
        pub method_decls: Vec<statement::FunctionDeclaration>,
        pub super_class: Option<super::statement::Expression>
    }

    #[derive(Debug, PartialEq, Clone)]
    pub struct VariableDeclaration {
        pub id: Token,
        pub init: Option<Expression>
    }

    #[derive(Debug, PartialEq, Clone)]
    pub struct Block {
        pub statements: Vec<Statement>
    }

    #[derive(Debug, PartialEq, Clone)]
    pub enum Statement {
        Expression (Expression),
        If (If),
        While (While),
        For (For),
        Print (Print),
        Return (Return),
        FunctionDeclaration (FunctionDeclaration),
        ClassDeclaration (ClassDeclaration),
        VariableDeclaration (VariableDeclaration),
        Block (Block)
    }
}

use statement::*;

pub fn parse(tokens: &Vec<Token>) -> Vec<Statement> {

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

pub fn parse_declaration(tokens: &Vec<Token>, cursor: usize) -> (Statement, usize) {

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

pub fn parse_function_declaration(tokens: &Vec<Token>, cursor: usize) -> (Statement, usize) {

    let mut cursor = cursor;

    if tokens[cursor].kind == TokenKind::Identifier {

        let identifier = tokens[cursor].clone();
        cursor += 1;

        if tokens[cursor].kind == TokenKind::LeftParenthesis {

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
                        panic!("Syntax error in line {}: Expected identifier", tokens[cursor].line_number);
                    }
                    if tokens[cursor].kind == TokenKind::Comma {
                        cursor += 1;
                    }
                    else if tokens[cursor].kind == TokenKind::RightParenthesis {
                        cursor += 1;
                        break;
                    }
                    else {
                        panic!("Syntax error in line {}: Expected ')' or ','", tokens[cursor].line_number);
                    }
                }
                else {
                    panic!("Syntax error in line {}: Can't have more than 255 arguments in a function call", tokens[cursor].line_number);
                }
            }

            if tokens[cursor].kind == TokenKind::LeftBrace {
                cursor += 1;
                let (body, cursor) = parse_block(tokens, cursor);
                return (
                    Statement::FunctionDeclaration (
                        FunctionDeclaration {
                            id: identifier,
                            pars: parameters,
                            body: Box::new(body)
                        }
                    ),
                    cursor
                )
            }
            else {
                panic!("Syntax error in line {}: Expected '{{'", tokens[cursor].line_number);
            }
        }
        else {
            panic!("Syntax error in line {}: Expected '('", tokens[cursor].line_number);
        }
    }
    else {
        panic!("Syntax error in line {}: Expected identifier", tokens[cursor].line_number);
    }

}

pub fn parse_class_declaration(tokens: &Vec<Token>, cursor: usize) -> (Statement, usize) {

    let mut cursor = cursor;

    if tokens[cursor].kind == TokenKind::Identifier {

        let identifier = tokens[cursor].clone();
        cursor += 1;

        let super_class: Option<expression::Expression>;

        if tokens[cursor].kind == TokenKind::Less {
            cursor += 1;
            if tokens[cursor].kind == TokenKind::Identifier {
                if tokens[cursor] == identifier {
                    panic!("Syntax error in line {}: Super class cannot be the same as the inheriting class", tokens[cursor].line_number);
                }
                super_class = Some (
                    expression::Expression::Variable (
                        Variable {
                            identifier: tokens[cursor].clone(),
                            lookup_hop_count: None
                        }
                    )
                );
                cursor += 1;
            }
            else {
                panic!("Syntax error in line {}: Expected super class identifier after '<' in class declaration", tokens[cursor].line_number);
            }
        }
        else {
            super_class = None;
        }

        if tokens[cursor].kind == TokenKind::LeftBrace {

            let mut methods = Vec::<statement::FunctionDeclaration>::new();
            cursor += 1;

            loop {
                if tokens[cursor].kind == TokenKind::RightBrace {
                    return (
                        Statement::ClassDeclaration (
                            ClassDeclaration {
                                id: identifier,
                                method_decls: methods,
                                super_class: Some ( statement::Expression { expr: super_class.unwrap() } )
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
        else {
            panic!("Syntax error in line {}: Expected '{{'", tokens[cursor].line_number);
        }
    }
    else {
        panic!("Syntax error in line {}: Expected identifier", tokens[cursor].line_number);
    }

}

pub fn parse_variable_declaration(tokens: &Vec<Token>, cursor: usize) -> (Statement, usize) {

    let mut cursor = cursor;
    let id: Token;
    let expr: expression::Expression;

    if tokens[cursor].kind == TokenKind::Identifier {
        id = tokens[cursor].clone();
        cursor += 1;
        if tokens[cursor].kind == TokenKind::Equal {
            (expr, cursor) = parse_expression(tokens, cursor + 1);
            if tokens[cursor].kind == TokenKind::Semicolon {
                return (
                    Statement::VariableDeclaration ( VariableDeclaration { id: id, init: Some(statement::Expression { expr }) } ),
                    cursor + 1
                );
            }
            else {
                panic!("Syntax error in line {}: Expected ';'", tokens[cursor].line_number);
            }
        }
        else {
            if tokens[cursor].kind == TokenKind::Semicolon {
                return (
                    Statement::VariableDeclaration ( VariableDeclaration { id: id, init: None } ),
                    cursor + 1
                );
            }
            else {
                panic!("Syntax error in line {}: Expected ';'", tokens[cursor].line_number);
            }
        }
    }
    else {
        panic!("Syntax error in line {}: Expected identifier", tokens[cursor].line_number);
    }

}

pub fn parse_statement(tokens: &Vec<Token>, cursor: usize) -> (Statement, usize) {

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

pub fn parse_while_statement(tokens: &Vec<Token>, cursor: usize) -> (Statement, usize) {

    let mut cursor = cursor;

    if tokens[cursor].kind != TokenKind::LeftParenthesis {
        panic!("Syntax error in line {}: Expected '('", tokens[cursor].line_number);
    }

    cursor += 1;

    let (condition_expression, mut cursor) = parse_expression(tokens, cursor);

    if tokens[cursor].kind != TokenKind::RightParenthesis {
        panic!("Syntax error in line {}: Expected ')'", tokens[cursor].line_number);
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

pub fn parse_for_statement(tokens: &Vec<Token>, cursor: usize) -> (Statement, usize) {

    let mut cursor = cursor;

    let initializer_statement: Option<Box<Statement>>;
    let condition_expression: Option<statement::Expression>;
    let increment_expression: Option<statement::Expression>;
    let body_statement: Box<Statement>;

    if tokens[cursor].kind != TokenKind::LeftParenthesis {
        panic!("Syntax error in line {}: Expected '('", tokens[cursor].line_number);
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

    if tokens[cursor].kind == TokenKind::Semicolon {
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

pub fn parse_expression_statement(tokens: &Vec<Token>, cursor: usize) -> (Statement, usize) {

    let (expr, cursor) = parse_expression(tokens, cursor);

    if tokens[cursor].kind == TokenKind::Semicolon {
        return (
            Statement::Expression ( statement::Expression { expr } ),
            cursor + 1
        );
    } else {
        panic!("Syntax error in line {}: Expected ';'", tokens[cursor].line_number);
    }

}

pub fn parse_block(tokens: &Vec<Token>, cursor: usize) -> (Statement, usize) {

    let mut statements = Vec::<Statement>::new();
    let mut cursor = cursor;

    loop {

        if tokens[cursor].kind == TokenKind::RightBrace {
            cursor += 1;
            break;
        }

        if tokens[cursor].kind == TokenKind::EOF {
            panic!("Syntax error in line {}: Expected '}}'", tokens[cursor].line_number);
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

pub fn parse_print_statement(tokens: &Vec<Token>, cursor: usize) -> (Statement, usize) {

    let (expr, cursor) = parse_expression(tokens, cursor);

    if tokens[cursor].kind == TokenKind::Semicolon {
        return (
            Statement::Print ( Print { expr } ),
            cursor + 1
        );
    } else {
        panic!("Syntax error in line {}: Expected ';'", tokens[cursor].line_number);
    }

}

pub fn parse_return_statement(tokens: &Vec<Token>, cursor: usize) -> (Statement, usize) {

    if tokens[cursor].kind == TokenKind::Semicolon {
        return (
            Statement::Return ( Return { expr: None } ),
            cursor + 1
        );
    }

    let (expr, cursor) = parse_expression(tokens, cursor);

    if tokens[cursor].kind == TokenKind::Semicolon {
        return (
            Statement::Return ( Return { expr: Some(statement::Expression { expr }) } ),
            cursor + 1
        );
    } else {
        panic!("Syntax error in line {}: Expected ';'", tokens[cursor].line_number);
    }

}

pub fn parse_if_statement(tokens: &Vec<Token>, cursor: usize) -> (Statement, usize) {

    let mut cursor = cursor;

    if tokens[cursor].kind != TokenKind::LeftParenthesis {
        panic!("Syntax error in line {}: Expected '('", tokens[cursor].line_number);
    }

    cursor += 1;

    let (condition_expression, mut cursor) = parse_expression(tokens, cursor);

    if tokens[cursor].kind != TokenKind::RightParenthesis {
        panic!("Syntax error in line {}: Expected ')'", tokens[cursor].line_number);
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

pub fn parse_expression(tokens: &Vec<Token>, cursor: usize) -> ( expression::Expression, usize ) {
    return parse_assignment(tokens, cursor);
}

pub fn parse_assignment(tokens: &Vec<Token>, cursor: usize) -> ( expression::Expression, usize ) {

    let (expr, mut cursor) = parse_logic_or(tokens, cursor);

    if tokens[cursor].kind == TokenKind::Equal {
        cursor += 1;
        let value_expr: expression::Expression;

        (value_expr, cursor) = parse_assignment(tokens, cursor);

        match expr {
            expression::Expression::Variable ( Variable { identifier: t, lookup_hop_count: None } )=> {
                return (
                    expression::Expression::Assignment (
                        Assignment {
                            left: t,
                            lookup_hop_count: None,
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
                panic!("Syntax error in line {}: Invalid assignment target", tokens[cursor].line_number);
            }
        }
    }
    else {
        return (expr, cursor);
    }

}

pub fn parse_logic_or(tokens: &Vec<Token>, cursor: usize) -> ( expression::Expression, usize ) {

    let (mut expr, mut cursor) = parse_logic_and(tokens, cursor);

    loop {

        if tokens[cursor].kind == TokenKind::Or
        {
            let left = Box::<expression::Expression>::new(expr);
            cursor += 1;
            let (right_expr, new_cursor) = parse_logic_and(tokens, cursor);
            let right = Box::<expression::Expression>::new(right_expr);
            expr = expression::Expression::LogicalOr ( LogicalOr { left, right } );
            cursor = new_cursor;
        }
        else {
            break;
        }

    }

    return (expr, cursor);

}

pub fn parse_logic_and(tokens: &Vec<Token>, cursor: usize) -> ( expression::Expression, usize ) {

    let (mut expr, mut cursor) = parse_equality(tokens, cursor);

    loop {

        if tokens[cursor].kind == TokenKind::And
        {
            let left = Box::<expression::Expression>::new(expr);
            cursor += 1;
            let (right_expr, new_cursor) = parse_equality(tokens, cursor);
            let right = Box::<expression::Expression>::new(right_expr);
            expr = expression::Expression::LogicalAnd ( LogicalAnd { left, right } );
            cursor = new_cursor;
        }
        else {
            break;
        }

    }

    return (expr, cursor);

}

pub fn parse_equality(tokens: &Vec<Token>, cursor: usize) -> ( expression::Expression, usize ) {

    let (mut expr, mut cursor) = parse_comparison(tokens, cursor);

    loop {

        if
            tokens[cursor].kind == TokenKind::EqualEqual ||
            tokens[cursor].kind == TokenKind::BangEqual
        {
            let left = Box::<expression::Expression>::new(expr);
            let operator = tokens[cursor].clone();
            cursor += 1;
            let (right_expr, new_cursor) = parse_comparison(tokens, cursor);
            let right = Box::<expression::Expression>::new(right_expr);
            expr = expression::Expression::BinaryOperation ( BinaryOperation { left, operator, right } );
            cursor = new_cursor;
        }
        else {
            break;
        }

    }

    return (expr, cursor);

}

pub fn parse_comparison(tokens: &Vec<Token>, cursor: usize) -> ( expression::Expression, usize ) {

    let (mut expr, mut cursor) = parse_term(tokens, cursor);

    loop {

        if
            tokens[cursor].kind == TokenKind::Greater      ||
            tokens[cursor].kind == TokenKind::GreaterEqual ||
            tokens[cursor].kind == TokenKind::Less         ||
            tokens[cursor].kind == TokenKind::LessEqual
        {
            let left = Box::<expression::Expression>::new(expr);
            let operator = tokens[cursor].clone();
            cursor += 1;
            let (right_expr, new_cursor) = parse_term(tokens, cursor);
            let right = Box::<expression::Expression>::new(right_expr);
            expr = expression::Expression::BinaryOperation ( BinaryOperation { left, operator, right } );
            cursor = new_cursor;
        }
        else {
            break;
        }

    }

    return (expr, cursor);

}

pub fn parse_term(tokens: &Vec<Token>, cursor: usize) -> ( expression::Expression, usize ) {

    let (mut expr, mut cursor) = parse_factor(tokens, cursor);

    loop {

        if
            tokens[cursor].kind == TokenKind::Plus  ||
            tokens[cursor].kind == TokenKind::Minus
        {
            let left = Box::<expression::Expression>::new(expr);
            let operator = tokens[cursor].clone();
            cursor += 1;
            let (right_expr, new_cursor) = parse_factor(tokens, cursor);
            let right = Box::<expression::Expression>::new(right_expr);
            expr = expression::Expression::BinaryOperation ( BinaryOperation { left, operator, right } );
            cursor = new_cursor;
        }
        else {
            break;
        }

    }

    return (expr, cursor);
}

pub fn parse_factor(tokens: &Vec<Token>, cursor: usize) -> ( expression::Expression, usize ) {

    let (mut expr, mut cursor) = parse_unary(tokens, cursor);

    loop {

        if
            tokens[cursor].kind == TokenKind::Star  ||
            tokens[cursor].kind == TokenKind::Slash
        {
            let left = Box::<expression::Expression>::new(expr);
            let operator = tokens[cursor].clone();
            cursor += 1;
            let (right_expr, new_cursor) = parse_unary(tokens, cursor);
            let right = Box::<expression::Expression>::new(right_expr);
            expr = expression::Expression::BinaryOperation ( BinaryOperation { left, operator, right } );
            cursor = new_cursor;
        }
        else {
            break;
        }

    }

    return (expr, cursor);

}

pub fn parse_unary(tokens: &Vec<Token>, cursor: usize) -> ( expression::Expression, usize ) {

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

pub fn parse_call(tokens: &Vec<Token>, cursor: usize) -> ( expression::Expression, usize ) {

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
                        panic!("Syntax error in line {}: Expected ')' or ','", tokens[cursor].line_number);
                    }
                }
                else {
                    panic!("Syntax error in line {}: Can't have more than 255 arguments in a function call", tokens[cursor].line_number);
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
                panic!("Syntax error in line {}: Expected property name after '.'", tokens[cursor].line_number);
            }
        }
        else {
            break;
        }
    }

    return (expr, cursor);
}

pub fn parse_primary(tokens: &Vec<Token>, mut cursor: usize) -> ( expression::Expression, usize ) {

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
            panic!("Syntax error in line {}: Expected '.' after 'super'", tokens[cursor].line_number);
        }

        if tokens[cursor].kind == TokenKind::Identifier {
            return (
                expression::Expression::Super (
                    Super {
                        property: tokens[cursor].clone(),
                        lookup_hop_count: None
                    }
                ),
                cursor + 1
            );
        }
        else {
            panic!("Syntax error in line {}: Expected property name after 'super.'", tokens[cursor].line_number);
        }

    }

    if tokens[cursor].kind == TokenKind::Identifier {
        return (
            expression::Expression::Variable (
                Variable {
                    identifier: tokens[cursor].clone(),
                    lookup_hop_count: None
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
            panic!("Syntax error in line {}: Expected ')'", tokens[cursor].line_number);
        }

        cursor += 1;

        return ( expression::Expression::Parentheses ( Parentheses { expression: Box::<expression::Expression>::new(expr) } ), cursor);
    }

    panic!("Syntax error in line {}: Expected expression", tokens[cursor].line_number);
}
