use crate::lexer::*;
use crate::ast as ast;

pub fn syntax_error<'a>(tokens: &[Token<'a>], cursor: usize, message: &str) -> ! {
    eprintln!("Syntax error in line {}, '{}': {}", tokens[cursor].line_number, tokens[cursor].lexeme, message);
    std::process::exit(1);
}

pub fn parse<'a>(tokens: &[Token<'a>]) -> Vec<ast::stmt::Statement<'a>> {

    let mut cursor: usize = 0;
    let mut stmt: ast::stmt::Statement;

    let mut statements = Vec::<ast::stmt::Statement>::new();

    loop {

        if tokens[cursor].kind == TokenKind::EOF {
            break;
        }

        (stmt, cursor) = parse_declaration(tokens, cursor);
        statements.push(stmt);

    }

    return statements;
}

pub fn parse_declaration<'a>(tokens: &[Token<'a>], cursor: usize) -> (ast::stmt::Statement<'a>, usize) {

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

pub fn parse_function_declaration<'a>(tokens: &[Token<'a>], cursor: usize) -> (ast::stmt::Statement<'a>, usize) {

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
        ast::stmt::Statement::FunctionDeclaration (
            ast::stmt::FunctionDeclaration {
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

pub fn parse_class_declaration<'a>(tokens: &[Token<'a>], cursor: usize) -> (ast::stmt::Statement<'a>, usize) {

    let mut cursor = cursor;

    if tokens[cursor].kind != TokenKind::Identifier {
        syntax_error(tokens, cursor, &"Expected identifier");
    }

    let identifier = tokens[cursor].clone();
    cursor += 1;

    let super_class: Option<ast::expr::Expression>;

    if tokens[cursor].kind == TokenKind::Less {
        cursor += 1;
        if tokens[cursor].kind != TokenKind::Identifier {
            syntax_error(tokens, cursor, &"Expect superclass name.");
        }
        if tokens[cursor] == identifier {
            syntax_error(tokens, cursor, &"A class can't inherit from itself.");
        }
        super_class = Some (
            ast::expr::Expression::Variable (
                ast::expr::Variable {
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

    let mut methods = Vec::<ast::stmt::FunctionDeclaration>::new();
    cursor += 1;

    loop {
        if tokens[cursor].kind == TokenKind::RightBrace {
            return (
                ast::stmt::Statement::ClassDeclaration (
                    ast::stmt::ClassDeclaration {
                        id: identifier,
                        method_decls: methods,
                        super_class: super_class,
                        context_class_slot_idx: None
                    }
                ),
                cursor + 1
            );
        }
        else {
            let method: ast::stmt::Statement;
            (method, cursor) = parse_function_declaration(tokens, cursor);

            if let ast::stmt::Statement::FunctionDeclaration (m) = method {
                methods.push(m);
            }
        }
    }
}

pub fn parse_variable_declaration<'a>(tokens: &[Token<'a>], cursor: usize) -> (ast::stmt::Statement<'a>, usize) {

    let mut cursor = cursor;
    let id: Token;
    let expr: ast::expr::Expression;

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
            ast::stmt::Statement::VariableDeclaration ( ast::stmt::VariableDeclaration { id: id, init: Some(ast::stmt::Expression { expr }) } ),
            cursor + 1
        );
    }
    else {
        if tokens[cursor].kind != TokenKind::Semicolon {
            syntax_error(tokens, cursor, &"Expected ';'");
        }
        return (
            ast::stmt::Statement::VariableDeclaration ( ast::stmt::VariableDeclaration { id: id, init: None } ),
            cursor + 1
        );
    }
}

pub fn parse_statement<'a>(tokens: &[Token<'a>], cursor: usize) -> (ast::stmt::Statement<'a>, usize) {

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

pub fn parse_while_statement<'a>(tokens: &[Token<'a>], cursor: usize) -> (ast::stmt::Statement<'a>, usize) {

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
        ast::stmt::Statement::While (
            ast::stmt::While {
                condition: ast::stmt::Expression { expr: condition_expression },
                body: Box::new(body_statement)
            },
        ),
        cursor
    );
}

