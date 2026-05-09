use assert_cmd::Command;

#[test]
fn binary_wrapper_writes_stdout_and_exits_successfully() {
    let mut command = Command::cargo_bin("patholog").expect("patholog binary");

    command
        .arg("--version")
        .assert()
        .success()
        .stdout("patholog 0.5.2\n");
}

#[test]
fn binary_wrapper_writes_stderr_and_preserves_exit_code() {
    let mut command = Command::cargo_bin("patholog").expect("patholog binary");

    command
        .arg("clean")
        .assert()
        .code(1)
        .stderr("patholog: clean requires --stdout or --export\n");
}
