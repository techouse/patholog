use super::{Diagnostic, PathEntry};

/// PATH health report.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DoctorReport {
    /// Parsed PATH entries.
    pub entries: Vec<PathEntry>,
    /// Diagnostics emitted for the entries.
    pub diagnostics: Vec<Diagnostic>,
}
