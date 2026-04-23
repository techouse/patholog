use crate::model::PathEntry;

use super::shared::finish_lines;

pub(crate) fn format_print(entries: &[PathEntry]) -> String {
    finish_lines(
        entries
            .iter()
            .map(|entry| format!("{}  {}", entry.index, entry_display(entry))),
    )
}

fn entry_display(entry: &PathEntry) -> &str {
    if entry.is_empty {
        "<empty>"
    } else {
        &entry.display
    }
}
