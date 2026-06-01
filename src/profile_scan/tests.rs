use crate::model::PlatformMode;

use super::{MAX_LINE_DISPLAY_CHARS, classify_path_mutation, line_display, scan_shell_profiles};

#[test]
fn scan_shell_profiles_reports_path_mutations_in_existing_profiles() {
    let directory = tempfile::tempdir().expect("create tempdir");
    let zshrc = directory.path().join(".zshrc");
    std::fs::write(
        &zshrc,
        "# PATH=/ignored\nexport PATH=\"$HOME/bin:$PATH\"\npath+=(/opt/tools)\n",
    )
    .expect("write zshrc");

    let report = scan_shell_profiles(directory.path(), PlatformMode::Posix);
    let profile = report
        .profiles
        .iter()
        .find(|profile| profile.path == zshrc.display().to_string())
        .expect("zshrc profile");

    assert!(profile.readable);
    assert_eq!(profile.path_mutations.len(), 2);
    assert_eq!(profile.path_mutations[0].kind, "path_assignment");
    assert_eq!(profile.path_mutations[1].kind, "zsh_path_array");
}

#[test]
fn scan_shell_profiles_reports_powershell_path_mutations() {
    let directory = tempfile::tempdir().expect("create tempdir");
    let profile_path = directory
        .path()
        .join("Documents/PowerShell/Microsoft.PowerShell_profile.ps1");
    std::fs::create_dir_all(profile_path.parent().expect("profile has parent"))
        .expect("create profile parent");
    std::fs::write(&profile_path, "$env:Path = \"C:\\Tools;$env:Path\"\n").expect("write profile");

    let report = scan_shell_profiles(directory.path(), PlatformMode::Windows);
    let profile = report
        .profiles
        .iter()
        .find(|profile| profile.path == profile_path.display().to_string())
        .expect("powershell profile");

    assert_eq!(profile.path_mutations.len(), 1);
    assert_eq!(profile.path_mutations[0].kind, "powershell_env_path");
}

#[test]
fn scan_shell_profiles_reports_posix_powershell_all_hosts_profile() {
    let directory = tempfile::tempdir().expect("create tempdir");
    let profile_path = directory.path().join(".config/powershell/profile.ps1");
    std::fs::create_dir_all(profile_path.parent().expect("profile has parent"))
        .expect("create profile parent");
    std::fs::write(&profile_path, "$env:Path = \"/opt/tools:$env:Path\"\n").expect("write profile");

    let report = scan_shell_profiles(directory.path(), PlatformMode::Posix);
    let profile = report
        .profiles
        .iter()
        .find(|profile| profile.path == profile_path.display().to_string())
        .expect("powershell all-host profile");

    assert_eq!(profile.path_mutations.len(), 1);
    assert_eq!(profile.path_mutations[0].kind, "powershell_env_path");
}

#[test]
fn scan_shell_profiles_reports_windows_powershell_all_hosts_profiles() {
    let directory = tempfile::tempdir().expect("create tempdir");
    let powershell_profile = directory.path().join("Documents/PowerShell/profile.ps1");
    let windows_powershell_profile = directory
        .path()
        .join("Documents/WindowsPowerShell/profile.ps1");
    for profile_path in [&powershell_profile, &windows_powershell_profile] {
        std::fs::create_dir_all(profile_path.parent().expect("profile has parent"))
            .expect("create profile parent");
        std::fs::write(profile_path, "$env:Path = \"C:\\Tools;$env:Path\"\n")
            .expect("write profile");
    }

    let report = scan_shell_profiles(directory.path(), PlatformMode::Windows);

    for profile_path in [powershell_profile, windows_powershell_profile] {
        let profile = report
            .profiles
            .iter()
            .find(|profile| profile.path == profile_path.display().to_string())
            .expect("powershell all-host profile");
        assert_eq!(profile.path_mutations.len(), 1);
        assert_eq!(profile.path_mutations[0].kind, "powershell_env_path");
    }
}

#[test]
fn scan_shell_profiles_treats_invalid_utf8_profiles_as_readable_lossy_text() {
    let directory = tempfile::tempdir().expect("create tempdir");
    let zshrc = directory.path().join(".zshrc");
    std::fs::write(&zshrc, b"PATH=/ok:\xff\n").expect("write zshrc");

    let report = scan_shell_profiles(directory.path(), PlatformMode::Posix);
    let profile = report
        .profiles
        .iter()
        .find(|profile| profile.path == zshrc.display().to_string())
        .expect("zshrc profile");

    assert!(profile.readable);
    assert_eq!(profile.path_mutations.len(), 1);
    assert_eq!(profile.path_mutations[0].kind, "path_assignment");
    assert!(profile.path_mutations[0].text.starts_with("PATH=/ok:"));
}

#[test]
fn scan_shell_profiles_reports_directory_profiles_as_non_files() {
    let directory = tempfile::tempdir().expect("create tempdir");
    let bashrc = directory.path().join(".bashrc");
    std::fs::create_dir(&bashrc).expect("create bashrc directory");

    let report = scan_shell_profiles(directory.path(), PlatformMode::Posix);
    let profile = report
        .profiles
        .iter()
        .find(|profile| profile.path == bashrc.display().to_string())
        .expect("bashrc profile");

    assert!(profile.exists);
    assert!(!profile.is_file);
    assert!(!profile.readable);
    assert!(profile.path_mutations.is_empty());
}

#[test]
fn classify_path_mutation_recognizes_supported_profile_mutation_forms() {
    assert_eq!(
        classify_path_mutation("eval \"$(/usr/libexec/path_helper)\""),
        Some("path_helper")
    );
    assert_eq!(
        classify_path_mutation("[Environment]::SetEnvironmentVariable('Path', 'C:\\Tools')"),
        Some("powershell_environment_update")
    );
    assert_eq!(
        classify_path_mutation("PATH+=:/opt/tools"),
        Some("path_assignment")
    );
    assert_eq!(
        classify_path_mutation("path=(/opt/tools $path)"),
        Some("zsh_path_array")
    );
}

#[test]
fn classify_path_mutation_ignores_comments_blank_lines_and_identifier_substrings() {
    assert_eq!(classify_path_mutation(""), None);
    assert_eq!(classify_path_mutation("  # export PATH=/ignored"), None);
    assert_eq!(classify_path_mutation("MYPATH=/ignored"), None);
    assert_eq!(classify_path_mutation("mypath=(/ignored)"), None);
}

#[test]
fn line_display_truncates_at_character_boundary() {
    let prefix = "é".repeat(MAX_LINE_DISPLAY_CHARS);
    let line = format!("  {prefix}x  ");

    assert_eq!(line_display(&line), format!("{prefix}..."));
}
