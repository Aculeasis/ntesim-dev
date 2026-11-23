use super::cursor::Parser;
use super::error::ParserError;
use crate::dsl::ast::*;
use crate::dsl::lexer::{Token, TokenKind};
use std::collections::BTreeSet;

impl Parser {
    pub fn parse_script(&mut self) -> Result<Script, ParserError> {
        let options = if let TokenKind::Ident(name) = self.peek() {
            if name == "options" {
                self.parse_options()?
            } else {
                Vec::new()
            }
        } else {
            Vec::new()
        };

        let mut declarations = Vec::new();
        let mut statements = Vec::new();
        let mut rotation_started = false;

        while self.peek() != &TokenKind::Eof {
            if self.is_declaration_start() {
                if rotation_started {
                    return Err(ParserError {
                        message: "configuration must precede rotation statements".to_string(),
                        loc: self.current_loc(),
                    });
                }
                declarations.push(self.parse_declaration()?);
            } else {
                let statement = self.parse_statement()?;
                if !matches!(
                    statement.kind,
                    StatementKind::Variable { is_const: true, .. }
                ) {
                    rotation_started = true;
                }
                statements.push(statement);
            }
        }

        Ok(Script {
            options,
            declarations,
            statements,
        })
    }

    fn is_declaration_start(&self) -> bool {
        match self.peek() {
            TokenKind::Ident(name) => {
                if name == "target" {
                    return self
                        .tokens
                        .get(self.pos + 2)
                        .is_some_and(|token| token.kind == TokenKind::Eq);
                }
                if name == "active"
                    || name == "hurt"
                    || name == "incoming"
                    || name == "options"
                    || name == "dummy"
                    || name == "interrupt"
                    || name == "combat"
                {
                    return true;
                }
                if let Some(Token {
                    kind: TokenKind::Ident(next),
                    ..
                }) = self.tokens.get(self.pos + 1)
                {
                    return next == "char" || next == "add";
                }
                false
            }
            _ => false,
        }
    }

    fn parse_options(&mut self) -> Result<Vec<OptionPair>, ParserError> {
        self.advance(); // consume 'options'
        let mut pairs = Vec::new();
        let mut keys = BTreeSet::new();
        while self.peek() != &TokenKind::Semicolon && self.peek() != &TokenKind::Eof {
            let loc = self.current_loc();
            let key = self.unique_key(&mut keys)?;
            self.expect(&TokenKind::Eq)?;
            let mut value_range = self.current_loc();
            let value = self.parse_expression()?;
            let last = &self.tokens[self.pos - 1].loc;
            value_range.length = last.offset + last.length - value_range.offset;
            pairs.push(OptionPair {
                key,
                value,
                loc,
                value_range,
            });
        }
        self.expect(&TokenKind::Semicolon)?;
        Ok(pairs)
    }

    fn parse_declaration(&mut self) -> Result<Declaration, ParserError> {
        let loc = self.current_loc();
        let name = self.expect_ident()?;
        match name.as_str() {
            "target" => self.parse_target_decl(loc),
            "active" => self.parse_active_decl(loc),
            "hurt" => self.parse_hurt_decl(loc),
            "incoming" => self.parse_incoming_decl(loc),
            "dummy" => self.parse_dummy_decl(loc),
            "interrupt" => self.parse_interrupt_decl(loc),
            "combat" => self.parse_combat_decl(loc),
            actor => {
                let second = self.expect_ident()?;
                if second == "char" {
                    self.parse_character_decl(actor.to_string(), loc)
                } else if second == "add" {
                    self.parse_add_decl(actor.to_string(), loc)
                } else {
                    Err(ParserError {
                        message: format!(
                            "unexpected declaration keyword '{second}' after '{actor}'"
                        ),
                        loc,
                    })
                }
            }
        }
    }

    fn parse_character_decl(
        &mut self,
        actor: String,
        loc: SourceLocation,
    ) -> Result<Declaration, ParserError> {
        let mut level = None;
        let mut awakening = None;
        let mut talents = None;
        let mut qte = None;
        let mut extra_fields = Vec::new();
        let mut keys = BTreeSet::new();

        while self.peek() != &TokenKind::Semicolon && self.peek() != &TokenKind::Eof {
            if self.match_token(&TokenKind::Plus) {
                // +params=[...]
                let p = self.expect_ident()?;
                if p != "params" {
                    return Err(ParserError {
                        message: format!("expected 'params' after '+', got '{p}'"),
                        loc: self.current_loc(),
                    });
                }
                self.expect(&TokenKind::Eq)?;
                self.expect(&TokenKind::LBracket)?;
                while self.peek() != &TokenKind::RBracket && self.peek() != &TokenKind::Eof {
                    let k = self.unique_key(&mut keys)?;
                    self.expect(&TokenKind::Eq)?;
                    let v = self.parse_expression()?;
                    extra_fields.push((k, v));
                    if !self.match_token(&TokenKind::Comma) {
                        break;
                    }
                }
                self.expect(&TokenKind::RBracket)?;
                continue;
            }

            let key = self.unique_key(&mut keys)?;
            self.expect(&TokenKind::Eq)?;
            match key.as_str() {
                "lvl" => {
                    level = Some(self.parse_level()?);
                }
                "awakening" => {
                    awakening = Some(self.parse_awakening()?);
                }
                "talent" => {
                    talents = Some(Box::new(self.parse_talent_triple()?));
                }
                "qte" => {
                    qte = Some(Box::new(self.parse_expression()?));
                }
                _ => {
                    let val = self.parse_expression()?;
                    extra_fields.push((key, val));
                }
            }
        }
        self.expect(&TokenKind::Semicolon)?;
        Ok(Declaration {
            kind: DeclarationKind::Character {
                actor,
                level,
                awakening,
                talents,
                qte,
                extra_fields,
            },
            loc,
        })
    }

