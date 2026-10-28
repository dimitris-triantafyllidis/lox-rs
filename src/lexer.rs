use std::process;

#[derive(Debug, PartialEq, Clone, Hash, Eq)]
pub enum TokenKind {

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

pub fn print_token_stream(tokens: &[Token]) {

    for token in tokens {
        match token.kind {
            TokenKind::LeftParenthesis  => println!("LEFT_PAREN {} null", token.lexeme),
            TokenKind::RightParenthesis => println!("RIGHT_PAREN {} null", token.lexeme),
            TokenKind::LeftBrace        => println!("LEFT_BRACE {} null", token.lexeme),
            TokenKind::RightBrace       => println!("RIGHT_BRACE {} null", token.lexeme),
            TokenKind::Semicolon        => println!("SEMICOLON {} null", token.lexeme),
            TokenKind::Comma            => println!("COMMA {} null", token.lexeme),
            TokenKind::Dot              => println!("DOT {} null", token.lexeme),
            TokenKind::Minus            => println!("MINUS {} null", token.lexeme),
            TokenKind::Plus             => println!("PLUS {} null", token.lexeme),
            TokenKind::Slash            => println!("SLASH {} null", token.lexeme),
            TokenKind::Star             => println!("STAR {} null", token.lexeme),
            TokenKind::Bang             => println!("BANG {} null", token.lexeme),
            TokenKind::BangEqual        => println!("BANG_EQUAL {} null", token.lexeme),
            TokenKind::Equal            => println!("EQUAL {} null", token.lexeme),
            TokenKind::EqualEqual       => println!("EQUAL_EQUAL {} null", token.lexeme),
            TokenKind::Greater          => println!("GREATER {} null", token.lexeme),
            TokenKind::GreaterEqual     => println!("GREATER_EQUAL {} null", token.lexeme),
            TokenKind::Less             => println!("LESS {} null", token.lexeme),
            TokenKind::LessEqual        => println!("LESS_EQUAL {} null", token.lexeme),
            TokenKind::Identifier       => println!("IDENTIFIER {} null", token.lexeme),
            TokenKind::String           => println!("STRING {} {}", token.lexeme, &token.lexeme[1..token.lexeme.len()-1]),
            TokenKind::Number           => {
                let value = token.lexeme.parse::<f64>().unwrap();
                if value.fract() == 0.0 {
                    println!("NUMBER {} {:.1}", token.lexeme, token.lexeme.parse::<f64>().unwrap());
                }
                else {
                    println!("NUMBER {} {}", token.lexeme, token.lexeme.parse::<f64>().unwrap());
                }
            },
            TokenKind::And              => println!("AND {} null", token.lexeme),
            TokenKind::Class            => println!("CLASS {} null", token.lexeme),
            TokenKind::Else             => println!("ELSE {} null", token.lexeme),
            TokenKind::False            => println!("FALSE {} null", token.lexeme),
            TokenKind::Fun              => println!("FUN {} null", token.lexeme),
            TokenKind::For              => println!("FOR {} null", token.lexeme),
            TokenKind::If               => println!("IF {} null", token.lexeme),
            TokenKind::Nil              => println!("NIL {} null", token.lexeme),
            TokenKind::Or               => println!("OR {} null", token.lexeme),
            TokenKind::Print            => println!("PRINT {} null", token.lexeme),
            TokenKind::Return           => println!("RETURN {} null", token.lexeme),
            TokenKind::Super            => println!("SUPER {} null", token.lexeme),
            TokenKind::This             => println!("THIS {} null", token.lexeme),
            TokenKind::True             => println!("TRUE {} null", token.lexeme),
            TokenKind::Var              => println!("VAR {} null", token.lexeme),
            TokenKind::While            => println!("WHILE {} null", token.lexeme),
            TokenKind::EOF              => println!("EOF  null"),
        }
    }

}

#[derive(Debug, PartialEq, Clone, Hash, Eq)]
pub struct Token<'a> {
    pub kind: TokenKind,
    pub lexeme: &'a str,
    pub line_number: usize
}

