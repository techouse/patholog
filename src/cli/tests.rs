use std::path::Path;

use crate::model::{ExitCode, IssueKind};

use super::fail_on::parse_fail_on;
use super::{CommandContext, run};

#[test]
fn clean_requires_output_mode() {
    let result = run(["clean"], context("", None));

    assert_eq!(result.exit_code, ExitCode::GeneralError);
    assert_eq!(
        result.stderr,
        "patholog: clean requires --stdout or --export\n"
    );
}

#[test]
fn clean_stdout_output_is_unchanged() {
    let result = run(
        ["clean", "--stdout", "--platform", "posix"],
        context("first::second:first", None),
    );

    assert_eq!(result.exit_code, ExitCode::Success);
    assert_eq!(result.stdout, "first:second\n");
}

#[test]
fn clean_export_outputs_shell_snippets() {
    for (shell, expected_stdout) in [
        ("bash", "export PATH='first:second'\n"),
        ("zsh", "export PATH='first:second'\n"),
        ("fish", "set -gx PATH 'first' 'second'\n"),
        ("pwsh", "$env:Path = 'first:second'\n"),
    ] {
        let result = run(
            ["clean", "--export", "--shell", shell, "--platform", "posix"],
            context("first::second:first", None),
        );

        assert_eq!(result.exit_code, ExitCode::Success);
        assert_eq!(result.stdout, expected_stdout);
    }
}

#[test]
fn clean_export_quotes_posix_shell_paths() {
    let result = run(
        [
            "clean",
            "--export",
            "--shell",
            "bash",
            "--platform",
            "posix",
        ],
        context("dir one:weird'quote:back\\slash", None),
    );

    assert_eq!(result.exit_code, ExitCode::Success);
    assert_eq!(
        result.stdout,
        "export PATH='dir one:weird'\\''quote:back\\slash'\n"
    );
}

#[test]
fn clean_export_quotes_fish_paths() {
    let result = run(
        [
            "clean",
            "--export",
            "--shell",
            "fish",
            "--platform",
            "posix",
        ],
        context("dir one:weird'quote:back\\slash", None),
    );

    assert_eq!(result.exit_code, ExitCode::Success);
    assert_eq!(
        result.stdout,
        "set -gx PATH 'dir one' 'weird\\'quote' 'back\\\\slash'\n"
    );
}

#[test]
fn clean_export_requires_shell() {
    let result = run(["clean", "--export"], context("", None));

    assert_eq!(result.exit_code, ExitCode::GeneralError);
    assert_eq!(result.stderr, "patholog: clean --export requires --shell\n");
}

#[test]
fn clean_shell_requires_export() {
    let result = run(["clean", "--shell", "bash"], context("", None));

    assert_eq!(result.exit_code, ExitCode::GeneralError);
    assert_eq!(result.stderr, "patholog: clean --shell requires --export\n");
}

#[test]
fn clean_rejects_conflicting_output_modes() {
    let result = run(
        ["clean", "--stdout", "--export", "--shell", "bash"],
        context("", None),
    );

    assert_eq!(result.exit_code, ExitCode::GeneralError);
    assert_eq!(
        result.stderr,
        "patholog: clean output modes --stdout and --export are mutually exclusive\n"
    );
}

#[test]
fn completions_outputs_scripts_for_supported_shells() {
    for shell in ["bash", "fish", "pwsh", "zsh"] {
        let result = run(["completions", shell], context("", None));

        assert_eq!(result.exit_code, ExitCode::Success);
        assert!(result.stdout.contains("clean"));
        assert!(result.stdout.contains("completions"));
    }
}

#[test]
fn apply_requires_dry_run() {
    let result = run(["apply", "--shell", "zsh"], context("", None));

    assert_eq!(result.exit_code, ExitCode::GeneralError);
    assert_eq!(
        result.stderr,
        "patholog: apply requires --dry-run in v0.4.0\n"
    );
}

#[test]
fn apply_requires_shell() {
    let result = run(["apply", "--dry-run"], context("", None));

    assert_eq!(result.exit_code, ExitCode::GeneralError);
    assert_eq!(result.stderr, "patholog: apply requires --shell\n");
}

