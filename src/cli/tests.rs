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
fn clean_stdout_drops_unwanted_entries() {
    let result = run(
        [
            "clean",
            "--stdout",
            "--platform",
            "posix",
            "--drop",
            "first",
        ],
        context("first::second:first:third", None),
    );

    assert_eq!(result.exit_code, ExitCode::Success);
    assert_eq!(result.stdout, "second:third\n");
}

#[test]
fn clean_stdout_applies_repeated_fink_preset_once() {
    let result = run(
        [
            "clean",
            "--stdout",
            "--platform",
            "posix",
            "--preset",
            "fink",
            "--preset",
            "fink",
        ],
        context("/usr/bin:/sw/bin:/sw/sbin:/sw/share/man", None),
    );

    assert_eq!(result.exit_code, ExitCode::Success);
    assert_eq!(result.stdout, "/usr/bin:/sw/share/man\n");
}

#[test]
fn clean_stdout_combines_drop_and_preset_policy() {
    let result = run(
        [
            "clean",
            "--stdout",
            "--platform",
            "posix",
            "--drop",
            "/custom/bin",
            "--preset",
            "fink",
        ],
        context("/custom/bin:/sw/bin:/usr/bin", None),
    );

    assert_eq!(result.exit_code, ExitCode::Success);
    assert_eq!(result.stdout, "/usr/bin\n");
}

