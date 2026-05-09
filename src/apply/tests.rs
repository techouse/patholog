use std::path::Path;

use crate::model::{ApplyAction, PlatformMode, ShellKind};
use crate::policy::PathPolicy;

use super::{
    ApplyPlanOptions, END_MARKER, START_MARKER, existing_managed_block, managed_block, plan_apply,
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

fn plan(
    home: &Path,
    profile: Option<&Path>,
    platform_mode: PlatformMode,
    shell: ShellKind,
) -> Result<crate::model::ApplyPlan, String> {
    plan_apply(&ApplyPlanOptions {
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
