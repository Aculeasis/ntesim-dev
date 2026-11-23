use super::*;

#[derive(Debug, PartialEq, Eq)]
pub struct Error {
    pub position: Position,
    pub message: String,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "rotation {}:{}: {}",
            self.position.line, self.position.column, self.message
        )
    }
}

impl std::error::Error for Error {}

pub(crate) fn error(position: Position, message: impl Into<String>) -> Error {
    Error {
        position,
        message: message.into(),
    }
}
