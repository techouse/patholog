use std::path::Path;

use crate::model::{PlatformMode, ResolutionReport};
use crate::platform::resolve_platform_rules;

use super::names::candidate_names;
use super::resolve_command;

#[test]
fn posix_command_found_once() {
    let (directory, root) = relative_tempdir();
    let bin = root.join("bin");
    make_executable(&bin.join("tool"));

    let report = resolve_command(
        &bin.display().to_string(),
        "tool",
        PlatformMode::Posix,
        None,
        directory.path(),
        true,
    );

    assert_eq!(
        candidate_paths(&report),
        [bin.join("tool").display().to_string()]
    );
}

#[test]
fn posix_command_found_multiple_times_marks_first_winner() {
    let (directory, root) = relative_tempdir();
    let first = root.join("first");
    let second = root.join("second");
    make_executable(&first.join("tool"));
    make_executable(&second.join("tool"));

    let report = resolve_command(
        &format!("{}:{}", first.display(), second.display()),
        "tool",
        PlatformMode::Posix,
        None,
        directory.path(),
        true,
    );

    assert_eq!(
        report
            .candidates
            .iter()
            .map(|candidate| candidate.wins)
            .collect::<Vec<_>>(),
        [true, false]
    );
}

#[test]
fn command_not_found_records_searched_directories() {
    let (directory, root) = relative_tempdir();
    let bin = root.join("bin");
    std::fs::create_dir(&bin).expect("create bin");

    let report = resolve_command(
        &bin.display().to_string(),
        "missing",
        PlatformMode::Posix,
        None,
        directory.path(),
        true,
    );

    assert_eq!(report.searched_directories, [bin.display().to_string()]);
}

#[test]
fn empty_posix_entry_searches_current_directory() {
    let directory = tempfile::tempdir().expect("create tempdir");
    make_executable(&directory.path().join("tool"));

    let report = resolve_command(
        "",
        "tool",
        PlatformMode::Posix,
        None,
        directory.path(),
        true,
    );

    assert_eq!(
        report.searched_directories,
        ["<empty entry: current directory>"]
    );
    assert_eq!(
        candidate_paths(&report),
        [directory.path().join("tool").display().to_string()]
    );
}

#[test]
fn non_directory_path_entries_are_not_searched() {
    let (directory, root) = relative_tempdir();
    let path_file = root.join("not-a-directory");
    std::fs::write(&path_file, "not a directory").expect("write file");

    let report = resolve_command(
        &path_file.display().to_string(),
        "tool",
        PlatformMode::Posix,
        None,
        directory.path(),
        true,
    );

    assert!(report.candidates.is_empty());
    assert_eq!(
        report.searched_directories,
        [path_file.display().to_string()]
    );
}

#[test]
fn posix_directory_named_like_command_is_not_executable() {
    let (directory, root) = relative_tempdir();
    let bin = root.join("bin");
    std::fs::create_dir_all(bin.join("tool")).expect("create command directory");

    let report = resolve_command(
        &bin.display().to_string(),
        "tool",
        PlatformMode::Posix,
        None,
        directory.path(),
        true,
    );

    assert!(report.candidates.is_empty());
}

#[test]
fn windows_pathext_resolution_is_modelled() {
    let directory = tempfile::tempdir().expect("create tempdir");
    let bin = directory.path().join("bin");
    std::fs::create_dir(&bin).expect("create bin");
    std::fs::write(bin.join("tool.exe"), "windows binary").expect("write exe");

    let report = resolve_command(
        &bin.display().to_string(),
        "tool",
        PlatformMode::Windows,
        Some(".EXE;.CMD"),
        directory.path(),
        true,
    );

    assert_eq!(
        candidate_paths(&report),
        [bin.join("tool.exe").display().to_string()]
    );
}

#[test]
fn windows_pathext_order_decides_winner() {
    let directory = tempfile::tempdir().expect("create tempdir");
    let bin = directory.path().join("bin");
    std::fs::create_dir(&bin).expect("create bin");
    std::fs::write(bin.join("tool.cmd"), "cmd binary").expect("write cmd");
    std::fs::write(bin.join("tool.exe"), "exe binary").expect("write exe");

    let report = resolve_command(
        &bin.display().to_string(),
        "tool",
        PlatformMode::Windows,
        Some(".CMD;.EXE"),
        directory.path(),
        true,
    );

    assert_eq!(
        candidate_paths(&report),
        [
            bin.join("tool.cmd").display().to_string(),
            bin.join("tool.exe").display().to_string()
        ]
    );
}

#[test]
fn windows_command_with_extension_does_not_append_pathext() {
    let directory = tempfile::tempdir().expect("create tempdir");
    let bin = directory.path().join("bin");
    std::fs::create_dir(&bin).expect("create bin");
    std::fs::write(bin.join("tool.exe"), "exe binary").expect("write exe");
    std::fs::write(bin.join("tool.exe.cmd"), "cmd binary").expect("write cmd");

    let report = resolve_command(
        &bin.display().to_string(),
        "tool.exe",
        PlatformMode::Windows,
        Some(".CMD;.EXE"),
        directory.path(),
        true,
    );

    assert_eq!(
        candidate_paths(&report),
        [bin.join("tool.exe").display().to_string()]
    );
}

#[test]
fn windows_executable_lookup_is_case_insensitive() {
    let directory = tempfile::tempdir().expect("create tempdir");
    let bin = directory.path().join("bin");
    std::fs::create_dir(&bin).expect("create bin");
    std::fs::write(bin.join("Tool.EXE"), "windows binary").expect("write exe");

    let report = resolve_command(
        &bin.display().to_string(),
        "tool",
        PlatformMode::Windows,
        Some(".EXE"),
        directory.path(),
        true,
    );

    assert_eq!(
        candidate_paths(&report),
        [bin.join("Tool.EXE").display().to_string()]
    );
}

#[test]
fn related_name_hint_is_not_a_match() {
    let (directory, root) = relative_tempdir();
    let bin = root.join("bin");
    make_executable(&bin.join("python3"));

    let report = resolve_command(
        &bin.display().to_string(),
        "python",
        PlatformMode::Posix,
        None,
        directory.path(),
        true,
    );

    assert!(report.candidates.is_empty());
    assert_eq!(report.related_hints[0].command, "python3");
}

#[test]
fn related_hints_omit_missing_related_commands() {
    let directory = tempfile::tempdir().expect("create tempdir");

    let report = resolve_command(
        "",
        "python",
        PlatformMode::Posix,
        None,
        directory.path(),
        true,
    );

    assert!(report.candidates.is_empty());
    assert!(report.related_hints.is_empty());
}

#[test]
fn windows_command_suffix_after_separator_is_not_expanded() {
    let rules = resolve_platform_rules(PlatformMode::Windows, Some(".EXE;.CMD"));

    assert_eq!(
        candidate_names(r"Scripts\tool.exe", &rules),
        [r"Scripts\tool.exe"]
    );
}

fn candidate_paths(report: &ResolutionReport) -> Vec<String> {
    report
        .candidates
        .iter()
        .map(|candidate| candidate.path.clone())
        .collect()
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
