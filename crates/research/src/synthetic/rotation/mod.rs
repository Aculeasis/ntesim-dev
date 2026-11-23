//! Explicit synthetic timelines. This language does not infer game action legality.
use crate::{
    scheduler::ClockId,
    synthetic::{self, ActorInput, ClockInput, Command, Effect, PoolInput, TargetInput},
};
use ntesim_contracts::sha256;
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, fmt};

mod limits;
pub use limits::LANGUAGE_VERSION;
pub(crate) use limits::{MAX_DEPTH, MAX_REPEAT, MAX_SOURCE_BYTES, MAX_WORK};
mod model;
pub(crate) use model::Fixture;
pub use model::{Metadata, Position, Report};
mod diagnostics;
pub use diagnostics::Error;
pub(crate) use diagnostics::error;
mod lexer;
pub(crate) use lexer::{Token, tokenize};
mod parser;
pub(crate) use parser::{Node, Parser, Statement};
mod compiler;
pub use compiler::{Compiled, Origin, compile};
mod runtime;
pub(crate) use runtime::{Prepared, prepare};
pub use runtime::{run, run_seeded};

#[cfg(test)]
mod tests;
