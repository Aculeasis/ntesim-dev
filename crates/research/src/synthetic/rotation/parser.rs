use super::*;

pub(crate) enum Statement {
    Clock(ClockId),
    Wait(u64),
    Do(String),
    Repeat(u64, Vec<Node>),
}

pub(crate) struct Node {
    pub(crate) position: Position,
    pub(crate) statement: Statement,
}

pub(crate) struct Parser {
    pub(crate) tokens: Vec<Token>,
    pub(crate) next: usize,
    pub(crate) nodes: usize,
}

impl Parser {
    pub(crate) fn peek(&self) -> &Token {
        &self.tokens[self.next]
    }
    pub(crate) fn take(&mut self) -> Result<Token, Error> {
        if self.peek().text.is_empty() {
            return Err(error(self.peek().position, "unexpected end of source"));
        }
        let token = Token {
            text: self.peek().text.clone(),
            position: self.peek().position,
        };
        self.next += 1;
        Ok(token)
    }
    pub(crate) fn expect(&mut self, text: &str) -> Result<(), Error> {
        if self.peek().text != text {
            return Err(error(self.peek().position, format!("expected {text:?}")));
        }
        self.next += 1;
        Ok(())
    }
    pub(crate) fn integer(&mut self) -> Result<u64, Error> {
        let token = self.take()?;
        if !token.text.bytes().all(|c| c.is_ascii_digit()) {
            return Err(error(token.position, "expected unsigned decimal integer"));
        }
        token
            .text
            .parse()
            .map_err(|_| error(token.position, "integer overflow"))
    }
    pub(crate) fn block(&mut self, depth: usize) -> Result<Vec<Node>, Error> {
        let mut nodes = Vec::new();
        while !self.peek().text.is_empty() && self.peek().text != "}" {
            let token = self.take()?;
            self.nodes += 1;
            if self.nodes > MAX_WORK {
                return Err(error(token.position, "statement limit exceeded"));
            }
            let statement = match token.text.as_str() {
                "clock" => {
                    let id = self.integer()?;
                    let id = u32::try_from(id)
                        .map_err(|_| error(token.position, "clock ID overflow"))?;
                    self.expect(";")?;
                    Statement::Clock(id)
                }
                "wait" => {
                    let time = self.integer()?;
                    self.expect(";")?;
                    Statement::Wait(time)
                }
                "do" => {
                    let action = self.take()?;
                    if !action
                        .text
                        .starts_with(|c: char| c.is_ascii_alphabetic() || c == '_')
                        || !action
                            .text
                            .chars()
                            .all(|c| c.is_ascii_alphanumeric() || "_.:-".contains(c))
                    {
                        return Err(error(action.position, "expected action identifier"));
                    }
                    self.expect(";")?;
                    Statement::Do(action.text)
                }
                "repeat" => {
                    let count = self.integer()?;
                    if count == 0 || count > MAX_REPEAT {
                        return Err(error(token.position, "repeat count must be 1..10000"));
                    }
                    if depth >= MAX_DEPTH {
                        return Err(error(token.position, "repeat nesting exceeds 32"));
                    }
                    self.expect("{")?;
                    let body = self.block(depth + 1)?;
                    self.expect("}")?;
                    if body.is_empty() {
                        return Err(error(token.position, "empty repeat body"));
                    }
                    Statement::Repeat(count, body)
                }
                _ => {
                    return Err(error(
                        token.position,
                        format!("unknown statement {:?}", token.text),
                    ));
                }
            };
            nodes.push(Node {
                position: token.position,
                statement,
            });
        }
        Ok(nodes)
    }
}
