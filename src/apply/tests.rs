use std::path::Path;

use crate::model::{ApplyAction, PlatformMode, ShellKind};
use crate::policy::PathPolicy;

use super::{
    ApplyPlanOptions, END_MARKER, START_MARKER, WriteMode, appended_profile_content,
    backup_path_candidate, create_profile_backup_for_seconds, existing_managed_block,
    existing_managed_block_span, managed_block, plan_apply, plan_apply_operation,
    replaced_profile_content, write_apply_plan, write_profile_atomically,
};

#[test]
fn plan_apply_defaults_to_zshrc_for_zsh() {
    let directory = tempfile::tempdir().expect("create tempdir");
    let plan =
        plan(directory.path(), None, PlatformMode::Posix, ShellKind::Zsh).expect("plan apply");

    assert_eq!(plan.action, ApplyAction::CreateProfile);
    assert_eq!(
        plan.profile_path,
        directory.path().join(".zshrc").display().to_string()
    );
}

#[test]
fn plan_apply_defaults_to_interactive_profiles() {
    let directory = tempfile::tempdir().expect("create tempdir");

    for (shell, expected_path) in [
        (ShellKind::Bash, ".bashrc"),
        (ShellKind::Fish, ".config/fish/config.fish"),
        (
            ShellKind::Pwsh,
            ".config/powershell/Microsoft.PowerShell_profile.ps1",
        ),
    ] {
        let plan = plan(directory.path(), None, PlatformMode::Posix, shell).expect("plan apply");

        assert_eq!(
            plan.profile_path,
            directory.path().join(expected_path).display().to_string()
        );
    }
}

#[test]
fn plan_apply_uses_windows_user_profile_for_pwsh_windows_mode() {
    let directory = tempfile::tempdir().expect("create tempdir");
    let user_profile = directory.path().join("userprofile");
    let plan = plan_apply(&ApplyPlanOptions {
        path_value: r"C:\One;C:\Two",
        platform_mode: PlatformMode::Windows,
        pathext: None,
        shell: ShellKind::Pwsh,
        home_dir: None,
        user_profile_dir: Some(&user_profile),
        profile: None,
        policy: PathPolicy::default(),
    })
    .expect("plan apply");

    assert_eq!(
        plan.profile_path,
        user_profile
            .join("Documents/PowerShell/Microsoft.PowerShell_profile.ps1")
            .display()
            .to_string()
    );
}

#[test]
fn plan_apply_uses_home_for_posix_shells_in_windows_mode() {
    let directory = tempfile::tempdir().expect("create tempdir");
    let home = directory.path().join("home");
    let user_profile = directory.path().join("userprofile");

    for (shell, expected_path) in [
        (ShellKind::Bash, ".bashrc"),
        (ShellKind::Fish, ".config/fish/config.fish"),
        (ShellKind::Zsh, ".zshrc"),
    ] {
        let plan = plan_apply(&ApplyPlanOptions {
            path_value: r"C:\One;C:\Two",
            platform_mode: PlatformMode::Windows,
            pathext: None,
            shell,
            home_dir: Some(&home),
            user_profile_dir: Some(&user_profile),
            profile: None,
            policy: PathPolicy::default(),
        })
        .expect("plan apply");

        assert_eq!(
            plan.profile_path,
            home.join(expected_path).display().to_string()
        );
    }
}

#[test]
fn plan_apply_profile_override_does_not_require_home() {
    let directory = tempfile::tempdir().expect("create tempdir");
    let profile = directory.path().join("custom.profile");
    let plan = plan_apply(&ApplyPlanOptions {
        path_value: "/a:/b",
        platform_mode: PlatformMode::Posix,
        pathext: None,
        shell: ShellKind::Bash,
        home_dir: None,
        user_profile_dir: None,
        profile: Some(&profile),
        policy: PathPolicy::default(),
    })
    .expect("plan apply");

    assert_eq!(plan.profile_path, profile.display().to_string());
}

#[test]
fn plan_apply_appends_when_profile_has_no_managed_block() {
    let directory = tempfile::tempdir().expect("create tempdir");
    let profile = directory.path().join(".bashrc");
    std::fs::write(&profile, "export PATH=\"$HOME/bin:$PATH\"\n").expect("write profile");

    let plan = plan(
        directory.path(),
        Some(&profile),
        PlatformMode::Posix,
        ShellKind::Bash,
    )
    .expect("plan apply");

    assert_eq!(plan.action, ApplyAction::AppendBlock);
    assert_eq!(plan.existing_block, None);
}

