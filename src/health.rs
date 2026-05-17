use std::collections::BTreeMap;

use crate::model::{Diagnostic, DoctorReport, HealthReport, HealthSeverity, IssueKind};

pub(crate) fn summarize_health(report: DoctorReport) -> HealthReport {
    let entry_count = report.entries.len();
    let issue_count = report.diagnostics.len();
    let counts = count_diagnostics(&report.diagnostics);
    let worst_severity = report
        .diagnostics
        .iter()
        .map(|diagnostic| severity_for(diagnostic.kind))
        .max()
        .unwrap_or(HealthSeverity::None);
    let penalty = report.diagnostics.iter().fold(0u8, |penalty, diagnostic| {
        penalty.saturating_add(penalty_for(diagnostic.kind))
    });
    let score = 100u8.saturating_sub(penalty);
    let healthy = score == 100 && issue_count == 0;

    HealthReport {
        variable: report.variable,
        score,
        healthy,
        entry_count,
        issue_count,
        worst_severity,
        counts,
        diagnostics: report.diagnostics,
    }
}

fn count_diagnostics(diagnostics: &[Diagnostic]) -> BTreeMap<IssueKind, usize> {
    let mut counts = BTreeMap::new();
    for diagnostic in diagnostics {
        *counts.entry(diagnostic.kind).or_default() += 1;
    }
    counts
}

fn severity_for(kind: IssueKind) -> HealthSeverity {
    match kind {
        IssueKind::Empty | IssueKind::Missing | IssueKind::NotDirectory | IssueKind::Unreadable => {
            HealthSeverity::Error
        }
        IssueKind::Duplicate
        | IssueKind::SuspiciousOrder
        | IssueKind::ShadowedCommand
        | IssueKind::Unwanted => HealthSeverity::Warning,
    }
}

fn penalty_for(kind: IssueKind) -> u8 {
    match severity_for(kind) {
        HealthSeverity::Error => 15,
        HealthSeverity::Warning => 5,
        HealthSeverity::None => 0,
    }
}

#[cfg(test)]
#[path = "health/tests.rs"]
mod tests;
