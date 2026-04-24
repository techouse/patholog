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
fn doctor_command_reports_shadowed_candidates() {
    let directory = tempfile::tempdir().expect("create tempdir");
    let first = directory.path().join("first");
    let second = directory.path().join("second");
    make_executable(&first.join("tool.exe"));
    make_executable(&second.join("tool.exe"));

    let result = run(
        ["doctor", "--platform", "windows", "--command", "tool"],
        context_with_cwd(
            &format!("{};{}", first.display(), second.display()),
            None,
            directory.path(),
        ),
    );

    assert_eq!(result.exit_code, ExitCode::Success);
    assert!(result.stdout.contains("Shadowed commands:"));
    assert!(result.stdout.contains("tool at"));
}

#[test]
fn scan_reports_path_changes_from_home_profiles() {
    let directory = tempfile::tempdir().expect("create tempdir");
    let zshrc = directory.path().join(".zshrc");
    std::fs::write(&zshrc, "export PATH=\"$HOME/bin:$PATH\"\n").expect("write zshrc");

    let result = run(
        ["scan", "--platform", "posix"],
        context_with_home("", None, directory.path()),
    );

    assert_eq!(result.exit_code, ExitCode::Success);
    assert!(result.stdout.contains("PATH changes:"));
    assert!(result.stdout.contains(".zshrc"));
}

#[test]
fn scan_home_option_overrides_missing_context_home() {
    let directory = tempfile::tempdir().expect("create tempdir");
    let home = directory.path().display().to_string();
    let result = run(
        ["scan", "--platform", "posix", "--home", home.as_str()],
        context("", None),
    );

    assert_eq!(result.exit_code, ExitCode::Success);
    assert!(result.stdout.contains("No PATH changes found"));
}

#[test]
fn scan_requires_home_directory() {
    let result = run(["scan"], context("", None));

    assert_eq!(result.exit_code, ExitCode::GeneralError);
    assert_eq!(
        result.stderr,
        "patholog: scan requires a home directory; set HOME or pass --home\n"
    );
}

#[test]
fn version_uses_injected_stdout() {
    let result = run(["--version"], context("", None));

    assert_eq!(result.exit_code, ExitCode::Success);
    assert_eq!(result.stdout, "patholog 0.2.0\n");
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
        parse_fail_on(
            " duplicate, ,not_directory,unreadable,suspicious_order,shadowed_command,duplicate ",
        )
        .expect("parse fail-on"),
        [
            IssueKind::Duplicate,
            IssueKind::NotDirectory,
            IssueKind::Unreadable,
            IssueKind::SuspiciousOrder,
            IssueKind::ShadowedCommand
        ]
    );
}

#[test]
fn command_context_from_env_reads_process_context() {
    let context = CommandContext::from_env();

    assert!(context.cwd.is_dir() || context.cwd == Path::new("."));
}

fn context(path_value: &str, pathext: Option<&str>) -> CommandContext {
    context_with_cwd(path_value, pathext, Path::new("."))
}

fn context_with_cwd(path_value: &str, pathext: Option<&str>, cwd: &Path) -> CommandContext {
    CommandContext {
        path_value: path_value.to_owned(),
        pathext: pathext.map(str::to_owned),
        cwd: cwd.to_path_buf(),
        home_dir: None,
    }
}

fn context_with_home(path_value: &str, pathext: Option<&str>, home: &Path) -> CommandContext {
    CommandContext {
        path_value: path_value.to_owned(),
        pathext: pathext.map(str::to_owned),
        cwd: Path::new(".").to_path_buf(),
        home_dir: Some(home.to_path_buf()),
    }
}

fn make_executable(path: &Path) {
    std::fs::create_dir_all(path.parent().expect("path has parent")).expect("create parent");
    std::fs::write(path, "#!/bin/sh\nexit 0\n").expect("write executable");
    make_permissions_executable(path);
}

#[cfg(unix)]
fn make_permissions_executable(path: &Path) {
    use std::os::unix::fs::PermissionsExt;

    let mut permissions = std::fs::metadata(path)
        .expect("read metadata")
        .permissions();
    permissions.set_mode(0o755);
    std::fs::set_permissions(path, permissions).expect("set executable");
}

#[cfg(not(unix))]
fn make_permissions_executable(_path: &Path) {}
