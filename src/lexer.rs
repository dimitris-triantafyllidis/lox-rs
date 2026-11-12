use std::fmt;
use std::process;

#[derive(Debug, PartialEq, Clone, Hash, Eq)]
pub enum TokenKind {

    // One-character tokens

    LeftParenthesis, RightParenthesis,
    LeftBrace, RightBrace,
    Semicolon, Comma,
    Dot, Minus,
    Plus, Slash,
    Star,

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

    And, Or,
    Class, Fun,
    If, Else, For, While,
    True, False, Nil,
    Return, Print,
    Super, This, Var,

    //

    EOF,
}

#[derive(Debug, PartialEq, Clone, Hash, Eq)]
pub struct Token<'a> {
    pub kind:        TokenKind,
    pub lexeme:      &'a str,
    pub line_number: usize,
}

impl<'a> Token<'a> {
    pub fn new(kind: TokenKind, lexeme: &'a str, line_number: usize) -> Self {
        Self { kind, lexeme, line_number }
    }
}

impl fmt::Display for Token<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.kind {
            TokenKind::LeftParenthesis  => write!(f, "LEFT_PAREN {} null",    self.lexeme),
            TokenKind::RightParenthesis => write!(f, "RIGHT_PAREN {} null",   self.lexeme),
            TokenKind::LeftBrace        => write!(f, "LEFT_BRACE {} null",    self.lexeme),
            TokenKind::RightBrace       => write!(f, "RIGHT_BRACE {} null",   self.lexeme),
            TokenKind::Semicolon        => write!(f, "SEMICOLON {} null",     self.lexeme),
            TokenKind::Comma            => write!(f, "COMMA {} null",         self.lexeme),
            TokenKind::Dot              => write!(f, "DOT {} null",           self.lexeme),
            TokenKind::Minus            => write!(f, "MINUS {} null",         self.lexeme),
            TokenKind::Plus             => write!(f, "PLUS {} null",          self.lexeme),
            TokenKind::Slash            => write!(f, "SLASH {} null",         self.lexeme),
            TokenKind::Star             => write!(f, "STAR {} null",          self.lexeme),
            TokenKind::Bang             => write!(f, "BANG {} null",          self.lexeme),
            TokenKind::BangEqual        => write!(f, "BANG_EQUAL {} null",    self.lexeme),
            TokenKind::Equal            => write!(f, "EQUAL {} null",         self.lexeme),
            TokenKind::EqualEqual       => write!(f, "EQUAL_EQUAL {} null",   self.lexeme),
            TokenKind::Greater          => write!(f, "GREATER {} null",       self.lexeme),
            TokenKind::GreaterEqual     => write!(f, "GREATER_EQUAL {} null", self.lexeme),
            TokenKind::Less             => write!(f, "LESS {} null",          self.lexeme),
            TokenKind::LessEqual        => write!(f, "LESS_EQUAL {} null",    self.lexeme),
            TokenKind::Identifier       => write!(f, "IDENTIFIER {} null",    self.lexeme),
            TokenKind::String           => write!(f, "STRING {} {}",          self.lexeme, &self.lexeme[1..self.lexeme.len() - 1]),
            TokenKind::And              => write!(f, "AND {} null",           self.lexeme),
            TokenKind::Class            => write!(f, "CLASS {} null",         self.lexeme),
            TokenKind::Else             => write!(f, "ELSE {} null",          self.lexeme),
            TokenKind::False            => write!(f, "FALSE {} null",         self.lexeme),
            TokenKind::Fun              => write!(f, "FUN {} null",           self.lexeme),
            TokenKind::For              => write!(f, "FOR {} null",           self.lexeme),
            TokenKind::If               => write!(f, "IF {} null",            self.lexeme),
            TokenKind::Nil              => write!(f, "NIL {} null",           self.lexeme),
            TokenKind::Or               => write!(f, "OR {} null",            self.lexeme),
            TokenKind::Print            => write!(f, "PRINT {} null",         self.lexeme),
            TokenKind::Return           => write!(f, "RETURN {} null",        self.lexeme),
            TokenKind::Super            => write!(f, "SUPER {} null",         self.lexeme),
            TokenKind::This             => write!(f, "THIS {} null",          self.lexeme),
            TokenKind::True             => write!(f, "TRUE {} null",          self.lexeme),
            TokenKind::Var              => write!(f, "VAR {} null",           self.lexeme),
            TokenKind::While            => write!(f, "WHILE {} null",         self.lexeme),
            TokenKind::EOF              => write!(f, "EOF  null"),
            TokenKind::Number           => {
                let value = self.lexeme.parse::<f64>().unwrap();
                let formatted = if value.fract() == 0.0 {
                    format!("{value:.1}")
                } else {
                    value.to_string()
                };
                write!(f, "NUMBER {} {}", self.lexeme, formatted)
            }
        }
    }
}

