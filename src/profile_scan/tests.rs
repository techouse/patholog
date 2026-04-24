use crate::model::PlatformMode;

use super::scan_shell_profiles;

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
