use crate::model::{Diagnostic, IssueKind, PathEntry};

pub(super) fn diagnostics(entries: &[PathEntry]) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();
    for entry in entries {
        if entry.is_empty {
            diagnostics.push(empty_diagnostic(entry));
        } else if !entry.exists {
            diagnostics.push(missing_diagnostic(entry));
        } else if !entry.is_dir {
            diagnostics.push(not_directory_diagnostic(entry));
        } else if !is_readable_directory(entry) {
            diagnostics.push(unreadable_diagnostic(entry));
        }
    }
    diagnostics
}

fn is_readable_directory(entry: &PathEntry) -> bool {
    std::fs::read_dir(&entry.raw).is_ok()
}

fn empty_diagnostic(entry: &PathEntry) -> Diagnostic {
    Diagnostic {
        kind: IssueKind::Empty,
        message: format!("entry {} is empty", entry.index),
        entry_index: Some(entry.index),
        entry_value: Some(entry.display.clone()),
        related_indexes: Vec::new(),
    }
}

fn missing_diagnostic(entry: &PathEntry) -> Diagnostic {
    Diagnostic {
        kind: IssueKind::Missing,
        message: format!("{} does not exist", entry.display),
        entry_index: Some(entry.index),
        entry_value: Some(entry.display.clone()),
        related_indexes: Vec::new(),
    }
}

fn not_directory_diagnostic(entry: &PathEntry) -> Diagnostic {
    Diagnostic {
        kind: IssueKind::NotDirectory,
        message: format!("{} is not a directory", entry.display),
        entry_index: Some(entry.index),
        entry_value: Some(entry.display.clone()),
        related_indexes: Vec::new(),
    }
}

fn unreadable_diagnostic(entry: &PathEntry) -> Diagnostic {
    Diagnostic {
        kind: IssueKind::Unreadable,
        message: format!("{} cannot be read", entry.display),
        entry_index: Some(entry.index),
        entry_value: Some(entry.display.clone()),
        related_indexes: Vec::new(),
    }
}
