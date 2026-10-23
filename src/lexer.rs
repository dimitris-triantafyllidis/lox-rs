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

#[derive(Debug, PartialEq, Clone, Hash, Eq)]
pub struct Token<'a> {
    pub kind: TokenKind,
    pub lexeme: &'a str,
    pub line_number: usize
}

pub fn lexer_match_one_or_two_character_token<'a>(characters: &'a str, cursor: usize, line_number: usize) -> Option<Token<'a>> {

    // One-character tokens

    match characters.as_bytes()[cursor] {
        b'(' => { return Some ( Token { kind: TokenKind::LeftParenthesis  , lexeme: &characters[cursor..cursor+1], line_number } ); },
        b')' => { return Some ( Token { kind: TokenKind::RightParenthesis , lexeme: &characters[cursor..cursor+1], line_number } ); },
        b'{' => { return Some ( Token { kind: TokenKind::LeftBrace        , lexeme: &characters[cursor..cursor+1], line_number } ); },
        b'}' => { return Some ( Token { kind: TokenKind::RightBrace       , lexeme: &characters[cursor..cursor+1], line_number } ); },
        b',' => { return Some ( Token { kind: TokenKind::Comma            , lexeme: &characters[cursor..cursor+1], line_number } ); },
        b'.' => { return Some ( Token { kind: TokenKind::Dot              , lexeme: &characters[cursor..cursor+1], line_number } ); },
        b'-' => { return Some ( Token { kind: TokenKind::Minus            , lexeme: &characters[cursor..cursor+1], line_number } ); },
        b'+' => { return Some ( Token { kind: TokenKind::Plus             , lexeme: &characters[cursor..cursor+1], line_number } ); },
        b';' => { return Some ( Token { kind: TokenKind::Semicolon        , lexeme: &characters[cursor..cursor+1], line_number } ); },
        b'/' => { return Some ( Token { kind: TokenKind::Slash            , lexeme: &characters[cursor..cursor+1], line_number } ); },
        b'*' => { return Some ( Token { kind: TokenKind::Star             , lexeme: &characters[cursor..cursor+1], line_number } ); },
        _   => ()
    }

    // One-character or two-character tokens

    if cursor < characters.len() - 1 {
        match (characters.as_bytes()[cursor], characters.as_bytes()[cursor + 1]) {
            (b'!', b'=') => { return Some ( Token { kind: TokenKind::BangEqual    , lexeme: &characters[cursor..cursor+2], line_number } ); },
            (b'=', b'=') => { return Some ( Token { kind: TokenKind::EqualEqual   , lexeme: &characters[cursor..cursor+2], line_number } ); },
            (b'>', b'=') => { return Some ( Token { kind: TokenKind::GreaterEqual , lexeme: &characters[cursor..cursor+2], line_number } ); },
            (b'<', b'=') => { return Some ( Token { kind: TokenKind::LessEqual    , lexeme: &characters[cursor..cursor+2], line_number } ); },
            _          => ()
        }
    }

    match characters.as_bytes()[cursor] {
        b'!' => { return Some ( Token { kind: TokenKind::Bang    , lexeme: &characters[cursor..cursor+1], line_number } ); },
        b'=' => { return Some ( Token { kind: TokenKind::Equal   , lexeme: &characters[cursor..cursor+1], line_number } ); },
        b'>' => { return Some ( Token { kind: TokenKind::Greater , lexeme: &characters[cursor..cursor+1], line_number } ); },
        b'<' => { return Some ( Token { kind: TokenKind::Less    , lexeme: &characters[cursor..cursor+1], line_number } ); },
        _   => ()
    }

    return None;

}

pub fn lexer_match_keyword_or_identifier<'a>(characters: &'a str, cursor_start: usize, line_number: usize) -> Option<Token<'a>> {

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

            "and"    => { return Some ( Token { kind: TokenKind::And        , lexeme, line_number } ); }
            "class"  => { return Some ( Token { kind: TokenKind::Class      , lexeme, line_number } ); }
            "else"   => { return Some ( Token { kind: TokenKind::Else       , lexeme, line_number } ); }
            "false"  => { return Some ( Token { kind: TokenKind::False      , lexeme, line_number } ); }
            "fun"    => { return Some ( Token { kind: TokenKind::Fun        , lexeme, line_number } ); }
            "for"    => { return Some ( Token { kind: TokenKind::For        , lexeme, line_number } ); }
            "if"     => { return Some ( Token { kind: TokenKind::If         , lexeme, line_number } ); }
            "nil"    => { return Some ( Token { kind: TokenKind::Nil        , lexeme, line_number } ); }
            "or"     => { return Some ( Token { kind: TokenKind::Or         , lexeme, line_number } ); }
            "print"  => { return Some ( Token { kind: TokenKind::Print      , lexeme, line_number } ); }
            "return" => { return Some ( Token { kind: TokenKind::Return     , lexeme, line_number } ); }
            "super"  => { return Some ( Token { kind: TokenKind::Super      , lexeme, line_number } ); }
            "this"   => { return Some ( Token { kind: TokenKind::This       , lexeme, line_number } ); }
            "true"   => { return Some ( Token { kind: TokenKind::True       , lexeme, line_number } ); }
            "var"    => { return Some ( Token { kind: TokenKind::Var        , lexeme, line_number } ); }
            "while"  => { return Some ( Token { kind: TokenKind::While      , lexeme, line_number } ); }
            _ =>        { return Some ( Token { kind: TokenKind::Identifier , lexeme, line_number } ); }

        }

    }

    return None;

}

pub fn lexer_match_string_literal<'a>(characters: &'a str, cursor_start: usize, line_number: usize) -> Option<Token<'a>> {

    let mut cursor_end = cursor_start;

    if characters.as_bytes()[cursor_end] == b'"' {

        cursor_end += 1;

        while cursor_end < characters.len() {
            if characters.as_bytes()[cursor_end] != b'"' {
                cursor_end += 1;
            }
            else if characters.as_bytes()[cursor_end] == b'"' {
                cursor_end += 1;
                return Some ( Token { kind: TokenKind::String, lexeme: &characters[cursor_start..cursor_end], line_number } );
            }
        }
    }

    return None;

}

pub fn lexer_match_number_literal<'a>(characters: &'a str, cursor_start: usize, line_number: usize) -> Option<Token<'a>> {

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
            return Some (
                Token {
                    kind: TokenKind::Number,
                    lexeme: &characters[cursor_start..cursor_end],
                    line_number
                }
            );
        }
        else {
            return Some (
                Token {
                    kind: TokenKind::Number,
                    lexeme: &characters[cursor_start..cursor_end - 1],
                    line_number
                }
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
