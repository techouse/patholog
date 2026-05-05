use crate::model::{Diagnostic, IssueKind, PathEntry, PresetKind};
use crate::policy::PathPolicy;

const USER_TOOL_DIR_SUFFIXES: &[&str] = &["/.cargo/bin", "/.local/bin", "/.pyenv/shims"];
const SYSTEM_DIRS: &[&str] = &["/bin", "/usr/bin", "/usr/sbin", "/sbin"];
const HOMEBREW_DIRS: &[&str] = &["/opt/homebrew/bin", "/usr/local/bin"];

pub(super) fn diagnostics(entries: &[PathEntry]) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();

    let usr_bin = first_raw_entry(entries, "/usr/bin");
    let homebrew = first_raw_entry(entries, "/opt/homebrew/bin");
    if let (Some(usr_bin), Some(homebrew)) = (usr_bin, homebrew)
        && usr_bin.index < homebrew.index
    {
        diagnostics.push(homebrew_order_diagnostic(usr_bin, homebrew));
    }

    let first_system = first_matching_entry(entries, SYSTEM_DIRS);
    let first_user_tool = first_user_tool_entry(entries);
    if let (Some(first_system), Some(first_user_tool)) = (first_system, first_user_tool)
        && first_system.index < first_user_tool.index
    {
        diagnostics.push(user_tool_order_diagnostic(first_system, first_user_tool));
    }

    diagnostics
}

pub(super) fn preset_diagnostics(entries: &[PathEntry], policy: &PathPolicy) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();
    for preset in policy.ordering_presets() {
        match preset {
            PresetKind::Homebrew => {
                if let Some(diagnostic) = homebrew_preset_diagnostic(entries) {
                    diagnostics.push(diagnostic);
                }
            }
            PresetKind::Cargo => {
                if let Some(diagnostic) = user_tool_preset_diagnostic(entries, "/.cargo/bin") {
                    diagnostics.push(diagnostic);
                }
            }
            PresetKind::Pyenv => {
                if let Some(diagnostic) = user_tool_preset_diagnostic(entries, "/.pyenv/shims") {
                    diagnostics.push(diagnostic);
                }
            }
            PresetKind::Fink => {}
        }
    }
    diagnostics
}

fn homebrew_order_diagnostic(usr_bin: &PathEntry, homebrew: &PathEntry) -> Diagnostic {
    Diagnostic {
        kind: IssueKind::SuspiciousOrder,
        message: "/usr/bin appears before /opt/homebrew/bin".to_owned(),
        entry_index: Some(usr_bin.index),
        entry_value: Some(usr_bin.display.clone()),
        related_indexes: vec![usr_bin.index, homebrew.index],
    }
}

fn homebrew_preset_diagnostic(entries: &[PathEntry]) -> Option<Diagnostic> {
    let first_system = first_matching_entry(entries, SYSTEM_DIRS)?;
    let first_homebrew = first_matching_entry(entries, HOMEBREW_DIRS)?;
    if first_system.index < first_homebrew.index {
        return Some(user_tool_order_diagnostic(first_system, first_homebrew));
    }
    None
}

fn user_tool_preset_diagnostic(entries: &[PathEntry], suffix: &str) -> Option<Diagnostic> {
    let first_system = first_matching_entry(entries, SYSTEM_DIRS)?;
    let first_user_tool = first_entry_with_suffix(entries, suffix)?;
    if first_system.index < first_user_tool.index {
        return Some(user_tool_order_diagnostic(first_system, first_user_tool));
    }
    None
}

fn user_tool_order_diagnostic(first_system: &PathEntry, first_user_tool: &PathEntry) -> Diagnostic {
    Diagnostic {
        kind: IssueKind::SuspiciousOrder,
        message: format!(
            "{} appears before {}",
            first_system.display, first_user_tool.display
        ),
        entry_index: Some(first_system.index),
        entry_value: Some(first_system.display.clone()),
        related_indexes: vec![first_system.index, first_user_tool.index],
    }
}

fn first_raw_entry<'a>(entries: &'a [PathEntry], raw: &str) -> Option<&'a PathEntry> {
    entries
        .iter()
        .find(|entry| !entry.is_empty && entry.raw == raw)
}

fn first_matching_entry<'a>(
    entries: &'a [PathEntry],
    raw_values: &[&str],
) -> Option<&'a PathEntry> {
    entries
        .iter()
        .find(|entry| !entry.is_empty && raw_values.contains(&entry.raw.as_str()))
}

fn first_user_tool_entry(entries: &[PathEntry]) -> Option<&PathEntry> {
    entries.iter().find(|entry| {
        USER_TOOL_DIR_SUFFIXES
            .iter()
            .any(|suffix| entry_has_suffix(entry, suffix))
    })
}

fn first_entry_with_suffix<'a>(entries: &'a [PathEntry], suffix: &str) -> Option<&'a PathEntry> {
    entries.iter().find(|entry| entry_has_suffix(entry, suffix))
}

fn entry_has_suffix(entry: &PathEntry, suffix: &str) -> bool {
    !entry.is_empty && entry.raw.ends_with(suffix)
}
