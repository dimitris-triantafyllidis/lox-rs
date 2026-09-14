use crate::lexer::*;

#[derive(Debug, PartialEq, Clone)]
pub enum Expression {
    Literal {
        token: Token
    },
    Variable {
        identifier: Token,
    },
    UnaryOperation {
        operator: Token,
        right:    Box<Expression>
    },
    BinaryOperation {
        operator: Token,
        left:     Box<Expression>,
        right:    Box<Expression>
    },
    LogicalOr {
        left:     Box<Expression>,
        right:    Box<Expression>
    },
    LogicalAnd {
        left:     Box<Expression>,
        right:    Box<Expression>
    },
    Parentheses {
        expression: Box<Expression>
    },
    Assignment {
        left: Token,
        expression: Box<Expression>
    },
    Call {
        callee: Box<Expression>,
        arguments: Vec<Expression>
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum Statement {
    Expression(Expression),
    If(Expression, Box<Statement>, Option<Box<Statement>>),
    While(Expression, Box<Statement>),
    For(Option<Box<Statement>>, Option<Expression>, Option<Expression>, Box<Statement>),
    Print(Expression),
    FunctionDeclaration(Token, Vec<Token>, Box<Statement>),
    VariableDeclaration(Token, Option<Expression>),
    Block(Vec<Statement>)
}

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

            #[derive(PartialEq)]
            enum ParameterParsingState {
                Start,
                GetParameter,
                Finish
            }

            let mut state = ParameterParsingState::Start;

            loop {
                match state {
                    ParameterParsingState::Start => {
                        if tokens[cursor].kind == TokenKind::RightParenthesis {
                            cursor += 1;
                            state = ParameterParsingState::Finish;
                        }
                        else {
                            state = ParameterParsingState::GetParameter;
                        }
                    },
                    ParameterParsingState::Finish => {
                        break;
                    },
                    ParameterParsingState::GetParameter => {
                        if parameters.len() < 255 {
                            if tokens[cursor].kind == TokenKind::Identifier {
                                parameters.push(tokens[cursor].clone());
                                cursor += 1;
                            }
                            else {
                                panic!("Expected identifier");
                            }

                            if tokens[cursor].kind == TokenKind::Comma {
                                cursor += 1;
                                state = ParameterParsingState::GetParameter;
                            }
                            else if tokens[cursor].kind == TokenKind::RightParenthesis {
                                cursor += 1;
                                state = ParameterParsingState::Finish;
                            }
                            else {
                                panic!("Expected ')' or ','");
                            }
                        }
                        else {
                            panic!("Can't have more than 255 parameters in a function call")
                        }
                    }
                }
            }

            if tokens[cursor].kind == TokenKind::LeftBrace {
                cursor += 1;
                let (body, cursor) = parse_block(tokens, cursor);
                return (
                    Statement::FunctionDeclaration (
                        identifier,
                        parameters,
                        Box::new(body)
                    ),
                    cursor
                )
            }
            else {
                panic!("Expected '{{'");
            }
        }
        else {
            panic!("Expected '('");
        }
    }
    else {
        panic!("Expected identifier");
    }

}

pub fn parse_variable_declaration(tokens: &Vec<Token>, cursor: usize) -> (Statement, usize) {

    let mut cursor = cursor;
    let id: Token;
    let expr: Expression;

    if tokens[cursor].kind == TokenKind::Identifier {
        id = tokens[cursor].clone();
        cursor += 1;
        if tokens[cursor].kind == TokenKind::Equal {
            (expr, cursor) = parse_expression(tokens, cursor + 1);
            if tokens[cursor].kind == TokenKind::Semicolon {
                return (Statement::VariableDeclaration(id, Some(expr)), cursor + 1);
            }
            else {
                panic!("Expected ';'");
            }
        }
        else {
            if tokens[cursor].kind == TokenKind::Semicolon {
                return (Statement::VariableDeclaration(id, None), cursor + 1);
            }
            else {
                panic!("Expected ';'");
            }
        }
    }
    else {
        panic!("Expected identifier");
    }

}