pub fn parse_for_statement<'a>(tokens: &[Token<'a>], cursor: usize) -> (ast::stmt::Statement<'a>, usize) {

    let mut cursor = cursor;

    let initializer_statement: Option<Box<ast::stmt::Statement>>;
    let condition_expression: Option<ast::stmt::Expression>;
    let increment_expression: Option<ast::stmt::Expression>;
    let body_statement: Box<ast::stmt::Statement>;

    if tokens[cursor].kind != TokenKind::LeftParenthesis {
        syntax_error(tokens, cursor, &"Expected '('");
    }

    cursor += 1;

    if tokens[cursor].kind == TokenKind::Semicolon {
        initializer_statement = None;
        cursor += 1;
    }
    else {
        let is: ast::stmt::Statement;
        (is, cursor) = parse_declaration(tokens, cursor);
        initializer_statement = Some(Box::new(is));
    }

    if tokens[cursor].kind == TokenKind::Semicolon {
        condition_expression = None;
    }
    else {
        let ce: ast::expr::Expression;
        (ce, cursor) = parse_expression(tokens, cursor);
        condition_expression = Some ( ast::stmt::Expression { expr: ce } );
    }

    cursor += 1;

    if tokens[cursor].kind == TokenKind::RightParenthesis {
        increment_expression = None;
    }
    else {
        let ie: ast::expr::Expression;
        (ie, cursor) = parse_expression(tokens, cursor);
        increment_expression = Some ( ast::stmt::Expression { expr: ie } );
    }

    cursor += 1;

    let bs: ast::stmt::Statement;
    (bs, cursor) = parse_statement(tokens, cursor);
    body_statement = Box::new(bs);

    return (
        ast::stmt::Statement::For (
            ast::stmt::For {
                init: initializer_statement,
                cond: condition_expression,
                incr: increment_expression,
                body: body_statement
            }
        ),
        cursor
    );

}

pub fn parse_expression_statement<'a>(tokens: &[Token<'a>], cursor: usize) -> (ast::stmt::Statement<'a>, usize) {

    let (expr, cursor) = parse_expression(tokens, cursor);

    if tokens[cursor].kind != TokenKind::Semicolon {
        syntax_error(tokens, cursor, &"Expected ';'");
    }
    return (
        ast::stmt::Statement::Expression ( ast::stmt::Expression { expr } ),
        cursor + 1
    );
}

pub fn parse_block<'a>(tokens: &[Token<'a>], cursor: usize) -> (ast::stmt::Statement<'a>, usize) {

    let mut statements = Vec::<ast::stmt::Statement>::new();
    let mut cursor = cursor;

    loop {

        if tokens[cursor].kind == TokenKind::RightBrace {
            cursor += 1;
            break;
        }

        if tokens[cursor].kind == TokenKind::EOF {
            syntax_error(tokens, cursor, &"Expected '}}'");
        }

        let statement: ast::stmt::Statement;

        (statement, cursor) = parse_declaration(tokens, cursor);

        statements.push(statement);

    }

    return (
        ast::stmt::Statement::Block ( ast::stmt::Block { statements } ),
        cursor
    )

}

pub fn parse_print_statement<'a>(tokens: &[Token<'a>], cursor: usize) -> (ast::stmt::Statement<'a>, usize) {

    let (expr, cursor) = parse_expression(tokens, cursor);

    if tokens[cursor].kind != TokenKind::Semicolon {
        syntax_error(tokens, cursor, &"Expected ';'");
    }
    return (
        ast::stmt::Statement::Print ( ast::stmt::Print { expr } ),
        cursor + 1
    );
}

pub fn parse_return_statement<'a>(tokens: &[Token<'a>], cursor: usize) -> (ast::stmt::Statement<'a>, usize) {

    if tokens[cursor].kind == TokenKind::Semicolon {
        return (
            ast::stmt::Statement::Return ( ast::stmt::Return { expr: None } ),
            cursor + 1
        );
    }

    let (expr, cursor) = parse_expression(tokens, cursor);

    if tokens[cursor].kind != TokenKind::Semicolon {
        syntax_error(tokens, cursor, &"Expected ';'");
    }
    return (
        ast::stmt::Statement::Return ( ast::stmt::Return { expr: Some(ast::stmt::Expression { expr }) } ),
        cursor + 1
    );
}

pub fn parse_if_statement<'a>(tokens: &[Token<'a>], cursor: usize) -> (ast::stmt::Statement<'a>, usize) {

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
            ast::stmt::Statement::If (
                ast::stmt::If {
                    condition: ast::stmt::Expression { expr: condition_expression },
                    then_statement: Box::new(then_statement),
                    else_statement: Some(Box::new(else_statement))
                }
            ),
            cursor
        );
    }
    else {
        return (
            ast::stmt::Statement::If (
                ast::stmt::If {
                    condition: ast::stmt::Expression { expr: condition_expression },
                    then_statement: Box::new(then_statement),
                    else_statement: None
                }
            ),
            cursor
        );
    }

}

