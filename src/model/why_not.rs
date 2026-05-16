use super::{Diagnostic, RelatedExecutableHint, ResolutionCandidate};

/// Result of missing-command analysis.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WhyNotReport {
    /// Requested command.
    pub command: String,
    /// Exact executable candidates.
    pub candidates: Vec<ResolutionCandidate>,
    /// PATH directories searched, in order.
    pub searched_directories: Vec<String>,
    /// Advisory related executable hints.
    pub related_hints: Vec<RelatedExecutableHint>,
    /// PATH health diagnostics relevant to missing command lookup.
    pub path_diagnostics: Vec<Diagnostic>,
    /// Ordered advisory next checks.
    pub advice: Vec<String>,
}

impl WhyNotReport {
    /// Returns true when an exact executable candidate was found.
    #[must_use]
    pub fn found(&self) -> bool {
        self.winner().is_some()
    }

    /// Returns the winning exact executable candidate, if any.
    #[must_use]
    pub fn winner(&self) -> Option<&ResolutionCandidate> {
        self.candidates.iter().find(|candidate| candidate.wins)
    }
}
