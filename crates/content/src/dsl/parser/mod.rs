//! Recursive descent parser for NTE domain-specific language.

mod cursor;
mod declarations;
mod error;
mod expressions;
mod statements;

pub use cursor::Parser;
pub use error::ParserError;
