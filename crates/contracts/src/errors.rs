use std::fmt;

/// Strongly typed contract errors.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ContractError {
    InvalidSchemaVersion { expected: u32, actual: u32 },
    PackageMismatch { expected: String, actual: String },
    ZeroDuration,
    EmptyDependencies,
    DuplicateDependency(String),
    UnknownDependency(String),
    ValidationFailure(String),
    Custom(String),
}

impl fmt::Display for ContractError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidSchemaVersion { expected, actual } => {
                write!(
                    f,
                    "unsupported scenario schema: expected {expected}, got {actual}"
                )
            }
            Self::PackageMismatch { expected, actual } => {
                write!(
                    f,
                    "scenario/data version mismatch: expected {expected}, got {actual}"
                )
            }
            Self::ZeroDuration => write!(f, "duration must be positive"),
            Self::EmptyDependencies => {
                write!(
                    f,
                    "empty execution dependency graph is not a validated scenario"
                )
            }
            Self::DuplicateDependency(id) => write!(f, "duplicate dependency: {id}"),
            Self::UnknownDependency(id) => {
                write!(f, "dependency is absent from model registry: {id}")
            }
            Self::ValidationFailure(msg) => write!(f, "validation failure: {msg}"),
            Self::Custom(msg) => write!(f, "{msg}"),
        }
    }
}

impl std::error::Error for ContractError {}

impl From<String> for ContractError {
    fn from(s: String) -> Self {
        Self::Custom(s)
    }
}

impl From<&str> for ContractError {
    fn from(s: &str) -> Self {
        Self::Custom(s.to_string())
    }
}
