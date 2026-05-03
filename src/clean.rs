use crate::model::{PlatformMode, ShellKind};
use crate::normalize::first_unique_entries;
use crate::path_env::parse_path;
use crate::platform::resolve_platform_rules;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CleanedPath {
    entries: Vec<String>,
    separator: char,
}

impl CleanedPath {
    fn raw_path(&self) -> String {
        self.entries.join(&self.separator.to_string())
    }
}

pub(crate) fn clean_path(
    path_value: &str,
    platform_mode: PlatformMode,
    pathext: Option<&str>,
) -> String {
    cleaned_path(path_value, platform_mode, pathext).raw_path()
}

pub(crate) fn clean_export(
    path_value: &str,
    platform_mode: PlatformMode,
    pathext: Option<&str>,
    shell: ShellKind,
) -> String {
    format_clean_export(&cleaned_path(path_value, platform_mode, pathext), shell)
}

fn cleaned_path(
    path_value: &str,
    platform_mode: PlatformMode,
    pathext: Option<&str>,
) -> CleanedPath {
    let rules = resolve_platform_rules(platform_mode, pathext);
    let entries = parse_path(path_value, platform_mode, pathext);
    let entries = first_unique_entries(&entries)
        .into_iter()
        .map(|entry| entry.raw)
        .collect();
    CleanedPath {
        entries,
        separator: rules.separator,
    }
}

fn format_clean_export(cleaned: &CleanedPath, shell: ShellKind) -> String {
    match shell {
        ShellKind::Bash | ShellKind::Zsh => {
            format!("export PATH={}", posix_single_quote(&cleaned.raw_path()))
        }
        ShellKind::Fish => format_fish_export(&cleaned.entries),
        ShellKind::Pwsh => {
            format!(
                "$env:Path = {}",
                powershell_single_quote(&cleaned.raw_path())
            )
        }
    }
}

fn format_fish_export(entries: &[String]) -> String {
    let quoted_entries = entries
        .iter()
        .map(|entry| fish_single_quote(entry))
        .collect::<Vec<_>>();
    if quoted_entries.is_empty() {
        "set -gx PATH".to_owned()
    } else {
        format!("set -gx PATH {}", quoted_entries.join(" "))
    }
}

fn posix_single_quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\\''"))
}

fn fish_single_quote(value: &str) -> String {
    let mut quoted = String::from("'");
    for character in value.chars() {
        match character {
            '\\' => quoted.push_str("\\\\"),
            '\'' => quoted.push_str("\\'"),
            _ => quoted.push(character),
        }
    }
    quoted.push('\'');
    quoted
}

fn powershell_single_quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "''"))
}

#[cfg(test)]
#[path = "clean/tests.rs"]
mod tests;
