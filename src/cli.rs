use std::ffi::OsString;

mod execute;
mod fail_on;
mod parse;
mod types;

pub use types::{CliResult, CommandContext};

/// Runs the patholog CLI with already-split arguments.
pub fn run<I, S>(argv: I, context: CommandContext) -> CliResult
where
    I: IntoIterator<Item = S>,
    S: Into<OsString>,
{
    match parse::parse_args(argv) {
        Ok(cli) => execute::execute(cli, &context),
        Err(result) => result,
    }
}

#[cfg(test)]
#[path = "cli/tests.rs"]
mod tests;