#[test]
fn plan_apply_replaces_one_complete_managed_block() {
    let directory = tempfile::tempdir().expect("create tempdir");
    let profile = directory.path().join(".bashrc");
    let existing_block = managed_block("export PATH='/old'");
    std::fs::write(&profile, format!("before\n{existing_block}\nafter\n")).expect("write profile");

    let plan = plan(
        directory.path(),
        Some(&profile),
        PlatformMode::Posix,
        ShellKind::Bash,
    )
    .expect("plan apply");

    assert_eq!(plan.action, ApplyAction::ReplaceBlock);
    assert_eq!(plan.existing_block, Some(existing_block));
}

#[test]
fn plan_apply_rejects_non_file_profile() {
    let directory = tempfile::tempdir().expect("create tempdir");
    let profile = directory.path().join(".bashrc");
    std::fs::create_dir(&profile).expect("create profile directory");

    let error = plan(
        directory.path(),
        Some(&profile),
        PlatformMode::Posix,
        ShellKind::Bash,
    )
    .expect_err("plan should fail");

    assert!(error.contains("not a file"));
}

#[cfg(unix)]
#[test]
fn plan_apply_rejects_unreadable_profile() {
    use std::os::unix::fs::PermissionsExt;

    let directory = tempfile::tempdir().expect("create tempdir");
    let profile = directory.path().join(".bashrc");
    std::fs::write(&profile, "export PATH=\"$PATH\"\n").expect("write profile");
    let mut permissions = std::fs::metadata(&profile)
        .expect("read metadata")
        .permissions();
    permissions.set_mode(0o000);
    std::fs::set_permissions(&profile, permissions).expect("make unreadable");

    let error = plan(
        directory.path(),
        Some(&profile),
        PlatformMode::Posix,
        ShellKind::Bash,
    )
    .expect_err("plan should fail");

    let mut permissions = std::fs::metadata(&profile)
        .expect("read metadata")
        .permissions();
    permissions.set_mode(0o600);
    std::fs::set_permissions(&profile, permissions).expect("restore permissions");

    assert!(error.contains("not readable"));
}

#[test]
fn existing_managed_block_detects_crlf_block() {
    let content =
        format!("before\r\n{START_MARKER}\r\nexport PATH='/old'\r\n{END_MARKER}\r\nafter\r\n");

    assert_eq!(
        existing_managed_block(&content).expect("detect block"),
        Some(format!(
            "{START_MARKER}\r\nexport PATH='/old'\r\n{END_MARKER}\r"
        ))
    );
}

#[test]
fn existing_managed_block_rejects_malformed_block() {
    let error = existing_managed_block(START_MARKER).expect_err("malformed block");

    assert!(error.contains("malformed"));
}

#[test]
fn existing_managed_block_rejects_indented_marker_lines() {
    let content = format!("{START_MARKER}\nexport PATH='/old'\n  {END_MARKER}");
    let error = existing_managed_block(&content).expect_err("indented marker should not match");

    assert!(error.contains("malformed"));
}

#[test]
fn existing_managed_block_rejects_duplicate_blocks() {
    let block = managed_block("export PATH='/old'");
    let error = existing_managed_block(&format!("{block}\n{block}")).expect_err("duplicate blocks");

    assert!(error.contains("duplicate"));
}

#[test]
fn existing_managed_block_span_supports_exact_replacement() {
    let existing_block = managed_block("export PATH='/old'");
    let content = format!("before\n{existing_block}\nafter\n");
    let span = existing_managed_block_span(&content)
        .expect("detect block")
        .expect("block exists");

    assert_eq!(&content[span], existing_block);
    assert_eq!(
        replaced_profile_content(&content, &managed_block("export PATH='/new'"))
            .expect("replace block"),
        "before\n# >>> patholog PATH >>>\nexport PATH='/new'\n# <<< patholog PATH <<<\nafter\n"
    );
}

#[test]
fn replaced_profile_content_preserves_crlf_line_endings() {
    let existing_block = "# >>> patholog PATH >>>\r\nexport PATH='/old'\r\n# <<< patholog PATH <<<";
    let content = format!("before\r\n{existing_block}\r\nafter\r\n");
    let planned_block = managed_block("export PATH='/new'");
    let span = existing_managed_block_span(&content)
        .expect("detect block")
        .expect("block exists");

    assert_eq!(&content[span], format!("{existing_block}\r"));
    assert_eq!(
        replaced_profile_content(&content, &planned_block).expect("replace block"),
        "before\r\n# >>> patholog PATH >>>\r\nexport PATH='/new'\r\n# <<< patholog PATH <<<\r\nafter\r\n"
    );
}

#[test]
fn appended_profile_content_preserves_existing_text_with_separator() {
    let block = managed_block("export PATH='/new'");

    assert_eq!(
        appended_profile_content("", &block),
        "# >>> patholog PATH >>>\nexport PATH='/new'\n# <<< patholog PATH <<<\n"
    );
    assert_eq!(
        appended_profile_content("before\n", &block),
        "before\n\n# >>> patholog PATH >>>\nexport PATH='/new'\n# <<< patholog PATH <<<\n"
    );
    assert_eq!(
        appended_profile_content("before", &block),
        "before\n\n# >>> patholog PATH >>>\nexport PATH='/new'\n# <<< patholog PATH <<<\n"
    );
}

