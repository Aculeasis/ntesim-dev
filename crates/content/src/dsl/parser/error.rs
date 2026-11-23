use crate::dsl::ast::SourceLocation;
use std::fmt;

#[derive(Clone, Debug, PartialEq)]
pub struct ParserError {
    pub message: String,
    pub loc: SourceLocation,
}

impl fmt::Display for ParserError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "at line {}, column {}: {}",
            self.loc.line, self.loc.column, self.message
        )
    }
}

impl std::error::Error for ParserError {}