pub fn scan<'a>(characters: &'a str) -> Vec<Token<'a>> {
    let mut tokens = Vec::<Token<'a>>::new();

    let mut cursor: usize = 0;
    let mut line_number: usize = 1;

    let source_length = characters.len();

    while cursor < source_length {

        // Skip comments

        if source_length - cursor >= 2 && ( characters.as_bytes()[cursor], characters.as_bytes()[cursor + 1], ) == (b'/', b'/')
        {
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
            match_one_or_two_character_token ( characters, cursor, line_number ),
            match_keyword_or_identifier      ( characters, cursor, line_number ),
            match_string_literal             ( characters, cursor, line_number ),
            match_number_literal             ( characters, cursor, line_number ),
        ];

        if let Some((token, new_line_number)) = matches.into_iter().flatten().max_by_key(|(token, _)| token.lexeme.len()) {
            cursor += token.lexeme.len();
            line_number = new_line_number;
            tokens.push(token);
        }
        else {
            panic!("Lexical error in line {line_number}: Unrecognizable token");
        }
    }

    tokens.push ( Token::new ( TokenKind::EOF, "end", line_number ) );
    tokens
}

pub fn print_token_stream(tokens: &[Token]) {
    for token in tokens {
        println!("{token}");
    }
}

fn match_one_or_two_character_token(characters: &str, cursor: usize, line_number: usize) -> Option<(Token<'_>, usize)> {

    let ln = line_number;

    // One-character tokens

    let lexeme = &characters[cursor..=cursor];

    match characters.as_bytes()[cursor] {
        b'(' => return Some ( ( Token::new ( TokenKind::LeftParenthesis,  lexeme, ln ), ln) ),
        b')' => return Some ( ( Token::new ( TokenKind::RightParenthesis, lexeme, ln ), ln) ),
        b'{' => return Some ( ( Token::new ( TokenKind::LeftBrace,        lexeme, ln ), ln) ),
        b'}' => return Some ( ( Token::new ( TokenKind::RightBrace,       lexeme, ln ), ln) ),
        b',' => return Some ( ( Token::new ( TokenKind::Comma,            lexeme, ln ), ln) ),
        b'.' => return Some ( ( Token::new ( TokenKind::Dot,              lexeme, ln ), ln) ),
        b'-' => return Some ( ( Token::new ( TokenKind::Minus,            lexeme, ln ), ln) ),
        b'+' => return Some ( ( Token::new ( TokenKind::Plus,             lexeme, ln ), ln) ),
        b';' => return Some ( ( Token::new ( TokenKind::Semicolon,        lexeme, ln ), ln) ),
        b'/' => return Some ( ( Token::new ( TokenKind::Slash,            lexeme, ln ), ln) ),
        b'*' => return Some ( ( Token::new ( TokenKind::Star,             lexeme, ln ), ln) ),
        _    => ()
    }

    // One-character or two-character tokens

    let lexeme = &characters[cursor..=cursor + 1];

    if cursor < characters.len() - 1 {
        match (
            characters.as_bytes()[cursor],
            characters.as_bytes()[cursor + 1],
        ) {
            (b'!', b'=') => return Some ( ( Token::new ( TokenKind::BangEqual,    lexeme, ln ), ln ) ),
            (b'=', b'=') => return Some ( ( Token::new ( TokenKind::EqualEqual,   lexeme, ln ), ln ) ),
            (b'>', b'=') => return Some ( ( Token::new ( TokenKind::GreaterEqual, lexeme, ln ), ln ) ),
            (b'<', b'=') => return Some ( ( Token::new ( TokenKind::LessEqual,    lexeme, ln ), ln ) ),
            _            => ()
        }
    }

    let lexeme = &characters[cursor..=cursor];

    match characters.as_bytes()[cursor] {
        b'!' => return Some ( ( Token::new ( TokenKind::Bang,    lexeme, ln ), ln ) ),
        b'=' => return Some ( ( Token::new ( TokenKind::Equal,   lexeme, ln ), ln ) ),
        b'>' => return Some ( ( Token::new ( TokenKind::Greater, lexeme, ln ), ln ) ),
        b'<' => return Some ( ( Token::new ( TokenKind::Less,    lexeme, ln ), ln ) ),
        _    => ()
    }

    None
}