#[test]
fn appended_profile_content_preserves_crlf_line_endings() {
    let block = managed_block("export PATH='/new'");

    assert_eq!(
        appended_profile_content("before\r\n", &block),
        "before\r\n\r\n# >>> patholog PATH >>>\r\nexport PATH='/new'\r\n# <<< patholog PATH <<<\r\n"
    );
    assert_eq!(
        appended_profile_content("before", &block),
        "before\n\n# >>> patholog PATH >>>\nexport PATH='/new'\n# <<< patholog PATH <<<\n"
    );
}

#[test]
fn backup_path_for_seconds_uses_base_name() {
    let directory = tempfile::tempdir().expect("create tempdir");
    let profile = directory.path().join(".zshrc");

    assert_eq!(
        backup_path_candidate(&profile, 123, 0),
        directory.path().join(".zshrc.patholog-backup.123")
    );
}

#[test]
fn create_profile_backup_uses_create_new_suffix_without_overwriting() {
    let directory = tempfile::tempdir().expect("create tempdir");
    let profile = directory.path().join(".bashrc");
    let first_backup = backup_path_candidate(&profile, 123, 0);
    std::fs::write(&profile, "before\n").expect("write profile");
    std::fs::write(&first_backup, "existing backup\n").expect("write backup");

    let backup = create_profile_backup_for_seconds(&profile, 123).expect("create backup");

    assert_eq!(
        std::fs::read_to_string(&first_backup).expect("read original backup"),
        "existing backup\n"
    );
    assert_eq!(
        backup,
        directory.path().join(".bashrc.patholog-backup.123.1")
    );
    assert_eq!(
        std::fs::read_to_string(&backup).expect("read new backup"),
        "before\n"
    );
}

#[cfg(target_os = "linux")]
#[test]
fn create_profile_backup_preserves_non_utf8_base_name() {
    use std::ffi::OsString;
    use std::os::unix::ffi::{OsStrExt, OsStringExt};

    let directory = tempfile::tempdir().expect("create tempdir");
    let profile = directory
        .path()
        .join(OsString::from_vec(b"profile-\xff".to_vec()));
    std::fs::write(&profile, "before\n").expect("write profile");

    let backup = create_profile_backup_for_seconds(&profile, 123).expect("create backup");

    assert_eq!(
        backup.file_name().expect("backup has file name").as_bytes(),
        b"profile-\xff.patholog-backup.123"
    );
    assert_eq!(
        std::fs::read_to_string(&backup).expect("read backup"),
        "before\n"
    );
}

#[cfg(unix)]
#[test]
fn create_profile_backup_preserves_source_permissions() {
    use std::os::unix::fs::PermissionsExt;

    let directory = tempfile::tempdir().expect("create tempdir");
    let profile = directory.path().join(".bashrc");
    std::fs::write(&profile, "before\n").expect("write profile");
    let mut permissions = std::fs::metadata(&profile)
        .expect("read metadata")
        .permissions();
    permissions.set_mode(0o600);
    std::fs::set_permissions(&profile, permissions).expect("set permissions");

    let backup = create_profile_backup_for_seconds(&profile, 123).expect("create backup");

    assert_eq!(
        std::fs::metadata(&backup)
            .expect("read backup metadata")
            .permissions()
            .mode()
            & 0o777,
        0o600
    );
}

#[test]
fn write_apply_plan_creates_parent_directories_and_profile() {
    let directory = tempfile::tempdir().expect("create tempdir");
    let profile = directory.path().join(".config/fish/config.fish");
    let planned = plan_apply_operation(&ApplyPlanOptions {
        path_value: "/a:/b:/a",
        platform_mode: PlatformMode::Posix,
        pathext: None,
        shell: ShellKind::Fish,
        home_dir: Some(directory.path()),
        user_profile_dir: Some(directory.path()),
        profile: Some(&profile),
        policy: PathPolicy::default(),
    })
    .expect("plan apply");

    let outcome = write_apply_plan(planned, true).expect("write apply");

    assert!(outcome.wrote);
    assert!(!outcome.backup_created);
    assert_eq!(
        std::fs::read_to_string(&profile).expect("read profile"),
        "# >>> patholog PATH >>>\nset -gx PATH '/a' '/b'\n# <<< patholog PATH <<<\n"
    );
}

