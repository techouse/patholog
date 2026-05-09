use super::{Diagnostic, PathEntry, PathVariable};

/// Path-like environment variable health report.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DoctorReport {
    /// Path-like variable being diagnosed.
    pub variable: PathVariable,
    /// Parsed path-like variable entries.
    pub entries: Vec<PathEntry>,
    /// Diagnostics emitted for the entries.
    pub diagnostics: Vec<Diagnostic>,
}
