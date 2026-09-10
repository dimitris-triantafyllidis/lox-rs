use std::{env, fs, io, process::ExitCode};

use std::collections::HashMap;

use rustyline::error::ReadlineError;
use rustyline::DefaultEditor;

fn main() -> ExitCode {

    let args: Vec<String> = env::args().collect();

    if args.len() > 2 {
        println!("Usage: lox [script]");
        return ExitCode::from(64);
    }
    else if args.len() == 2 {
        run_file(&args[1]);
    }
    else {
        run_repl();
    }

    return ExitCode::from(0);
}

fn run_file(file_path: &String) {

    match fs::read_to_string(file_path) {
        io::Result::Ok(s) => {
            run(&s);
        },
        io::Result::Err(e) => {
            eprintln!("io error: {}", e);
        }
    }
}

fn run_repl() {

    let mut rl = DefaultEditor::new().unwrap();

    loop {
        match rl.readline("lox > ") {

            Ok(line) => {
                run(&line);
            }
            Err(ReadlineError::Interrupted) => {
                continue;
            }
            Err(ReadlineError::Eof) => {
                break;
            }
            Err(e) => {
                eprintln!("rustyline error: {}", e);
            }
        }
    }
}

fn run(s: &String) {

    let tokens = lexer_scan(&s);
    let parsed = parse(&tokens);

    // println!("{:#?}", &parsed);

    let mut interpreter = Interpreter::new();

    interpreter.execute(&parsed);

}

#[derive(Debug, PartialEq, Clone)]
enum TokenKind {

    // One-character tokens

    LeftParenthesis, RightParenthesis,
    LeftBrace, RightBrace,
    Semicolon, Comma, Dot,
    Minus, Plus, Slash, Star,

    // One-character or two-character tokens

    Bang, BangEqual,
    Equal, EqualEqual,
    Greater, GreaterEqual,
    Less, LessEqual,

    // Literals

    Identifier,
    String,
    Number,

    // Keywords

    And, Class, Else, False,
    Fun, For, If, Nil,
    Or, Print, Return, Super,
    This, True, Var, While,

    //

    EOF

}

#[derive(Debug, Clone)]
struct Token {
    kind: TokenKind,
    lexeme: String
}

fn lexer_match_one_or_two_character_token(characters: &Vec<char>, cursor: usize) -> Option<Token> {

    // One-character tokens

    match characters[cursor] {
        '(' => { return Some ( Token { kind: TokenKind::LeftParenthesis  , lexeme: "(".to_string() } ); },
        ')' => { return Some ( Token { kind: TokenKind::RightParenthesis , lexeme: ")".to_string() } ); },
        '{' => { return Some ( Token { kind: TokenKind::LeftBrace        , lexeme: "{".to_string() } ); },
        '}' => { return Some ( Token { kind: TokenKind::RightBrace       , lexeme: "}".to_string() } ); },
        ',' => { return Some ( Token { kind: TokenKind::Comma            , lexeme: ",".to_string() } ); },
        '.' => { return Some ( Token { kind: TokenKind::Dot              , lexeme: ".".to_string() } ); },
        '-' => { return Some ( Token { kind: TokenKind::Minus            , lexeme: "-".to_string() } ); },
        '+' => { return Some ( Token { kind: TokenKind::Plus             , lexeme: "+".to_string() } ); },
        ';' => { return Some ( Token { kind: TokenKind::Semicolon        , lexeme: ";".to_string() } ); },
        '/' => { return Some ( Token { kind: TokenKind::Slash            , lexeme: "/".to_string() } ); },
        '*' => { return Some ( Token { kind: TokenKind::Star             , lexeme: "*".to_string() } ); },
        _   => ()
    }

    // One-character or two-character tokens

    if cursor < characters.len() - 1 {
        match (characters[cursor], characters[cursor + 1]) {
            ('!', '=') => { return Some ( Token { kind: TokenKind::BangEqual    , lexeme: "!=".to_string() } ); },
            ('=', '=') => { return Some ( Token { kind: TokenKind::EqualEqual   , lexeme: "==".to_string() } ); },
            ('>', '=') => { return Some ( Token { kind: TokenKind::GreaterEqual , lexeme: ">=".to_string() } ); },
            ('<', '=') => { return Some ( Token { kind: TokenKind::LessEqual    , lexeme: "<=".to_string() } ); },
            _          => ()
        }
    }

    match characters[cursor] {
        '!' => { return Some ( Token { kind: TokenKind::Bang    , lexeme: "!".to_string() } ); },
        '=' => { return Some ( Token { kind: TokenKind::Equal   , lexeme: "=".to_string() } ); },
        '>' => { return Some ( Token { kind: TokenKind::Greater , lexeme: ">".to_string() } ); },
        '<' => { return Some ( Token { kind: TokenKind::Less    , lexeme: "<".to_string() } ); },
        _   => ()
    }

    return None;

}

