mod clap_app;
mod commands;
mod profiling;
mod report_context;
mod reports;

use clap_app::build_cli;
use commands::execute;
use serde_json::Value;
use std::{
    env,
    io::{self, Write},
    process::ExitCode,
};

fn write_report(writer: &mut impl Write, value: &Value) -> io::Result<()> {
    serde_json::to_writer_pretty(&mut *writer, value)?;
    writer.write_all(b"\n")
}

fn main() -> ExitCode {
    let args: Vec<String> = env::args().skip(1).collect();
    if args.iter().any(|s| s == "--help" || s == "-h") {
        let mut cli = build_cli();
        let cmd_args = std::iter::once("ntesim".to_string()).chain(args.clone());
        if let Err(err) = cli.try_get_matches_from_mut(cmd_args) {
            let _ = err.print();
            return ExitCode::SUCCESS;
        }
    }
    if args.first().is_some_and(|s| s == "--version" || s == "-V") {
        println!("ntesim {}", env!("CARGO_PKG_VERSION"));
        return ExitCode::SUCCESS;
    }
    let profile = match profiling::Profile::start(&args) {
        Ok(profile) => profile,
        Err(error) => {
            let _ = writeln!(io::stderr().lock(), "{error}");
            return ExitCode::from(1);
        }
    };
    match execute(&args) {
        Ok((value, valid)) => {
            if let Some(profile) = profile
                && let Err(error) = profile.finish(&value, valid)
            {
                let _ = writeln!(io::stderr().lock(), "profiling failed: {error}");
                return ExitCode::from(1);
            }
            if let Err(error) = write_report(&mut io::stdout().lock(), &value) {
                let _ = writeln!(io::stderr().lock(), "failed to write JSON report: {error}");
                return ExitCode::from(1);
            }
            if valid {
                ExitCode::SUCCESS
            } else {
                ExitCode::from(2)
            }
        }
        Err(error) => {
            let _ = writeln!(io::stderr().lock(), "{error}");
            ExitCode::from(1)
        }
    }
}

#[cfg(test)]
mod tests;