    fn parse_level(&mut self) -> Result<LevelPair, ParserError> {
        let current = self.expect_integer()?;
        self.expect(&TokenKind::Slash)?;
        let cap = self.expect_integer()?;
        Ok(LevelPair { current, cap })
    }

    fn parse_awakening(&mut self) -> Result<AwakeningSelection, ParserError> {
        if self.match_token(&TokenKind::LBracket) {
            let mut items = Vec::new();
            while self.peek() != &TokenKind::RBracket && self.peek() != &TokenKind::Eof {
                items.push(self.expect_integer()?);
                if !self.match_token(&TokenKind::Comma) {
                    break;
                }
            }
            self.expect(&TokenKind::RBracket)?;
            Ok(AwakeningSelection::Explicit(items))
        } else {
            Ok(AwakeningSelection::Rank(self.expect_integer()?))
        }
    }

    fn parse_talent_triple(&mut self) -> Result<TalentTriple, ParserError> {
        let normal = self.parse_expression()?;
        self.expect(&TokenKind::Comma)?;
        let skill = self.parse_expression()?;
        self.expect(&TokenKind::Comma)?;
        let burst = self.parse_expression()?;
        Ok(TalentTriple {
            normal,
            skill,
            burst,
        })
    }

    fn parse_add_decl(
        &mut self,
        actor: String,
        loc: SourceLocation,
    ) -> Result<Declaration, ParserError> {
        let add_type = self.expect_ident()?;
        match add_type.as_str() {
            "weapon" => {
                self.expect(&TokenKind::Eq)?;
                let weapon_id = self.expect_ident()?;
                let mut refinement = None;
                let mut level = None;
                let mut extra_fields = Vec::new();
                let mut keys = BTreeSet::from(["weapon".to_string()]);
                while self.peek() != &TokenKind::Semicolon && self.peek() != &TokenKind::Eof {
                    let k = self.unique_key(&mut keys)?;
                    self.expect(&TokenKind::Eq)?;
                    if k == "refine" {
                        refinement = Some(self.expect_integer()?);
                    } else if k == "lvl" {
                        level = Some(self.parse_level()?);
                    } else {
                        extra_fields.push((k, self.parse_expression()?));
                    }
                }
                self.expect(&TokenKind::Semicolon)?;
                Ok(Declaration {
                    kind: DeclarationKind::Weapon {
                        actor,
                        weapon_id,
                        refinement,
                        level,
                        extra_fields,
                    },
                    loc,
                })
            }
            "cartridge" => {
                self.expect(&TokenKind::Eq)?;
                let cartridge_id = self.expect_ident()?;
                let count_key = self.expect_ident()?;
                if count_key != "count" {
                    return Err(ParserError {
                        message: format!(
                            "expected 'count' in cartridge declaration, got '{count_key}'"
                        ),
                        loc: self.current_loc(),
                    });
                }
                self.expect(&TokenKind::Eq)?;
                let count = self.expect_integer()?;
                self.expect(&TokenKind::Semicolon)?;
                Ok(Declaration {
                    kind: DeclarationKind::Cartridge {
                        actor,
                        cartridge_id,
                        count,
                    },
                    loc,
                })
            }
            "stats" => {
                let mut pairs = Vec::new();
                let mut keys = BTreeSet::new();
                while self.peek() != &TokenKind::Semicolon && self.peek() != &TokenKind::Eof {
                    let k = self.unique_key(&mut keys)?;
                    self.expect(&TokenKind::Eq)?;
                    pairs.push((k, self.parse_expression()?));
                }
                self.expect(&TokenKind::Semicolon)?;
                Ok(Declaration {
                    kind: DeclarationKind::Stats { actor, pairs },
                    loc,
                })
            }
            other => Err(ParserError {
                message: format!("unknown add type '{other}'"),
                loc,
            }),
        }
    }

