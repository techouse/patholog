use std::path::Path;

use crate::model::{IssueKind, PlatformMode};

use super::{WhyNotOptions, analyze_why_not};

#[test]
fn analyze_why_not_preserves_found_candidate_and_winner() {
    let directory = tempfile::tempdir().expect("create tempdir");
    let bin = directory.path().join("bin");
    make_executable(&bin.join("tool"));

    let report = analyze_why_not(&WhyNotOptions {
        path_value: &bin.display().to_string(),
        command: "tool",
        platform_mode: PlatformMode::Posix,
        pathext: None,
        cwd: directory.path(),
        home_dir: Some(directory.path()),
        user_profile_dir: None,
    });
    let expected_path = bin.join("tool").display().to_string();

    assert!(report.found());
    assert_eq!(
        report.winner().map(|candidate| candidate.path.as_str()),
        Some(expected_path.as_str())
    );
    assert_eq!(
        report.advice,
        ["The exact command is already available from PATH."]
    );
}

#[test]
fn analyze_why_not_reports_missing_command_context() {
    let directory = tempfile::tempdir().expect("create tempdir");
    let bin = directory.path().join("bin");
    let missing = directory.path().join("missing");
    std::fs::create_dir(&bin).expect("create bin");
    make_executable(&bin.join("python3"));

    let report = analyze_why_not(&WhyNotOptions {
        path_value: &format!("{}:{}", missing.display(), bin.display()),
        command: "python",
        platform_mode: PlatformMode::Posix,
        pathext: None,
        cwd: directory.path(),
        home_dir: Some(directory.path()),
        user_profile_dir: None,
    });

    assert!(!report.found());
    assert_eq!(report.related_hints[0].command, "python3");
    assert!(
        report
            .path_diagnostics
            .iter()
            .any(|diagnostic| diagnostic.kind == IssueKind::Missing)
    );
    assert!(
        report
            .advice
            .iter()
            .any(|advice| advice.contains("Common tool directories absent from PATH"))
    );
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
