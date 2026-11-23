use super::*;

#[derive(Debug)]
pub(crate) struct Token {
    pub(crate) text: String,
    pub(crate) position: Position,
}

pub(crate) fn tokenize(source: &str) -> Result<Vec<Token>, Error> {
    let mut position = Position { line: 1, column: 1 };
    if source.len() > MAX_SOURCE_BYTES {
        return Err(error(position, "source exceeds 1 MiB"));
    }
    let mut chars = source.chars().peekable();
    let mut tokens = Vec::new();
    while let Some(c) = chars.next() {
        let start = position;
        position.column += 1;
        match c {
            '\n' => {
                position.line += 1;
                position.column = 1;
            }
            c if c.is_ascii_whitespace() => {}
            '#' => {
                while chars.peek().is_some_and(|c| *c != '\n') {
                    chars.next();
                    position.column += 1;
                }
            }
            ';' | '{' | '}' => tokens.push(Token {
                text: c.to_string(),
                position: start,
            }),
            c if c.is_ascii_alphanumeric() || c == '_' => {
                let mut text = c.to_string();
                while chars
                    .peek()
                    .is_some_and(|c| c.is_ascii_alphanumeric() || "_.:-".contains(*c))
                {
                    text.push(chars.next().unwrap());
                    position.column += 1;
                }
                tokens.push(Token {
                    text,
                    position: start,
                });
            }
            _ => return Err(error(start, format!("unexpected character {c:?}"))),
        }
    }
    tokens.push(Token {
        text: String::new(),
        position,
    });
    Ok(tokens)
}
