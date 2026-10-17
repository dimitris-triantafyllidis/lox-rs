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
    pub lexeme: &'a [char],
    pub line_number: usize
}

pub fn lexer_match_one_or_two_character_token<'a>(characters: &'a [char], cursor: usize, line_number: usize) -> Option<Token<'a>> {

    // One-character tokens

    match characters[cursor] {
        '(' => { return Some ( Token { kind: TokenKind::LeftParenthesis  , lexeme: &characters[cursor..cursor+1], line_number } ); },
        ')' => { return Some ( Token { kind: TokenKind::RightParenthesis , lexeme: &characters[cursor..cursor+1], line_number } ); },
        '{' => { return Some ( Token { kind: TokenKind::LeftBrace        , lexeme: &characters[cursor..cursor+1], line_number } ); },
        '}' => { return Some ( Token { kind: TokenKind::RightBrace       , lexeme: &characters[cursor..cursor+1], line_number } ); },
        ',' => { return Some ( Token { kind: TokenKind::Comma            , lexeme: &characters[cursor..cursor+1], line_number } ); },
        '.' => { return Some ( Token { kind: TokenKind::Dot              , lexeme: &characters[cursor..cursor+1], line_number } ); },
        '-' => { return Some ( Token { kind: TokenKind::Minus            , lexeme: &characters[cursor..cursor+1], line_number } ); },
        '+' => { return Some ( Token { kind: TokenKind::Plus             , lexeme: &characters[cursor..cursor+1], line_number } ); },
        ';' => { return Some ( Token { kind: TokenKind::Semicolon        , lexeme: &characters[cursor..cursor+1], line_number } ); },
        '/' => { return Some ( Token { kind: TokenKind::Slash            , lexeme: &characters[cursor..cursor+1], line_number } ); },
        '*' => { return Some ( Token { kind: TokenKind::Star             , lexeme: &characters[cursor..cursor+1], line_number } ); },
        _   => ()
    }

    // One-character or two-character tokens

    if cursor < characters.len() - 1 {
        match (characters[cursor], characters[cursor + 1]) {
            ('!', '=') => { return Some ( Token { kind: TokenKind::BangEqual    , lexeme: &characters[cursor..cursor+2], line_number } ); },
            ('=', '=') => { return Some ( Token { kind: TokenKind::EqualEqual   , lexeme: &characters[cursor..cursor+2], line_number } ); },
            ('>', '=') => { return Some ( Token { kind: TokenKind::GreaterEqual , lexeme: &characters[cursor..cursor+2], line_number } ); },
            ('<', '=') => { return Some ( Token { kind: TokenKind::LessEqual    , lexeme: &characters[cursor..cursor+2], line_number } ); },
            _          => ()
        }
    }

    match characters[cursor] {
        '!' => { return Some ( Token { kind: TokenKind::Bang    , lexeme: &characters[cursor..cursor+1], line_number } ); },
        '=' => { return Some ( Token { kind: TokenKind::Equal   , lexeme: &characters[cursor..cursor+1], line_number } ); },
        '>' => { return Some ( Token { kind: TokenKind::Greater , lexeme: &characters[cursor..cursor+1], line_number } ); },
        '<' => { return Some ( Token { kind: TokenKind::Less    , lexeme: &characters[cursor..cursor+1], line_number } ); },
        _   => ()
    }

    return None;

}

