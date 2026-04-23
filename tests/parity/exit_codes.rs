use std::path::Path;

use patholog::cli::run;
use patholog::model::ExitCode;

use crate::support::context;

#[test]
fn doctor_fail_on_returns_exit_code_two() {
    let directory = tempfile::tempdir().expect("create tempdir");
    let missing = directory.path().join("missing");

    let result = run(
        ["doctor", "--platform", "posix", "--fail-on=missing"],
        context(&missing.display().to_string(), None, directory.path()),
    );

    assert_eq!(result.exit_code, ExitCode::DiagnosticsFound);
    assert_eq!(result.stderr, "");
}

#[test]
fn why_not_found_returns_exit_code_three() {
    let directory = tempfile::tempdir().expect("create tempdir");
    let bin = directory.path().join("bin");
    std::fs::create_dir(&bin).expect("create bin");

    let result = run(
        ["why", "missing", "--platform", "posix"],
        context(&bin.display().to_string(), None, directory.path()),
    );

    assert_eq!(result.exit_code, ExitCode::CommandNotFound);
    assert_eq!(result.stderr, "");
}

#[test]
fn usage_error_returns_exit_code_one() {
    let result = run(["bad"], context("", None, Path::new(".")));

    assert_eq!(result.exit_code, ExitCode::GeneralError);
    assert_eq!(result.stdout, "");
}
