use crate::model::{PathVariable, PlatformMode, PresetKind, ShellKind};
use crate::policy::PathPolicy;

use super::{
    clean_export, clean_export_with_policy, clean_path, clean_path_with_policy, fish_single_quote,
    posix_single_quote, powershell_single_quote,
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
fn clean_path_drops_unwanted_entries_before_deduping() {
    let policy = PathPolicy::new(&["first".to_owned()], &[], PathVariable::Path);

    assert_eq!(
        clean_path_with_policy(
            "first:second:first:third",
            PlatformMode::Posix,
            None,
            PathVariable::Path,
            &policy,
        ),
        "second:third"
    );
}

#[test]
fn clean_path_applies_fink_preset_drop_rules() {
    let policy = PathPolicy::new(&[], &[PresetKind::Fink], PathVariable::Path);

    assert_eq!(
        clean_path_with_policy(
            "/usr/bin:/sw/bin:/sw/sbin:/sw/share/man",
            PlatformMode::Posix,
            None,
            PathVariable::Path,
            &policy,
        ),
        "/usr/bin:/sw/share/man"
    );
}

#[test]
fn clean_path_applies_fink_manpath_preset_drop_rules() {
    let policy = PathPolicy::new(&[], &[PresetKind::Fink], PathVariable::Manpath);

    assert_eq!(
        clean_path_with_policy(
            "/usr/share/man:/sw/share/man::/usr/share/man",
            PlatformMode::Posix,
            None,
            PathVariable::Manpath,
            &policy,
        ),
        "/usr/share/man:"
    );
}

#[test]
fn clean_path_preserves_manpath_empty_default_placeholders() {
    assert_eq!(
        clean_path_with_policy(
            ":/usr/share/man::/opt/share/man:/usr/share/man:",
            PlatformMode::Posix,
            None,
            PathVariable::Manpath,
            &PathPolicy::default(),
        ),
        ":/usr/share/man::/opt/share/man:"
    );
}

#[test]
fn clean_path_preserves_manpath_empty_placeholders_after_drops() {
    let policy = PathPolicy::new(&["/drop/man".to_owned()], &[], PathVariable::Manpath);

    assert_eq!(
        clean_path_with_policy(
            "/keep/man:/drop/man::/keep/man",
            PlatformMode::Posix,
            None,
            PathVariable::Manpath,
            &policy,
        ),
        "/keep/man:"
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
fn clean_export_formats_manpath_with_variable_name() {
    assert_eq!(
        clean_export_with_policy(
            "/usr/share/man:/opt/share/man",
            PlatformMode::Posix,
            None,
            ShellKind::Bash,
            PathVariable::Manpath,
            &PathPolicy::default(),
        ),
        "export MANPATH='/usr/share/man:/opt/share/man'"
    );
    assert_eq!(
        clean_export_with_policy(
            "/usr/share/man:/opt/share/man",
            PlatformMode::Posix,
            None,
            ShellKind::Fish,
            PathVariable::Manpath,
            &PathPolicy::default(),
        ),
        "set -gx MANPATH '/usr/share/man' '/opt/share/man'"
    );
    assert_eq!(
        clean_export_with_policy(
            "/usr/share/man:/opt/share/man",
            PlatformMode::Posix,
            None,
            ShellKind::Pwsh,
            PathVariable::Manpath,
            &PathPolicy::default(),
        ),
        "$env:MANPATH = '/usr/share/man:/opt/share/man'"
    );
}

#[test]
fn clean_export_preserves_manpath_empty_default_placeholders() {
    assert_eq!(
        clean_export_with_policy(
            "/usr/share/man::/opt/share/man:/usr/share/man",
            PlatformMode::Posix,
            None,
            ShellKind::Bash,
            PathVariable::Manpath,
            &PathPolicy::default(),
        ),
        "export MANPATH='/usr/share/man::/opt/share/man'"
    );
    assert_eq!(
        clean_export_with_policy(
            "/usr/share/man::/opt/share/man:/usr/share/man",
            PlatformMode::Posix,
            None,
            ShellKind::Fish,
            PathVariable::Manpath,
            &PathPolicy::default(),
        ),
        "set -gx MANPATH '/usr/share/man' '' '/opt/share/man'"
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
