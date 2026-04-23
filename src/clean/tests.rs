use crate::model::PlatformMode;

use super::clean_path;

#[test]
fn clean_path_removes_empty_entries_and_later_duplicates() {
    let directory = tempfile::tempdir().expect("create tempdir");
    let first = directory.path().join("first");
    let second = directory.path().join("second");
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
    let directory = tempfile::tempdir().expect("create tempdir");
    let missing = directory.path().join("missing");
    let file_path = directory.path().join("file");
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
