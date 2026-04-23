use crate::model::{Diagnostic, IssueKind, PathEntry};

const USER_TOOL_DIR_SUFFIXES: &[&str] = &["/.cargo/bin", "/.local/bin", "/.pyenv/shims"];
const SYSTEM_DIRS: &[&str] = &["/bin", "/usr/bin", "/usr/sbin", "/sbin"];

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

fn homebrew_order_diagnostic(usr_bin: &PathEntry, homebrew: &PathEntry) -> Diagnostic {
    Diagnostic {
        kind: IssueKind::SuspiciousOrder,
        message: "/usr/bin appears before /opt/homebrew/bin".to_owned(),
        entry_index: Some(usr_bin.index),
        entry_value: Some(usr_bin.display.clone()),
        related_indexes: vec![usr_bin.index, homebrew.index],
    }
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
        !entry.is_empty
            && USER_TOOL_DIR_SUFFIXES
                .iter()
                .any(|suffix| entry.raw.ends_with(suffix))
    })
}
