//! NTE domain-specific language parser and AST.

pub(crate) mod action_parameters;
mod actor;
pub mod ast;
mod catalog;
pub(crate) mod combat;
pub mod compiler;
pub mod configuration;
pub(crate) mod dummy;
pub mod editing;
pub mod evaluate;
pub(crate) mod hurt;
mod incoming;
pub mod inspection;
pub(crate) mod interrupt;
pub mod lexer;
pub mod live;
pub mod number;
pub mod options;
pub mod parser;
mod rotation;
pub mod source;
#[cfg(test)]
mod source_tests;
mod source_validation;
mod symbols;
pub mod target;
pub mod validation;

#[cfg(test)]
mod compiler_tests;
#[cfg(test)]
mod configuration_tests;
#[cfg(test)]
mod editing_tests;
#[cfg(test)]
mod evaluation_tests;
#[cfg(test)]
mod incoming_tests;
#[cfg(test)]
mod inspection_tests;
#[cfg(test)]
mod options_tests;
#[cfg(test)]
mod plunge_tests;
#[cfg(test)]
mod target_tests;
#[cfg(test)]
mod tests;
#[cfg(test)]
mod validation_tests;

pub use ast::*;
pub use lexer::{Lexer, LexerError, Token, TokenKind};
pub use parser::{Parser, ParserError};

#[derive(Clone, Debug, PartialEq)]
pub enum DslError {
    Lexer(LexerError),
    Parser(ParserError),
}

impl std::fmt::Display for DslError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Lexer(e) => write!(f, "DSL lex error: {e}"),
            Self::Parser(e) => write!(f, "DSL parse error: {e}"),
        }
    }
}

impl std::error::Error for DslError {}

impl From<LexerError> for DslError {
    fn from(e: LexerError) -> Self {
        Self::Lexer(e)
    }
}

impl From<ParserError> for DslError {
    fn from(e: ParserError) -> Self {
        Self::Parser(e)
    }
}

/// Parses full NTE script source code into an AST `Script`.
pub fn parse_nte(source: &str) -> Result<Script, DslError> {
    let mut lexer = Lexer::new(source);
    let tokens = lexer.tokenize_all()?;
    let mut parser = Parser::new(tokens);
    let script = parser.parse_script()?;
    Ok(script)
}