pub fn lexer_match_one_or_two_character_token<'a>(characters: &'a str, cursor: usize, line_number: usize) -> Option<(Token<'a>, usize)> {

    // One-character tokens

    match characters.as_bytes()[cursor] {
        b'(' => { return Some ( ( Token { kind: TokenKind::LeftParenthesis  , lexeme: &characters[cursor..cursor+1], line_number }, line_number ) ) },
        b')' => { return Some ( ( Token { kind: TokenKind::RightParenthesis , lexeme: &characters[cursor..cursor+1], line_number }, line_number ) ) },
        b'{' => { return Some ( ( Token { kind: TokenKind::LeftBrace        , lexeme: &characters[cursor..cursor+1], line_number }, line_number ) ) },
        b'}' => { return Some ( ( Token { kind: TokenKind::RightBrace       , lexeme: &characters[cursor..cursor+1], line_number }, line_number ) ) },
        b',' => { return Some ( ( Token { kind: TokenKind::Comma            , lexeme: &characters[cursor..cursor+1], line_number }, line_number ) ) },
        b'.' => { return Some ( ( Token { kind: TokenKind::Dot              , lexeme: &characters[cursor..cursor+1], line_number }, line_number ) ) },
        b'-' => { return Some ( ( Token { kind: TokenKind::Minus            , lexeme: &characters[cursor..cursor+1], line_number }, line_number ) ) },
        b'+' => { return Some ( ( Token { kind: TokenKind::Plus             , lexeme: &characters[cursor..cursor+1], line_number }, line_number ) ) },
        b';' => { return Some ( ( Token { kind: TokenKind::Semicolon        , lexeme: &characters[cursor..cursor+1], line_number }, line_number ) ) },
        b'/' => { return Some ( ( Token { kind: TokenKind::Slash            , lexeme: &characters[cursor..cursor+1], line_number }, line_number ) ) },
        b'*' => { return Some ( ( Token { kind: TokenKind::Star             , lexeme: &characters[cursor..cursor+1], line_number }, line_number ) ) },
        _   => ()
    }

    // One-character or two-character tokens

    if cursor < characters.len() - 1 {
        match (characters.as_bytes()[cursor], characters.as_bytes()[cursor + 1]) {
            (b'!', b'=') => { return Some ( ( Token { kind: TokenKind::BangEqual    , lexeme: &characters[cursor..cursor+2], line_number }, line_number ) ) },
            (b'=', b'=') => { return Some ( ( Token { kind: TokenKind::EqualEqual   , lexeme: &characters[cursor..cursor+2], line_number }, line_number ) ) },
            (b'>', b'=') => { return Some ( ( Token { kind: TokenKind::GreaterEqual , lexeme: &characters[cursor..cursor+2], line_number }, line_number ) ) },
            (b'<', b'=') => { return Some ( ( Token { kind: TokenKind::LessEqual    , lexeme: &characters[cursor..cursor+2], line_number }, line_number ) ) },
            _          => ()
        }
    }

    match characters.as_bytes()[cursor] {
        b'!' => { return Some ( ( Token { kind: TokenKind::Bang    , lexeme: &characters[cursor..cursor+1], line_number }, line_number ) ) },
        b'=' => { return Some ( ( Token { kind: TokenKind::Equal   , lexeme: &characters[cursor..cursor+1], line_number }, line_number ) ) },
        b'>' => { return Some ( ( Token { kind: TokenKind::Greater , lexeme: &characters[cursor..cursor+1], line_number }, line_number ) ) },
        b'<' => { return Some ( ( Token { kind: TokenKind::Less    , lexeme: &characters[cursor..cursor+1], line_number }, line_number ) ) },
        _   => ()
    }

    return None;

}

pub fn lexer_match_keyword_or_identifier<'a>(characters: &'a str, cursor_start: usize, line_number: usize) -> Option<(Token<'a>, usize)> {

    let mut cursor_end = cursor_start;

    if characters.as_bytes()[cursor_end].is_ascii_alphabetic() || characters.as_bytes()[cursor_end] == b'_' {

        cursor_end += 1;

        while cursor_end < characters.len() {
            if characters.as_bytes()[cursor_end].is_ascii_alphanumeric() || characters.as_bytes()[cursor_end] == b'_' {
                cursor_end += 1;
            }
            else {
                break;
            }
        }

        let lexeme = &characters[cursor_start..cursor_end];

        match lexeme {

            "and"    => { return Some ( ( Token { kind: TokenKind::And        , lexeme, line_number }, line_number ) ); }
            "class"  => { return Some ( ( Token { kind: TokenKind::Class      , lexeme, line_number }, line_number ) ); }
            "else"   => { return Some ( ( Token { kind: TokenKind::Else       , lexeme, line_number }, line_number ) ); }
            "false"  => { return Some ( ( Token { kind: TokenKind::False      , lexeme, line_number }, line_number ) ); }
            "fun"    => { return Some ( ( Token { kind: TokenKind::Fun        , lexeme, line_number }, line_number ) ); }
            "for"    => { return Some ( ( Token { kind: TokenKind::For        , lexeme, line_number }, line_number ) ); }
            "if"     => { return Some ( ( Token { kind: TokenKind::If         , lexeme, line_number }, line_number ) ); }
            "nil"    => { return Some ( ( Token { kind: TokenKind::Nil        , lexeme, line_number }, line_number ) ); }
            "or"     => { return Some ( ( Token { kind: TokenKind::Or         , lexeme, line_number }, line_number ) ); }
            "print"  => { return Some ( ( Token { kind: TokenKind::Print      , lexeme, line_number }, line_number ) ); }
            "return" => { return Some ( ( Token { kind: TokenKind::Return     , lexeme, line_number }, line_number ) ); }
            "super"  => { return Some ( ( Token { kind: TokenKind::Super      , lexeme, line_number }, line_number ) ); }
            "this"   => { return Some ( ( Token { kind: TokenKind::This       , lexeme, line_number }, line_number ) ); }
            "true"   => { return Some ( ( Token { kind: TokenKind::True       , lexeme, line_number }, line_number ) ); }
            "var"    => { return Some ( ( Token { kind: TokenKind::Var        , lexeme, line_number }, line_number ) ); }
            "while"  => { return Some ( ( Token { kind: TokenKind::While      , lexeme, line_number }, line_number ) ); }
            _ =>        { return Some ( ( Token { kind: TokenKind::Identifier , lexeme, line_number }, line_number ) ); }

        }

    }

    return None;

}

