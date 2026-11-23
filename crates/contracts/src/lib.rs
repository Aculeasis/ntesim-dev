pub mod errors;
pub mod hypotheses;
pub mod ids;
pub mod models;
pub mod validation;

#[cfg(test)]
mod tests;

pub use errors::*;
pub use hypotheses::*;
pub use ids::*;
pub use models::*;

pub const SCHEMA_VERSION: u32 = 1;
pub const ENGINE_VERSION: &str = env!("CARGO_PKG_VERSION");
pub const ENGINE_SOURCE_SHA256: &str = match option_env!("NTESIM_ENGINE_SOURCE_SHA256") {
    Some(hash) => hash,
    None => "unavailable: build did not supply a source fingerprint",
};

pub fn sha256(bytes: &[u8]) -> String {
    use sha2::{Digest, Sha256};
    format!("{:x}", Sha256::digest(bytes))
}
