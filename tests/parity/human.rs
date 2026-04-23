use patholog::cli::run;

use crate::support::{assert_success_with_golden, context, golden_text, make_executable};

#[test]
fn print_golden_output() {
    let directory = tempfile::tempdir().expect("create tempdir");
    let root = directory.path().join("print");
    let first = root.join("first");
    let second = root.join("second");
    std::fs::create_dir_all(&first).expect("create first");
    std::fs::create_dir(&second).expect("create second");

    let result = run(
        ["print", "--platform", "posix"],
        context(
            &format!("{}:{}", first.display(), second.display()),
            None,
            &root,
        ),
    );

    assert_success_with_golden(result.exit_code, &result.stderr);
    assert_eq!(
        result.stdout,
        golden_text("print.stdout").replace("{root}", &root.display().to_string())
    );
}

#[test]
fn doctor_golden_output() {
    let directory = tempfile::tempdir().expect("create tempdir");
    let root = directory.path().join("doctor");
    let keep = root.join("keep");
    std::fs::create_dir_all(&keep).expect("create keep");
    let file_path = root.join("file");
    std::fs::write(&file_path, "not a directory").expect("write file");
    let missing = root.join("missing");

    let result = run(
        ["doctor", "--platform", "posix"],
        context(
            &format!(
                "{}:{}:{}::{}",
                keep.display(),
                missing.display(),
                file_path.display(),
                keep.display()
            ),
            None,
            &root,
        ),
    );

    assert_success_with_golden(result.exit_code, &result.stderr);
    assert_eq!(
        result.stdout,
        golden_text("doctor.stdout").replace("{root}", &root.display().to_string())
    );
}

#[test]
fn why_golden_output() {
    let directory = tempfile::tempdir().expect("create tempdir");
    let root = directory.path().join("why");
    let first = root.join("first");
    let second = root.join("second");
    make_executable(&first.join("tool"));
    make_executable(&second.join("tool"));

    let result = run(
        ["why", "tool", "--platform", "posix"],
        context(
            &format!("{}:{}", first.display(), second.display()),
            None,
            &root,
        ),
    );

    assert_success_with_golden(result.exit_code, &result.stderr);
    assert_eq!(
        result.stdout,
        golden_text("why.stdout").replace("{root}", &root.display().to_string())
    );
}

#[test]
fn conflicts_golden_output() {
    let directory = tempfile::tempdir().expect("create tempdir");
    let root = directory.path().join("conflicts");
    let first = root.join("first");
    let second = root.join("second");
    make_executable(&first.join("tool"));
    make_executable(&second.join("tool"));

    let result = run(
        ["conflicts", "tool", "--platform", "posix"],
        context(
            &format!("{}:{}", first.display(), second.display()),
            None,
            &root,
        ),
    );

    assert_success_with_golden(result.exit_code, &result.stderr);
    assert_eq!(
        result.stdout,
        golden_text("conflicts.stdout").replace("{root}", &root.display().to_string())
    );
}
