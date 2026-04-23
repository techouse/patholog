use std::path::Path;

use crate::model::ExitCode;

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
fn usage_error_maps_to_general_error() {
    let result = run(["bad"], context("", None));

    assert_eq!(result.exit_code, ExitCode::GeneralError);
    assert_eq!(result.stdout, "");
    assert!(result.stderr.contains("bad"));
}

fn context(path_value: &str, pathext: Option<&str>) -> CommandContext {
    CommandContext {
        path_value: path_value.to_owned(),
        pathext: pathext.map(str::to_owned),
        cwd: Path::new(".").to_path_buf(),
    }
}
