//! Lexer for Neverness to Everness (NTE) domain-specific language.

use super::ast::{SourceLocation, TimeUnit};
use std::fmt;

#[derive(Clone, Debug, PartialEq)]
pub enum TokenKind {
    Ident(String),
    Number {
        decimal: String,
        unit: Option<TimeUnit>,
        raw: String,
    },
    StringLit(String),
    BoolLit(bool),
    // Punctuation & operators
    EqEq,
    NotEq,
    LtEq,
    GtEq,
    AndAnd,
    OrOr,
    Eq,
    Plus,
    Minus,
    Star,
    Slash,
    Percent,
    Exclamation,
    Lt,
    Gt,
    Semicolon,
    Colon,
    Comma,
    Dot,
    LParen,
    RParen,
    LBrace,
    RBrace,
    LBracket,
    RBracket,
    Eof,
}

impl fmt::Display for TokenKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Ident(s) => write!(f, "identifier '{s}'"),
            Self::Number { raw, .. } => write!(f, "number '{raw}'"),
            Self::StringLit(s) => write!(f, "string \"{s}\""),
            Self::BoolLit(b) => write!(f, "boolean '{b}'"),
            Self::EqEq => write!(f, "'=='"),
            Self::NotEq => write!(f, "'!='"),
            Self::LtEq => write!(f, "'<='"),
            Self::GtEq => write!(f, "'>='"),
            Self::AndAnd => write!(f, "'&&'"),
            Self::OrOr => write!(f, "'||'"),
            Self::Eq => write!(f, "'='"),
            Self::Plus => write!(f, "'+'"),
            Self::Minus => write!(f, "'-'"),
            Self::Star => write!(f, "'*'"),
            Self::Slash => write!(f, "'/'"),
            Self::Percent => write!(f, "'%'"),
            Self::Exclamation => write!(f, "'!'"),
            Self::Lt => write!(f, "'<'"),
            Self::Gt => write!(f, "'>'"),
            Self::Semicolon => write!(f, "';'"),
            Self::Colon => write!(f, "':'"),
            Self::Comma => write!(f, "','"),
            Self::Dot => write!(f, "'.'"),
            Self::LParen => write!(f, "'('"),
            Self::RParen => write!(f, "')'"),
            Self::LBrace => write!(f, "'{{'"),
            Self::RBrace => write!(f, "'}}'"),
            Self::LBracket => write!(f, "'['"),
            Self::RBracket => write!(f, "']'"),
            Self::Eof => write!(f, "end of input"),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Token {
    pub kind: TokenKind,
    pub loc: SourceLocation,
}

#[derive(Clone, Debug, PartialEq)]
pub struct LexerError {
    pub message: String,
    pub loc: SourceLocation,
}

impl fmt::Display for LexerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "at line {}, column {}: {}",
            self.loc.line, self.loc.column, self.message
        )
    }
}

impl std::error::Error for LexerError {}

pub struct Lexer<'a> {
    chars: Vec<char>,
    len: usize,
    pos: usize,
    line: usize,
    column: usize,
    offset: usize,
    _marker: std::marker::PhantomData<&'a str>,
}

impl<'a> Lexer<'a> {
    pub fn new(source: &'a str) -> Self {
        let chars: Vec<char> = source.chars().collect();
        let len = chars.len();
        Self {
            chars,
            len,
            pos: 0,
            line: 1,
            column: 1,
            offset: 0,
            _marker: std::marker::PhantomData,
        }
    }

    fn current_loc(&self) -> SourceLocation {
        SourceLocation {
            line: self.line,
            column: self.column,
            offset: self.offset,
            length: self.peek().map_or(0, char::len_utf16),
        }
    }

    fn peek(&self) -> Option<char> {
        if self.pos < self.len {
            Some(self.chars[self.pos])
        } else {
            None
        }
    }

    fn peek_next(&self) -> Option<char> {
        if self.pos + 1 < self.len {
            Some(self.chars[self.pos + 1])
        } else {
            None
        }
    }

