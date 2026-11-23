use super::cursor::Parser;
use super::error::ParserError;
use crate::dsl::ast::*;
use crate::dsl::lexer::TokenKind;
use std::collections::BTreeSet;

impl Parser {
    pub fn parse_statement(&mut self) -> Result<Statement, ParserError> {
        let loc = self.current_loc();
        match self.peek() {
            TokenKind::Ident(name) => {
                let name = name.clone();
                match name.as_str() {
                    "target" => {
                        self.advance();
                        self.parse_target_gauge(loc)
                    }
                    "let" => {
                        self.advance();
                        self.parse_let_or_const(false, loc)
                    }
                    "const" => {
                        self.advance();
                        self.parse_let_or_const(true, loc)
                    }
                    "if" => {
                        self.advance();
                        self.parse_if(loc)
                    }
                    "repeat" => {
                        self.advance();
                        self.parse_repeat(loc)
                    }
                    "while" => {
                        self.advance();
                        self.parse_while(loc)
                    }
                    "for" => {
                        self.advance();
                        self.parse_for(loc)
                    }
                    "break" => {
                        self.advance();
                        self.expect(&TokenKind::Semicolon)?;
                        Ok(Statement {
                            kind: StatementKind::Break,
                            loc,
                        })
                    }
                    "continue" => {
                        self.advance();
                        self.expect(&TokenKind::Semicolon)?;
                        Ok(Statement {
                            kind: StatementKind::Continue,
                            loc,
                        })
                    }
                    _ => {
                        // Could be action_list (e.g. mint attack:5, skill;), assignment, or expression
                        if let Some(next_tok) = self.tokens.get(self.pos + 1) {
                            match &next_tok.kind {
                                TokenKind::Eq => {
                                    self.advance();
                                    self.advance(); // consume '='
                                    let value = self.parse_expression()?;
                                    self.expect(&TokenKind::Semicolon)?;
                                    return Ok(Statement {
                                        kind: StatementKind::Assignment {
                                            target: name,
                                            value,
                                        },
                                        loc,
                                    });
                                }
                                TokenKind::Ident(_) => {
                                    self.advance(); // consume actor name
                                    return self.parse_action_list(name, loc);
                                }
                                _ => {}
                            }
                        }
                        let expr = self.parse_expression()?;
                        self.expect(&TokenKind::Semicolon)?;
                        Ok(Statement {
                            kind: StatementKind::Expression(expr),
                            loc,
                        })
                    }
                }
            }
            _ => {
                let expr = self.parse_expression()?;
                self.expect(&TokenKind::Semicolon)?;
                Ok(Statement {
                    kind: StatementKind::Expression(expr),
                    loc,
                })
            }
        }
    }

    fn parse_target_gauge(&mut self, loc: SourceLocation) -> Result<Statement, ParserError> {
        let target_loc = self.current_loc();
        let target = Expression {
            kind: ExpressionKind::Identifier(self.expect_ident()?),
            loc: target_loc,
        };
        let operation_loc = self.current_loc();
        let gauge = match self.expect_ident()?.as_str() {
            "break" => None,
            "break_gauge" => {
                self.expect(&TokenKind::Eq)?;
                Some(self.parse_expression()?)
            }
            _ => {
                return Err(ParserError {
                    message: "Target input requires break or break_gauge.".into(),
                    loc: operation_loc,
                });
            }
        };
        self.expect(&TokenKind::Semicolon)?;
        Ok(Statement {
            kind: StatementKind::TargetGauge { target, gauge },
            loc,
        })
    }

    fn parse_action_list(
        &mut self,
        actor: String,
        loc: SourceLocation,
    ) -> Result<Statement, ParserError> {
        let mut actions = Vec::new();
        loop {
            let act_loc = self.current_loc();
            let action = self.expect_ident()?;
            let mut params = Vec::new();
            let mut keys = BTreeSet::new();
            if self.match_token(&TokenKind::LBracket) {
                while self.peek() != &TokenKind::RBracket && self.peek() != &TokenKind::Eof {
                    let k = self.unique_key(&mut keys)?;
                    self.expect(&TokenKind::Eq)?;
                    let v = self.parse_expression()?;
                    params.push((k, v));
                    if !self.match_token(&TokenKind::Comma) {
                        break;
                    }
                }
                self.expect(&TokenKind::RBracket)?;
            }
            let count = if self.match_token(&TokenKind::Colon) {
                let loc = self.current_loc();
                let count = self.expect_integer()?;
                if count == 0 {
                    return Err(ParserError {
                        message: "action repeat count must be positive".to_string(),
                        loc,
                    });
                }
                Some(Expression {
                    kind: ExpressionKind::Number {
                        decimal: count.to_string(),
                        unit: None,
                    },
                    loc,
                })
            } else {
                None
            };
            actions.push(ActionInvocation {
                action,
                params,
                count,
                loc: act_loc,
            });
            if !self.match_token(&TokenKind::Comma) {
                break;
            }
        }
        self.expect(&TokenKind::Semicolon)?;
        Ok(Statement {
            kind: StatementKind::ActionList { actor, actions },
            loc,
        })
    }

