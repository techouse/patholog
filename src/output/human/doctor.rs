use crate::model::{Diagnostic, DoctorReport, IssueKind};

use super::shared::finish_lines;

pub(crate) fn format_doctor(report: &DoctorReport) -> String {
    let mut lines = vec![format!(
        "{} entries: {}",
        report.variable.env_name(),
        report.entries.len()
    )];
    if report.diagnostics.is_empty() {
        lines.push(String::new());
        lines.push("No issues found.".to_owned());
        return finish_lines(lines);
    }

    append_diagnostic_group(
        &mut lines,
        "Duplicates:",
        &report.diagnostics,
        IssueKind::Duplicate,
        format_duplicate,
    );
    append_diagnostic_group(
        &mut lines,
        "Missing directories:",
        &report.diagnostics,
        IssueKind::Missing,
        format_indexed,
    );
    append_diagnostic_group(
        &mut lines,
        "Non-directories:",
        &report.diagnostics,
        IssueKind::NotDirectory,
        format_indexed,
    );
    append_diagnostic_group(
        &mut lines,
        "Unreadable directories:",
        &report.diagnostics,
        IssueKind::Unreadable,
        format_indexed,
    );
    append_diagnostic_group(
        &mut lines,
        "Unwanted entries:",
        &report.diagnostics,
        IssueKind::Unwanted,
        format_indexed,
    );
    append_diagnostic_group(
        &mut lines,
        "Empty entries:",
        &report.diagnostics,
        IssueKind::Empty,
        format_empty,
    );
    append_diagnostic_group(
        &mut lines,
        "Ordering warnings:",
        &report.diagnostics,
        IssueKind::SuspiciousOrder,
        format_message,
    );
    append_diagnostic_group(
        &mut lines,
        "Shadowed commands:",
        &report.diagnostics,
        IssueKind::ShadowedCommand,
        format_message,
    );

    finish_lines(lines)
}

fn append_diagnostic_group(
    lines: &mut Vec<String>,
    heading: &str,
    diagnostics: &[Diagnostic],
    kind: IssueKind,
    formatter: fn(&Diagnostic) -> String,
) {
    let matching = diagnostics
        .iter()
        .filter(|diagnostic| diagnostic.kind == kind)
        .collect::<Vec<_>>();
    if matching.is_empty() {
        return;
    }

    lines.push(String::new());
    lines.push(heading.to_owned());
    lines.extend(matching.iter().map(|diagnostic| formatter(diagnostic)));
}

fn format_duplicate(diagnostic: &Diagnostic) -> String {
    let indexes = diagnostic
        .related_indexes
        .iter()
        .map(usize::to_string)
        .collect::<Vec<_>>()
        .join(", ");
    let value = diagnostic.entry_value.as_deref().unwrap_or("");
    format!(
        "  {value} ({}x; entries {indexes})",
        diagnostic.related_indexes.len()
    )
}

fn format_indexed(diagnostic: &Diagnostic) -> String {
    format!(
        "  {}  {}",
        diagnostic.entry_index.unwrap_or_default(),
        diagnostic.entry_value.as_deref().unwrap_or("")
    )
}

fn format_empty(diagnostic: &Diagnostic) -> String {
    format!("  {}  <empty>", diagnostic.entry_index.unwrap_or_default())
}

fn format_message(diagnostic: &Diagnostic) -> String {
    format!("  {}", diagnostic.message)
}
