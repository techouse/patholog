use crate::model::{ResolutionCandidate, ResolutionReport};

use super::shared::finish_lines;

pub(crate) fn format_why(report: &ResolutionReport) -> String {
    let mut lines = vec![format!("Command: {}", report.command), String::new()];
    let Some(winner) = winner(&report.candidates) else {
        lines.push("Not found in PATH.".to_owned());
        lines.push(String::new());
        lines.push("Searched directories:".to_owned());
        lines.extend(
            report
                .searched_directories
                .iter()
                .enumerate()
                .map(|(index, directory)| format!("  {}  {directory}", index + 1)),
        );
        append_related_hints(&mut lines, report);
        return finish_lines(lines);
    };

    lines.extend([
        "Resolved to:".to_owned(),
        format!("  {}", winner.path),
        String::new(),
        "Why:".to_owned(),
        format!(
            "  entry {} ({}) appears before all other matching PATH entries.",
            winner.entry_index, winner.directory
        ),
        String::new(),
        "Other matches:".to_owned(),
    ]);

    append_other_matches(&mut lines, report);
    finish_lines(lines)
}

pub(crate) fn format_conflicts(report: &ResolutionReport) -> String {
    if report.candidates.is_empty() {
        return format!("No matches for {}\n", report.command);
    }

    finish_lines(report.candidates.iter().map(|candidate| {
        let marker = if candidate.wins { "*" } else { " " };
        format!("{}{marker} {}", candidate.entry_index, candidate.path)
    }))
}

fn winner(candidates: &[ResolutionCandidate]) -> Option<&ResolutionCandidate> {
    candidates.iter().find(|candidate| candidate.wins)
}

fn append_other_matches(lines: &mut Vec<String>, report: &ResolutionReport) {
    let other_matches = report
        .candidates
        .iter()
        .filter(|candidate| !candidate.wins)
        .collect::<Vec<_>>();
    if other_matches.is_empty() {
        lines.push("  none".to_owned());
    } else {
        lines.extend(
            other_matches
                .iter()
                .map(|candidate| format!("  {}  {}", candidate.entry_index, candidate.path)),
        );
    }
}

fn append_related_hints(lines: &mut Vec<String>, report: &ResolutionReport) {
    if report.related_hints.is_empty() {
        return;
    }

    lines.push(String::new());
    lines.push("Related executables, not PATH matches:".to_owned());
    for hint in &report.related_hints {
        lines.push(format!("  {}", hint.command));
        lines.extend(hint.paths.iter().map(|path| format!("    {path}")));
    }
}