pub fn parse_expression<'a>(tokens: &[Token<'a>], cursor: usize) -> ( ast::expr::Expression<'a>, usize ) {
    return parse_assignment(tokens, cursor);
}

pub fn parse_assignment<'a>(tokens: &[Token<'a>], cursor: usize) -> ( ast::expr::Expression<'a>, usize ) {

    let (expr, mut cursor) = parse_logic_or(tokens, cursor);

    if tokens[cursor].kind != TokenKind::Equal {
        return (expr, cursor);
    }

    let cursor_equals = cursor;
    cursor += 1;
    let value_expr: ast::expr::Expression;

    (value_expr, cursor) = parse_assignment(tokens, cursor);

    match expr {
        ast::expr::Expression::Variable ( ast::expr::Variable { identifier: t, lookup_hop_count, env_slot_idx } )=> {
            return (
                ast::expr::Expression::Assignment (
                    ast::expr::Assignment {
                        left: t,
                        lookup_hop_count,
                        env_slot_idx,
                        expression: Box::<ast::expr::Expression>::new(value_expr)
                    }
                ),
                cursor
            );
        },
        ast::expr::Expression::Get ( ast::expr::Get { instance, property } ) => {
            return (
                ast::expr::Expression::Set (
                    ast::expr::Set {
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

pub fn parse_logic_or<'a>(tokens: &[Token<'a>], cursor: usize) -> ( ast::expr::Expression<'a>, usize ) {

    let (mut expr, mut cursor) = parse_logic_and(tokens, cursor);

    loop {
        if tokens[cursor].kind != TokenKind::Or {
            break;
        }
        let left = Box::<ast::expr::Expression>::new(expr);
        cursor += 1;
        let (right_expr, new_cursor) = parse_logic_and(tokens, cursor);
        let right = Box::<ast::expr::Expression>::new(right_expr);
        expr = ast::expr::Expression::LogicalOr ( ast::expr::LogicalOr { left, right } );
        cursor = new_cursor;
    }

    return (expr, cursor);

}

pub fn parse_logic_and<'a>(tokens: &[Token<'a>], cursor: usize) -> ( ast::expr::Expression<'a>, usize ) {

    let (mut expr, mut cursor) = parse_equality(tokens, cursor);

    loop {
        if tokens[cursor].kind != TokenKind::And {
            break;
        }
        let left = Box::<ast::expr::Expression>::new(expr);
        cursor += 1;
        let (right_expr, new_cursor) = parse_equality(tokens, cursor);
        let right = Box::<ast::expr::Expression>::new(right_expr);
        expr = ast::expr::Expression::LogicalAnd ( ast::expr::LogicalAnd { left, right } );
        cursor = new_cursor;
    }

    return (expr, cursor);

}

pub fn parse_equality<'a>(tokens: &[Token<'a>], cursor: usize) -> ( ast::expr::Expression<'a>, usize ) {

    let (mut expr, mut cursor) = parse_comparison(tokens, cursor);

    loop {
        if
            tokens[cursor].kind != TokenKind::EqualEqual &&
            tokens[cursor].kind != TokenKind::BangEqual
        {
            break;
        }
        let left = Box::<ast::expr::Expression>::new(expr);
        let operator = tokens[cursor].clone();
        cursor += 1;
        let (right_expr, new_cursor) = parse_comparison(tokens, cursor);
        let right = Box::<ast::expr::Expression>::new(right_expr);
        expr = ast::expr::Expression::BinaryOperation ( ast::expr::BinaryOperation { left, operator, right } );
        cursor = new_cursor;
    }

    return (expr, cursor);

}

pub fn parse_comparison<'a>(tokens: &[Token<'a>], cursor: usize) -> ( ast::expr::Expression<'a>, usize ) {

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
        let left = Box::<ast::expr::Expression>::new(expr);
        let operator = tokens[cursor].clone();
        cursor += 1;
        let (right_expr, new_cursor) = parse_term(tokens, cursor);
        let right = Box::<ast::expr::Expression>::new(right_expr);
        expr = ast::expr::Expression::BinaryOperation ( ast::expr::BinaryOperation { left, operator, right } );
        cursor = new_cursor;
    }

    return (expr, cursor);

}

pub fn parse_term<'a>(tokens: &[Token<'a>], cursor: usize) -> ( ast::expr::Expression<'a>, usize ) {

    let (mut expr, mut cursor) = parse_factor(tokens, cursor);

    loop {

        if
            tokens[cursor].kind != TokenKind::Plus  &&
            tokens[cursor].kind != TokenKind::Minus
        {
            break;
        }
        let left = Box::<ast::expr::Expression>::new(expr);
        let operator = tokens[cursor].clone();
        cursor += 1;
        let (right_expr, new_cursor) = parse_factor(tokens, cursor);
        let right = Box::<ast::expr::Expression>::new(right_expr);
        expr = ast::expr::Expression::BinaryOperation ( ast::expr::BinaryOperation { left, operator, right } );
        cursor = new_cursor;
    }

    return (expr, cursor);
}

