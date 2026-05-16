use crate::model::{HealthReport, IssueKind};

use super::shared::finish_lines;

pub(crate) fn format_health(report: &HealthReport) -> String {
    let mut lines = vec![
        format!(
            "{} health: {}/100",
            report.variable.env_name(),
            report.score
        ),
        format!(
            "Status: {}",
            if report.healthy {
                "healthy"
            } else {
                "issues found"
            }
        ),
        format!("Entries: {}", report.entry_count),
        format!("Issues: {}", report.issue_count),
        format!("Worst severity: {}", report.worst_severity.as_str()),
    ];

    if !report.counts.is_empty() {
        lines.push(String::new());
        lines.push("Counts:".to_owned());
        for kind in count_order() {
            if let Some(count) = report.counts.get(&kind) {
                lines.push(format!("  {}  {count}", kind.as_str()));
            }
        }
    }

    finish_lines(lines)
}

fn count_order() -> [IssueKind; 8] {
    [
        IssueKind::Missing,
        IssueKind::NotDirectory,
        IssueKind::Unreadable,
        IssueKind::Empty,
        IssueKind::Duplicate,
        IssueKind::Unwanted,
        IssueKind::SuspiciousOrder,
        IssueKind::ShadowedCommand,
    ]
}
