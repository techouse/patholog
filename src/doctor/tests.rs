use crate::model::{IssueKind, PlatformMode};

use super::diagnose_path;

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
