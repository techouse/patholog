use std::ffi::OsString;

use clap::{Parser, error::ErrorKind};

use super::types::{Cli, CliResult};

pub(super) fn parse_args<I, S>(argv: I) -> Result<Cli, CliResult>
where
    I: IntoIterator<Item = S>,
    S: Into<OsString>,
{
    let mut parser_argv = vec![OsString::from("patholog")];
    parser_argv.extend(argv.into_iter().map(Into::into));

    match Cli::try_parse_from(parser_argv) {
        Ok(cli) => Ok(cli),
        Err(error) if error.kind() == ErrorKind::DisplayHelp => {
            Err(CliResult::success(error.to_string()))
        }
        Err(error) if error.kind() == ErrorKind::DisplayVersion => {
            Err(CliResult::success(error.to_string()))
        }
        Err(error) => Err(CliResult::clap_error(error.to_string())),
    }
}