fn lexer_match_keyword_or_identifier(characters: &Vec<char>, cursor: usize) -> Option<Token> {

    let mut cursor = cursor;

    if characters[cursor].is_alphabetic() || characters[cursor] == '_' {

        let mut lexeme = String::new();

        lexeme.push(characters[cursor]);
        cursor += 1;

        while cursor < characters.len() {
            if characters[cursor].is_alphanumeric() || characters[cursor] == '_' {
                lexeme.push(characters[cursor]);
                cursor += 1;
            }
            else {
                break;
            }
        }

        if      lexeme == "and"    { return Some ( Token { kind: TokenKind::And        , lexeme: lexeme.clone() } ); }
        else if lexeme == "class"  { return Some ( Token { kind: TokenKind::Class      , lexeme: lexeme.clone() } ); }
        else if lexeme == "else"   { return Some ( Token { kind: TokenKind::Else       , lexeme: lexeme.clone() } ); }
        else if lexeme == "false"  { return Some ( Token { kind: TokenKind::False      , lexeme: lexeme.clone() } ); }
        else if lexeme == "fun"    { return Some ( Token { kind: TokenKind::Fun        , lexeme: lexeme.clone() } ); }
        else if lexeme == "for"    { return Some ( Token { kind: TokenKind::For        , lexeme: lexeme.clone() } ); }
        else if lexeme == "if"     { return Some ( Token { kind: TokenKind::If         , lexeme: lexeme.clone() } ); }
        else if lexeme == "nil"    { return Some ( Token { kind: TokenKind::Nil        , lexeme: lexeme.clone() } ); }
        else if lexeme == "or"     { return Some ( Token { kind: TokenKind::Or         , lexeme: lexeme.clone() } ); }
        else if lexeme == "print"  { return Some ( Token { kind: TokenKind::Print      , lexeme: lexeme.clone() } ); }
        else if lexeme == "return" { return Some ( Token { kind: TokenKind::Return     , lexeme: lexeme.clone() } ); }
        else if lexeme == "super"  { return Some ( Token { kind: TokenKind::Super      , lexeme: lexeme.clone() } ); }
        else if lexeme == "this"   { return Some ( Token { kind: TokenKind::This       , lexeme: lexeme.clone() } ); }
        else if lexeme == "true"   { return Some ( Token { kind: TokenKind::True       , lexeme: lexeme.clone() } ); }
        else if lexeme == "var"    { return Some ( Token { kind: TokenKind::Var        , lexeme: lexeme.clone() } ); }
        else if lexeme == "while"  { return Some ( Token { kind: TokenKind::While      , lexeme: lexeme.clone() } ); }
        else                       { return Some ( Token { kind: TokenKind::Identifier , lexeme: lexeme.clone() } ); }
    }

    return None;

}