pub fn parse_statement(tokens: &Vec<Token>, cursor: usize) -> (Statement, usize) {

    if tokens[cursor].kind == TokenKind::Print {
        return parse_print_statement(tokens, cursor + 1);
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
        panic!("Expected '('");
    }

    cursor += 1;

    let (condition_expression, mut cursor) = parse_expression(tokens, cursor);

    if tokens[cursor].kind != TokenKind::RightParenthesis {
        panic!("Expected ')'");
    }

    cursor += 1;

    let (body_statement, cursor) = parse_statement(tokens, cursor);

    return (
        Statement::While (
            condition_expression,
            Box::new(body_statement)
        ),
        cursor
    );
}

pub fn parse_for_statement(tokens: &Vec<Token>, cursor: usize) -> (Statement, usize) {

    let mut cursor = cursor;

    let initializer_statement: Option<Box<Statement>>;
    let condition_expression: Option<Expression>;
    let increment_expression: Option<Expression>;
    let body_statement: Box<Statement>;

    if tokens[cursor].kind != TokenKind::LeftParenthesis {
        panic!("Expected '('");
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
        let ce: Expression;
        (ce, cursor) = parse_expression(tokens, cursor);
        condition_expression = Some(ce);
    }

    cursor += 1;

    if tokens[cursor].kind == TokenKind::Semicolon {
        increment_expression = None;
    }
    else {
        let ie: Expression;
        (ie, cursor) = parse_expression(tokens, cursor);
        increment_expression = Some(ie);
    }

    cursor += 1;

    let bs: Statement;
    (bs, cursor) = parse_statement(tokens, cursor);
    body_statement = Box::new(bs);

    return (
        Statement::For (
            initializer_statement,
            condition_expression,
            increment_expression,
            body_statement
        ),
        cursor
    );

}

pub fn parse_expression_statement(tokens: &Vec<Token>, cursor: usize) -> (Statement, usize) {

    let (expr, cursor) = parse_expression(tokens, cursor);

    if tokens[cursor].kind == TokenKind::Semicolon {
        return (
            Statement::Expression(expr),
            cursor + 1
        );
    } else {
        panic!("Expected ';'");
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
            panic!("Expected '}}'");
        }

        let statement: Statement;

        (statement, cursor) = parse_declaration(tokens, cursor);

        statements.push(statement);

    }

    return (
        Statement::Block(statements),
        cursor
    )

}

pub fn parse_print_statement(tokens: &Vec<Token>, cursor: usize) -> (Statement, usize) {

    let (expr, cursor) = parse_expression(tokens, cursor);

    if tokens[cursor].kind == TokenKind::Semicolon {
        return (
            Statement::Print(expr),
            cursor + 1
        );
    } else {
        panic!("Expected ';'");
    }

}

pub fn parse_if_statement(tokens: &Vec<Token>, cursor: usize) -> (Statement, usize) {

    let mut cursor = cursor;

    if tokens[cursor].kind != TokenKind::LeftParenthesis {
        panic!("Expected '('");
    }

    cursor += 1;

    let (condition_expression, mut cursor) = parse_expression(tokens, cursor);

    if tokens[cursor].kind != TokenKind::RightParenthesis {
        panic!("Expected ')'");
    }

    cursor += 1;

    let (then_statement, mut cursor) = parse_statement(tokens, cursor);

    if tokens[cursor].kind == TokenKind::Else {

        cursor += 1;

        let (else_statement, cursor) = parse_statement(tokens, cursor);

        return (
            Statement::If (
                condition_expression,
                Box::new(then_statement),
                Some(Box::new(else_statement))
            ),
            cursor
        );
    }
    else {
        return (
            Statement::If (
                condition_expression,
                Box::new(then_statement),
                None
            ),
            cursor
        );
    }

}

pub fn parse_expression(tokens: &Vec<Token>, cursor: usize) -> ( Expression, usize ) {
    return parse_assignment(tokens, cursor);
}

