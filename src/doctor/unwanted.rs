use crate::model::{Diagnostic, IssueKind, PathEntry, PlatformMode};
use crate::policy::PathPolicy;

pub(super) fn diagnostics(
    entries: &[PathEntry],
    policy: &PathPolicy,
    platform_mode: PlatformMode,
    pathext: Option<&str>,
) -> Vec<Diagnostic> {
    if policy.is_empty() {
        return Vec::new();
    }
    let policy = policy.compile(platform_mode, pathext);
    entries
        .iter()
        .filter(|entry| policy.matches_entry(entry))
        .map(unwanted_diagnostic)
        .collect()
}

fn unwanted_diagnostic(entry: &PathEntry) -> Diagnostic {
    Diagnostic {
        kind: IssueKind::Unwanted,
        message: format!("{} is marked for removal", entry.display),
        entry_index: Some(entry.index),
        entry_value: Some(entry.display.clone()),
        related_indexes: vec![entry.index],
    }
}
