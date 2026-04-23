use crate::model::{Diagnostic, IssueKind, PathEntry};

pub(super) fn diagnostics(entries: &[PathEntry]) -> Vec<Diagnostic> {
    let mut groups: Vec<(&str, Vec<&PathEntry>)> = Vec::new();
    for entry in entries {
        if entry.is_empty {
            continue;
        }
        if let Some((_, group_entries)) = groups
            .iter_mut()
            .find(|(key, _entries)| *key == entry.comparison_key)
        {
            group_entries.push(entry);
        } else {
            groups.push((&entry.comparison_key, vec![entry]));
        }
    }

    groups
        .into_iter()
        .filter_map(|(_key, duplicate_entries)| duplicate_diagnostic(&duplicate_entries))
        .collect()
}

fn duplicate_diagnostic(duplicate_entries: &[&PathEntry]) -> Option<Diagnostic> {
    if duplicate_entries.len() < 2 {
        return None;
    }
    let first = duplicate_entries[0];
    let related_indexes = duplicate_entries
        .iter()
        .map(|entry| entry.index)
        .collect::<Vec<_>>();
    Some(Diagnostic {
        kind: IssueKind::Duplicate,
        message: format!(
            "{} appears {} times",
            first.display,
            duplicate_entries.len()
        ),
        entry_index: Some(first.index),
        entry_value: Some(first.display.clone()),
        related_indexes,
    })
}