    fn parse_let_or_const(
        &mut self,
        is_const: bool,
        loc: SourceLocation,
    ) -> Result<Statement, ParserError> {
        let name = self.expect_ident()?;
        self.expect(&TokenKind::Eq)?;
        let value = self.parse_expression()?;
        self.expect(&TokenKind::Semicolon)?;
        Ok(Statement {
            kind: StatementKind::Variable {
                name,
                is_const,
                value,
            },
            loc,
        })
    }

    fn parse_block(&mut self) -> Result<Vec<Statement>, ParserError> {
        self.nested(Self::parse_block_inner)
    }

    fn parse_block_inner(&mut self) -> Result<Vec<Statement>, ParserError> {
        self.expect(&TokenKind::LBrace)?;
        let mut statements = Vec::new();
        while self.peek() != &TokenKind::RBrace && self.peek() != &TokenKind::Eof {
            statements.push(self.parse_statement()?);
        }
        self.expect(&TokenKind::RBrace)?;
        Ok(statements)
    }

    fn parse_if(&mut self, loc: SourceLocation) -> Result<Statement, ParserError> {
        self.nested(|parser| parser.parse_if_inner(loc))
    }

    fn parse_if_inner(&mut self, loc: SourceLocation) -> Result<Statement, ParserError> {
        let condition = self.parse_expression()?;
        let then_branch = self.parse_block()?;
        let else_branch = if let TokenKind::Ident(name) = self.peek() {
            if name == "else" {
                self.advance();
                if let TokenKind::Ident(next) = self.peek() {
                    if next == "if" {
                        let if_loc = self.current_loc();
                        self.advance();
                        Some(vec![self.parse_if(if_loc)?])
                    } else {
                        Some(self.parse_block()?)
                    }
                } else {
                    Some(self.parse_block()?)
                }
            } else {
                None
            }
        } else {
            None
        };
        Ok(Statement {
            kind: StatementKind::If {
                condition,
                then_branch,
                else_branch,
            },
            loc,
        })
    }

    fn parse_repeat(&mut self, loc: SourceLocation) -> Result<Statement, ParserError> {
        let count = self.parse_expression()?;
        let body = self.parse_block()?;
        Ok(Statement {
            kind: StatementKind::Repeat { count, body },
            loc,
        })
    }

    fn parse_while(&mut self, loc: SourceLocation) -> Result<Statement, ParserError> {
        let condition = self.parse_expression()?;
        let body = self.parse_block()?;
        Ok(Statement {
            kind: StatementKind::While { condition, body },
            loc,
        })
    }

    fn parse_for(&mut self, loc: SourceLocation) -> Result<Statement, ParserError> {
        let let_kw = self.expect_ident()?;
        if let_kw != "let" {
            return Err(ParserError {
                message: format!("expected 'let' in for loop, got '{let_kw}'"),
                loc: self.current_loc(),
            });
        }
        let var_name = self.expect_ident()?;
        self.expect(&TokenKind::Eq)?;
        let init = self.parse_expression()?;
        self.expect(&TokenKind::Semicolon)?;
        let condition = self.parse_expression()?;
        self.expect(&TokenKind::Semicolon)?;
        let update_var = self.expect_ident()?;
        self.expect(&TokenKind::Eq)?;
        let update_expr = self.parse_expression()?;
        let body = self.parse_block()?;
        Ok(Statement {
            kind: StatementKind::For(Box::new(ForStatement {
                var_name,
                init,
                condition,
                update_var,
                update_expr,
                body,
            })),
            loc,
        })
    }
}
