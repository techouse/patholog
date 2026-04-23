use patholog::cli::run;

use crate::support::{assert_success_with_golden, context, golden_text};

#[test]
fn clean_stdout_removes_empty_and_later_duplicates() {
    let directory = tempfile::tempdir().expect("create tempdir");
    let root = directory.path().join("clean");
    let first = root.join("first");
    let second = root.join("second");
    std::fs::create_dir_all(&first).expect("create first");
    std::fs::create_dir(&second).expect("create second");

    let result = run(
        ["clean", "--stdout", "--platform", "posix"],
        context(
            &format!(
                "{}:{}::{}",
                first.display(),
                second.display(),
                first.display()
            ),
            None,
            &root,
        ),
    );

    assert_success_with_golden(result.exit_code, &result.stderr);
    assert_eq!(
        result.stdout,
        golden_text("clean.stdout").replace("{root}", &root.display().to_string())
    );
}