#[test]
fn apply_dry_run_uses_default_interactive_profiles() {
    let directory = tempfile::tempdir().expect("create tempdir");

    for (shell, expected_path) in [
        ("zsh", ".zshrc"),
        ("bash", ".bashrc"),
        ("fish", ".config/fish/config.fish"),
        (
            "pwsh",
            ".config/powershell/Microsoft.PowerShell_profile.ps1",
        ),
    ] {
        let result = run(
            [
                "apply",
                "--dry-run",
                "--shell",
                shell,
                "--platform",
                "posix",
            ],
            context_with_home("/a:/b", None, directory.path()),
        );

        assert_eq!(result.exit_code, ExitCode::Success);
        assert!(
            result
                .stdout
                .contains(&directory.path().join(expected_path).display().to_string())
        );
    }
}

#[test]
fn apply_dry_run_windows_pwsh_uses_user_profile_dir() {
    let directory = tempfile::tempdir().expect("create tempdir");
    let home = directory.path().join("home");
    let user_profile = directory.path().join("userprofile");

    let result = run(
        [
            "apply",
            "--dry-run",
            "--shell",
            "pwsh",
            "--platform",
            "windows",
        ],
        context_with_home_dirs(r"C:\A;C:\B", None, &home, &user_profile),
    );

    assert_eq!(result.exit_code, ExitCode::Success);
    assert!(
        result.stdout.contains(
            &user_profile
                .join("Documents/PowerShell/Microsoft.PowerShell_profile.ps1")
                .display()
                .to_string()
        )
    );
}

#[test]
fn apply_profile_overrides_home() {
    let directory = tempfile::tempdir().expect("create tempdir");
    let home = directory.path().join("home");
    let profile = directory.path().join("custom.profile");

    let result = run(
        [
            "apply",
            "--dry-run",
            "--shell",
            "bash",
            "--home",
            home.to_str().expect("utf-8 home"),
            "--profile",
            profile.to_str().expect("utf-8 profile"),
        ],
        context("", None),
    );

    assert_eq!(result.exit_code, ExitCode::Success);
    assert!(result.stdout.contains(&profile.display().to_string()));
    assert!(
        !result
            .stdout
            .contains(&home.join(".bashrc").display().to_string())
    );
}

#[test]
fn apply_dry_run_reports_create_append_and_replace_actions() {
    let directory = tempfile::tempdir().expect("create tempdir");
    let append_profile = directory.path().join("append.profile");
    let replace_profile = directory.path().join("replace.profile");
    std::fs::write(&append_profile, "export PATH=\"$HOME/bin:$PATH\"\n").expect("write profile");
    std::fs::write(
        &replace_profile,
        "# >>> patholog PATH >>>\nexport PATH='/old'\n# <<< patholog PATH <<<\n",
    )
    .expect("write profile");

    for (profile, expected_action) in [
        (directory.path().join("missing.profile"), "create_profile"),
        (append_profile, "append_block"),
        (replace_profile, "replace_block"),
    ] {
        let result = run(
            [
                "apply",
                "--dry-run",
                "--shell",
                "bash",
                "--profile",
                profile.to_str().expect("utf-8 profile"),
            ],
            context("/a:/b:/a", None),
        );

        assert_eq!(result.exit_code, ExitCode::Success);
        assert!(result.stdout.contains(expected_action));
        assert!(result.stdout.contains("# >>> patholog PATH >>>"));
        assert!(result.stdout.contains("export PATH='/a:/b'"));
    }
}

#[test]
fn apply_dry_run_json_includes_plan_fields() {
    let directory = tempfile::tempdir().expect("create tempdir");
    let result = run(
        [
            "apply",
            "--dry-run",
            "--shell",
            "zsh",
            "--json",
            "--home",
            directory.path().to_str().expect("utf-8 home"),
        ],
        context("/a:/b:/a", None),
    );

    assert_eq!(result.exit_code, ExitCode::Success);
    assert!(result.stdout.contains("\"action\": \"create_profile\""));
    assert!(result.stdout.contains("\"cleaned_path\": \"/a:/b\""));
    assert!(result.stdout.contains("\"existing_block\": null"));
    assert!(result.stdout.contains("\"would_write\": false"));
}

#[test]
fn apply_rejects_non_file_profile() {
    let directory = tempfile::tempdir().expect("create tempdir");
    let profile = directory.path().join("profile.d");
    std::fs::create_dir(&profile).expect("create profile dir");

    let result = run(
        [
            "apply",
            "--dry-run",
            "--shell",
            "bash",
            "--profile",
            profile.to_str().expect("utf-8 profile"),
        ],
        context("/a:/b", None),
    );

    assert_eq!(result.exit_code, ExitCode::GeneralError);
    assert!(result.stderr.contains("not a file"));
}

