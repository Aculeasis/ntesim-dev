use super::error::ParserError;
use crate::dsl::ast::SourceLocation;
use crate::dsl::lexer::{Token, TokenKind};
use std::collections::BTreeSet;

pub struct Parser {
    pub(super) tokens: Vec<Token>,
    pub(super) pos: usize,
    pub(super) nesting: usize,
    pub(super) expression_depth: usize,
    pub(super) expression_nodes: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self {
            tokens,
            pos: 0,
            nesting: 0,
            expression_depth: 0,
            expression_nodes: 0,
        }
    }

    pub(super) fn nested<T>(
        &mut self,
        parse: impl FnOnce(&mut Self) -> Result<T, ParserError>,
    ) -> Result<T, ParserError> {
        if self.nesting >= 64 {
            return Err(ParserError {
                message: "source nesting limit exceeded".into(),
                loc: self.current_loc(),
            });
        }
        self.nesting += 1;
        let result = parse(self);
        self.nesting -= 1;
        result
    }

    pub(super) fn expression_node(&mut self) -> Result<(), ParserError> {
        self.expression_nodes += 1;
        if self.expression_nodes > 256 {
            return Err(ParserError {
                message: "expression size limit exceeded".into(),
                loc: self.current_loc(),
            });
        }
        Ok(())
    }

    pub(super) fn current_token(&self) -> &Token {
        if self.pos < self.tokens.len() {
            &self.tokens[self.pos]
        } else {
            &self.tokens[self.tokens.len() - 1]
        }
    }

    pub(super) fn current_loc(&self) -> SourceLocation {
        self.current_token().loc.clone()
    }

    pub(super) fn peek(&self) -> &TokenKind {
        &self.current_token().kind
    }

    pub(super) fn advance(&mut self) -> Token {
        let tok = self.current_token().clone();
        if self.pos < self.tokens.len() && tok.kind != TokenKind::Eof {
            self.pos += 1;
        }
        tok
    }

    pub(super) fn check(&self, expected: &TokenKind) -> bool {
        self.peek() == expected
    }

    pub(super) fn match_token(&mut self, expected: &TokenKind) -> bool {
        if self.check(expected) {
            self.advance();
            true
        } else {
            false
        }
    }

    pub(super) fn expect(&mut self, expected: &TokenKind) -> Result<Token, ParserError> {
        let current = self.current_token().clone();
        if &current.kind == expected {
            Ok(self.advance())
        } else {
            Err(ParserError {
                message: format!("expected {expected}, found {}", current.kind),
                loc: current.loc,
            })
        }
    }

    pub(super) fn expect_ident(&mut self) -> Result<String, ParserError> {
        let current = self.current_token().clone();
        match current.kind {
            TokenKind::Ident(s) => {
                self.advance();
                Ok(s)
            }
            _ => Err(ParserError {
                message: format!("expected identifier, found {}", current.kind),
                loc: current.loc,
            }),
        }
    }

    pub(super) fn expect_integer(&mut self) -> Result<u32, ParserError> {
        let token = self.advance();
        if let TokenKind::Number {
            decimal,
            unit: None,
            ..
        } = &token.kind
            && decimal.bytes().all(|c| c.is_ascii_digit())
            && let Ok(value) = decimal.parse()
        {
            return Ok(value);
        }
        Err(ParserError {
            message: "expected an unsigned 32-bit integer literal without a time unit".to_string(),
            loc: token.loc,
        })
    }

    pub(super) fn expect_key(&mut self) -> Result<String, ParserError> {
        let mut key = self.expect_ident()?;
        if self.match_token(&TokenKind::Percent) {
            key.push('%');
        }
        Ok(key)
    }

    pub(super) fn unique_key(
        &mut self,
        keys: &mut BTreeSet<String>,
    ) -> Result<String, ParserError> {
        let loc = self.current_loc();
        let key = self.expect_key()?;
        if !keys.insert(key.clone()) {
            return Err(ParserError {
                message: format!("duplicate field '{key}'"),
                loc,
            });
        }
        Ok(key)
    }
}
