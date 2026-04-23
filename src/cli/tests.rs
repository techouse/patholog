use std::path::Path;

use crate::model::{ExitCode, IssueKind};

use super::fail_on::parse_fail_on;
use super::{CommandContext, run};

#[test]
fn clean_requires_stdout() {
    let result = run(["clean"], context("", None));

    assert_eq!(result.exit_code, ExitCode::GeneralError);
    assert_eq!(result.stderr, "patholog: clean requires --stdout\n");
}

#[test]
fn doctor_fail_on_returns_diagnostics_found() {
    let directory = tempfile::tempdir().expect("create tempdir");
    let missing = directory.path().join("missing");

    let result = run(
        ["doctor", "--platform", "posix", "--fail-on=missing"],
        context(&missing.display().to_string(), None),
    );

    assert_eq!(result.exit_code, ExitCode::DiagnosticsFound);
    assert!(result.stdout.contains("Missing directories:"));
}

#[test]
fn version_uses_injected_stdout() {
    let result = run(["--version"], context("", None));

    assert_eq!(result.exit_code, ExitCode::Success);
    assert_eq!(result.stdout, "patholog 0.1.0\n");
    assert_eq!(result.stderr, "");
}

#[test]
fn help_uses_injected_stdout() {
    let result = run(["--help"], context("", None));

    assert_eq!(result.exit_code, ExitCode::Success);
    assert!(result.stdout.contains("Usage: patholog <COMMAND>"));
    assert_eq!(result.stderr, "");
}

#[test]
fn usage_error_maps_to_general_error() {
    let result = run(["bad"], context("", None));

    assert_eq!(result.exit_code, ExitCode::GeneralError);
    assert_eq!(result.stdout, "");
    assert!(result.stderr.contains("bad"));
}

#[test]
fn invalid_fail_on_returns_runtime_error() {
    let result = run(["doctor", "--fail-on=bogus"], context("", None));

    assert_eq!(result.exit_code, ExitCode::GeneralError);
    assert_eq!(result.stdout, "");
    assert!(result.stderr.contains("unsupported issue kind \"bogus\""));
}

#[test]
fn parse_fail_on_trims_ignores_empty_and_deduplicates() {
    assert_eq!(
        parse_fail_on(" duplicate, ,not_directory,suspicious_order,duplicate ")
            .expect("parse fail-on"),
        [
            IssueKind::Duplicate,
            IssueKind::NotDirectory,
            IssueKind::SuspiciousOrder
        ]
    );
}

#[test]
fn command_context_from_env_reads_process_context() {
    let context = CommandContext::from_env();

    assert!(context.cwd.is_dir() || context.cwd == Path::new("."));
}

fn context(path_value: &str, pathext: Option<&str>) -> CommandContext {
    CommandContext {
        path_value: path_value.to_owned(),
        pathext: pathext.map(str::to_owned),
        cwd: Path::new(".").to_path_buf(),
    }
}