pub fn lexer_match_string_literal<'a>(characters: &'a str, cursor_start: usize, line_number: usize) -> Option<(Token<'a>, usize)>  {

    let mut cursor_end = cursor_start;

    let mut line_number = line_number;

    if characters.as_bytes()[cursor_end] == b'"' {

        cursor_end += 1;

        while cursor_end < characters.len() {
            if characters.as_bytes()[cursor_end] != b'"' {
                if characters.as_bytes()[cursor_end] == b'\n' {
                    line_number += 1;
                }
                if cursor_end == characters.len() - 1 {
                    eprintln!("[line {}] Error: Unterminated string.", line_number);
                    process::exit(1);
                }
                cursor_end += 1;
            }
            else if characters.as_bytes()[cursor_end] == b'"' {
                cursor_end += 1;
                return
                    Some (
                        (
                            Token {
                                kind: TokenKind::String,
                                lexeme: &characters[cursor_start..cursor_end],
                                line_number
                            },
                            line_number
                        )
                    );
            }
        }
    }

    return None;

}

pub fn lexer_match_number_literal<'a>(characters: &'a str, cursor_start: usize, line_number: usize) -> Option<(Token<'a>, usize)> {

    let mut cursor_end = cursor_start;

    if characters.as_bytes()[cursor_end].is_ascii_digit() {

        let mut seen_dot = false;

        cursor_end += 1;

        while cursor_end < characters.len() {

            if characters.as_bytes()[cursor_end] == b'.' {
                if !seen_dot {
                    seen_dot = true;
                    cursor_end += 1;
                }
                else {
                    return None;
                }
            }
            else if characters.as_bytes()[cursor_end].is_ascii_digit() {
                cursor_end += 1;
            }
            else if !characters.as_bytes()[cursor_end].is_ascii_digit() {
                break;
            }
            else {
                return None;
            }
        }
    }

    if !characters[cursor_start..cursor_end].is_empty() {
        if characters.as_bytes()[cursor_end - 1] != b'.' {
            return
                Some (
                    (
                        Token {
                            kind: TokenKind::Number,
                            lexeme: &characters[cursor_start..cursor_end],
                            line_number
                        },
                        line_number
                    )
                );
        }
        else {
            return
                Some (
                    (
                        Token {
                            kind: TokenKind::Number,
                            lexeme: &characters[cursor_start..cursor_end - 1],
                            line_number
                        },
                        line_number
                    )
                );
        }
    }
    else {
        return None;
    }
}

pub fn lexer_scan<'a>(characters: &'a str) -> Vec<Token<'a>> {

    let mut tokens = Vec::<Token<'a>>::new();

    let mut cursor: usize = 0;
    let mut line_number: usize = 1;

    let source_length = characters.len();

    while cursor < source_length {

        // Skip comments

        if source_length - cursor >= 2 && (characters.as_bytes()[cursor], characters.as_bytes()[cursor + 1]) == (b'/', b'/') {
            while cursor < source_length && characters.as_bytes()[cursor] != b'\n' {
                cursor += 1;
            }
            continue;
        }

        // Skip whitespace

        if characters.as_bytes()[cursor] == b'\n' {
            line_number += 1;
        }

        if characters.as_bytes()[cursor].is_ascii_whitespace() {
            cursor += 1;
            continue;
        }

        //

        let matches = [

            lexer_match_one_or_two_character_token (characters, cursor, line_number ),
            lexer_match_keyword_or_identifier      (characters, cursor, line_number ),
            lexer_match_string_literal             (characters, cursor, line_number ),
            lexer_match_number_literal             (characters, cursor, line_number )

        ];

        if let Some((token, new_line_number)) =
            matches
                .into_iter()
                .flatten()
                .max_by_key(|(token, _)| token.lexeme.len())
        {
            cursor += token.lexeme.len();
            line_number = new_line_number;
            tokens.push(token);
        }
        else
        {
            panic!("Lexical error in line {}: Unrecognizable token", line_number);
        }

    }

    tokens.push (
        Token {
            kind: TokenKind::EOF,
            lexeme: &"end",
            line_number
        }
    );

    tokens
}