#[cfg(unix)]
#[test]
fn apply_rejects_unreadable_profile() {
    use std::os::unix::fs::PermissionsExt;

    let directory = tempfile::tempdir().expect("create tempdir");
    let profile = directory.path().join("profile");
    std::fs::write(&profile, "export PATH=\"$PATH\"\n").expect("write profile");
    let mut permissions = std::fs::metadata(&profile)
        .expect("read metadata")
        .permissions();
    permissions.set_mode(0o000);
    std::fs::set_permissions(&profile, permissions).expect("make unreadable");

    let result = run(
        [
            "apply",
            "--dry-run",
            "--shell",
            "bash",
            "--profile",
            profile.to_str().expect("utf-8 profile"),
        ],
        context("/a:/b", None),
    );

    let mut permissions = std::fs::metadata(&profile)
        .expect("read metadata")
        .permissions();
    permissions.set_mode(0o600);
    std::fs::set_permissions(&profile, permissions).expect("restore permissions");

    assert_eq!(result.exit_code, ExitCode::GeneralError);
    assert!(result.stderr.contains("not readable"));
}

#[test]
fn apply_rejects_malformed_or_duplicate_managed_blocks() {
    let directory = tempfile::tempdir().expect("create tempdir");
    let malformed = directory.path().join("malformed.profile");
    let duplicate = directory.path().join("duplicate.profile");
    std::fs::write(&malformed, "# >>> patholog PATH >>>\nexport PATH='/old'\n")
        .expect("write malformed");
    std::fs::write(
        &duplicate,
        "# >>> patholog PATH >>>\nexport PATH='/old'\n# <<< patholog PATH <<<\n# >>> patholog PATH >>>\nexport PATH='/old'\n# <<< patholog PATH <<<\n",
    )
    .expect("write duplicate");

    for profile in [malformed, duplicate] {
        let result = run(
            [
                "apply",
                "--dry-run",
                "--shell",
                "bash",
                "--profile",
                profile.to_str().expect("utf-8 profile"),
            ],
            context("/a:/b", None),
        );

        assert_eq!(result.exit_code, ExitCode::GeneralError);
        assert!(result.stderr.contains("managed block"));
    }
}

#[test]
fn doctor_fail_on_returns_diagnostics_found() {
    let directory = tempfile::tempdir().expect("create tempdir");
    let missing = directory.path().join("missing");

    let result = run(
        ["doctor", "--platform", "posix", "--fail-on=missing"],
        context(&missing.display().to_string(), None),
    );

    assert_eq!(result.exit_code, ExitCode::DiagnosticsFound);
    assert!(result.stdout.contains("Missing directories:"));
}

#[test]
fn doctor_command_reports_shadowed_candidates() {
    let directory = tempfile::tempdir().expect("create tempdir");
    let first = directory.path().join("first");
    let second = directory.path().join("second");
    make_executable(&first.join("tool.exe"));
    make_executable(&second.join("tool.exe"));

    let result = run(
        ["doctor", "--platform", "windows", "--command", "tool"],
        context_with_cwd(
            &format!("{};{}", first.display(), second.display()),
            None,
            directory.path(),
        ),
    );

    assert_eq!(result.exit_code, ExitCode::Success);
    assert!(result.stdout.contains("Shadowed commands:"));
    assert!(result.stdout.contains("tool at"));
}

#[test]
fn scan_reports_path_changes_from_home_profiles() {
    let directory = tempfile::tempdir().expect("create tempdir");
    let zshrc = directory.path().join(".zshrc");
    std::fs::write(&zshrc, "export PATH=\"$HOME/bin:$PATH\"\n").expect("write zshrc");

    let result = run(
        ["scan", "--platform", "posix"],
        context_with_home("", None, directory.path()),
    );

    assert_eq!(result.exit_code, ExitCode::Success);
    assert!(result.stdout.contains("PATH changes:"));
    assert!(result.stdout.contains(".zshrc"));
}

#[test]
fn scan_home_option_overrides_missing_context_home() {
    let directory = tempfile::tempdir().expect("create tempdir");
    let home = directory.path().display().to_string();
    let result = run(
        ["scan", "--platform", "posix", "--home", home.as_str()],
        context("", None),
    );

    assert_eq!(result.exit_code, ExitCode::Success);
    assert!(result.stdout.contains("No PATH changes found"));
}