    fn parse_target_decl(&mut self, loc: SourceLocation) -> Result<Declaration, ParserError> {
        let mut pairs = Vec::new();
        let mut keys = BTreeSet::new();
        while self.peek() != &TokenKind::Semicolon && self.peek() != &TokenKind::Eof {
            let k = self.unique_key(&mut keys)?;
            self.expect(&TokenKind::Eq)?;
            pairs.push((k, self.parse_expression()?));
        }
        self.expect(&TokenKind::Semicolon)?;
        Ok(Declaration {
            kind: DeclarationKind::Target { pairs },
            loc,
        })
    }

    fn parse_active_decl(&mut self, loc: SourceLocation) -> Result<Declaration, ParserError> {
        let actor = self.expect_ident()?;
        self.expect(&TokenKind::Semicolon)?;
        Ok(Declaration {
            kind: DeclarationKind::Active { actor },
            loc,
        })
    }

    fn parse_hurt_decl(&mut self, loc: SourceLocation) -> Result<Declaration, ParserError> {
        let every = self.expect_ident()?;
        if every != "every" {
            return Err(ParserError {
                message: format!("expected 'every' after 'hurt', got '{every}'"),
                loc: self.current_loc(),
            });
        }
        let mut pairs = Vec::new();
        let mut keys = BTreeSet::new();
        while self.peek() != &TokenKind::Semicolon && self.peek() != &TokenKind::Eof {
            let k = self.unique_key(&mut keys)?;
            self.expect(&TokenKind::Eq)?;
            let first = self.parse_expression()?;
            let expr = if self.peek() == &TokenKind::Comma {
                self.advance();
                let second = self.parse_expression()?;
                Expression {
                    loc: first.loc.clone(),
                    kind: ExpressionKind::Range {
                        min: Box::new(first),
                        max: Box::new(second),
                    },
                }
            } else {
                first
            };
            pairs.push((k, expr));
        }
        self.expect(&TokenKind::Semicolon)?;
        Ok(Declaration {
            kind: DeclarationKind::Hurt { pairs },
            loc,
        })
    }

    fn parse_dummy_decl(&mut self, loc: SourceLocation) -> Result<Declaration, ParserError> {
        let first = self.expect_ident()?;
        let (target, kind) = if first.starts_with('t')
            && first.len() > 1
            && first[1..].chars().all(|c| c.is_ascii_digit())
        {
            (Some(first), self.expect_ident()?)
        } else {
            (None, first)
        };
        let mut pairs = Vec::new();
        let mut keys = BTreeSet::new();
        while self.peek() != &TokenKind::Semicolon && self.peek() != &TokenKind::Eof {
            let k = self.unique_key(&mut keys)?;
            self.expect(&TokenKind::Eq)?;
            pairs.push((k, self.parse_expression()?));
        }
        self.expect(&TokenKind::Semicolon)?;
        Ok(Declaration {
            kind: DeclarationKind::Dummy {
                kind,
                target,
                pairs,
            },
            loc,
        })
    }

    fn parse_interrupt_decl(&mut self, loc: SourceLocation) -> Result<Declaration, ParserError> {
        let actor = self.expect_ident()?;
        let mut pairs = Vec::new();
        let mut keys = BTreeSet::new();
        while self.peek() != &TokenKind::Semicolon && self.peek() != &TokenKind::Eof {
            let k = self.unique_key(&mut keys)?;
            self.expect(&TokenKind::Eq)?;
            pairs.push((k, self.parse_expression()?));
        }
        self.expect(&TokenKind::Semicolon)?;
        Ok(Declaration {
            kind: DeclarationKind::Interrupt { actor, pairs },
            loc,
        })
    }

    fn parse_combat_decl(&mut self, loc: SourceLocation) -> Result<Declaration, ParserError> {
        let actor = self.expect_ident()?;
        let mut pairs = Vec::new();
        let mut keys = BTreeSet::new();
        while self.peek() != &TokenKind::Semicolon && self.peek() != &TokenKind::Eof {
            let k = self.unique_key(&mut keys)?;
            self.expect(&TokenKind::Eq)?;
            pairs.push((k, self.parse_expression()?));
        }
        self.expect(&TokenKind::Semicolon)?;
        Ok(Declaration {
            kind: DeclarationKind::Combat { actor, pairs },
            loc,
        })
    }

    fn parse_incoming_decl(&mut self, loc: SourceLocation) -> Result<Declaration, ParserError> {
        let actor = self.expect_ident()?;
        let mut pairs = Vec::new();
        let mut keys = BTreeSet::new();
        while self.peek() != &TokenKind::Semicolon && self.peek() != &TokenKind::Eof {
            let k = self.unique_key(&mut keys)?;
            self.expect(&TokenKind::Eq)?;
            pairs.push((k, self.parse_expression()?));
        }
        self.expect(&TokenKind::Semicolon)?;
        Ok(Declaration {
            kind: DeclarationKind::Incoming { actor, pairs },
            loc,
        })
    }
}
