use std::fs;
use std::path::{Path, PathBuf};

use crate::clean::{clean_export_with_policy, clean_path_with_policy};
use crate::model::{ApplyAction, ApplyPlan, PathVariable, PlatformMode, ShellKind};
use crate::platform::resolve_platform_rules;
use crate::policy::PathPolicy;

pub(crate) const START_MARKER: &str = "# >>> patholog PATH >>>";
pub(crate) const END_MARKER: &str = "# <<< patholog PATH <<<";

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ApplyPlanOptions<'a> {
    pub(crate) path_value: &'a str,
    pub(crate) platform_mode: PlatformMode,
    pub(crate) pathext: Option<&'a str>,
    pub(crate) shell: ShellKind,
    pub(crate) home_dir: Option<&'a Path>,
    pub(crate) user_profile_dir: Option<&'a Path>,
    pub(crate) profile: Option<&'a Path>,
    pub(crate) policy: PathPolicy,
}

pub(crate) fn plan_apply(options: &ApplyPlanOptions<'_>) -> Result<ApplyPlan, String> {
    let profile_path = target_profile(options)?;
    let cleaned_path = clean_path_with_policy(
        options.path_value,
        options.platform_mode,
        options.pathext,
        PathVariable::Path,
        &options.policy,
    );
    let planned_block = managed_block(&clean_export_with_policy(
        options.path_value,
        options.platform_mode,
        options.pathext,
        options.shell,
        PathVariable::Path,
        &options.policy,
    ));

    match fs::metadata(&profile_path) {
        Ok(metadata) if !metadata.is_file() => Err(format!(
            "apply target profile is not a file: {}",
            profile_path.display()
        )),
        Ok(_metadata) => {
            plan_existing_profile(options.shell, &profile_path, planned_block, cleaned_path)
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(ApplyPlan {
            shell: options.shell,
            profile_path: profile_path.display().to_string(),
            action: ApplyAction::CreateProfile,
            existing_block: None,
            planned_block,
            cleaned_path,
            would_write: false,
        }),
        Err(error) => Err(format!(
            "apply target profile is not readable: {} ({error})",
            profile_path.display()
        )),
    }
}

fn target_profile(options: &ApplyPlanOptions<'_>) -> Result<PathBuf, String> {
    if let Some(profile) = options.profile {
        return Ok(profile.to_path_buf());
    }

    let Some(home) = default_home(options) else {
        return Err(missing_home_message(options).to_owned());
    };

    Ok(match options.shell {
        ShellKind::Bash => home.join(".bashrc"),
        ShellKind::Fish => home.join(".config/fish/config.fish"),
        ShellKind::Pwsh => pwsh_profile(home, options.platform_mode),
        ShellKind::Zsh => home.join(".zshrc"),
    })
}

fn default_home<'a>(options: &'a ApplyPlanOptions<'_>) -> Option<&'a Path> {
    match (
        options.shell,
        resolve_platform_rules(options.platform_mode, None).mode,
    ) {
        (ShellKind::Pwsh, PlatformMode::Windows) => options.user_profile_dir,
        _ => options.home_dir,
    }
}

fn missing_home_message(options: &ApplyPlanOptions<'_>) -> &'static str {
    match (
        options.shell,
        resolve_platform_rules(options.platform_mode, None).mode,
    ) {
        (ShellKind::Pwsh, PlatformMode::Windows) => {
            "apply requires a home directory; set USERPROFILE, pass --home, or pass --profile"
        }
        _ => "apply requires a home directory; set HOME, pass --home, or pass --profile",
    }
}

fn pwsh_profile(home: &Path, platform_mode: PlatformMode) -> PathBuf {
    match resolve_platform_rules(platform_mode, None).mode {
        PlatformMode::Windows => home.join("Documents/PowerShell/Microsoft.PowerShell_profile.ps1"),
        PlatformMode::Auto | PlatformMode::Posix => {
            home.join(".config/powershell/Microsoft.PowerShell_profile.ps1")
        }
    }
}

fn plan_existing_profile(
    shell: ShellKind,
    profile_path: &Path,
    planned_block: String,
    cleaned_path: String,
) -> Result<ApplyPlan, String> {
    let content = fs::read_to_string(profile_path).map_err(|error| {
        format!(
            "apply target profile is not readable: {} ({error})",
            profile_path.display()
        )
    })?;
    let existing_block = existing_managed_block(&content)?;
    let action = if existing_block.is_some() {
        ApplyAction::ReplaceBlock
    } else {
        ApplyAction::AppendBlock
    };

    Ok(ApplyPlan {
        shell,
        profile_path: profile_path.display().to_string(),
        action,
        existing_block,
        planned_block,
        cleaned_path,
        would_write: false,
    })
}

pub(crate) fn managed_block(snippet: &str) -> String {
    format!("{START_MARKER}\n{snippet}\n{END_MARKER}")
}

pub(crate) fn existing_managed_block(content: &str) -> Result<Option<String>, String> {
    let starts = marker_offsets(content, START_MARKER);
    let ends = marker_offsets(content, END_MARKER);
    match (starts.as_slice(), ends.as_slice()) {
        ([], []) => Ok(None),
        ([start], [end]) if start < end => Ok(Some(existing_block_content(content, *start, *end))),
        ([start], [end]) if start >= end => {
            Err("apply target profile contains a malformed patholog managed block".to_owned())
        }
        _ => Err(
            "apply target profile contains duplicate or malformed patholog managed blocks"
                .to_owned(),
        ),
    }
}

fn marker_offsets(content: &str, marker: &str) -> Vec<usize> {
    let mut offsets = Vec::new();
    let mut offset = 0;
    for segment in content.split_inclusive('\n') {
        let without_newline = segment.strip_suffix('\n').unwrap_or(segment);
        let line = without_newline
            .strip_suffix('\r')
            .unwrap_or(without_newline);
        if line == marker {
            offsets.push(offset);
        }
        offset += segment.len();
    }
    offsets
}

fn existing_block_content(content: &str, start: usize, end: usize) -> String {
    let after_end_marker = end + END_MARKER.len();
    let block_end = content[after_end_marker..]
        .find('\n')
        .map_or(after_end_marker, |newline| after_end_marker + newline);
    content[start..block_end].to_owned()
}

#[cfg(test)]
#[path = "apply/tests.rs"]
mod tests;
