use std::{env, fs, io, process::ExitCode};

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
    let value = evaluate_expression(&parsed);

    println!("{:?}", value);

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
    UnaryOperation {
        operator: Token,
        right:    Box<Expression>
    },
    BinaryOperation {
        operator: Token,
        left:     Box<Expression>,
        right:    Box<Expression>
    },
    Parentheses {
        expression: Box<Expression>
    }
}

/*

expression -> equality ;
equality   -> comparison ( ( "!=" | "==" ) comparison )* ;
comparison -> term ( ( ">" | ">=" | "<" | "<=" ) term )* ;
term       -> factor ( ( "-" | "+" ) factor )* ;
factor     -> unary ( ( "/" | "*" ) unary )* ;
unary      -> ( "!" | "-" ) unary | primary ;
primary    -> NUMBER | STRING | "true" | "false" | "nil" | "(" expression ")" ;

*/

fn parse_expression(tokens: &Vec<Token>, cursor: usize) -> ( Expression, usize ) {
    return parse_equality(tokens, cursor);
}

fn parse_equality(tokens: &Vec<Token>, cursor: usize) -> ( Expression, usize ) {

    let (mut expr, mut cursor) = parse_comparison(tokens, cursor);

    loop {

        if cursor >= tokens.len() { break; }

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

        if cursor >= tokens.len() { break; }

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

        if cursor >= tokens.len() { break; }

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

        if cursor >= tokens.len() { break; }

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

fn parse(tokens: &Vec<Token>) -> Expression {
    let (expr, cursor) = parse_expression(tokens, 0);

    if tokens[cursor].kind != TokenKind::EOF {
        panic!("Unexpected token '{}'", tokens[cursor].lexeme);
    }

    expr
}


#[derive(Debug, Clone, PartialEq)]
enum Value {
    Nil,
    Boolean (bool),
    Number (f64),
    String (String)
}

fn value_to_bool(v: Value) -> bool {
    match v {
        Value::Nil => false,
        Value::Boolean(false) => false,
        _ => true
    }
}

fn evaluate_literal(expr: &Expression) -> Value {

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
        }
        _ => panic!()
    }

}

fn evaluate_parentheses(expr: &Expression) -> Value {

    if let Expression::Parentheses { expression: e } = expr {
        return evaluate_expression(e);
    }

    panic!()

}

fn evaluate_unary(expr: &Expression) -> Value {

    if let Expression::UnaryOperation { operator: op, right: rhs } = expr {
        let rhs_value = evaluate_expression(rhs);
        match rhs_value {
            Value::Number (v) if op.kind == TokenKind::Minus => {
                return Value::Number (-v)
            }
            _ if op.kind == TokenKind::Bang => {
                return Value::Boolean (!value_to_bool(rhs_value))
            }
            _ => panic!()
        }
    }

    panic!()

}

fn evaluate_binary(expr: &Expression) -> Value {
    if let Expression::BinaryOperation { operator: op, left: lhs, right: rhs } = expr {

        let lhs_value = evaluate_expression(lhs);
        let rhs_value = evaluate_expression(rhs);

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

fn evaluate_expression(expr: &Expression) -> Value {

    return match expr {
        Expression::Literal { token } => evaluate_literal(expr),
        Expression::UnaryOperation { operator, right }  => evaluate_unary(expr),
        Expression::BinaryOperation { left, operator, right } => evaluate_binary(expr),
        Expression::Parentheses { expression } => evaluate_parentheses(expr)
    }

}
