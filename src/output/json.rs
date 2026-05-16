use serde_json::{Value, json};

use crate::config::{ConfigPolicy, LoadedConfig};
use crate::model::{
    ApplyOutcome, ApplyPlan, Diagnostic, DoctorReport, PathEntry, PathMutation,
    ResolutionCandidate, ResolutionReport, ShellProfile, ShellProfileScanReport, WhyNotReport,
};

pub(crate) fn dumps_json(value: &Value) -> Result<String, serde_json::Error> {
    serde_json::to_string_pretty(value).map(|json| format!("{json}\n"))
}

pub(crate) fn entries_to_json(entries: &[PathEntry]) -> Value {
    Value::Array(entries.iter().map(entry_to_json).collect())
}

pub(crate) fn doctor_to_json(report: &DoctorReport) -> Value {
    json!({
        "variable": report.variable.as_str(),
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

pub(crate) fn why_not_to_json(report: &WhyNotReport) -> Value {
    json!({
        "command": report.command,
        "found": report.found(),
        "winner": report.winner().map(candidate_to_json),
        "candidates": report.candidates.iter().map(candidate_to_json).collect::<Vec<_>>(),
        "searched_directories": report.searched_directories,
        "related_hints": report.related_hints.iter().map(|hint| {
            json!({
                "command": hint.command,
                "paths": hint.paths,
            })
        }).collect::<Vec<_>>(),
        "path_diagnostics": report
            .path_diagnostics
            .iter()
            .map(diagnostic_to_json)
            .collect::<Vec<_>>(),
        "advice": report.advice,
    })
}

pub(crate) fn shell_profile_scan_to_json(report: &ShellProfileScanReport) -> Value {
    json!({
        "home": report.home,
        "profiles": report.profiles.iter().map(shell_profile_to_json).collect::<Vec<_>>(),
    })
}

pub(crate) fn apply_plan_to_json(plan: &ApplyPlan) -> Value {
    json!({
        "shell": plan.shell.as_str(),
        "profile_path": plan.profile_path,
        "action": plan.action.as_str(),
        "existing_block": plan.existing_block,
        "planned_block": plan.planned_block,
        "cleaned_path": plan.cleaned_path,
        "would_write": plan.would_write,
    })
}

pub(crate) fn apply_outcome_to_json(outcome: &ApplyOutcome) -> Value {
    let mut value = apply_plan_to_json(&outcome.plan);
    if let Some(object) = value.as_object_mut() {
        object.insert("wrote".to_owned(), json!(outcome.wrote));
        object.insert("backup_path".to_owned(), json!(outcome.backup_path));
        object.insert("backup_created".to_owned(), json!(outcome.backup_created));
    }
    value
}

pub(crate) fn config_to_json(config: &LoadedConfig) -> Value {
    json!({
        "config_path": config.path.display().to_string(),
        "version": config.config.version,
        "path": config_policy_to_json(&config.config.path),
        "manpath": config_policy_to_json(&config.config.manpath),
    })
}

fn config_policy_to_json(policy: &ConfigPolicy) -> Value {
    json!({
        "drop": policy.drop_entries,
        "preset": policy
            .presets
            .iter()
            .map(|preset| preset.as_str())
            .collect::<Vec<_>>(),
        "fail_on": policy
            .fail_on
            .iter()
            .map(|kind| kind.as_str())
            .collect::<Vec<_>>(),
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

fn shell_profile_to_json(profile: &ShellProfile) -> Value {
    json!({
        "shell": profile.shell,
        "path": profile.path,
        "exists": profile.exists,
        "is_file": profile.is_file,
        "readable": profile.readable,
        "path_mutations": profile.path_mutations.iter().map(path_mutation_to_json).collect::<Vec<_>>(),
    })
}

fn path_mutation_to_json(mutation: &PathMutation) -> Value {
    json!({
        "line": mutation.line,
        "kind": mutation.kind,
        "text": mutation.text,
    })
}