    fn advance(&mut self) -> Option<char> {
        if self.pos < self.len {
            let ch = self.chars[self.pos];
            self.pos += 1;
            self.offset += ch.len_utf16();
            if ch == '\n' {
                self.line += 1;
                self.column = 1;
            } else {
                self.column += ch.len_utf16();
            }
            Some(ch)
        } else {
            None
        }
    }

    fn skip_whitespace_and_comments(&mut self) {
        while let Some(ch) = self.peek() {
            if ch.is_whitespace() {
                self.advance();
            } else if ch == '#' {
                // Line comment
                while let Some(c) = self.peek() {
                    self.advance();
                    if c == '\n' {
                        break;
                    }
                }
            } else if ch == '/' && self.peek_next() == Some('/') {
                // Line comment
                self.advance();
                self.advance();
                while let Some(c) = self.peek() {
                    self.advance();
                    if c == '\n' {
                        break;
                    }
                }
            } else {
                break;
            }
        }
    }

    fn lex_number(&mut self) -> Result<Token, LexerError> {
        let loc = self.current_loc();
        let mut raw = String::new();
        while let Some(ch) = self.peek() {
            if ch.is_ascii_digit() {
                raw.push(ch);
                self.advance();
            } else {
                break;
            }
        }

        if self.peek() == Some('.') && self.peek_next().is_some_and(|c| c.is_ascii_digit()) {
            raw.push('.');
            self.advance();
            while self.peek().is_some_and(|c| c.is_ascii_digit()) {
                raw.push(self.advance().expect("digit exists"));
            }
        }
        if raw.bytes().filter(u8::is_ascii_digit).count() > 1_000 {
            return Err(LexerError {
                message: "numeric literals are limited to 1000 digits".to_string(),
                loc,
            });
        }

        // Check for time unit (s, ms, us)
        let unit = if self.peek() == Some('u') && self.peek_next() == Some('s') {
            self.advance();
            self.advance();
            raw.push_str("us");
            Some(TimeUnit::Microseconds)
        } else if self.peek() == Some('m') && self.peek_next() == Some('s') {
            self.advance();
            self.advance();
            raw.push_str("ms");
            Some(TimeUnit::Milliseconds)
        } else if self.peek() == Some('s') {
            let next = self.peek_next();
            if next.is_none_or(|c| !c.is_alphanumeric() && c != '_') {
                self.advance();
                raw.push('s');
                Some(TimeUnit::Seconds)
            } else {
                None
            }
        } else {
            None
        };

        let num_str = match unit {
            Some(TimeUnit::Microseconds) => &raw[..raw.len() - 2],
            Some(TimeUnit::Milliseconds) => &raw[..raw.len() - 2],
            Some(TimeUnit::Seconds) => &raw[..raw.len() - 1],
            None => &raw[..],
        };

        let decimal = num_str.to_string();
        if self
            .peek()
            .is_some_and(|c| c.is_ascii_alphanumeric() || c == '_' || c == '.')
        {
            return Err(LexerError {
                message: format!("invalid numeric literal starting with '{raw}'"),
                loc,
            });
        }

        Ok(Token {
            kind: TokenKind::Number { decimal, unit, raw },
            loc,
        })
    }

    fn lex_ident_or_keyword(&mut self) -> Token {
        let loc = self.current_loc();
        let mut s = String::new();
        while let Some(ch) = self.peek() {
            if ch.is_ascii_alphanumeric() || ch == '_' {
                s.push(ch);
                self.advance();
            } else {
                break;
            }
        }

        let kind = match s.as_str() {
            "true" => TokenKind::BoolLit(true),
            "false" => TokenKind::BoolLit(false),
            _ => TokenKind::Ident(s),
        };

        Token { kind, loc }
    }

