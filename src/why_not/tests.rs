use std::path::Path;

use crate::model::{IssueKind, PlatformMode};

use super::{WhyNotOptions, analyze_why_not};

#[test]
fn analyze_why_not_preserves_found_candidate_and_winner() {
    let (directory, root) = relative_tempdir();
    let bin = root.join("bin");
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
    let (directory, root) = relative_tempdir();
    let bin = root.join("bin");
    let missing = root.join("missing");
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

#[test]
fn analyze_why_not_omits_diagnostic_and_related_advice_when_absent() {
    let (directory, root) = relative_tempdir();
    let bin = root.join("bin");
    std::fs::create_dir(&bin).expect("create bin");

    let report = analyze_why_not(&WhyNotOptions {
        path_value: &bin.display().to_string(),
        command: "tool",
        platform_mode: PlatformMode::Posix,
        pathext: None,
        cwd: directory.path(),
        home_dir: None,
        user_profile_dir: None,
    });

    assert!(!report.found());
    assert!(report.path_diagnostics.is_empty());
    assert!(report.related_hints.is_empty());
    assert!(
        report
            .advice
            .iter()
            .all(|advice| !advice.contains("PATH diagnostics") && !advice.contains("related"))
    );
    assert!(
        report
            .advice
            .iter()
            .any(|advice| advice.contains("Common tool directories absent from PATH"))
    );
}

#[test]
fn analyze_why_not_reports_windows_common_tool_directories() {
    let (directory, root) = relative_tempdir();
    let bin = root.join("bin");
    std::fs::create_dir(&bin).expect("create bin");
    let user_profile = root.join("Users").join("me");
    let cargo_bin = user_profile.join(".cargo/bin").display().to_string();
    let windows_apps = user_profile
        .join("AppData/Local/Microsoft/WindowsApps")
        .display()
        .to_string();

    let report = analyze_why_not(&WhyNotOptions {
        path_value: &bin.display().to_string(),
        command: "cargo",
        platform_mode: PlatformMode::Windows,
        pathext: None,
        cwd: directory.path(),
        home_dir: None,
        user_profile_dir: Some(&user_profile),
    });

    assert!(!report.found());
    let common_directory_advice = report
        .advice
        .iter()
        .find(|advice| advice.contains("Common tool directories absent from PATH"))
        .expect("common directory advice");
    assert!(common_directory_advice.contains(&cargo_bin));
    assert!(common_directory_advice.contains(&windows_apps));
}

#[test]
fn analyze_why_not_reports_posix_rust_common_tool_directory() {
    let (directory, root) = relative_tempdir();
    let bin = root.join("bin");
    std::fs::create_dir(&bin).expect("create bin");
    let cargo_bin = directory.path().join(".cargo/bin").display().to_string();

    let report = analyze_why_not(&WhyNotOptions {
        path_value: &bin.display().to_string(),
        command: "cargo",
        platform_mode: PlatformMode::Posix,
        pathext: None,
        cwd: directory.path(),
        home_dir: Some(directory.path()),
        user_profile_dir: None,
    });

    assert!(!report.found());
    assert!(
        report
            .advice
            .iter()
            .any(|advice| advice.contains(&cargo_bin))
    );
}

#[test]
fn analyze_why_not_omits_common_directory_advice_when_common_dirs_are_present() {
    let directory = tempfile::tempdir().expect("create tempdir");

    let report = analyze_why_not(&WhyNotOptions {
        path_value: "/opt/homebrew/bin:/usr/local/bin",
        command: "tool",
        platform_mode: PlatformMode::Posix,
        pathext: None,
        cwd: directory.path(),
        home_dir: None,
        user_profile_dir: None,
    });

    assert!(!report.found());
    assert!(
        report
            .advice
            .iter()
            .all(|advice| !advice.contains("Common tool directories absent from PATH"))
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

fn relative_tempdir() -> (tempfile::TempDir, std::path::PathBuf) {
    let directory = tempfile::Builder::new()
        .prefix(".patholog-test-")
        .tempdir_in(".")
        .expect("create relative tempdir");
    let relative_path = directory
        .path()
        .file_name()
        .expect("tempdir has directory name")
        .into();
    (directory, relative_path)
}
