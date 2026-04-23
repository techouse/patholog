use patholog::cli::run;
use patholog::model::ExitCode;

use crate::support::{assert_success_with_golden, context, golden_text, make_executable};

#[test]
fn print_json_golden_output() {
    let directory = tempfile::tempdir().expect("create tempdir");
    let root = directory.path().join("print_json");
    let first = root.join("first");
    let second = root.join("second");
    std::fs::create_dir_all(&first).expect("create first");
    std::fs::create_dir(&second).expect("create second");

    let result = run(
        ["print", "--platform", "posix", "--json"],
        context(
            &format!("{}:{}", first.display(), second.display()),
            None,
            &root,
        ),
    );

    assert_success_with_golden(result.exit_code, &result.stderr);
    assert_eq!(
        result.stdout,
        golden_text("print.json").replace("{root}", &root.display().to_string())
    );
}

#[test]
fn doctor_json_golden_output() {
    let directory = tempfile::tempdir().expect("create tempdir");
    let root = directory.path().join("doctor_json");
    let missing = root.join("missing");

    let result = run(
        ["doctor", "--platform", "posix", "--json"],
        context(&missing.display().to_string(), None, &root),
    );

    assert_success_with_golden(result.exit_code, &result.stderr);
    assert_eq!(
        result.stdout,
        golden_text("doctor.json").replace("{root}", &root.display().to_string())
    );
}

#[test]
fn why_json_golden_output() {
    let directory = tempfile::tempdir().expect("create tempdir");
    let root = directory.path().join("why_json");
    let first = root.join("first");
    let second = root.join("second");
    make_executable(&first.join("tool"));
    make_executable(&second.join("tool"));

    let result = run(
        ["why", "tool", "--platform", "posix", "--json"],
        context(
            &format!("{}:{}", first.display(), second.display()),
            None,
            &root,
        ),
    );

    assert_success_with_golden(result.exit_code, &result.stderr);
    assert_eq!(
        result.stdout,
        golden_text("why.json").replace("{root}", &root.display().to_string())
    );
}

#[test]
fn conflicts_json_golden_output() {
    let directory = tempfile::tempdir().expect("create tempdir");
    let root = directory.path().join("conflicts_json");
    let first = root.join("first");
    let second = root.join("second");
    make_executable(&first.join("tool"));
    make_executable(&second.join("tool"));

    let result = run(
        ["conflicts", "tool", "--platform", "posix", "--json"],
        context(
            &format!("{}:{}", first.display(), second.display()),
            None,
            &root,
        ),
    );

    assert_success_with_golden(result.exit_code, &result.stderr);
    assert_eq!(
        result.stdout,
        golden_text("conflicts.json").replace("{root}", &root.display().to_string())
    );
}

#[test]
fn why_related_hint_json_golden_output() {
    let directory = tempfile::tempdir().expect("create tempdir");
    let root = directory.path().join("why_related_json");
    let bin = root.join("bin");
    make_executable(&bin.join("python3"));

    let result = run(
        ["why", "python", "--platform", "posix", "--json"],
        context(&bin.display().to_string(), None, &root),
    );

    assert_eq!(result.exit_code, ExitCode::CommandNotFound);
    assert_eq!(result.stderr, "");
    assert_eq!(
        result.stdout,
        golden_text("why_related.json").replace("{root}", &root.display().to_string())
    );
}