#[test]
fn clean_stdout_applies_config_policy_before_cli_policy() {
    let directory = tempfile::tempdir().expect("create tempdir");
    let config = write_config(
        directory.path(),
        "version = 1\n[path]\ndrop = [\"config-drop\"]\n",
    );

    let result = run(
        [
            "clean",
            "--stdout",
            "--platform",
            "posix",
            "--config",
            config.to_str().expect("utf-8 config"),
            "--drop",
            "cli-drop",
        ],
        context("config-drop:cli-drop:keep", None),
    );

    assert_eq!(result.exit_code, ExitCode::Success);
    assert_eq!(result.stdout, "keep\n");
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
fn clean_export_outputs_manpath_shell_snippet() {
    let result = run(
        [
            "clean",
            "--export",
            "--var",
            "manpath",
            "--shell",
            "bash",
            "--platform",
            "posix",
        ],
        context_with_manpath("", "/usr/share/man::/opt/share/man:/usr/share/man", None),
    );

    assert_eq!(result.exit_code, ExitCode::Success);
    assert_eq!(
        result.stdout,
        "export MANPATH='/usr/share/man::/opt/share/man'\n"
    );
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
fn clean_export_manpath_applies_config_policy() {
    let directory = tempfile::tempdir().expect("create tempdir");
    let config = write_config(
        directory.path(),
        "version = 1\n[manpath]\ndrop = [\"/drop/man\"]\n",
    );

    let result = run(
        [
            "clean",
            "--export",
            "--var",
            "manpath",
            "--shell",
            "bash",
            "--platform",
            "posix",
            "--config",
            config.to_str().expect("utf-8 config"),
        ],
        context_with_manpath("", "/drop/man:/keep/man", None),
    );

    assert_eq!(result.exit_code, ExitCode::Success);
    assert_eq!(result.stdout, "export MANPATH='/keep/man'\n");
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
        assert!(result.stdout.contains("config"));
        assert!(result.stdout.contains("completions"));
        assert!(result.stdout.contains("health"));
        assert!(result.stdout.contains("why-not"));
        assert!(result.stdout.contains("yes"));
        assert!(result.stdout.contains("no-backup"));
    }
}

#[test]
fn health_clean_path_returns_success() {
    let (directory, root) = relative_tempdir();
    let bin = root.join("bin");
    std::fs::create_dir_all(&bin).expect("create bin");

    let result = run(
        ["health", "--platform", "posix"],
        context_with_home(&bin.display().to_string(), None, directory.path()),
    );

    assert_eq!(result.exit_code, ExitCode::Success);
    assert!(result.stdout.contains("PATH health: 100/100"));
    assert!(result.stdout.contains("Status: healthy"));
}

#[test]
fn health_unhealthy_path_still_returns_success() {
    let (directory, root) = relative_tempdir();
    let missing = root.join("missing");
    let file = root.join("not-dir");
    std::fs::write(&file, "not a directory").expect("write file");

    let result = run(
        ["health", "--platform", "posix"],
        context_with_home(
            &format!("{}:{}:", missing.display(), file.display()),
            None,
            directory.path(),
        ),
    );

    assert_eq!(result.exit_code, ExitCode::Success);
    assert!(result.stdout.contains("PATH health:"));
    assert!(result.stdout.contains("Status: issues found"));
    assert!(result.stdout.contains("missing"));
    assert!(result.stdout.contains("not_directory"));
    assert!(result.stdout.contains("empty"));
}

#[test]
fn health_json_includes_stable_fields() {
    let (directory, root) = relative_tempdir();
    let missing = root.join("missing");

    let result = run(
        ["health", "--platform", "posix", "--json"],
        context_with_home(&missing.display().to_string(), None, directory.path()),
    );

    assert_eq!(result.exit_code, ExitCode::Success);
    assert!(result.stdout.contains("\"variable\": \"path\""));
    assert!(result.stdout.contains("\"score\": 85"));
    assert!(result.stdout.contains("\"healthy\": false"));
    assert!(result.stdout.contains("\"entry_count\": 1"));
    assert!(result.stdout.contains("\"issue_count\": 1"));
    assert!(result.stdout.contains("\"worst_severity\": \"error\""));
    assert!(result.stdout.contains("\"counts\": {"));
    assert!(result.stdout.contains("\"missing\": 1"));
    assert!(result.stdout.contains("\"diagnostics\": ["));
}

#[test]
fn health_var_manpath_reports_manpath() {
    let (directory, root) = relative_tempdir();
    let manpath = root.join("share").join("man");
    std::fs::create_dir_all(&manpath).expect("create manpath");

    let result = run(
        ["health", "--var", "manpath", "--platform", "posix"],
        context_with_manpath("", &manpath.display().to_string(), None),
    );

    assert_eq!(result.exit_code, ExitCode::Success);
    assert!(result.stdout.contains("MANPATH health: 100/100"));
    drop(directory);
}

#[test]
fn health_drop_reports_unwanted_entries() {
    let (directory, root) = relative_tempdir();
    let drop_entry = root.join("drop");
    let keep_entry = root.join("keep");
    std::fs::create_dir_all(&drop_entry).expect("create drop entry");
    std::fs::create_dir_all(&keep_entry).expect("create keep entry");

    let result = run(
        [
            "health",
            "--platform",
            "posix",
            "--drop",
            drop_entry.to_str().expect("utf-8 path"),
        ],
        context_with_home(
            &format!("{}:{}", drop_entry.display(), keep_entry.display()),
            None,
            directory.path(),
        ),
    );

    assert_eq!(result.exit_code, ExitCode::Success);
    assert!(result.stdout.contains("Worst severity: warning"));
    assert!(result.stdout.contains("unwanted  1"));
}

#[test]
fn health_preset_reports_unwanted_entries() {
    let result = run(
        ["health", "--platform", "posix", "--preset", "fink"],
        context("/sw/bin:/usr/bin", None),
    );

    assert_eq!(result.exit_code, ExitCode::Success);
    assert!(result.stdout.contains("unwanted  1"));
}

#[test]
fn health_config_applies_policy_without_fail_on_exit_code() {
    let directory = tempfile::tempdir().expect("create tempdir");
    let config = write_config(
        directory.path(),
        "version = 1\n[path]\ndrop = [\"config-drop\"]\nfail_on = [\"unwanted\"]\n",
    );

    let result = run(
        [
            "health",
            "--platform",
            "posix",
            "--config",
            config.to_str().expect("utf-8 config"),
        ],
        context("config-drop:keep", None),
    );

    assert_eq!(result.exit_code, ExitCode::Success);
    assert!(result.stdout.contains("unwanted  1"));
}

#[test]
fn health_config_auto_applies_local_policy() {
    let directory = tempfile::tempdir().expect("create tempdir");
    write_config(
        directory.path(),
        "version = 1\n[path]\ndrop = [\"auto-drop\"]\n",
    );

    let result = run(
        ["health", "--platform", "posix", "--config", "auto"],
        context_with_cwd("auto-drop:keep", None, directory.path()),
    );

    assert_eq!(result.exit_code, ExitCode::Success);
    assert!(result.stdout.contains("unwanted  1"));
}

#[test]
fn why_not_found_command_reports_available() {
    let (directory, root) = relative_tempdir();
    let bin = root.join("bin");
    make_executable(&bin.join("tool"));

    let result = run(
        ["why-not", "tool", "--platform", "posix"],
        context_with_home(&bin.display().to_string(), None, directory.path()),
    );

    assert_eq!(result.exit_code, ExitCode::Success);
    assert!(result.stdout.contains("Available in PATH:"));
    assert!(
        result
            .stdout
            .contains("The exact command is already available.")
    );
}

#[test]
fn why_not_missing_command_reports_searched_directories_and_related_hints() {
    let (directory, root) = relative_tempdir();
    let bin = root.join("bin");
    make_executable(&bin.join("python3"));

    let result = run(
        ["why-not", "python", "--platform", "posix"],
        context_with_home(&bin.display().to_string(), None, directory.path()),
    );

    assert_eq!(result.exit_code, ExitCode::CommandNotFound);
    assert!(result.stdout.contains("Not found in PATH."));
    assert!(result.stdout.contains("Searched directories:"));
    assert!(result.stdout.contains(&bin.display().to_string()));
    assert!(
        result
            .stdout
            .contains("Related executables, not PATH matches:")
    );
    assert!(result.stdout.contains("python3"));
}

#[test]
fn why_not_missing_command_reports_path_diagnostics() {
    let (directory, root) = relative_tempdir();
    let missing = root.join("missing");
    let file = root.join("not-dir");
    std::fs::write(&file, "not a directory").expect("write file");

    let result = run(
        ["why-not", "tool", "--platform", "posix"],
        context_with_home(
            &format!("{}:{}:", missing.display(), file.display()),
            None,
            directory.path(),
        ),
    );

    assert_eq!(result.exit_code, ExitCode::CommandNotFound);
    assert!(result.stdout.contains("PATH diagnostics:"));
    assert!(result.stdout.contains("missing"));
    assert!(result.stdout.contains("not_directory"));
    assert!(result.stdout.contains("empty"));
}

#[test]
fn why_not_json_includes_stable_fields() {
    let (directory, root) = relative_tempdir();
    let missing = root.join("missing");

    let result = run(
        ["why-not", "tool", "--platform", "posix", "--json"],
        context_with_home(&missing.display().to_string(), None, directory.path()),
    );

    assert_eq!(result.exit_code, ExitCode::CommandNotFound);
    assert!(result.stdout.contains("\"command\": \"tool\""));
    assert!(result.stdout.contains("\"found\": false"));
    assert!(result.stdout.contains("\"winner\": null"));
    assert!(result.stdout.contains("\"candidates\": []"));
    assert!(result.stdout.contains("\"searched_directories\": ["));
    assert!(result.stdout.contains("\"related_hints\": []"));
    assert!(result.stdout.contains("\"path_diagnostics\": ["));
    assert!(result.stdout.contains("\"advice\": ["));
}

#[test]
fn apply_requires_output_mode() {
    let result = run(["apply", "--shell", "zsh"], context("", None));

    assert_eq!(result.exit_code, ExitCode::GeneralError);
    assert_eq!(
        result.stderr,
        "patholog: apply requires --dry-run or --yes\n"
    );
}

#[test]
fn apply_rejects_conflicting_modes() {
    let result = run(
        ["apply", "--dry-run", "--yes", "--shell", "zsh"],
        context("", None),
    );

    assert_eq!(result.exit_code, ExitCode::GeneralError);
    assert_eq!(
        result.stderr,
        "patholog: apply cannot use --dry-run with --yes\n"
    );
}

#[test]
fn apply_no_backup_requires_yes() {
    let result = run(
        ["apply", "--dry-run", "--no-backup", "--shell", "zsh"],
        context("", None),
    );

    assert_eq!(result.exit_code, ExitCode::GeneralError);
    assert_eq!(
        result.stderr,
        "patholog: apply --no-backup requires --yes\n"
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
fn apply_dry_run_windows_posix_shells_use_home_dir() {
    let directory = tempfile::tempdir().expect("create tempdir");
    let home = directory.path().join("home");
    let user_profile = directory.path().join("userprofile");

    for (shell, expected_path) in [
        ("bash", ".bashrc"),
        ("fish", ".config/fish/config.fish"),
        ("zsh", ".zshrc"),
    ] {
        let result = run(
            [
                "apply",
                "--dry-run",
                "--shell",
                shell,
                "--platform",
                "windows",
            ],
            context_with_home_dirs(r"C:\A;C:\B", None, &home, &user_profile),
        );

        assert_eq!(result.exit_code, ExitCode::Success);
        assert!(
            result
                .stdout
                .contains(&home.join(expected_path).display().to_string())
        );
        assert!(
            !result
                .stdout
                .contains(&user_profile.join(expected_path).display().to_string())
        );
    }
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
                "--platform",
                "posix",
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
fn apply_dry_run_uses_drop_policy_in_cleaned_block() {
    let directory = tempfile::tempdir().expect("create tempdir");
    let profile = directory.path().join("missing.profile");

    let result = run(
        [
            "apply",
            "--dry-run",
            "--shell",
            "bash",
            "--platform",
            "posix",
            "--profile",
            profile.to_str().expect("utf-8 profile"),
            "--drop",
            "/a",
        ],
        context("/a:/b:/a", None),
    );

    assert_eq!(result.exit_code, ExitCode::Success);
    assert!(result.stdout.contains("Cleaned PATH:\n  /b"));
    assert!(result.stdout.contains("export PATH='/b'"));
}

#[test]
fn apply_dry_run_uses_fink_preset_drop_policy() {
    let directory = tempfile::tempdir().expect("create tempdir");
    let profile = directory.path().join("missing.profile");

    let result = run(
        [
            "apply",
            "--dry-run",
            "--shell",
            "bash",
            "--platform",
            "posix",
            "--profile",
            profile.to_str().expect("utf-8 profile"),
            "--preset",
            "fink",
        ],
        context("/usr/bin:/sw/bin:/sw/sbin", None),
    );

    assert_eq!(result.exit_code, ExitCode::Success);
    assert!(result.stdout.contains("Cleaned PATH:\n  /usr/bin"));
    assert!(result.stdout.contains("export PATH='/usr/bin'"));
}

#[test]
fn apply_dry_run_uses_config_path_policy() {
    let directory = tempfile::tempdir().expect("create tempdir");
    let profile = directory.path().join("missing.profile");
    let config = write_config(
        directory.path(),
        "version = 1\n[path]\ndrop = [\"/drop\"]\n[manpath]\ndrop = [\"/keep\"]\n",
    );

    let result = run(
        [
            "apply",
            "--dry-run",
            "--shell",
            "bash",
            "--platform",
            "posix",
            "--profile",
            profile.to_str().expect("utf-8 profile"),
            "--config",
            config.to_str().expect("utf-8 config"),
        ],
        context("/drop:/keep", None),
    );

    assert_eq!(result.exit_code, ExitCode::Success);
    assert!(result.stdout.contains("Cleaned PATH:\n  /keep"));
    assert!(result.stdout.contains("export PATH='/keep'"));
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
            "--platform",
            "posix",
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
fn apply_yes_creates_profile_without_backup() {
    let directory = tempfile::tempdir().expect("create tempdir");
    let profile = directory.path().join("nested/.zshrc");

    let result = run(
        [
            "apply",
            "--yes",
            "--shell",
            "zsh",
            "--platform",
            "posix",
            "--profile",
            profile.to_str().expect("utf-8 profile"),
        ],
        context("/a:/b:/a", None),
    );

    assert_eq!(result.exit_code, ExitCode::Success);
    assert!(result.stdout.contains("Apply: zsh"));
    assert!(result.stdout.contains("Wrote:\n  true"));
    assert!(result.stdout.contains("Backup:\n  none"));
    assert_eq!(
        std::fs::read_to_string(&profile).expect("read profile"),
        "# >>> patholog PATH >>>\nexport PATH='/a:/b'\n# <<< patholog PATH <<<\n"
    );
    assert!(backup_paths_for(&profile).is_empty());
}

#[test]
fn apply_yes_uses_config_path_policy() {
    let directory = tempfile::tempdir().expect("create tempdir");
    let profile = directory.path().join(".zshrc");
    let config = write_config(
        directory.path(),
        "version = 1\n[path]\ndrop = [\"/drop\"]\n",
    );

    let result = run(
        [
            "apply",
            "--yes",
            "--shell",
            "zsh",
            "--platform",
            "posix",
            "--profile",
            profile.to_str().expect("utf-8 profile"),
            "--config",
            config.to_str().expect("utf-8 config"),
        ],
        context("/drop:/keep", None),
    );

    assert_eq!(result.exit_code, ExitCode::Success);
    assert_eq!(
        std::fs::read_to_string(&profile).expect("read profile"),
        "# >>> patholog PATH >>>\nexport PATH='/keep'\n# <<< patholog PATH <<<\n"
    );
}

#[test]
fn apply_yes_appends_block_and_creates_backup_by_default() {
    let directory = tempfile::tempdir().expect("create tempdir");
    let profile = directory.path().join(".bashrc");
    let original = "export PATH=\"$HOME/bin:$PATH\"\n";
    std::fs::write(&profile, original).expect("write profile");

    let result = run(
        [
            "apply",
            "--yes",
            "--shell",
            "bash",
            "--platform",
            "posix",
            "--profile",
            profile.to_str().expect("utf-8 profile"),
        ],
        context("/a:/b:/a", None),
    );

    assert_eq!(result.exit_code, ExitCode::Success);
    assert!(result.stdout.contains("Action:\n  append_block"));
    assert!(result.stdout.contains("Backup:\n  "));
    assert_eq!(
        std::fs::read_to_string(&profile).expect("read profile"),
        "export PATH=\"$HOME/bin:$PATH\"\n\n# >>> patholog PATH >>>\nexport PATH='/a:/b'\n# <<< patholog PATH <<<\n"
    );
    let backups = backup_paths_for(&profile);
    assert_eq!(backups.len(), 1);
    assert_eq!(
        std::fs::read_to_string(&backups[0]).expect("read backup"),
        original
    );
}

#[test]
fn apply_yes_replaces_block_and_creates_backup_by_default() {
    let directory = tempfile::tempdir().expect("create tempdir");
    let profile = directory.path().join(".bashrc");
    let original =
        "before\n# >>> patholog PATH >>>\nexport PATH='/old'\n# <<< patholog PATH <<<\nafter\n";
    std::fs::write(&profile, original).expect("write profile");

    let result = run(
        [
            "apply",
            "--yes",
            "--shell",
            "bash",
            "--platform",
            "posix",
            "--profile",
            profile.to_str().expect("utf-8 profile"),
        ],
        context("/a:/b:/a", None),
    );

    assert_eq!(result.exit_code, ExitCode::Success);
    assert!(result.stdout.contains("Action:\n  replace_block"));
    assert_eq!(
        std::fs::read_to_string(&profile).expect("read profile"),
        "before\n# >>> patholog PATH >>>\nexport PATH='/a:/b'\n# <<< patholog PATH <<<\nafter\n"
    );
    let backups = backup_paths_for(&profile);
    assert_eq!(backups.len(), 1);
    assert_eq!(
        std::fs::read_to_string(&backups[0]).expect("read backup"),
        original
    );
}

#[test]
fn apply_yes_no_backup_writes_without_backup() {
    let directory = tempfile::tempdir().expect("create tempdir");
    let profile = directory.path().join(".bashrc");
    std::fs::write(&profile, "before\n").expect("write profile");

    let result = run(
        [
            "apply",
            "--yes",
            "--no-backup",
            "--shell",
            "bash",
            "--platform",
            "posix",
            "--profile",
            profile.to_str().expect("utf-8 profile"),
        ],
        context("/a:/b:/a", None),
    );

    assert_eq!(result.exit_code, ExitCode::Success);
    assert!(result.stdout.contains("Backup:\n  disabled"));
    assert!(backup_paths_for(&profile).is_empty());
}

#[test]
fn apply_yes_json_includes_write_fields() {
    let directory = tempfile::tempdir().expect("create tempdir");
    let profile = directory.path().join(".zshrc");

    let result = run(
        [
            "apply",
            "--yes",
            "--shell",
            "zsh",
            "--platform",
            "posix",
            "--json",
            "--profile",
            profile.to_str().expect("utf-8 profile"),
        ],
        context("/a:/b:/a", None),
    );

    assert_eq!(result.exit_code, ExitCode::Success);
    assert!(result.stdout.contains("\"would_write\": true"));
    assert!(result.stdout.contains("\"wrote\": true"));
    assert!(result.stdout.contains("\"backup_created\": false"));
    assert!(result.stdout.contains("\"backup_path\": null"));
}

#[test]
fn print_can_read_manpath() {
    let result = run(
        ["print", "--var", "manpath", "--platform", "posix"],
        context_with_manpath("", "/usr/share/man:/opt/share/man", None),
    );

    assert_eq!(result.exit_code, ExitCode::Success);
    assert_eq!(result.stdout, "1  /usr/share/man\n2  /opt/share/man\n");
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

#[cfg(unix)]
#[test]
fn apply_dry_run_rejects_symlink_profile() {
    use std::os::unix::fs::symlink;

    let directory = tempfile::tempdir().expect("create tempdir");
    let target = directory.path().join("real.profile");
    let profile = directory.path().join("profile");
    std::fs::write(&target, "export PATH=\"$PATH\"\n").expect("write target");
    symlink(&target, &profile).expect("create profile symlink");

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
    assert!(result.stderr.contains("symlink"));
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
fn apply_yes_rejects_malformed_block_without_writing() {
    let directory = tempfile::tempdir().expect("create tempdir");
    let profile = directory.path().join("malformed.profile");
    let original = "# >>> patholog PATH >>>\nexport PATH='/old'\n";
    std::fs::write(&profile, original).expect("write malformed");

    let result = run(
        [
            "apply",
            "--yes",
            "--shell",
            "bash",
            "--profile",
            profile.to_str().expect("utf-8 profile"),
        ],
        context("/a:/b", None),
    );

    assert_eq!(result.exit_code, ExitCode::GeneralError);
    assert!(result.stderr.contains("managed block"));
    assert_eq!(
        std::fs::read_to_string(&profile).expect("read profile"),
        original
    );
    assert!(backup_paths_for(&profile).is_empty());
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
fn doctor_reports_unwanted_entries_and_fail_on() {
    let result = run(
        [
            "doctor",
            "--platform",
            "posix",
            "--drop",
            "drop",
            "--fail-on=unwanted",
        ],
        context("drop:keep", None),
    );

    assert_eq!(result.exit_code, ExitCode::DiagnosticsFound);
    assert!(result.stdout.contains("Unwanted entries:"));
    assert!(result.stdout.contains("drop"));
}

#[test]
fn doctor_config_reports_unwanted_entries_and_fail_on() {
    let directory = tempfile::tempdir().expect("create tempdir");
    let config = write_config(
        directory.path(),
        "version = 1\n[path]\ndrop = [\"drop\"]\nfail_on = [\"unwanted\"]\n",
    );

    let result = run(
        [
            "doctor",
            "--platform",
            "posix",
            "--config",
            config.to_str().expect("utf-8 config"),
        ],
        context("drop:keep", None),
    );

    assert_eq!(result.exit_code, ExitCode::DiagnosticsFound);
    assert!(result.stdout.contains("Unwanted entries:"));
    assert!(result.stdout.contains("drop"));
}

#[test]
fn doctor_config_fail_on_combines_with_cli_fail_on() {
    let directory = tempfile::tempdir().expect("create tempdir");
    let missing = directory.path().join("missing");
    let config = write_config(
        directory.path(),
        "version = 1\n[path]\nfail_on = [\"unwanted\"]\n",
    );

    let result = run(
        [
            "doctor",
            "--platform",
            "posix",
            "--config",
            config.to_str().expect("utf-8 config"),
            "--fail-on=missing",
        ],
        context(&missing.display().to_string(), None),
    );

    assert_eq!(result.exit_code, ExitCode::DiagnosticsFound);
    assert!(result.stdout.contains("Missing directories:"));
}

#[test]
fn doctor_config_var_manpath_uses_manpath_section() {
    let directory = tempfile::tempdir().expect("create tempdir");
    let config = write_config(
        directory.path(),
        "version = 1\n[path]\ndrop = [\"path-drop\"]\n[manpath]\ndrop = [\"man-drop\"]\n",
    );

    let result = run(
        [
            "doctor",
            "--var",
            "manpath",
            "--platform",
            "posix",
            "--config",
            config.to_str().expect("utf-8 config"),
        ],
        context_with_manpath("path-drop", "man-drop:keep", None),
    );

    assert_eq!(result.exit_code, ExitCode::Success);
    assert!(result.stdout.contains("Unwanted entries:"));
    assert!(result.stdout.contains("man-drop"));
    assert!(!result.stdout.contains("path-drop"));
}

#[test]
fn doctor_config_command_uses_path_section() {
    let directory = tempfile::tempdir().expect("create tempdir");
    let bin = directory.path().join("bin");
    std::fs::create_dir(&bin).expect("create bin");
    let config = write_config(
        directory.path(),
        "version = 1\n[path]\ndrop = [\"drop\"]\n[manpath]\ndrop = [\"man-drop\"]\n",
    );

    let result = run(
        [
            "doctor",
            "--command",
            "tool",
            "--platform",
            "posix",
            "--config",
            config.to_str().expect("utf-8 config"),
        ],
        context(&format!("drop:{}", bin.display()), None),
    );

    assert_eq!(result.exit_code, ExitCode::Success);
    assert!(result.stdout.contains("Unwanted entries:"));
    assert!(result.stdout.contains("drop"));
    assert!(!result.stdout.contains("man-drop"));
}

#[test]
fn doctor_var_manpath_reports_manpath_entries() {
    let result = run(
        ["doctor", "--var", "manpath", "--platform", "posix"],
        context_with_manpath("", "man:man", None),
    );

    assert_eq!(result.exit_code, ExitCode::Success);
    assert!(result.stdout.contains("MANPATH entries: 2"));
    assert!(result.stdout.contains("Duplicates:"));
}

#[test]
fn doctor_var_manpath_rejects_command_resolution() {
    let result = run(
        ["doctor", "--var", "manpath", "--command", "tool"],
        context_with_manpath("", "/usr/share/man", None),
    );

    assert_eq!(result.exit_code, ExitCode::GeneralError);
    assert_eq!(
        result.stderr,
        "patholog: doctor --command only supports --var path\n"
    );
}

#[test]
fn doctor_var_manpath_does_not_fail_on_path_ordering() {
    let result = run(
        [
            "doctor",
            "--var",
            "manpath",
            "--platform",
            "posix",
            "--fail-on=suspicious_order",
        ],
        context_with_manpath("", "/bin:/Users/me/.cargo/bin", None),
    );

    assert_eq!(result.exit_code, ExitCode::Success);
    assert!(!result.stdout.contains("Ordering warnings:"));
}

#[test]
fn doctor_fink_preset_reports_unwanted_entries() {
    let result = run(
        ["doctor", "--platform", "posix", "--preset", "fink"],
        context("/usr/bin:/sw/bin:/sw/sbin", None),
    );

    assert_eq!(result.exit_code, ExitCode::Success);
    assert!(result.stdout.contains("Unwanted entries:"));
    assert!(result.stdout.contains("/sw/bin"));
    assert!(result.stdout.contains("/sw/sbin"));
}

#[test]
fn doctor_fink_preset_reports_manpath_unwanted_entries() {
    let result = run(
        [
            "doctor",
            "--var",
            "manpath",
            "--platform",
            "posix",
            "--preset",
            "fink",
        ],
        context_with_manpath("", "/sw/share/man", None),
    );

    assert_eq!(result.exit_code, ExitCode::Success);
    assert!(result.stdout.contains("Unwanted entries:"));
    assert!(result.stdout.contains("/sw/share/man"));
}

#[test]
fn doctor_ordering_presets_are_accepted() {
    for (preset, path, expected_warning) in [
        (
            "homebrew",
            "/bin:/opt/homebrew/bin",
            "/bin appears before /opt/homebrew/bin",
        ),
        (
            "cargo",
            "/usr/bin:/Users/me/.local/bin:/Users/me/.cargo/bin",
            "/usr/bin appears before /Users/me/.cargo/bin",
        ),
        (
            "pyenv",
            "/usr/bin:/Users/me/.cargo/bin:/Users/me/.pyenv/shims",
            "/usr/bin appears before /Users/me/.pyenv/shims",
        ),
    ] {
        let result = run(
            ["doctor", "--platform", "posix", "--preset", preset],
            context(path, None),
        );

        assert_eq!(result.exit_code, ExitCode::Success);
        assert!(result.stdout.contains("Ordering warnings:"));
        assert!(result.stdout.contains(expected_warning));
    }
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
    assert_eq!(result.stdout, "patholog 0.9.0\n");
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
fn missing_config_returns_runtime_error() {
    let directory = tempfile::tempdir().expect("create tempdir");
    let config = directory.path().join("missing.toml");

    let result = run(
        ["doctor", "--config", config.to_str().expect("utf-8 config")],
        context("", None),
    );

    assert_eq!(result.exit_code, ExitCode::GeneralError);
    assert!(result.stderr.contains("config file is not readable"));
}

#[test]
fn malformed_config_returns_runtime_error() {
    let directory = tempfile::tempdir().expect("create tempdir");
    let config = write_config(directory.path(), "version = ");

    let result = run(
        [
            "clean",
            "--stdout",
            "--config",
            config.to_str().expect("utf-8 config"),
        ],
        context("", None),
    );

    assert_eq!(result.exit_code, ExitCode::GeneralError);
    assert!(result.stderr.contains("config file is invalid"));
}

#[test]
fn unknown_config_key_returns_runtime_error() {
    let directory = tempfile::tempdir().expect("create tempdir");
    let config = write_config(directory.path(), "version = 1\nunknown = true\n");

    let result = run(
        [
            "apply",
            "--dry-run",
            "--shell",
            "bash",
            "--config",
            config.to_str().expect("utf-8 config"),
        ],
        context("", None),
    );

    assert_eq!(result.exit_code, ExitCode::GeneralError);
    assert!(result.stderr.contains("config file is invalid"));
    assert!(result.stderr.contains("unknown"));
}

#[test]
fn config_check_validates_config_file() {
    let directory = tempfile::tempdir().expect("create tempdir");
    let config = write_config(
        directory.path(),
        "version = 1\n[path]\ndrop = [\"/drop\"]\n",
    );

    let result = run(
        [
            "config",
            "check",
            "--config",
            config.to_str().expect("utf-8 config"),
        ],
        context("", None),
    );

    assert_eq!(result.exit_code, ExitCode::Success);
    assert!(result.stdout.contains("Config OK:"));
}

#[test]
fn config_print_outputs_normalized_json() {
    let directory = tempfile::tempdir().expect("create tempdir");
    let config = write_config(
        directory.path(),
        "version = 1\n[path]\ndrop = [\"/drop\", \"/drop\"]\npreset = [\"cargo\"]\nfail_on = [\"duplicate\"]\n",
    );

    let result = run(
        [
            "config",
            "print",
            "--json",
            "--config",
            config.to_str().expect("utf-8 config"),
        ],
        context("", None),
    );

    assert_eq!(result.exit_code, ExitCode::Success);
    assert!(result.stdout.contains("\"version\": 1"));
    assert!(
        result
            .stdout
            .contains("\"drop\": [\n      \"/drop\"\n    ]")
    );
    assert!(
        result
            .stdout
            .contains("\"preset\": [\n      \"cargo\"\n    ]")
    );
    assert!(
        result
            .stdout
            .contains("\"fail_on\": [\n      \"duplicate\"\n    ]")
    );
}

#[test]
fn config_auto_discovers_local_patholog_toml_for_operations() {
    let directory = tempfile::tempdir().expect("create tempdir");
    write_config(directory.path(), "version = 1\n[path]\ndrop = [\"drop\"]\n");

    let result = run(
        [
            "clean",
            "--stdout",
            "--platform",
            "posix",
            "--config",
            "auto",
        ],
        context_with_cwd("drop:keep", None, directory.path()),
    );

    assert_eq!(result.exit_code, ExitCode::Success);
    assert_eq!(result.stdout, "keep\n");
}

#[test]
fn config_auto_not_found_is_ignored_for_operations() {
    let directory = tempfile::tempdir().expect("create tempdir");

    let result = run(
        [
            "clean",
            "--stdout",
            "--platform",
            "posix",
            "--config",
            "auto",
        ],
        context_with_cwd("drop:keep", None, directory.path()),
    );

    assert_eq!(result.exit_code, ExitCode::Success);
    assert_eq!(result.stdout, "drop:keep\n");
}

#[test]
fn config_auto_not_found_fails_for_config_commands() {
    let directory = tempfile::tempdir().expect("create tempdir");

    let result = run(
        ["config", "check", "--config", "auto"],
        context_with_cwd("", None, directory.path()),
    );

    assert_eq!(result.exit_code, ExitCode::GeneralError);
    assert!(result.stderr.contains("config auto did not find"));
}

#[test]
fn parse_fail_on_trims_ignores_empty_and_deduplicates() {
    assert_eq!(
        parse_fail_on(
            " duplicate, ,not_directory,unreadable,suspicious_order,shadowed_command,unwanted,duplicate ",
        )
        .expect("parse fail-on"),
        [
            IssueKind::Duplicate,
            IssueKind::NotDirectory,
            IssueKind::Unreadable,
            IssueKind::SuspiciousOrder,
            IssueKind::ShadowedCommand,
            IssueKind::Unwanted
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

fn context_with_manpath(
    path_value: &str,
    manpath_value: &str,
    pathext: Option<&str>,
) -> CommandContext {
    CommandContext {
        path_value: path_value.to_owned(),
        manpath_value: manpath_value.to_owned(),
        pathext: pathext.map(str::to_owned),
        cwd: Path::new(".").to_path_buf(),
        home_dir: None,
        user_profile_dir: None,
    }
}

fn context_with_cwd(path_value: &str, pathext: Option<&str>, cwd: &Path) -> CommandContext {
    CommandContext {
        path_value: path_value.to_owned(),
        manpath_value: String::new(),
        pathext: pathext.map(str::to_owned),
        cwd: cwd.to_path_buf(),
        home_dir: None,
        user_profile_dir: None,
    }
}

fn context_with_home(path_value: &str, pathext: Option<&str>, home: &Path) -> CommandContext {
    CommandContext {
        path_value: path_value.to_owned(),
        manpath_value: String::new(),
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
        manpath_value: String::new(),
        pathext: pathext.map(str::to_owned),
        cwd: Path::new(".").to_path_buf(),
        home_dir: Some(home.to_path_buf()),
        user_profile_dir: Some(user_profile.to_path_buf()),
    }
}

fn backup_paths_for(profile: &Path) -> Vec<std::path::PathBuf> {
    let parent = profile.parent().expect("profile has parent");
    let prefix = format!(
        "{}.patholog-backup.",
        profile
            .file_name()
            .expect("profile has file name")
            .to_string_lossy()
    );
    let mut paths = std::fs::read_dir(parent)
        .expect("read profile parent")
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| {
            path.file_name()
                .map(|name| name.to_string_lossy().starts_with(&prefix))
                .unwrap_or(false)
        })
        .collect::<Vec<_>>();
    paths.sort();
    paths
}

fn write_config(directory: &Path, content: &str) -> std::path::PathBuf {
    let config_path = directory.join("patholog.toml");
    std::fs::write(&config_path, content).expect("write config");
    config_path
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
