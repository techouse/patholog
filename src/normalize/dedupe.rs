use crate::model::PathEntry;

pub(crate) fn first_unique_entries(entries: &[PathEntry]) -> Vec<PathEntry> {
    let mut output = Vec::new();
    let mut seen = Vec::<String>::new();
    for entry in entries {
        if entry.is_empty || seen.contains(&entry.comparison_key) {
            continue;
        }
        seen.push(entry.comparison_key.clone());
        output.push(entry.clone());
    }
    output
}