pub fn parse_assignment(tokens: &Vec<Token>, cursor: usize) -> ( Expression, usize ) {

    let (expr, mut cursor) = parse_logic_or(tokens, cursor);

    if tokens[cursor].kind == TokenKind::Equal {
        cursor += 1;
        let value_expr: Expression;

        (value_expr, cursor) = parse_assignment(tokens, cursor);

        match expr {
            Expression::Variable{identifier: t} => {
                return (
                    Expression::Assignment { left: t, expression: Box::<Expression>::new(value_expr) },
                    cursor
                );
            },
            _ => {
                panic!("Invalid assignment target");
            }
        }
    }
    else {
        return (expr, cursor);
    }

}

pub fn parse_logic_or(tokens: &Vec<Token>, cursor: usize) -> ( Expression, usize ) {

    let (mut expr, mut cursor) = parse_logic_and(tokens, cursor);

    loop {

        if tokens[cursor].kind == TokenKind::Or
        {
            let left = Box::<Expression>::new(expr);
            cursor += 1;
            let (right_expr, new_cursor) = parse_logic_and(tokens, cursor);
            let right = Box::<Expression>::new(right_expr);
            expr = Expression::LogicalOr { left, right };
            cursor = new_cursor;
        }
        else {
            break;
        }

    }

    return (expr, cursor);

}

pub fn parse_logic_and(tokens: &Vec<Token>, cursor: usize) -> ( Expression, usize ) {

    let (mut expr, mut cursor) = parse_equality(tokens, cursor);

    loop {

        if tokens[cursor].kind == TokenKind::And
        {
            let left = Box::<Expression>::new(expr);
            cursor += 1;
            let (right_expr, new_cursor) = parse_equality(tokens, cursor);
            let right = Box::<Expression>::new(right_expr);
            expr = Expression::LogicalAnd { left, right };
            cursor = new_cursor;
        }
        else {
            break;
        }

    }

    return (expr, cursor);

}

pub fn parse_equality(tokens: &Vec<Token>, cursor: usize) -> ( Expression, usize ) {

    let (mut expr, mut cursor) = parse_comparison(tokens, cursor);

    loop {

        if
            tokens[cursor].kind == TokenKind::EqualEqual ||
            tokens[cursor].kind == TokenKind::BangEqual
        {
            let left = Box::<Expression>::new(expr);
            let operator = tokens[cursor].clone();
            cursor += 1;
            let (right_expr, new_cursor) = parse_comparison(tokens, cursor);
            let right = Box::<Expression>::new(right_expr);
            expr = Expression::BinaryOperation { left, operator, right };
            cursor = new_cursor;
        }
        else {
            break;
        }

    }

    return (expr, cursor);

}

pub fn parse_comparison(tokens: &Vec<Token>, cursor: usize) -> ( Expression, usize ) {

    let (mut expr, mut cursor) = parse_term(tokens, cursor);

    loop {

        if
            tokens[cursor].kind == TokenKind::Greater      ||
            tokens[cursor].kind == TokenKind::GreaterEqual ||
            tokens[cursor].kind == TokenKind::Less         ||
            tokens[cursor].kind == TokenKind::LessEqual
        {
            let left = Box::<Expression>::new(expr);
            let operator = tokens[cursor].clone();
            cursor += 1;
            let (right_expr, new_cursor) = parse_term(tokens, cursor);
            let right = Box::<Expression>::new(right_expr);
            expr = Expression::BinaryOperation { left, operator, right };
            cursor = new_cursor;
        }
        else {
            break;
        }

    }

    return (expr, cursor);

}

pub fn parse_term(tokens: &Vec<Token>, cursor: usize) -> ( Expression, usize ) {

    let (mut expr, mut cursor) = parse_factor(tokens, cursor);

    loop {

        if
            tokens[cursor].kind == TokenKind::Plus  ||
            tokens[cursor].kind == TokenKind::Minus
        {
            let left = Box::<Expression>::new(expr);
            let operator = tokens[cursor].clone();
            cursor += 1;
            let (right_expr, new_cursor) = parse_factor(tokens, cursor);
            let right = Box::<Expression>::new(right_expr);
            expr = Expression::BinaryOperation { left, operator, right };
            cursor = new_cursor;
        }
        else {
            break;
        }

    }

    return (expr, cursor);
}

