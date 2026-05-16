use crate::model::{Diagnostic, DoctorReport, HealthSeverity, IssueKind, PathEntry, PathVariable};

use super::summarize_health;

#[test]
fn summarize_health_reports_perfect_score_for_clean_report() {
    let report = summarize_health(doctor_report(vec![entry(1, "/ok")], Vec::new()));

    assert_eq!(report.score, 100);
    assert!(report.healthy);
    assert_eq!(report.worst_severity, HealthSeverity::None);
}

#[test]
fn summarize_health_clamps_score_at_zero() {
    let diagnostics = (0..8)
        .map(|index| diagnostic(IssueKind::Missing, index + 1, "/missing"))
        .collect();

    let report = summarize_health(doctor_report(Vec::new(), diagnostics));

    assert_eq!(report.score, 0);
    assert!(!report.healthy);
}

#[test]
fn summarize_health_maps_warning_and_error_severity() {
    let warning = summarize_health(doctor_report(
        Vec::new(),
        vec![diagnostic(IssueKind::Duplicate, 1, "/dup")],
    ));
    let error = summarize_health(doctor_report(
        Vec::new(),
        vec![
            diagnostic(IssueKind::Duplicate, 1, "/dup"),
            diagnostic(IssueKind::Unreadable, 2, "/blocked"),
        ],
    ));

    assert_eq!(warning.worst_severity, HealthSeverity::Warning);
    assert_eq!(error.worst_severity, HealthSeverity::Error);
}

#[test]
fn summarize_health_counts_issue_kinds_by_stable_names() {
    let report = summarize_health(doctor_report(
        Vec::new(),
        vec![
            diagnostic(IssueKind::Missing, 1, "/missing"),
            diagnostic(IssueKind::Missing, 2, "/missing-too"),
            diagnostic(IssueKind::Unwanted, 3, "/drop"),
        ],
    ));
    let names = report
        .counts
        .keys()
        .map(|kind| kind.as_str())
        .collect::<Vec<_>>();

    assert_eq!(report.counts[&IssueKind::Missing], 2);
    assert_eq!(report.counts[&IssueKind::Unwanted], 1);
    assert_eq!(names, ["missing", "unwanted"]);
}

fn doctor_report(entries: Vec<PathEntry>, diagnostics: Vec<Diagnostic>) -> DoctorReport {
    DoctorReport {
        variable: PathVariable::Path,
        entries,
        diagnostics,
    }
}

fn entry(index: usize, raw: &str) -> PathEntry {
    PathEntry {
        index,
        raw: raw.to_owned(),
        display: raw.to_owned(),
        comparison_key: raw.to_owned(),
        exists: true,
        is_dir: true,
        is_empty: false,
    }
}

fn diagnostic(kind: IssueKind, index: usize, value: &str) -> Diagnostic {
    Diagnostic {
        kind,
        message: format!("{value} has an issue"),
        entry_index: Some(index),
        entry_value: Some(value.to_owned()),
        related_indexes: vec![index],
    }
}
