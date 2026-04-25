use std::fs;
use std::path::{Path, PathBuf};

use crate::model::{PathMutation, PlatformMode, ShellProfile, ShellProfileScanReport};
use crate::platform::resolve_platform_rules;

const MAX_LINE_DISPLAY_CHARS: usize = 160;

pub(crate) fn scan_shell_profiles(
    home: &Path,
    platform_mode: PlatformMode,
) -> ShellProfileScanReport {
    let rules = resolve_platform_rules(platform_mode, None);
    let profiles = known_profiles(home, rules.mode)
        .into_iter()
        .map(|candidate| scan_profile(&candidate))
        .collect();

    ShellProfileScanReport {
        home: home.display().to_string(),
        profiles,
    }
}

struct ProfileCandidate {
    shell: &'static str,
    path: PathBuf,
}

fn known_profiles(home: &Path, platform_mode: PlatformMode) -> Vec<ProfileCandidate> {
    match platform_mode {
        PlatformMode::Windows => windows_profiles(home),
        PlatformMode::Auto | PlatformMode::Posix => posix_profiles(home),
    }
}

fn posix_profiles(home: &Path) -> Vec<ProfileCandidate> {
    [
        ("zsh", ".zshenv"),
        ("zsh", ".zprofile"),
        ("zsh", ".zshrc"),
        ("zsh", ".zlogin"),
        ("bash", ".bash_profile"),
        ("bash", ".bash_login"),
        ("bash", ".bashrc"),
        ("posix", ".profile"),
        ("powershell", ".config/powershell/profile.ps1"),
        (
            "powershell",
            ".config/powershell/Microsoft.PowerShell_profile.ps1",
        ),
    ]
    .into_iter()
    .map(|(shell, relative_path)| ProfileCandidate {
        shell,
        path: home.join(relative_path),
    })
    .collect()
}

fn windows_profiles(home: &Path) -> Vec<ProfileCandidate> {
    [
        "Documents/PowerShell/profile.ps1",
        "Documents/PowerShell/Microsoft.PowerShell_profile.ps1",
        "Documents/WindowsPowerShell/profile.ps1",
        "Documents/WindowsPowerShell/Microsoft.PowerShell_profile.ps1",
    ]
    .into_iter()
    .map(|relative_path| ProfileCandidate {
        shell: "powershell",
        path: home.join(relative_path),
    })
    .collect()
}

fn scan_profile(candidate: &ProfileCandidate) -> ShellProfile {
    let exists = candidate.path.exists();
    let is_file = exists && candidate.path.is_file();
    if !is_file {
        return ShellProfile {
            shell: candidate.shell,
            path: candidate.path.display().to_string(),
            exists,
            is_file,
            readable: false,
            path_mutations: Vec::new(),
        };
    }

    let Ok(bytes) = fs::read(&candidate.path) else {
        return ShellProfile {
            shell: candidate.shell,
            path: candidate.path.display().to_string(),
            exists,
            is_file,
            readable: false,
            path_mutations: Vec::new(),
        };
    };

    let content = String::from_utf8_lossy(&bytes);
    let path_mutations = content
        .lines()
        .enumerate()
        .filter_map(|(zero_based_line, line)| {
            classify_path_mutation(line).map(|kind| PathMutation {
                line: zero_based_line + 1,
                kind,
                text: line_display(line),
            })
        })
        .collect();

    ShellProfile {
        shell: candidate.shell,
        path: candidate.path.display().to_string(),
        exists,
        is_file,
        readable: true,
        path_mutations,
    }
}

fn classify_path_mutation(line: &str) -> Option<&'static str> {
    let trimmed = line.trim();
    if trimmed.is_empty() || trimmed.starts_with('#') {
        return None;
    }

    let lower = trimmed.to_ascii_lowercase();
    if lower.contains("/usr/libexec/path_helper") {
        return Some("path_helper");
    }
    if lower.contains("$env:path") {
        return Some("powershell_env_path");
    }
    if lower.contains("setenvironmentvariable") && lower.contains("path") {
        return Some("powershell_environment_update");
    }
    if has_shell_path_assignment(trimmed) {
        return Some("path_assignment");
    }
    if has_zsh_path_array_assignment(trimmed) {
        return Some("zsh_path_array");
    }

    None
}

fn has_shell_path_assignment(line: &str) -> bool {
    line.match_indices("PATH")
        .any(|(index, _match)| is_path_assignment_at(line, index))
}

fn is_path_assignment_at(line: &str, index: usize) -> bool {
    let previous = line[..index].chars().next_back();
    if previous.is_some_and(is_identifier_char) {
        return false;
    }

    let suffix = &line[index + "PATH".len()..];
    suffix.starts_with('=') || suffix.starts_with("+=")
}

fn has_zsh_path_array_assignment(line: &str) -> bool {
    line.match_indices("path")
        .any(|(index, _match)| is_zsh_path_array_assignment_at(line, index))
}

fn is_zsh_path_array_assignment_at(line: &str, index: usize) -> bool {
    let previous = line[..index].chars().next_back();
    if previous.is_some_and(is_identifier_char) {
        return false;
    }

    let suffix = &line[index + "path".len()..];
    suffix.starts_with('=') || suffix.starts_with("+=")
}

fn is_identifier_char(character: char) -> bool {
    character == '_' || character.is_ascii_alphanumeric()
}

fn line_display(line: &str) -> String {
    let trimmed = line.trim();
    let mut chars = trimmed.chars();
    let mut display = chars
        .by_ref()
        .take(MAX_LINE_DISPLAY_CHARS)
        .collect::<String>();
    if chars.next().is_some() {
        display.push_str("...");
    }
    display
}

#[cfg(test)]
#[path = "profile_scan/tests.rs"]
mod tests;
