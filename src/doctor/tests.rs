use crate::model::{IssueKind, PathVariable, PlatformMode};
use crate::policy::PathPolicy;

use std::path::Path;

use super::{diagnose_command_path_with_policy, diagnose_path, diagnose_path_with_policy};

#[test]
fn diagnose_path_detects_core_diagnostics() {
    let (_directory, root) = relative_tempdir();
    let bin = root.join("bin");
    std::fs::create_dir(&bin).expect("create bin");
    let file_path = root.join("file");
    std::fs::write(&file_path, "not a directory").expect("write file");
    let missing = root.join("missing");

    let report = diagnose_path(
        &format!(
            "{}:{}:{}::{}",
            bin.display(),
            missing.display(),
            file_path.display(),
            bin.display()
        ),
        PlatformMode::Posix,
        None,
    );

    assert_eq!(
        report
            .diagnostics
            .iter()
            .map(|diagnostic| diagnostic.kind)
            .collect::<Vec<_>>(),
        [
            IssueKind::Duplicate,
            IssueKind::Missing,
            IssueKind::NotDirectory,
            IssueKind::Empty
        ]
    );
}

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

#[test]
fn ordering_diagnostic_warns_for_usr_bin_before_homebrew() {
    let report = diagnose_path("/usr/bin:/opt/homebrew/bin", PlatformMode::Posix, None);

    assert!(
        report
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.kind == IssueKind::SuspiciousOrder
                && diagnostic.message == "/usr/bin appears before /opt/homebrew/bin")
    );
}

#[test]
fn ordering_diagnostic_warns_for_system_dir_before_user_tool_dir() {
    let report = diagnose_path("/bin:/Users/me/.cargo/bin", PlatformMode::Posix, None);

    assert!(
        report
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.kind == IssueKind::SuspiciousOrder
                && diagnostic.message == "/bin appears before /Users/me/.cargo/bin")
    );
}

#[test]
fn manpath_diagnostics_do_not_report_path_ordering() {
    let report = diagnose_path_with_policy(
        "/bin:/Users/me/.cargo/bin",
        PlatformMode::Posix,
        None,
        PathVariable::Manpath,
        &PathPolicy::default(),
    );

    assert!(
        report
            .diagnostics
            .iter()
            .all(|diagnostic| diagnostic.kind != IssueKind::SuspiciousOrder)
    );
}

#[test]
fn windows_duplicate_detection_is_case_insensitive() {
    let directory = tempfile::tempdir().expect("create tempdir");
    let tools = directory.path().join("Tools");
    std::fs::create_dir(&tools).expect("create tools");
    let alternate_case = tools.display().to_string().to_uppercase();

    let report = diagnose_path(
        &format!("{};{}", tools.display(), alternate_case),
        PlatformMode::Windows,
        None,
    );

    assert!(
        report
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.kind == IssueKind::Duplicate)
    );
}

#[test]
fn command_diagnostics_report_shadowed_candidates() {
    let directory = tempfile::tempdir().expect("create tempdir");
    let first = directory.path().join("first");
    let second = directory.path().join("second");
    make_executable(&first.join("tool.exe"));
    make_executable(&second.join("tool.exe"));

    let report = diagnose_command_path_with_policy(
        &format!("{};{}", first.display(), second.display()),
        "tool",
        PlatformMode::Windows,
        None,
        directory.path(),
        &crate::policy::PathPolicy::default(),
    );

    assert!(
        report
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.kind == IssueKind::ShadowedCommand
                && diagnostic.message.contains("tool at")
                && diagnostic.message.contains("is shadowed by"))
    );
}

#[test]
fn command_diagnostics_ignore_same_path_entry_pathext_matches() {
    let directory = tempfile::tempdir().expect("create tempdir");
    let bin = directory.path().join("bin");
    make_executable(&bin.join("tool.cmd"));
    make_executable(&bin.join("tool.exe"));

    let report = diagnose_command_path_with_policy(
        &bin.display().to_string(),
        "tool",
        PlatformMode::Windows,
        Some(".CMD;.EXE"),
        directory.path(),
        &crate::policy::PathPolicy::default(),
    );

    assert!(
        report
            .diagnostics
            .iter()
            .all(|diagnostic| diagnostic.kind != IssueKind::ShadowedCommand)
    );
}

#[cfg(unix)]
#[test]
fn diagnose_path_reports_unreadable_directories() {
    use std::os::unix::fs::PermissionsExt;

    let directory = tempfile::tempdir().expect("create tempdir");
    let restricted = directory.path().join("restricted");
    std::fs::create_dir(&restricted).expect("create restricted");

    let mut permissions = std::fs::metadata(&restricted)
        .expect("read metadata")
        .permissions();
    permissions.set_mode(0o000);
    std::fs::set_permissions(&restricted, permissions).expect("restrict directory");

    if std::fs::read_dir(&restricted).is_ok() {
        restore_directory_permissions(&restricted);
        return;
    }

    let report = diagnose_path(&restricted.display().to_string(), PlatformMode::Posix, None);
    restore_directory_permissions(&restricted);

    assert!(
        report
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.kind == IssueKind::Unreadable)
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

#[cfg(unix)]
fn restore_directory_permissions(path: &Path) {
    use std::os::unix::fs::PermissionsExt;

    let mut permissions = std::fs::metadata(path)
        .expect("read metadata")
        .permissions();
    permissions.set_mode(0o755);
    std::fs::set_permissions(path, permissions).expect("restore permissions");
}