#[test]
fn scan_windows_platform_prefers_userprofile_home() {
    let directory = tempfile::tempdir().expect("create tempdir");
    let home = directory.path().join("home");
    let user_profile = directory.path().join("userprofile");
    let profile = user_profile.join("Documents/PowerShell/profile.ps1");
    std::fs::create_dir_all(profile.parent().expect("profile has parent"))
        .expect("create profile parent");
    std::fs::write(&profile, "$env:Path = \"C:\\Tools;$env:Path\"\n").expect("write profile");

    let result = run(
        ["scan", "--platform", "windows"],
        context_with_home_dirs("", None, &home, &user_profile),
    );

    assert_eq!(result.exit_code, ExitCode::Success);
    assert!(result.stdout.contains("PATH changes:"));
    assert!(result.stdout.contains(&profile.display().to_string()));
}

#[test]
fn scan_requires_home_directory() {
    let result = run(["scan"], context("", None));

    assert_eq!(result.exit_code, ExitCode::GeneralError);
    assert_eq!(
        result.stderr,
        "patholog: scan requires a home directory; set HOME or pass --home\n"
    );
}

#[test]
fn version_uses_injected_stdout() {
    let result = run(["--version"], context("", None));

    assert_eq!(result.exit_code, ExitCode::Success);
    assert_eq!(result.stdout, "patholog 0.4.0\n");
    assert_eq!(result.stderr, "");
}

#[test]
fn help_uses_injected_stdout() {
    let result = run(["--help"], context("", None));

    assert_eq!(result.exit_code, ExitCode::Success);
    assert!(result.stdout.contains("Usage: patholog <COMMAND>"));
    assert_eq!(result.stderr, "");
}

#[test]
fn usage_error_maps_to_general_error() {
    let result = run(["bad"], context("", None));

    assert_eq!(result.exit_code, ExitCode::GeneralError);
    assert_eq!(result.stdout, "");
    assert!(result.stderr.contains("bad"));
}

#[test]
fn invalid_fail_on_returns_runtime_error() {
    let result = run(["doctor", "--fail-on=bogus"], context("", None));

    assert_eq!(result.exit_code, ExitCode::GeneralError);
    assert_eq!(result.stdout, "");
    assert!(result.stderr.contains("unsupported issue kind \"bogus\""));
    assert!(result.stderr.contains("unreadable"));
    assert!(result.stderr.contains("shadowed_command"));
}

#[test]
fn parse_fail_on_trims_ignores_empty_and_deduplicates() {
    assert_eq!(
        parse_fail_on(
            " duplicate, ,not_directory,unreadable,suspicious_order,shadowed_command,duplicate ",
        )
        .expect("parse fail-on"),
        [
            IssueKind::Duplicate,
            IssueKind::NotDirectory,
            IssueKind::Unreadable,
            IssueKind::SuspiciousOrder,
            IssueKind::ShadowedCommand
        ]
    );
}

#[test]
fn command_context_from_env_reads_process_context() {
    let context = CommandContext::from_env();

    assert!(context.cwd.is_dir() || context.cwd == Path::new("."));
}

fn context(path_value: &str, pathext: Option<&str>) -> CommandContext {
    context_with_cwd(path_value, pathext, Path::new("."))
}

fn context_with_cwd(path_value: &str, pathext: Option<&str>, cwd: &Path) -> CommandContext {
    CommandContext {
        path_value: path_value.to_owned(),
        pathext: pathext.map(str::to_owned),
        cwd: cwd.to_path_buf(),
        home_dir: None,
        user_profile_dir: None,
    }
}

fn context_with_home(path_value: &str, pathext: Option<&str>, home: &Path) -> CommandContext {
    CommandContext {
        path_value: path_value.to_owned(),
        pathext: pathext.map(str::to_owned),
        cwd: Path::new(".").to_path_buf(),
        home_dir: Some(home.to_path_buf()),
        user_profile_dir: None,
    }
}

fn context_with_home_dirs(
    path_value: &str,
    pathext: Option<&str>,
    home: &Path,
    user_profile: &Path,
) -> CommandContext {
    CommandContext {
        path_value: path_value.to_owned(),
        pathext: pathext.map(str::to_owned),
        cwd: Path::new(".").to_path_buf(),
        home_dir: Some(home.to_path_buf()),
        user_profile_dir: Some(user_profile.to_path_buf()),
    }
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
