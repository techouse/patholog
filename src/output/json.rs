use serde_json::{Value, json};

use crate::model::{Diagnostic, DoctorReport, PathEntry, ResolutionCandidate, ResolutionReport};

pub(crate) fn dumps_json(value: &Value) -> Result<String, serde_json::Error> {
    serde_json::to_string_pretty(value).map(|json| format!("{json}\n"))
}

pub(crate) fn entries_to_json(entries: &[PathEntry]) -> Value {
    Value::Array(entries.iter().map(entry_to_json).collect())
}

pub(crate) fn doctor_to_json(report: &DoctorReport) -> Value {
    json!({
        "entries": report.entries.iter().map(entry_to_json).collect::<Vec<_>>(),
        "diagnostics": report.diagnostics.iter().map(diagnostic_to_json).collect::<Vec<_>>(),
    })
}

pub(crate) fn resolution_to_json(report: &ResolutionReport) -> Value {
    let winner = report
        .candidates
        .iter()
        .find(|candidate| candidate.wins)
        .map(candidate_to_json);

    json!({
        "command": report.command,
        "found": winner.is_some(),
        "winner": winner,
        "candidates": report.candidates.iter().map(candidate_to_json).collect::<Vec<_>>(),
        "searched_directories": report.searched_directories,
        "related_hints": report.related_hints.iter().map(|hint| {
            json!({
                "command": hint.command,
                "paths": hint.paths,
            })
        }).collect::<Vec<_>>(),
    })
}

fn entry_to_json(entry: &PathEntry) -> Value {
    json!({
        "index": entry.index,
        "raw": entry.raw,
        "display": entry.display,
        "comparison_key": entry.comparison_key,
        "kind": entry_kind(entry),
        "exists": entry.exists,
        "is_dir": entry.is_dir,
        "is_empty": entry.is_empty,
    })
}

fn entry_kind(entry: &PathEntry) -> &'static str {
    if entry.is_empty {
        "empty"
    } else if !entry.exists {
        "missing"
    } else if !entry.is_dir {
        "not_directory"
    } else {
        "directory"
    }
}

fn diagnostic_to_json(diagnostic: &Diagnostic) -> Value {
    json!({
        "kind": diagnostic.kind.as_str(),
        "message": diagnostic.message,
        "entry_index": diagnostic.entry_index,
        "entry_value": diagnostic.entry_value,
        "related_indexes": diagnostic.related_indexes,
    })
}

fn candidate_to_json(candidate: &ResolutionCandidate) -> Value {
    json!({
        "entry_index": candidate.entry_index,
        "directory": candidate.directory,
        "path": candidate.path,
        "wins": candidate.wins,
    })
}
