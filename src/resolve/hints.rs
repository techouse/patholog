use std::path::Path;

use crate::model::{PathEntry, PlatformRules, RelatedExecutableHint};

use super::candidates::find_candidates;

pub(super) fn related_hints(
    entries: &[PathEntry],
    command: &str,
    rules: &PlatformRules,
    cwd: &Path,
) -> Vec<RelatedExecutableHint> {
    related_commands(command)
        .iter()
        .filter_map(|related_command| {
            let candidates = find_candidates(entries, related_command, rules, cwd);
            if candidates.is_empty() {
                None
            } else {
                Some(RelatedExecutableHint {
                    command: (*related_command).to_owned(),
                    paths: candidates
                        .into_iter()
                        .map(|candidate| candidate.path)
                        .collect(),
                })
            }
        })
        .collect()
}

fn related_commands(command: &str) -> &'static [&'static str] {
    match command {
        "python" => &["python3"],
        "python3" => &["python"],
        "pip" => &["pip3"],
        "pip3" => &["pip"],
        _ => &[],
    }
}
