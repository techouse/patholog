use std::collections::HashSet;
use std::path::Path;

use crate::doctor::diagnose_path_with_policy;
use crate::model::{
    Diagnostic, IssueKind, PathVariable, PlatformMode, ResolutionReport, WhyNotReport,
};
use crate::normalize::comparison_key;
use crate::path_env::parse_path;
use crate::platform::resolve_platform_rules;
use crate::policy::PathPolicy;
use crate::resolve::resolve_command;

pub(crate) struct WhyNotOptions<'a> {
    pub(crate) path_value: &'a str,
    pub(crate) command: &'a str,
    pub(crate) platform_mode: PlatformMode,
    pub(crate) pathext: Option<&'a str>,
    pub(crate) cwd: &'a Path,
    pub(crate) home_dir: Option<&'a Path>,
    pub(crate) user_profile_dir: Option<&'a Path>,
}

pub(crate) fn analyze_why_not(options: &WhyNotOptions<'_>) -> WhyNotReport {
    let resolution = resolve_command(
        options.path_value,
        options.command,
        options.platform_mode,
        options.pathext,
        options.cwd,
        true,
    );
    let path_report = diagnose_path_with_policy(
        options.path_value,
        options.platform_mode,
        options.pathext,
        PathVariable::Path,
        &PathPolicy::default(),
    );
    let path_diagnostics = path_report
        .diagnostics
        .into_iter()
        .filter(is_path_health_blocker)
        .collect::<Vec<_>>();
    let missing_common_dirs = missing_common_tool_directories(options);
    let advice = advice_for(&resolution, &path_diagnostics, &missing_common_dirs);

    WhyNotReport {
        command: resolution.command,
        candidates: resolution.candidates,
        searched_directories: resolution.searched_directories,
        related_hints: resolution.related_hints,
        path_diagnostics,
        advice,
    }
}

fn is_path_health_blocker(diagnostic: &Diagnostic) -> bool {
    matches!(
        diagnostic.kind,
        IssueKind::Empty | IssueKind::Missing | IssueKind::NotDirectory | IssueKind::Unreadable
    )
}

fn advice_for(
    resolution: &ResolutionReport,
    path_diagnostics: &[Diagnostic],
    missing_common_dirs: &[String],
) -> Vec<String> {
    if resolution.candidates.iter().any(|candidate| candidate.wins) {
        return vec!["The exact command is already available from PATH.".to_owned()];
    }

    let mut advice = Vec::new();
    advice.push(
        "Check that the command is installed and that its executable directory is present in PATH."
            .to_owned(),
    );
    if !path_diagnostics.is_empty() {
        advice.push(
            "Review the PATH diagnostics above; broken entries can hide expected tools.".to_owned(),
        );
    }
    if !resolution.related_hints.is_empty() {
        advice.push(
            "A related executable name was found; verify the exact command name you meant to run."
                .to_owned(),
        );
    }
    if !missing_common_dirs.is_empty() {
        advice.push(format!(
            "Common tool directories absent from PATH: {}.",
            missing_common_dirs.join(", ")
        ));
    }
    advice.push(
        "If the tool is installed in a custom location, add that directory through your shell startup configuration."
            .to_owned(),
    );
    advice
}

fn missing_common_tool_directories(options: &WhyNotOptions<'_>) -> Vec<String> {
    let rules = resolve_platform_rules(options.platform_mode, options.pathext);
    let path_keys = parse_path(options.path_value, options.platform_mode, options.pathext)
        .into_iter()
        .filter(|entry| !entry.is_empty)
        .map(|entry| entry.comparison_key)
        .collect::<HashSet<_>>();

    common_tool_directories(options, rules.mode)
        .into_iter()
        .filter(|directory| !path_keys.contains(&comparison_key(directory, &rules)))
        .collect()
}

fn common_tool_directories(
    options: &WhyNotOptions<'_>,
    platform_mode: PlatformMode,
) -> Vec<String> {
    match platform_mode {
        PlatformMode::Windows => windows_common_tool_directories(options),
        PlatformMode::Auto | PlatformMode::Posix => posix_common_tool_directories(options),
    }
}

fn posix_common_tool_directories(options: &WhyNotOptions<'_>) -> Vec<String> {
    let mut directories = Vec::new();
    if matches!(
        options.command,
        "cargo" | "clippy" | "rustc" | "rustfmt" | "rustup"
    ) {
        push_home_child(&mut directories, options.home_dir, ".cargo/bin");
    }
    if matches!(
        options.command,
        "python" | "python3" | "pip" | "pip3" | "pyenv"
    ) {
        push_home_child(&mut directories, options.home_dir, ".pyenv/shims");
    }
    push_home_child(&mut directories, options.home_dir, ".local/bin");
    push_unique(&mut directories, "/opt/homebrew/bin".to_owned());
    push_unique(&mut directories, "/usr/local/bin".to_owned());
    directories
}

fn windows_common_tool_directories(options: &WhyNotOptions<'_>) -> Vec<String> {
    let mut directories = Vec::new();
    if matches!(
        options.command,
        "cargo" | "clippy" | "rustc" | "rustfmt" | "rustup"
    ) {
        push_home_child(&mut directories, options.user_profile_dir, ".cargo/bin");
    }
    push_home_child(
        &mut directories,
        options.user_profile_dir,
        "AppData/Local/Microsoft/WindowsApps",
    );
    directories
}

fn push_home_child(directories: &mut Vec<String>, home_dir: Option<&Path>, child: &str) {
    if let Some(home_dir) = home_dir {
        push_unique(directories, home_dir.join(child).display().to_string());
    }
}

fn push_unique(values: &mut Vec<String>, value: String) {
    if !values.contains(&value) {
        values.push(value);
    }
}

#[cfg(test)]
#[path = "why_not/tests.rs"]
mod tests;
