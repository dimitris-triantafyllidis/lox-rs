use std::fmt;
use std::process;

#[derive(Debug, PartialEq, Clone, Hash, Eq)]
pub enum TokenKind {
    // One-character tokens
    LeftParenthesis,
    RightParenthesis,
    LeftBrace,
    RightBrace,
    Semicolon,
    Comma,
    Dot,
    Minus,
    Plus,
    Slash,
    Star,

    // One-character or two-character tokens
    Bang,
    BangEqual,
    Equal,
    EqualEqual,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,

    // Literals
    Identifier,
    String,
    Number,

    // Keywords
    And,
    Class,
    Else,
    False,
    Fun,
    For,
    If,
    Nil,
    Or,
    Print,
    Return,
    Super,
    This,
    True,
    Var,
    While,

    //
    EOF,
}

#[derive(Debug, PartialEq, Clone, Hash, Eq)]
pub struct Token<'a> {
    pub kind: TokenKind,
    pub lexeme: &'a str,
    pub line_number: usize,
}

impl fmt::Display for Token<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.kind {
            TokenKind::LeftParenthesis => write!(f, "LEFT_PAREN {} null", self.lexeme),
            TokenKind::RightParenthesis => write!(f, "RIGHT_PAREN {} null", self.lexeme),
            TokenKind::LeftBrace => write!(f, "LEFT_BRACE {} null", self.lexeme),
            TokenKind::RightBrace => write!(f, "RIGHT_BRACE {} null", self.lexeme),
            TokenKind::Semicolon => write!(f, "SEMICOLON {} null", self.lexeme),
            TokenKind::Comma => write!(f, "COMMA {} null", self.lexeme),
            TokenKind::Dot => write!(f, "DOT {} null", self.lexeme),
            TokenKind::Minus => write!(f, "MINUS {} null", self.lexeme),
            TokenKind::Plus => write!(f, "PLUS {} null", self.lexeme),
            TokenKind::Slash => write!(f, "SLASH {} null", self.lexeme),
            TokenKind::Star => write!(f, "STAR {} null", self.lexeme),
            TokenKind::Bang => write!(f, "BANG {} null", self.lexeme),
            TokenKind::BangEqual => write!(f, "BANG_EQUAL {} null", self.lexeme),
            TokenKind::Equal => write!(f, "EQUAL {} null", self.lexeme),
            TokenKind::EqualEqual => write!(f, "EQUAL_EQUAL {} null", self.lexeme),
            TokenKind::Greater => write!(f, "GREATER {} null", self.lexeme),
            TokenKind::GreaterEqual => write!(f, "GREATER_EQUAL {} null", self.lexeme),
            TokenKind::Less => write!(f, "LESS {} null", self.lexeme),
            TokenKind::LessEqual => write!(f, "LESS_EQUAL {} null", self.lexeme),
            TokenKind::Identifier => write!(f, "IDENTIFIER {} null", self.lexeme),
            TokenKind::String => {
                write!(
                    f,
                    "STRING {} {}",
                    self.lexeme,
                    &self.lexeme[1..self.lexeme.len() - 1]
                )
            }
            TokenKind::And => write!(f, "AND {} null", self.lexeme),
            TokenKind::Class => write!(f, "CLASS {} null", self.lexeme),
            TokenKind::Else => write!(f, "ELSE {} null", self.lexeme),
            TokenKind::False => write!(f, "FALSE {} null", self.lexeme),
            TokenKind::Fun => write!(f, "FUN {} null", self.lexeme),
            TokenKind::For => write!(f, "FOR {} null", self.lexeme),
            TokenKind::If => write!(f, "IF {} null", self.lexeme),
            TokenKind::Nil => write!(f, "NIL {} null", self.lexeme),
            TokenKind::Or => write!(f, "OR {} null", self.lexeme),
            TokenKind::Print => write!(f, "PRINT {} null", self.lexeme),
            TokenKind::Return => write!(f, "RETURN {} null", self.lexeme),
            TokenKind::Super => write!(f, "SUPER {} null", self.lexeme),
            TokenKind::This => write!(f, "THIS {} null", self.lexeme),
            TokenKind::True => write!(f, "TRUE {} null", self.lexeme),
            TokenKind::Var => write!(f, "VAR {} null", self.lexeme),
            TokenKind::While => write!(f, "WHILE {} null", self.lexeme),
            TokenKind::EOF => write!(f, "EOF  null"),
            TokenKind::Number => {
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

        if source_length - cursor >= 2
            && (
                characters.as_bytes()[cursor],
                characters.as_bytes()[cursor + 1],
            ) == (b'/', b'/')
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
            match_one_or_two_character_token(characters, cursor, line_number),
            match_keyword_or_identifier(characters, cursor, line_number),
            match_string_literal(characters, cursor, line_number),
            match_number_literal(characters, cursor, line_number),
        ];

