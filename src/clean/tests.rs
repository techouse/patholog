use crate::model::{PlatformMode, ShellKind};

use super::{
    clean_export, clean_path, fish_single_quote, posix_single_quote, powershell_single_quote,
};

#[test]
fn clean_path_removes_empty_entries_and_later_duplicates() {
    let (_directory, root) = relative_tempdir();
    let first = root.join("first");
    let second = root.join("second");
    std::fs::create_dir(&first).expect("create first");
    std::fs::create_dir(&second).expect("create second");

    assert_eq!(
        clean_path(
            &format!(
                "{}:{}::{}",
                first.display(),
                second.display(),
                first.display()
            ),
            PlatformMode::Posix,
            None,
        ),
        format!("{}:{}", first.display(), second.display())
    );
}

#[test]
fn clean_path_keeps_missing_and_non_directory_entries() {
    let (_directory, root) = relative_tempdir();
    let missing = root.join("missing");
    let file_path = root.join("file");
    std::fs::write(&file_path, "not a directory").expect("write file");

    assert_eq!(
        clean_path(
            &format!("{}:{}", missing.display(), file_path.display()),
            PlatformMode::Posix,
            None,
        ),
        format!("{}:{}", missing.display(), file_path.display())
    );
}

#[test]
fn clean_export_formats_bash_with_cleaned_raw_path() {
    assert_eq!(
        clean_export(
            "first::second:first",
            PlatformMode::Posix,
            None,
            ShellKind::Bash
        ),
        "export PATH='first:second'"
    );
}

#[test]
fn clean_export_formats_zsh_with_cleaned_raw_path() {
    assert_eq!(
        clean_export(
            "first::second:first",
            PlatformMode::Posix,
            None,
            ShellKind::Zsh
        ),
        "export PATH='first:second'"
    );
}

#[test]
fn clean_export_formats_fish_with_path_entries() {
    assert_eq!(
        clean_export(
            "first::second:first",
            PlatformMode::Posix,
            None,
            ShellKind::Fish
        ),
        "set -gx PATH 'first' 'second'"
    );
}

#[test]
fn clean_export_formats_powershell_with_cleaned_raw_path() {
    assert_eq!(
        clean_export(
            r"C:\First;C:\Second;C:\First",
            PlatformMode::Windows,
            None,
            ShellKind::Pwsh,
        ),
        r"$env:Path = 'C:\First;C:\Second'"
    );
}

#[test]
fn posix_single_quote_escapes_embedded_single_quotes() {
    assert_eq!(posix_single_quote("alpha'beta"), "'alpha'\\''beta'");
}

#[test]
fn fish_single_quote_escapes_backslashes_and_single_quotes() {
    assert_eq!(
        fish_single_quote(r"alpha\beta'gamma"),
        r#"'alpha\\beta\'gamma'"#
    );
}

#[test]
fn powershell_single_quote_escapes_embedded_single_quotes() {
    assert_eq!(powershell_single_quote("alpha'beta"), "'alpha''beta'");
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