pub fn parse_factor<'a>(tokens: &[Token<'a>], cursor: usize) -> ( ast::expr::Expression<'a>, usize ) {

    let (mut expr, mut cursor) = parse_unary(tokens, cursor);

    loop {

        if
            tokens[cursor].kind != TokenKind::Star  &&
            tokens[cursor].kind != TokenKind::Slash
        {
            break;
        }
        let left = Box::<ast::expr::Expression>::new(expr);
        let operator = tokens[cursor].clone();
        cursor += 1;
        let (right_expr, new_cursor) = parse_unary(tokens, cursor);
        let right = Box::<ast::expr::Expression>::new(right_expr);
        expr = ast::expr::Expression::BinaryOperation ( ast::expr::BinaryOperation { left, operator, right } );
        cursor = new_cursor;
    }

    return (expr, cursor);

}

pub fn parse_unary<'a>(tokens: &[Token<'a>], cursor: usize) -> ( ast::expr::Expression<'a>, usize ) {

    let mut cursor = cursor;

    if
        tokens[cursor].kind == TokenKind::Bang  ||
        tokens[cursor].kind == TokenKind::Minus
    {
        let operator = tokens[cursor].clone();

        cursor += 1;

        let (right_expr, new_cursor) = parse_unary(tokens, cursor);
        let right = Box::<ast::expr::Expression>::new(right_expr);

        cursor = new_cursor;

        return (ast::expr::Expression::UnaryOperation ( ast::expr::UnaryOperation { operator, right } ), cursor);
    }

    return parse_call(tokens, cursor);

}

pub fn parse_call<'a>(tokens: &[Token<'a>], cursor: usize) -> ( ast::expr::Expression<'a>, usize ) {

    let (mut expr, mut cursor) = parse_primary(tokens, cursor);

    loop {

        if tokens[cursor].kind == TokenKind::LeftParenthesis {

            cursor += 1;
            let mut arguments = Vec::<ast::expr::Expression>::new();

            loop {
                if tokens[cursor].kind == TokenKind::RightParenthesis {
                    cursor += 1;
                    expr = ast::expr::Expression::Call (
                        ast::expr::Call {
                            callee: Box::new(expr),
                            arguments
                        }
                    );
                    break;
                }
                else if arguments.len() < 255 {
                    let argument: ast::expr::Expression;
                    (argument, cursor) = parse_expression(tokens, cursor);
                    arguments.push(argument);
                    if tokens[cursor].kind == TokenKind::Comma {
                        cursor += 1;
                    }
                    else if tokens[cursor].kind == TokenKind::RightParenthesis {
                        cursor += 1;
                        expr = ast::expr::Expression::Call (
                            ast::expr::Call {
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
            if tokens[cursor].kind != TokenKind::Identifier {
                syntax_error(tokens, cursor, &"Expect property name after '.'.");
            }
            let identifier = tokens[cursor].clone();
            cursor += 1;
            expr = ast::expr::Expression::Get (
                ast::expr::Get {
                    instance: Box::new(expr),
                    property: identifier
                }
            )
        }
        else {
            break;
        }
    }

    return (expr, cursor);
}

pub fn parse_primary<'a>(tokens: &[Token<'a>], mut cursor: usize) -> ( ast::expr::Expression<'a>, usize ) {

    if
        tokens[cursor].kind == TokenKind::Nil    ||
        tokens[cursor].kind == TokenKind::False  ||
        tokens[cursor].kind == TokenKind::True   ||
        tokens[cursor].kind == TokenKind::Number ||
        tokens[cursor].kind == TokenKind::String
    {
        return (
            ast::expr::Expression::Literal (
                ast::expr::Literal {
                    token: tokens[cursor].clone()
                }
            ),
            cursor + 1
        );
    }

    if tokens[cursor].kind == TokenKind::This {
        return (
            ast::expr::Expression::This,
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
                ast::expr::Expression::Super (
                    ast::expr::Super {
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
            ast::expr::Expression::Variable (
                ast::expr::Variable {
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

        return ( ast::expr::Expression::Parentheses ( ast::expr::Parentheses { expression: Box::<ast::expr::Expression>::new(expr) } ), cursor);
    }

    syntax_error(tokens, cursor, &"Expect expression.");
}