fn lexer_match_string_literal(characters: &Vec<char>, cursor: usize) -> Option<Token> {

    let mut cursor = cursor;

    if characters[cursor] == '"' {

        let mut lexeme = String::new();
        lexeme.push(characters[cursor]);

        cursor += 1;

        while cursor < characters.len() {
            if characters[cursor] != '"' {
                lexeme.push(characters[cursor]);
                cursor += 1;
            }
            else if characters[cursor] == '"' {
                lexeme.push(characters[cursor]);
                return Some ( Token { kind: TokenKind::String, lexeme: lexeme.clone() } );
            }
        }
    }

    return None;

}

fn lexer_match_number_literal(characters: &Vec<char>, cursor: usize) -> Option<Token> {

    let mut cursor = cursor;

    let mut lexeme = String::new();

    if characters[cursor].is_ascii_digit() {

        let mut seen_dot = false;

        lexeme.push(characters[cursor]);
        cursor += 1;

        while cursor < characters.len() {

            if characters[cursor] == '.' {
                if !seen_dot {
                    seen_dot = true;
                    lexeme.push(characters[cursor]);
                    cursor += 1;
                }
                else {
                    return None;
                }
            }
            else if characters[cursor].is_ascii_digit() {
                lexeme.push(characters[cursor]);
                cursor += 1;
            }
            else if characters[cursor].is_whitespace() || characters[cursor].is_ascii_punctuation() {
                break;
            }
            else {
                return None;
            }
        }
    }

    if !lexeme.is_empty() && !lexeme.ends_with('.') {
        return Some ( Token { kind: TokenKind::Number, lexeme: lexeme.clone() } );
    }
    else {
        return None;
    }

}

fn lexer_scan(s: &String) -> Vec<Token> {

    let characters: Vec<char> = s.chars().collect();
    let mut tokens = Vec::<Token>::new();
    let mut cursor: usize = 0;

    while cursor < characters.len() {

        // Skip whitespace

        if characters[cursor].is_whitespace() {
            cursor += 1;
            continue;
        }

        //

        let matches = [

            lexer_match_one_or_two_character_token ( &characters, cursor ),
            lexer_match_keyword_or_identifier      ( &characters, cursor ),
            lexer_match_string_literal             ( &characters, cursor ),
            lexer_match_number_literal             ( &characters, cursor )

        ];

        if let Some(token) =
            matches
                .into_iter()
                .flatten()
                .max_by_key(|token| token.lexeme.len())
        {
            cursor += token.lexeme.len();
            tokens.push(token);
        }
        else
        {
            eprintln!("lexical error: unrecognizable token at character {}", cursor);
            break;
        }

    }

    tokens.push (
        Token {
            kind: TokenKind::EOF,
            lexeme: String::from("")
        }
    );

    tokens

}

#[derive(Debug)]
enum Expression {
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
    }
}

#[derive(Debug)]
enum Statement {
    Expression(Expression),
    If(Expression, Box<Statement>, Option<Box<Statement>>),
    While(Expression, Box<Statement>),
    For(Option<Box<Statement>>, Option<Expression>, Option<Expression>, Box<Statement>),
    Print(Expression),
    VariableDeclaration(Token, Option<Expression>),
    Block(Vec<Statement>)
}

fn parse(tokens: &Vec<Token>) -> Vec<Statement> {

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

fn parse_declaration(tokens: &Vec<Token>, cursor: usize) -> (Statement, usize) {

    if tokens[cursor].kind == TokenKind::Var {
        return parse_variable_declaration(tokens, cursor + 1);
    }
    else {
        return parse_statement(tokens, cursor);
    }
}

fn parse_variable_declaration(tokens: &Vec<Token>, cursor: usize) -> (Statement, usize) {

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
        panic!("Expected identifier")
    }

}

