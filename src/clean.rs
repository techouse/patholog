use crate::model::{PathVariable, PlatformMode, ShellKind};
use crate::normalize::first_unique_entries;
use crate::path_env::parse_path;
use crate::platform::resolve_platform_rules;
use crate::policy::PathPolicy;

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

#[cfg(any(test, feature = "fuzzing"))]
pub(crate) fn clean_path(
    path_value: &str,
    platform_mode: PlatformMode,
    pathext: Option<&str>,
) -> String {
    clean_path_with_policy(path_value, platform_mode, pathext, &PathPolicy::default())
}

pub(crate) fn clean_path_with_policy(
    path_value: &str,
    platform_mode: PlatformMode,
    pathext: Option<&str>,
    policy: &PathPolicy,
) -> String {
    cleaned_path(path_value, platform_mode, pathext, policy).raw_path()
}

#[cfg(any(test, feature = "fuzzing"))]
pub(crate) fn clean_export(
    path_value: &str,
    platform_mode: PlatformMode,
    pathext: Option<&str>,
    shell: ShellKind,
) -> String {
    clean_export_with_policy(
        path_value,
        platform_mode,
        pathext,
        shell,
        PathVariable::Path,
        &PathPolicy::default(),
    )
}

pub(crate) fn clean_export_with_policy(
    path_value: &str,
    platform_mode: PlatformMode,
    pathext: Option<&str>,
    shell: ShellKind,
    variable: PathVariable,
    policy: &PathPolicy,
) -> String {
    format_clean_export(
        &cleaned_path(path_value, platform_mode, pathext, policy),
        shell,
        variable,
    )
}

fn cleaned_path(
    path_value: &str,
    platform_mode: PlatformMode,
    pathext: Option<&str>,
    policy: &PathPolicy,
) -> CleanedPath {
    let rules = resolve_platform_rules(platform_mode, pathext);
    let policy = policy.compile(platform_mode, pathext);
    let entries = parse_path(path_value, platform_mode, pathext);
    let entries = entries
        .into_iter()
        .filter(|entry| !policy.matches_entry(entry))
        .collect::<Vec<_>>();
    let entries = first_unique_entries(&entries)
        .into_iter()
        .map(|entry| entry.raw)
        .collect();
    CleanedPath {
        entries,
        separator: rules.separator,
    }
}

fn format_clean_export(cleaned: &CleanedPath, shell: ShellKind, variable: PathVariable) -> String {
    match shell {
        ShellKind::Bash | ShellKind::Zsh => {
            format!(
                "export {}={}",
                variable.env_name(),
                posix_single_quote(&cleaned.raw_path())
            )
        }
        ShellKind::Fish => format_fish_export(&cleaned.entries, variable),
        ShellKind::Pwsh => {
            format!(
                "$env:{} = {}",
                variable.powershell_env_name(),
                powershell_single_quote(&cleaned.raw_path())
            )
        }
    }
}

fn format_fish_export(entries: &[String], variable: PathVariable) -> String {
    let quoted_entries = entries
        .iter()
        .map(|entry| fish_single_quote(entry))
        .collect::<Vec<_>>();
    if quoted_entries.is_empty() {
        format!("set -gx {}", variable.env_name())
    } else {
        format!(
            "set -gx {} {}",
            variable.env_name(),
            quoted_entries.join(" ")
        )
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