pub fn parse_factor(tokens: &Vec<Token>, cursor: usize) -> ( Expression, usize ) {

    let (mut expr, mut cursor) = parse_unary(tokens, cursor);

    loop {

        if
            tokens[cursor].kind == TokenKind::Star  ||
            tokens[cursor].kind == TokenKind::Slash
        {
            let left = Box::<Expression>::new(expr);
            let operator = tokens[cursor].clone();
            cursor += 1;
            let (right_expr, new_cursor) = parse_unary(tokens, cursor);
            let right = Box::<Expression>::new(right_expr);
            expr = Expression::BinaryOperation { left, operator, right };
            cursor = new_cursor;
        }
        else {
            break;
        }

    }

    return (expr, cursor);

}

pub fn parse_unary(tokens: &Vec<Token>, cursor: usize) -> ( Expression, usize ) {

    let mut cursor = cursor;

    if
        tokens[cursor].kind == TokenKind::Bang  ||
        tokens[cursor].kind == TokenKind::Minus
    {
        let operator = tokens[cursor].clone();

        cursor += 1;

        let (right_expr, new_cursor) = parse_unary(tokens, cursor);
        let right = Box::<Expression>::new(right_expr);

        cursor = new_cursor;

        return (Expression::UnaryOperation { operator, right }, cursor);
    }

    return parse_call(tokens, cursor);

}

pub fn parse_call(tokens: &Vec<Token>, cursor: usize) -> ( Expression, usize ) {

    let (primary_expr, mut cursor) = parse_primary(tokens, cursor);

    if tokens[cursor].kind == TokenKind::LeftParenthesis {

        cursor += 1;

        let mut arguments = Vec::<Expression>::new();

        #[derive(PartialEq)]
        enum State {
            Start,
            GetArgument,
            Finish
        }

        let mut state = State::Start;

        loop {
            match state {
                State::Start => {
                    if tokens[cursor].kind == TokenKind::RightParenthesis {
                        cursor += 1;
                        state = State::Finish;
                    }
                    else {
                        state = State::GetArgument;
                    }
                },
                State::Finish => {
                    return (
                        Expression::Call {
                            callee: Box::new(primary_expr),
                            arguments: arguments
                        },
                        cursor
                    )
                },
                State::GetArgument => {
                    if arguments.len() < 255 {
                        let argument: Expression;
                        (argument, cursor) = parse_expression(tokens, cursor);
                        arguments.push(argument);
                        if tokens[cursor].kind == TokenKind::Comma {
                            cursor += 1;
                            state = State::GetArgument;
                        }
                        else if tokens[cursor].kind == TokenKind::RightParenthesis {
                            cursor += 1;
                            state = State::Finish;
                        }
                        else {
                            panic!("Expected ')' or ','");
                        }
                    }
                    else {
                        panic!("Can't have more than 255 arguments in a function call")
                    }
                }
            }
        }
    }
    else {
        return (primary_expr, cursor);
    }

}

pub fn parse_primary(tokens: &Vec<Token>, mut cursor: usize) -> (Expression, usize) {

    if
        tokens[cursor].kind == TokenKind::Nil    ||
        tokens[cursor].kind == TokenKind::False  ||
        tokens[cursor].kind == TokenKind::True   ||
        tokens[cursor].kind == TokenKind::Number ||
        tokens[cursor].kind == TokenKind::String
    {
        return (
            Expression::Literal {
                token: tokens[cursor].clone()
            },
            cursor + 1
        );
    }

    if tokens[cursor].kind == TokenKind::Identifier {
        return (
            Expression::Variable {
                identifier: tokens[cursor].clone()
            },
            cursor + 1
        );
    }

    if tokens[cursor].kind == TokenKind::LeftParenthesis {
        cursor += 1;

        let (expr, new_cursor) = parse_expression(tokens, cursor);
        cursor = new_cursor;

        if tokens[cursor].kind != TokenKind::RightParenthesis {
            panic!("Expected ')'");
        }

        cursor += 1;

        return ( Expression::Parentheses { expression: Box::<Expression>::new(expr) }, cursor);
    }

    panic!("Expected expression");
}
