use std::path::{Path, PathBuf};

use crate::model::{PathEntry, PlatformRules, ResolutionCandidate};

use super::filesystem::find_executable;
use super::names::candidate_names;

pub(super) fn find_candidates(
    entries: &[PathEntry],
    command: &str,
    rules: &PlatformRules,
    cwd: &Path,
) -> Vec<ResolutionCandidate> {
    let mut candidates = Vec::new();
    let candidate_names = candidate_names(command, rules);

    for entry in entries {
        let Some(directory) = entry_directory(entry, cwd) else {
            continue;
        };
        for candidate_name in &candidate_names {
            let Some(candidate_path) = find_executable(&directory, candidate_name, rules) else {
                continue;
            };
            candidates.push(ResolutionCandidate {
                entry_index: entry.index,
                directory: searched_directory_display(entry),
                path: candidate_path.display().to_string(),
                wins: false,
            });
        }
    }

    mark_winner(candidates)
}

pub(super) fn searched_directory_display(entry: &PathEntry) -> String {
    if entry.is_empty {
        "<empty entry: current directory>".to_owned()
    } else {
        entry.display.clone()
    }
}

fn entry_directory(entry: &PathEntry, cwd: &Path) -> Option<PathBuf> {
    if entry.is_empty {
        return Some(cwd.to_path_buf());
    }
    if entry.is_dir {
        return Some(PathBuf::from(&entry.raw));
    }
    None
}

fn mark_winner(mut candidates: Vec<ResolutionCandidate>) -> Vec<ResolutionCandidate> {
    let Some(winning_key) = candidates
        .first()
        .map(|candidate| (candidate.entry_index, candidate.path.clone()))
    else {
        return Vec::new();
    };

    for candidate in &mut candidates {
        candidate.wins = candidate.entry_index == winning_key.0 && candidate.path == winning_key.1;
    }
    candidates
}
