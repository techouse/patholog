use crate::model::{Diagnostic, IssueKind, WhyNotReport};

use super::shared::finish_lines;

pub(crate) fn format_why_not(report: &WhyNotReport) -> String {
    let mut lines = vec![format!("Command: {}", report.command), String::new()];
    if let Some(winner) = report.winner() {
        lines.extend([
            "Available in PATH:".to_owned(),
            format!("  {}", winner.path),
            String::new(),
            "Status:".to_owned(),
            "  The exact command is already available.".to_owned(),
        ]);
        return finish_lines(lines);
    }

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
    append_path_diagnostics(&mut lines, report);
    append_related_hints(&mut lines, report);
    append_advice(&mut lines, report);
    finish_lines(lines)
}

fn append_path_diagnostics(lines: &mut Vec<String>, report: &WhyNotReport) {
    if report.path_diagnostics.is_empty() {
        return;
    }

    lines.push(String::new());
    lines.push("PATH diagnostics:".to_owned());
    lines.extend(report.path_diagnostics.iter().map(format_diagnostic));
}

fn format_diagnostic(diagnostic: &Diagnostic) -> String {
    match diagnostic.kind {
        IssueKind::Empty => format!(
            "  empty  {}  <empty>",
            diagnostic.entry_index.unwrap_or_default()
        ),
        IssueKind::Missing => format_indexed("missing", diagnostic),
        IssueKind::NotDirectory => format_indexed("not_directory", diagnostic),
        IssueKind::Unreadable => format_indexed("unreadable", diagnostic),
        IssueKind::Duplicate
        | IssueKind::SuspiciousOrder
        | IssueKind::ShadowedCommand
        | IssueKind::Unwanted => format!("  {}  {}", diagnostic.kind.as_str(), diagnostic.message),
    }
}

fn format_indexed(label: &str, diagnostic: &Diagnostic) -> String {
    format!(
        "  {label}  {}  {}",
        diagnostic.entry_index.unwrap_or_default(),
        diagnostic.entry_value.as_deref().unwrap_or("")
    )
}

fn append_related_hints(lines: &mut Vec<String>, report: &WhyNotReport) {
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

fn append_advice(lines: &mut Vec<String>, report: &WhyNotReport) {
    if report.advice.is_empty() {
        return;
    }

    lines.push(String::new());
    lines.push("Advice:".to_owned());
    lines.extend(report.advice.iter().map(|advice| format!("  {advice}")));
}