        if let Some((token, new_line_number)) = matches
            .into_iter()
            .flatten()
            .max_by_key(|(token, _)| token.lexeme.len())
        {
            cursor += token.lexeme.len();
            line_number = new_line_number;
            tokens.push(token);
        } else {
            panic!("Lexical error in line {line_number}: Unrecognizable token");
        }
    }

    tokens.push(Token {
        kind: TokenKind::EOF,
        lexeme: "end",
        line_number,
    });

    tokens
}

pub fn print_token_stream(tokens: &[Token]) {
    for token in tokens {
        println!("{token}");
    }
}

fn match_one_or_two_character_token(
    characters: &str,
    cursor: usize,
    line_number: usize,
) -> Option<(Token<'_>, usize)> {
    // One-character tokens

    match characters.as_bytes()[cursor] {
        b'(' => {
            return Some((
                Token {
                    kind: TokenKind::LeftParenthesis,
                    lexeme: &characters[cursor..=cursor],
                    line_number,
                },
                line_number,
            ));
        }
        b')' => {
            return Some((
                Token {
                    kind: TokenKind::RightParenthesis,
                    lexeme: &characters[cursor..=cursor],
                    line_number,
                },
                line_number,
            ));
        }
        b'{' => {
            return Some((
                Token {
                    kind: TokenKind::LeftBrace,
                    lexeme: &characters[cursor..=cursor],
                    line_number,
                },
                line_number,
            ));
        }
        b'}' => {
            return Some((
                Token {
                    kind: TokenKind::RightBrace,
                    lexeme: &characters[cursor..=cursor],
                    line_number,
                },
                line_number,
            ));
        }
        b',' => {
            return Some((
                Token {
                    kind: TokenKind::Comma,
                    lexeme: &characters[cursor..=cursor],
                    line_number,
                },
                line_number,
            ));
        }
        b'.' => {
            return Some((
                Token {
                    kind: TokenKind::Dot,
                    lexeme: &characters[cursor..=cursor],
                    line_number,
                },
                line_number,
            ));
        }
        b'-' => {
            return Some((
                Token {
                    kind: TokenKind::Minus,
                    lexeme: &characters[cursor..=cursor],
                    line_number,
                },
                line_number,
            ));
        }
        b'+' => {
            return Some((
                Token {
                    kind: TokenKind::Plus,
                    lexeme: &characters[cursor..=cursor],
                    line_number,
                },
                line_number,
            ));
        }
        b';' => {
            return Some((
                Token {
                    kind: TokenKind::Semicolon,
                    lexeme: &characters[cursor..=cursor],
                    line_number,
                },
                line_number,
            ));
        }
        b'/' => {
            return Some((
                Token {
                    kind: TokenKind::Slash,
                    lexeme: &characters[cursor..=cursor],
                    line_number,
                },
                line_number,
            ));
        }
        b'*' => {
            return Some((
                Token {
                    kind: TokenKind::Star,
                    lexeme: &characters[cursor..=cursor],
                    line_number,
                },
                line_number,
            ));
        }
        _ => (),
    }

    // One-character or two-character tokens

    if cursor < characters.len() - 1 {
        match (
            characters.as_bytes()[cursor],
            characters.as_bytes()[cursor + 1],
        ) {
            (b'!', b'=') => {
                return Some((
                    Token {
                        kind: TokenKind::BangEqual,
                        lexeme: &characters[cursor..=cursor + 1],
                        line_number,
                    },
                    line_number,
                ));
            }
            (b'=', b'=') => {
                return Some((
                    Token {
                        kind: TokenKind::EqualEqual,
                        lexeme: &characters[cursor..=cursor + 1],
                        line_number,
                    },
                    line_number,
                ));
            }
            (b'>', b'=') => {
                return Some((
                    Token {
                        kind: TokenKind::GreaterEqual,
                        lexeme: &characters[cursor..=cursor + 1],
                        line_number,
                    },
                    line_number,
                ));
            }
            (b'<', b'=') => {
                return Some((
                    Token {
                        kind: TokenKind::LessEqual,
                        lexeme: &characters[cursor..=cursor + 1],
                        line_number,
                    },
                    line_number,
                ));
            }
            _ => (),
        }
    }

    match characters.as_bytes()[cursor] {
        b'!' => {
            return Some((
                Token {
                    kind: TokenKind::Bang,
                    lexeme: &characters[cursor..=cursor],
                    line_number,
                },
                line_number,
            ));
        }
        b'=' => {
            return Some((
                Token {
                    kind: TokenKind::Equal,
                    lexeme: &characters[cursor..=cursor],
                    line_number,
                },
                line_number,
            ));
        }
        b'>' => {
            return Some((
                Token {
                    kind: TokenKind::Greater,
                    lexeme: &characters[cursor..=cursor],
                    line_number,
                },
                line_number,
            ));
        }
        b'<' => {
            return Some((
                Token {
                    kind: TokenKind::Less,
                    lexeme: &characters[cursor..=cursor],
                    line_number,
                },
                line_number,
            ));
        }
        _ => (),
    }

    None
}

