use std::collections::BTreeMap;

use super::{Diagnostic, IssueKind, PathVariable};

/// Severity summary for path health diagnostics.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum HealthSeverity {
    /// No diagnostics were reported.
    None,
    /// Only warning-level diagnostics were reported.
    Warning,
    /// At least one error-level diagnostic was reported.
    Error,
}

impl HealthSeverity {
    /// Stable string used in JSON and human output.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::Warning => "warning",
            Self::Error => "error",
        }
    }
}

/// Compact health summary derived from doctor diagnostics.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HealthReport {
    /// Path-like variable being scored.
    pub variable: PathVariable,
    /// Deterministic health score in the range 0..=100.
    pub score: u8,
    /// Whether there are no diagnostics and the score is perfect.
    pub healthy: bool,
    /// Number of parsed entries.
    pub entry_count: usize,
    /// Number of diagnostics.
    pub issue_count: usize,
    /// Worst diagnostic severity.
    pub worst_severity: HealthSeverity,
    /// Diagnostic counts grouped by stable issue kind.
    pub counts: BTreeMap<IssueKind, usize>,
    /// Diagnostics used to compute the summary.
    pub diagnostics: Vec<Diagnostic>,
}
