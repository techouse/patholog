#![forbid(unsafe_code)]

use std::io::{self, Write};

fn main() {
    let result = patholog::cli::run(
        std::env::args_os().skip(1),
        patholog::cli::CommandContext::from_env(),
    );

    let mut stdout = io::stdout().lock();
    let mut stderr = io::stderr().lock();
    let stdout_result = stdout.write_all(result.stdout.as_bytes());
    let stderr_result = stderr.write_all(result.stderr.as_bytes());

    if stdout_result.is_err() || stderr_result.is_err() {
        std::process::exit(1);
    }
    std::process::exit(result.exit_code.code());
}
