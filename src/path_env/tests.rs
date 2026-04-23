use crate::model::PlatformMode;

use super::parse_path;

#[test]
fn posix_path_parsing_preserves_order() {
    let directory = relative_tempdir();
    let first = directory.path().join("first");
    let second = directory.path().join("second");
    std::fs::create_dir(&first).expect("create first");
    std::fs::create_dir(&second).expect("create second");

    let entries = parse_path(
        &format!("{}:{}", first.display(), second.display()),
        PlatformMode::Posix,
        None,
    );

    assert_eq!(
        entries
            .iter()
            .map(|entry| entry.raw.as_str())
            .collect::<Vec<_>>(),
        [first.display().to_string(), second.display().to_string()]
    );
}

#[test]
fn windows_path_parsing_uses_semicolon_and_case_insensitive_keys() {
    let directory = tempfile::tempdir().expect("create tempdir");
    let first = directory.path().join("Tools");
    std::fs::create_dir(&first).expect("create tools");
    let upper = first.display().to_string().to_uppercase();

    let entries = parse_path(
        &format!("{};{}", first.display(), upper),
        PlatformMode::Windows,
        None,
    );

    assert_eq!(entries[0].comparison_key, entries[1].comparison_key);
}

#[test]
fn empty_missing_and_non_directory_entries_are_classified() {
    let directory = relative_tempdir();
    let file_path = directory.path().join("file");
    std::fs::write(&file_path, "not a dir").expect("write file");
    let missing = directory.path().join("missing");

    let entries = parse_path(
        &format!(":{}:{}", missing.display(), file_path.display()),
        PlatformMode::Posix,
        None,
    );

    assert!(entries[0].is_empty);
    assert!(!entries[1].exists);
    assert!(entries[2].exists);
    assert!(!entries[2].is_dir);
}

fn relative_tempdir() -> tempfile::TempDir {
    tempfile::Builder::new()
        .prefix(".patholog-test-")
        .tempdir_in(".")
        .expect("create relative tempdir")
}
