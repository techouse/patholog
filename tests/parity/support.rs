use std::path::Path;

use patholog::cli::CommandContext;
use patholog::model::ExitCode;

pub(crate) fn assert_success_with_golden(exit_code: ExitCode, stderr: &str) {
    assert_eq!(exit_code, ExitCode::Success);
    assert_eq!(stderr, "");
}

pub(crate) fn context(path_value: &str, pathext: Option<&str>, cwd: &Path) -> CommandContext {
    CommandContext {
        path_value: path_value.to_owned(),
        pathext: pathext.map(str::to_owned),
        cwd: cwd.to_path_buf(),
    }
}

pub(crate) fn golden_text(name: &str) -> String {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("golden")
        .join(name);
    std::fs::read_to_string(&path).unwrap_or_else(|error| {
        panic!("read golden fixture {}: {error}", path.display());
    })
}

pub(crate) fn make_executable(path: &Path) {
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
