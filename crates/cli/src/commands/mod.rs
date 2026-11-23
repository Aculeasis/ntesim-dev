pub(crate) mod definition;
pub(crate) mod dispatch;
pub(crate) mod parse;

#[cfg(test)]
pub use definition::CliCommand;
pub(crate) use dispatch::execute;
#[cfg(test)]
pub(crate) use parse::parse_command;
