use super::cursor::Parser;
use super::error::ParserError;
use crate::dsl::ast::*;
use crate::dsl::lexer::TokenKind;

impl Parser {
    pub fn parse_expression(&mut self) -> Result<Expression, ParserError> {
        if self.expression_depth == 0 {
            self.expression_nodes = 0;
        }
        self.expression_depth += 1;
        let result = self.nested(Self::parse_logical_or);
        self.expression_depth -= 1;
        result
    }

    fn parse_logical_or(&mut self) -> Result<Expression, ParserError> {
        let mut left = self.parse_logical_and()?;
        while self.peek() == &TokenKind::OrOr {
            self.expression_node()?;
            let loc = self.advance().loc;
            let right = self.parse_logical_and()?;
            left = Expression {
                kind: ExpressionKind::Binary {
                    op: "||".to_string(),
                    left: Box::new(left),
                    right: Box::new(right),
                },
                loc,
            };
        }
        Ok(left)
    }

    fn parse_logical_and(&mut self) -> Result<Expression, ParserError> {
        let mut left = self.parse_equality()?;
        while self.peek() == &TokenKind::AndAnd {
            self.expression_node()?;
            let loc = self.advance().loc;
            let right = self.parse_equality()?;
            left = Expression {
                kind: ExpressionKind::Binary {
                    op: "&&".to_string(),
                    left: Box::new(left),
                    right: Box::new(right),
                },
                loc,
            };
        }
        Ok(left)
    }

    fn parse_equality(&mut self) -> Result<Expression, ParserError> {
        let mut left = self.parse_comparison()?;
        while let TokenKind::EqEq | TokenKind::NotEq = self.peek() {
            self.expression_node()?;
            let op_tok = self.advance();
            let op = if op_tok.kind == TokenKind::EqEq {
                "=="
            } else {
                "!="
            };
            let right = self.parse_comparison()?;
            left = Expression {
                kind: ExpressionKind::Binary {
                    op: op.to_string(),
                    left: Box::new(left),
                    right: Box::new(right),
                },
                loc: op_tok.loc,
            };
        }
        Ok(left)
    }

    fn parse_comparison(&mut self) -> Result<Expression, ParserError> {
        let mut left = self.parse_sum()?;
        while let TokenKind::Lt | TokenKind::LtEq | TokenKind::Gt | TokenKind::GtEq = self.peek() {
            self.expression_node()?;
            let op_tok = self.advance();
            let op = match op_tok.kind {
                TokenKind::Lt => "<",
                TokenKind::LtEq => "<=",
                TokenKind::Gt => ">",
                TokenKind::GtEq => ">=",
                _ => unreachable!(),
            };
            let right = self.parse_sum()?;
            left = Expression {
                kind: ExpressionKind::Binary {
                    op: op.to_string(),
                    left: Box::new(left),
                    right: Box::new(right),
                },
                loc: op_tok.loc,
            };
        }
        Ok(left)
    }

    fn parse_sum(&mut self) -> Result<Expression, ParserError> {
        let mut left = self.parse_product()?;
        while let TokenKind::Plus | TokenKind::Minus = self.peek() {
            // +params starts a character declaration suffix, not arithmetic.
            if self.peek() == &TokenKind::Plus
                && matches!(self.tokens.get(self.pos + 1).map(|t| &t.kind), Some(TokenKind::Ident(name)) if name == "params")
                && self
                    .tokens
                    .get(self.pos + 2)
                    .is_some_and(|t| t.kind == TokenKind::Eq)
                && self
                    .tokens
                    .get(self.pos + 3)
                    .is_some_and(|t| t.kind == TokenKind::LBracket)
            {
                break;
            }
            self.expression_node()?;
            let op_tok = self.advance();
            let op = if op_tok.kind == TokenKind::Plus {
                "+"
            } else {
                "-"
            };
            let right = self.parse_product()?;
            left = Expression {
                kind: ExpressionKind::Binary {
                    op: op.to_string(),
                    left: Box::new(left),
                    right: Box::new(right),
                },
                loc: op_tok.loc,
            };
        }
        Ok(left)
    }

    fn parse_product(&mut self) -> Result<Expression, ParserError> {
        let mut left = self.parse_unary()?;
        while let TokenKind::Star | TokenKind::Slash | TokenKind::Percent = self.peek() {
            self.expression_node()?;
            let op_tok = self.advance();
            let op = match op_tok.kind {
                TokenKind::Star => "*",
                TokenKind::Slash => "/",
                TokenKind::Percent => "%",
                _ => unreachable!(),
            };
            let right = self.parse_unary()?;
            left = Expression {
                kind: ExpressionKind::Binary {
                    op: op.to_string(),
                    left: Box::new(left),
                    right: Box::new(right),
                },
                loc: op_tok.loc,
            };
        }
        Ok(left)
    }

    fn parse_unary(&mut self) -> Result<Expression, ParserError> {
        self.nested(Self::parse_unary_inner)
    }

    fn parse_unary_inner(&mut self) -> Result<Expression, ParserError> {
        if let TokenKind::Plus | TokenKind::Minus | TokenKind::Exclamation = self.peek() {
            let op_tok = self.advance();
            let op = match op_tok.kind {
                TokenKind::Plus => "+",
                TokenKind::Minus => "-",
                TokenKind::Exclamation => "!",
                _ => unreachable!(),
            };
            let expr = self.parse_unary()?;
            return Ok(Expression {
                kind: ExpressionKind::Unary {
                    op: op.to_string(),
                    expr: Box::new(expr),
                },
                loc: op_tok.loc,
            });
        }
        self.parse_primary()
    }

    fn parse_primary(&mut self) -> Result<Expression, ParserError> {
        self.expression_node()?;
        let tok = self.advance();
        match tok.kind {
            TokenKind::Number { decimal, unit, .. } => Ok(Expression {
                kind: ExpressionKind::Number { decimal, unit },
                loc: tok.loc,
            }),
            TokenKind::StringLit(s) => Ok(Expression {
                kind: ExpressionKind::StringLiteral(s),
                loc: tok.loc,
            }),
            TokenKind::BoolLit(b) => Ok(Expression {
                kind: ExpressionKind::Boolean(b),
                loc: tok.loc,
            }),
            TokenKind::Dot => {
                // Field access e.g. .target.hp or .actor.energy
                let mut parts = Vec::new();
                parts.push(self.expect_ident()?);
                while self.match_token(&TokenKind::Dot) {
                    parts.push(self.expect_ident()?);
                }
                Ok(Expression {
                    kind: ExpressionKind::FieldAccess(parts),
                    loc: tok.loc,
                })
            }
            TokenKind::Ident(name) => {
                // Check if call: e.g. wait(0.5)
                if self.match_token(&TokenKind::LParen) {
                    let mut args = Vec::new();
                    while self.peek() != &TokenKind::RParen && self.peek() != &TokenKind::Eof {
                        args.push(self.parse_expression()?);
                        if !self.match_token(&TokenKind::Comma) {
                            break;
                        }
                    }
                    self.expect(&TokenKind::RParen)?;
                    Ok(Expression {
                        kind: ExpressionKind::Call { callee: name, args },
                        loc: tok.loc,
                    })
                } else {
                    Ok(Expression {
                        kind: ExpressionKind::Identifier(name),
                        loc: tok.loc,
                    })
                }
            }
            TokenKind::LParen => {
                let expr = self.parse_expression()?;
                self.expect(&TokenKind::RParen)?;
                Ok(expr)
            }
            _ => Err(ParserError {
                message: format!("unexpected token in expression: {}", tok.kind),
                loc: tok.loc,
            }),
        }
    }
}