    fn lex_string(&mut self, quote: char) -> Result<Token, LexerError> {
        let loc = self.current_loc();
        self.advance(); // consume opening quote
        let mut s = String::new();
        while let Some(ch) = self.peek() {
            if ch == quote {
                self.advance();
                return Ok(Token {
                    kind: TokenKind::StringLit(s),
                    loc,
                });
            } else if ch == '\\' {
                self.advance();
                match self.advance() {
                    Some('n') => s.push('\n'),
                    Some('r') => s.push('\r'),
                    Some('t') => s.push('\t'),
                    Some('\\') => s.push('\\'),
                    Some('\'') => s.push('\''),
                    Some('"') => s.push('"'),
                    Some(c) => s.push(c),
                    None => {
                        return Err(LexerError {
                            message: "unterminated escape sequence in string".to_string(),
                            loc,
                        });
                    }
                }
            } else {
                s.push(ch);
                self.advance();
            }
        }
        Err(LexerError {
            message: "unterminated string literal".to_string(),
            loc,
        })
    }

    pub fn next_token(&mut self) -> Result<Token, LexerError> {
        self.skip_whitespace_and_comments();
        let start = self.offset;
        let result = self.read_token();
        match result {
            Ok(mut token) => {
                token.loc.length = self.offset - start;
                Ok(token)
            }
            Err(mut error) => {
                error.loc.length = (self.offset - start).max(error.loc.length);
                Err(error)
            }
        }
    }

    fn read_token(&mut self) -> Result<Token, LexerError> {
        let loc = self.current_loc();
        let ch = match self.peek() {
            Some(c) => c,
            None => {
                return Ok(Token {
                    kind: TokenKind::Eof,
                    loc,
                });
            }
        };

        if ch.is_ascii_digit() {
            return self.lex_number();
        }

        if ch.is_ascii_alphabetic() || ch == '_' {
            return Ok(self.lex_ident_or_keyword());
        }

        if ch == '"' || ch == '\'' {
            return self.lex_string(ch);
        }

        self.advance();
        let kind = match ch {
            '=' => {
                if self.peek() == Some('=') {
                    self.advance();
                    TokenKind::EqEq
                } else {
                    TokenKind::Eq
                }
            }
            '!' => {
                if self.peek() == Some('=') {
                    self.advance();
                    TokenKind::NotEq
                } else {
                    TokenKind::Exclamation
                }
            }
            '<' => {
                if self.peek() == Some('=') {
                    self.advance();
                    TokenKind::LtEq
                } else {
                    TokenKind::Lt
                }
            }
            '>' => {
                if self.peek() == Some('=') {
                    self.advance();
                    TokenKind::GtEq
                } else {
                    TokenKind::Gt
                }
            }
            '&' => {
                if self.peek() == Some('&') {
                    self.advance();
                    TokenKind::AndAnd
                } else {
                    return Err(LexerError {
                        message: "expected '&' after '&'".to_string(),
                        loc,
                    });
                }
            }
            '|' => {
                if self.peek() == Some('|') {
                    self.advance();
                    TokenKind::OrOr
                } else {
                    return Err(LexerError {
                        message: "expected '|' after '|'".to_string(),
                        loc,
                    });
                }
            }
            '+' => TokenKind::Plus,
            '-' => TokenKind::Minus,
            '*' => TokenKind::Star,
            '/' => TokenKind::Slash,
            '%' => TokenKind::Percent,
            ';' => TokenKind::Semicolon,
            ':' => TokenKind::Colon,
            ',' => TokenKind::Comma,
            '.' => TokenKind::Dot,
            '(' => TokenKind::LParen,
            ')' => TokenKind::RParen,
            '{' => TokenKind::LBrace,
            '}' => TokenKind::RBrace,
            '[' => TokenKind::LBracket,
            ']' => TokenKind::RBracket,
            other => {
                return Err(LexerError {
                    message: format!("unexpected character '{other}'"),
                    loc,
                });
            }
        };

        Ok(Token { kind, loc })
    }

    pub fn tokenize_all(&mut self) -> Result<Vec<Token>, LexerError> {
        if self.chars.iter().map(|c| c.len_utf16()).sum::<usize>() > 200_000 {
            return Err(LexerError {
                message: "script exceeds 200000 UTF-16 code units".to_string(),
                loc: self.current_loc(),
            });
        }
        let mut tokens = Vec::new();
        loop {
            let tok = self.next_token()?;
            let is_eof = tok.kind == TokenKind::Eof;
            tokens.push(tok);
            if is_eof {
                break;
            }
        }
        Ok(tokens)
    }
}