fn parse_statement(tokens: &Vec<Token>, cursor: usize) -> (Statement, usize) {

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

fn parse_while_statement(tokens: &Vec<Token>, cursor: usize) -> (Statement, usize) {

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

fn parse_for_statement(tokens: &Vec<Token>, cursor: usize) -> (Statement, usize) {

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

fn parse_expression_statement(tokens: &Vec<Token>, cursor: usize) -> (Statement, usize) {

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

fn parse_block(tokens: &Vec<Token>, cursor: usize) -> (Statement, usize) {

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

fn parse_print_statement(tokens: &Vec<Token>, cursor: usize) -> (Statement, usize) {

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

fn parse_if_statement(tokens: &Vec<Token>, cursor: usize) -> (Statement, usize) {

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

fn parse_expression(tokens: &Vec<Token>, cursor: usize) -> ( Expression, usize ) {
    return parse_assignment(tokens, cursor);
}

fn parse_assignment(tokens: &Vec<Token>, cursor: usize) -> ( Expression, usize ) {

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

fn parse_logic_or(tokens: &Vec<Token>, cursor: usize) -> ( Expression, usize ) {

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

fn parse_logic_and(tokens: &Vec<Token>, cursor: usize) -> ( Expression, usize ) {

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

fn parse_equality(tokens: &Vec<Token>, cursor: usize) -> ( Expression, usize ) {

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

fn parse_comparison(tokens: &Vec<Token>, cursor: usize) -> ( Expression, usize ) {

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

fn parse_term(tokens: &Vec<Token>, cursor: usize) -> ( Expression, usize ) {

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

fn parse_factor(tokens: &Vec<Token>, cursor: usize) -> ( Expression, usize ) {

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

fn parse_unary(tokens: &Vec<Token>, cursor: usize) -> ( Expression, usize ) {

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

    return parse_primary(tokens, cursor);

}

fn parse_primary(tokens: &Vec<Token>, mut cursor: usize) -> (Expression, usize) {

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

#[derive(Debug, Clone, PartialEq)]
enum Value {
    Nil,
    Boolean (bool),
    Number (f64),
    String (String)
}

fn is_truthy(v: &Value) -> bool {
    match v {
        Value::Nil => false,
        Value::Boolean(false) => false,
        _ => true
    }
}

struct Interpreter {
    stack: Vec::<HashMap::<String, Value>>
}

impl Interpreter {

    pub fn new() -> Self {
        Self {
            stack: Vec::<HashMap::<String, Value>>::new()
        }
    }

    fn evaluate_literal(self: &mut Self, expr: &Expression) -> Value {

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

    fn evaluate_variable(self: &mut Self, expr: &Expression) -> Value {

        match expr {
            Expression::Variable { identifier: id } => {
                for i in (0..self.stack.len()).rev() {
                    if self.stack[i].contains_key(&id.lexeme) {
                        return self.stack[i][&id.lexeme].clone();
                    }
                }
                panic!("Undeclared identifier");
            },
            _ => panic!()
        }

    }

    fn evaluate_unary(self: &mut Self, expr: &Expression) -> Value {

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

    fn evaluate_parentheses(self: &mut Self, expr: &Expression) -> Value {

        if let Expression::Parentheses { expression: e } = expr {
            return self.evaluate_expression(e);
        }

        panic!()

    }

    fn evaluate_assignment(self: &mut Self, expr: &Expression) -> Value {

        match expr {
            Expression::Assignment {
                left: lhs,
                expression: rhs
            } => {
                for i in (0..self.stack.len()).rev() {
                    if self.stack[i].contains_key(&lhs.lexeme) {
                        let rhs_value = self.evaluate_expression(rhs);
                        self.stack[i].insert(lhs.lexeme.clone(), rhs_value.clone());
                        return rhs_value;
                    }
                }
                panic!("Undeclared assignment target");
            },
            _ => panic!()
        }

    }

    fn evaluate_binary(self: &mut Self, expr: &Expression) -> Value {

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

    fn evaluate_logical_or(self: &mut Self, expr: &Expression) -> Value {

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

    fn evaluate_logical_and(self: &mut Self, expr: &Expression) -> Value {

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

    fn evaluate_expression(self: &mut Self, expr: &Expression) -> Value {

        return match expr {
            Expression::Literal { .. } => self.evaluate_literal(expr),
            Expression::UnaryOperation { .. }  => self.evaluate_unary(expr),
            Expression::BinaryOperation { .. } => self.evaluate_binary(expr),
            Expression::Parentheses { .. } => self.evaluate_parentheses(expr),
            Expression::Variable { .. } => self.evaluate_variable(expr),
            Expression::Assignment { .. } => self.evaluate_assignment(expr),
            Expression::LogicalOr { .. } => self.evaluate_logical_or(expr),
            Expression::LogicalAnd { .. } => self.evaluate_logical_and(expr),
            _ => panic!()
        }

    }

    fn execute_statement(self: &mut Self, statement: &Statement) {

        match statement {
            Statement::Expression(..) => self.execute_expression_statement(statement),
            Statement::Print(..) => self.execute_print_statement(statement),
            Statement::While(..) => self.execute_while_statement(statement),
            Statement::For(..) => self.execute_for_statement(statement),
            Statement::VariableDeclaration(..) => self.execute_variable_declaration_statement(statement),
            Statement::Block(..) => self.execute_block_statement(statement),
            Statement::If(..) => self.execute_if_statement(statement),
            _ => panic!()
        }

    }

    fn execute_expression_statement(self: &mut Self, statement: &Statement) {

        if let Statement::Expression(expr) = statement {
            self.evaluate_expression(&expr);
            return;
        }

        panic!();

    }

    fn execute_print_statement(self: &mut Self, statement: &Statement) {

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
                    println!("\"{s}\"")
                }
            }

            return;
        }

        panic!();

    }

    fn execute_while_statement(self: &mut Self, statement: &Statement) {

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

    fn execute_for_statement(self: &mut Self, statement: &Statement) {

        if let Statement::For (
            initializer_statement,
            condition_expression,
            increment_expression,
            body_statement,
        ) = statement {

            self.stack.push(HashMap::<String, Value>::new());

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

            self.stack.pop();

            return;
        }

        panic!();

    }

    fn execute_if_statement(self: &mut Self, statement: &Statement) {

        if let Statement::If(condition_expression, then_statement, else_statement) = statement {
            let condition_expression_value = self.evaluate_expression(condition_expression);
            if is_truthy(&condition_expression_value) {
                self.execute_statement(then_statement);
                return;
            }
            else {
                if let Some(statement) = else_statement {
                    self.execute_statement(statement);
                    return;
                }
            }
        }

        panic!();

    }

    fn execute_variable_declaration_statement(self: &mut Self, statement: &Statement) {

        if let Statement::VariableDeclaration(id, expr) = statement {

            let stack_top_frame_idx = self.stack.len() - 1;

            if self.stack[stack_top_frame_idx].contains_key(&id.lexeme) {
                panic!("Variable redeclaration");
            }

            let expr_value = match expr {
                None => Value::Nil,
                Some(expr) => self.evaluate_expression(&expr)
            };

            self.stack[stack_top_frame_idx].insert(id.lexeme.clone(), expr_value.clone());

            return;
        }

        panic!();

    }

    fn execute_block_statement(self: &mut Self, statement: &Statement) {

        if let Statement::Block(statements) = statement {
            self.stack.push(HashMap::<String, Value>::new());
            for statement in statements {
                self.execute_statement(statement);
            }
            self.stack.pop();
            return;
        }

        panic!();
    }

    fn execute(self: &mut Self, statements: &Vec<Statement>) {

        self.stack.push(HashMap::<String, Value>::new());
        for statement in statements {
            self.execute_statement(statement);
        }
        self.stack.pop();
        return;

    }

}