#[test]
fn write_apply_plan_does_not_clobber_create_target_that_appears_after_planning() {
    let directory = tempfile::tempdir().expect("create tempdir");
    let profile = directory.path().join(".zshrc");
    let planned = plan_apply_operation(&ApplyPlanOptions {
        path_value: "/a:/b:/a",
        platform_mode: PlatformMode::Posix,
        pathext: None,
        shell: ShellKind::Zsh,
        home_dir: Some(directory.path()),
        user_profile_dir: Some(directory.path()),
        profile: Some(&profile),
        policy: PathPolicy::default(),
    })
    .expect("plan apply");
    std::fs::write(&profile, "user-created\n").expect("write competing profile");

    let error = write_apply_plan(planned, true).expect_err("write should fail");

    assert!(error.contains("changed before write") || error.contains("could not write profile"));
    assert_eq!(
        std::fs::read_to_string(&profile).expect("read profile"),
        "user-created\n"
    );
}

#[test]
fn write_apply_plan_reports_parent_directory_creation_failures_generically() {
    let directory = tempfile::tempdir().expect("create tempdir");
    let parent_file = directory.path().join("profile-parent");
    std::fs::write(&parent_file, "not a directory\n").expect("write parent file");
    let profile = parent_file.join("child/.zshrc");

    let error = write_profile_atomically(&profile, "content\n", None, WriteMode::CreateNew)
        .expect_err("write should fail");

    assert!(error.contains("could not create profile parent directory"));
    assert!(!error.contains("not writable"));
}

#[cfg(target_os = "linux")]
#[test]
fn write_apply_plan_uses_raw_non_utf8_profile_path() {
    use std::ffi::OsString;
    use std::os::unix::ffi::OsStringExt;

    let directory = tempfile::tempdir().expect("create tempdir");
    let profile = directory
        .path()
        .join(OsString::from_vec(b"profile-\xff".to_vec()));
    let planned = plan_apply_operation(&ApplyPlanOptions {
        path_value: "/a:/b:/a",
        platform_mode: PlatformMode::Posix,
        pathext: None,
        shell: ShellKind::Zsh,
        home_dir: Some(directory.path()),
        user_profile_dir: Some(directory.path()),
        profile: Some(&profile),
        policy: PathPolicy::default(),
    })
    .expect("plan apply");

    write_apply_plan(planned, true).expect("write apply");

    assert_eq!(
        std::fs::read_to_string(&profile).expect("read non-utf8 profile"),
        "# >>> patholog PATH >>>\nexport PATH='/a:/b'\n# <<< patholog PATH <<<\n"
    );
}

#[cfg(unix)]
#[test]
fn plan_apply_rejects_symlink_profile() {
    use std::os::unix::fs::symlink;

    let directory = tempfile::tempdir().expect("create tempdir");
    let target = directory.path().join("real.profile");
    let profile = directory.path().join(".bashrc");
    std::fs::write(&target, "before\n").expect("write target profile");
    symlink(&target, &profile).expect("create profile symlink");

    let error = plan(
        directory.path(),
        Some(&profile),
        PlatformMode::Posix,
        ShellKind::Bash,
    )
    .expect_err("plan should fail");

    assert!(error.contains("symlink"));
    assert!(
        std::fs::symlink_metadata(&profile)
            .expect("read symlink metadata")
            .file_type()
            .is_symlink()
    );
    assert_eq!(
        std::fs::read_to_string(&target).expect("read target profile"),
        "before\n"
    );
}

#[cfg(unix)]
#[test]
fn write_apply_plan_preserves_existing_profile_permissions() {
    use std::os::unix::fs::PermissionsExt;

    let directory = tempfile::tempdir().expect("create tempdir");
    let profile = directory.path().join(".bashrc");
    std::fs::write(&profile, "before\n").expect("write profile");
    let mut permissions = std::fs::metadata(&profile)
        .expect("read metadata")
        .permissions();
    permissions.set_mode(0o640);
    std::fs::set_permissions(&profile, permissions).expect("set permissions");
    let planned = planned(
        directory.path(),
        Some(&profile),
        PlatformMode::Posix,
        ShellKind::Bash,
    )
    .expect("plan apply");

    write_apply_plan(planned, false).expect("write apply");

    assert_eq!(
        std::fs::metadata(&profile)
            .expect("read metadata")
            .permissions()
            .mode()
            & 0o777,
        0o640
    );
}

fn planned(
    home: &Path,
    profile: Option<&Path>,
    platform_mode: PlatformMode,
    shell: ShellKind,
) -> Result<super::PlannedApply, String> {
    plan_apply_operation(&ApplyPlanOptions {
        path_value: "/a:/b:/a",
        platform_mode,
        pathext: None,
        shell,
        home_dir: Some(home),
        user_profile_dir: Some(home),
        profile,
        policy: PathPolicy::default(),
    })
}

fn plan(
    home: &Path,
    profile: Option<&Path>,
    platform_mode: PlatformMode,
    shell: ShellKind,
) -> Result<crate::model::ApplyPlan, String> {
    planned(home, profile, platform_mode, shell).map(|planned| planned.plan)
}