fn match_keyword_or_identifier(
    characters: &str,
    cursor_start: usize,
    line_number: usize,
) -> Option<(Token<'_>, usize)> {
    let mut cursor_end = cursor_start;

    if characters.as_bytes()[cursor_end].is_ascii_alphabetic()
        || characters.as_bytes()[cursor_end] == b'_'
    {
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
            "and" => Some((
                Token {
                    kind: TokenKind::And,
                    lexeme,
                    line_number,
                },
                line_number,
            )),
            "class" => Some((
                Token {
                    kind: TokenKind::Class,
                    lexeme,
                    line_number,
                },
                line_number,
            )),
            "else" => Some((
                Token {
                    kind: TokenKind::Else,
                    lexeme,
                    line_number,
                },
                line_number,
            )),
            "false" => Some((
                Token {
                    kind: TokenKind::False,
                    lexeme,
                    line_number,
                },
                line_number,
            )),
            "fun" => Some((
                Token {
                    kind: TokenKind::Fun,
                    lexeme,
                    line_number,
                },
                line_number,
            )),
            "for" => Some((
                Token {
                    kind: TokenKind::For,
                    lexeme,
                    line_number,
                },
                line_number,
            )),
            "if" => Some((
                Token {
                    kind: TokenKind::If,
                    lexeme,
                    line_number,
                },
                line_number,
            )),
            "nil" => Some((
                Token {
                    kind: TokenKind::Nil,
                    lexeme,
                    line_number,
                },
                line_number,
            )),
            "or" => Some((
                Token {
                    kind: TokenKind::Or,
                    lexeme,
                    line_number,
                },
                line_number,
            )),
            "print" => Some((
                Token {
                    kind: TokenKind::Print,
                    lexeme,
                    line_number,
                },
                line_number,
            )),
            "return" => Some((
                Token {
                    kind: TokenKind::Return,
                    lexeme,
                    line_number,
                },
                line_number,
            )),
            "super" => Some((
                Token {
                    kind: TokenKind::Super,
                    lexeme,
                    line_number,
                },
                line_number,
            )),
            "this" => Some((
                Token {
                    kind: TokenKind::This,
                    lexeme,
                    line_number,
                },
                line_number,
            )),
            "true" => Some((
                Token {
                    kind: TokenKind::True,
                    lexeme,
                    line_number,
                },
                line_number,
            )),
            "var" => Some((
                Token {
                    kind: TokenKind::Var,
                    lexeme,
                    line_number,
                },
                line_number,
            )),
            "while" => Some((
                Token {
                    kind: TokenKind::While,
                    lexeme,
                    line_number,
                },
                line_number,
            )),
            _ => Some((
                Token {
                    kind: TokenKind::Identifier,
                    lexeme,
                    line_number,
                },
                line_number,
            )),
        }
    } else {
        None
    }
}

fn match_string_literal(
    characters: &str,
    cursor_start: usize,
    line_number: usize,
) -> Option<(Token<'_>, usize)> {
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

fn match_number_literal(
    characters: &str,
    cursor_start: usize,
    line_number: usize,
) -> Option<(Token<'_>, usize)> {
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
            } else if characters.as_bytes()[cursor_end].is_ascii_digit() {
                cursor_end += 1;
            } else if !characters.as_bytes()[cursor_end].is_ascii_digit() {
                break;
            } else {
                return None;
            }
        }
    }

    if characters[cursor_start..cursor_end].is_empty() {
        None
    } else if characters.as_bytes()[cursor_end - 1] != b'.' {
        Some((
            Token {
                kind: TokenKind::Number,
                lexeme: &characters[cursor_start..cursor_end],
                line_number,
            },
            line_number,
        ))
    } else {
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