fn match_keyword_or_identifier(characters: &str, cursor_start: usize, line_number: usize) -> Option<(Token<'_>, usize)> {

    let ln = line_number;

    let mut cursor_end = cursor_start;

    if characters.as_bytes()[cursor_end].is_ascii_alphabetic() || characters.as_bytes()[cursor_end] == b'_' {
        cursor_end += 1;

        while cursor_end < characters.len() {
            if characters.as_bytes()[cursor_end].is_ascii_alphanumeric()
                || characters.as_bytes()[cursor_end] == b'_'
            {
                cursor_end += 1;
            } else {
                break;
            }
        }

        let lexeme = &characters[cursor_start..cursor_end];

        match lexeme {
            "and"    => Some ( ( Token::new ( TokenKind::And,        lexeme, ln ), ln ) ),
            "class"  => Some ( ( Token::new ( TokenKind::Class,      lexeme, ln ), ln ) ),
            "else"   => Some ( ( Token::new ( TokenKind::Else,       lexeme, ln ), ln ) ),
            "false"  => Some ( ( Token::new ( TokenKind::False,      lexeme, ln ), ln ) ),
            "fun"    => Some ( ( Token::new ( TokenKind::Fun,        lexeme, ln ), ln ) ),
            "for"    => Some ( ( Token::new ( TokenKind::For,        lexeme, ln ), ln ) ),
            "if"     => Some ( ( Token::new ( TokenKind::If,         lexeme, ln ), ln ) ),
            "nil"    => Some ( ( Token::new ( TokenKind::Nil,        lexeme, ln ), ln ) ),
            "or"     => Some ( ( Token::new ( TokenKind::Or,         lexeme, ln ), ln ) ),
            "print"  => Some ( ( Token::new ( TokenKind::Print,      lexeme, ln ), ln ) ),
            "return" => Some ( ( Token::new ( TokenKind::Return,     lexeme, ln ), ln ) ),
            "super"  => Some ( ( Token::new ( TokenKind::Super,      lexeme, ln ), ln ) ),
            "this"   => Some ( ( Token::new ( TokenKind::This,       lexeme, ln ), ln ) ),
            "true"   => Some ( ( Token::new ( TokenKind::True,       lexeme, ln ), ln ) ),
            "var"    => Some ( ( Token::new ( TokenKind::Var,        lexeme, ln ), ln ) ),
            "while"  => Some ( ( Token::new ( TokenKind::While,      lexeme, ln ), ln ) ),
            _        => Some ( ( Token::new ( TokenKind::Identifier, lexeme, ln ), ln ) ),
        }
    } else {
        None
    }
}

fn match_string_literal(characters: &str, cursor_start: usize, line_number: usize) -> Option<(Token<'_>, usize)> {
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
                    eprintln!("[line {line_number}] Error: Unterminated string.");
                    process::exit(1);
                }
                cursor_end += 1;
            } else if characters.as_bytes()[cursor_end] == b'"' {
                cursor_end += 1;
                return Some((
                    Token {
                        kind: TokenKind::String,
                        lexeme: &characters[cursor_start..cursor_end],
                        line_number,
                    },
                    line_number,
                ));
            }
        }
    }

    None
}

fn match_number_literal(characters: &str, cursor_start: usize, line_number: usize) -> Option<(Token<'_>, usize)> {
    let mut cursor_end = cursor_start;

    if characters.as_bytes()[cursor_end].is_ascii_digit() {
        let mut seen_dot = false;

        cursor_end += 1;

        while cursor_end < characters.len() {
            if characters.as_bytes()[cursor_end] == b'.' {
                if seen_dot {
                    return None;
                }
                seen_dot = true;
                cursor_end += 1;
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

    if characters[cursor_start..cursor_end].is_empty() {
        None
    }
    else if characters.as_bytes()[cursor_end - 1] != b'.' {
        Some((
            Token {
                kind: TokenKind::Number,
                lexeme: &characters[cursor_start..cursor_end],
                line_number,
            },
            line_number,
        ))
    }
    else {
        Some((
            Token {
                kind: TokenKind::Number,
                lexeme: &characters[cursor_start..cursor_end - 1],
                line_number,
            },
            line_number,
        ))
    }
}