pub fn lexer_match_keyword_or_identifier<'a>(characters: &'a [char], cursor_start: usize, line_number: usize) -> Option<Token<'a>> {

    let mut cursor_end = cursor_start;

    if characters[cursor_end].is_alphabetic() || characters[cursor_end] == '_' {

        cursor_end += 1;

        while cursor_end < characters.len() {
            if characters[cursor_end].is_alphanumeric() || characters[cursor_end] == '_' {
                cursor_end += 1;
            }
            else {
                break;
            }
        }

        if      characters[cursor_start..cursor_end] == ['a', 'n', 'd'                ] { return Some ( Token { kind: TokenKind::And        , lexeme: &characters[cursor_start..cursor_end], line_number } ); }
        else if characters[cursor_start..cursor_end] == ['c', 'l', 'a', 's', 's'      ] { return Some ( Token { kind: TokenKind::Class      , lexeme: &characters[cursor_start..cursor_end], line_number } ); }
        else if characters[cursor_start..cursor_end] == ['e', 'l', 's', 'e'           ] { return Some ( Token { kind: TokenKind::Else       , lexeme: &characters[cursor_start..cursor_end], line_number } ); }
        else if characters[cursor_start..cursor_end] == ['f', 'a', 'l', 's', 'e'      ] { return Some ( Token { kind: TokenKind::False      , lexeme: &characters[cursor_start..cursor_end], line_number } ); }
        else if characters[cursor_start..cursor_end] == ['f', 'u', 'n'                ] { return Some ( Token { kind: TokenKind::Fun        , lexeme: &characters[cursor_start..cursor_end], line_number } ); }
        else if characters[cursor_start..cursor_end] == ['f', 'o', 'r'                ] { return Some ( Token { kind: TokenKind::For        , lexeme: &characters[cursor_start..cursor_end], line_number } ); }
        else if characters[cursor_start..cursor_end] == ['i', 'f',                    ] { return Some ( Token { kind: TokenKind::If         , lexeme: &characters[cursor_start..cursor_end], line_number } ); }
        else if characters[cursor_start..cursor_end] == ['n', 'i', 'l'                ] { return Some ( Token { kind: TokenKind::Nil        , lexeme: &characters[cursor_start..cursor_end], line_number } ); }
        else if characters[cursor_start..cursor_end] == ['o', 'r',                    ] { return Some ( Token { kind: TokenKind::Or         , lexeme: &characters[cursor_start..cursor_end], line_number } ); }
        else if characters[cursor_start..cursor_end] == ['p', 'r', 'i', 'n', 't'      ] { return Some ( Token { kind: TokenKind::Print      , lexeme: &characters[cursor_start..cursor_end], line_number } ); }
        else if characters[cursor_start..cursor_end] == ['r', 'e', 't', 'u', 'r', 'n' ] { return Some ( Token { kind: TokenKind::Return     , lexeme: &characters[cursor_start..cursor_end], line_number } ); }
        else if characters[cursor_start..cursor_end] == ['s', 'u', 'p', 'e', 'r'      ] { return Some ( Token { kind: TokenKind::Super      , lexeme: &characters[cursor_start..cursor_end], line_number } ); }
        else if characters[cursor_start..cursor_end] == ['t', 'h', 'i', 's'           ] { return Some ( Token { kind: TokenKind::This       , lexeme: &characters[cursor_start..cursor_end], line_number } ); }
        else if characters[cursor_start..cursor_end] == ['t', 'r', 'u', 'e'           ] { return Some ( Token { kind: TokenKind::True       , lexeme: &characters[cursor_start..cursor_end], line_number } ); }
        else if characters[cursor_start..cursor_end] == ['v', 'a', 'r',               ] { return Some ( Token { kind: TokenKind::Var        , lexeme: &characters[cursor_start..cursor_end], line_number } ); }
        else if characters[cursor_start..cursor_end] == ['w', 'h', 'i', 'l', 'e'      ] { return Some ( Token { kind: TokenKind::While      , lexeme: &characters[cursor_start..cursor_end], line_number } ); }
        else                                                                            { return Some ( Token { kind: TokenKind::Identifier , lexeme: &characters[cursor_start..cursor_end], line_number } ); }
    }

    return None;

}

pub fn lexer_match_string_literal<'a>(characters: &'a [char], cursor_start: usize, line_number: usize) -> Option<Token<'a>> {

    let mut cursor_end = cursor_start;

    if characters[cursor_end] == '"' {

        cursor_end += 1;

        while cursor_end < characters.len() {
            if characters[cursor_end] != '"' {
                cursor_end += 1;
            }
            else if characters[cursor_end] == '"' {
                cursor_end += 1;
                return Some ( Token { kind: TokenKind::String, lexeme: &characters[cursor_start..cursor_end], line_number } );
            }
        }
    }

    return None;

}

pub fn lexer_match_number_literal<'a>(characters: &'a [char], cursor_start: usize, line_number: usize) -> Option<Token<'a>> {

    let mut cursor_end = cursor_start;

    if characters[cursor_end].is_ascii_digit() {

        let mut seen_dot = false;

        cursor_end += 1;

        while cursor_end < characters.len() {

            if characters[cursor_end] == '.' {
                if !seen_dot {
                    seen_dot = true;
                    cursor_end += 1;
                }
                else {
                    return None;
                }
            }
            else if characters[cursor_end].is_ascii_digit() {
                cursor_end += 1;
            }
            else if characters[cursor_end].is_whitespace() || characters[cursor_end].is_ascii_punctuation() {
                break;
            }
            else {
                return None;
            }
        }
    }

    if !characters[cursor_start..cursor_end].is_empty() && characters[cursor_end] != '.' {
        return Some ( Token { kind: TokenKind::Number, lexeme: &characters[cursor_start..cursor_end], line_number } );
    }
    else {
        return None;
    }

}

pub fn lexer_scan<'a>(characters: &'a [char]) -> Vec<Token<'a>> {

    let mut tokens = Vec::<Token<'a>>::new();

    let mut cursor: usize = 0;
    let mut line_number: usize = 1;

    let source_length = characters.len();

    while cursor < source_length {

        // Skip comments

        if source_length - cursor >= 2 && (characters[cursor], characters[cursor + 1]) == ('/', '/') {
            while cursor < source_length && characters[cursor] != '\n' {
                cursor += 1;
            }
            continue;
        }

        // Skip whitespace

        if characters[cursor] == '\n' {
            line_number += 1;
        }

        if characters[cursor].is_whitespace() {
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
            lexeme: &[],
            line_number
        }
    );

    tokens
}
