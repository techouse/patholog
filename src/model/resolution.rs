/// A command executable found through PATH lookup.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ResolutionCandidate {
    /// PATH entry index where the executable was found.
    pub entry_index: usize,
    /// PATH directory display value.
    pub directory: String,
    /// Full executable path.
    pub path: String,
    /// Whether this candidate is the winning executable.
    pub wins: bool,
}

/// Advisory hint for related command names that were not exact PATH matches.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RelatedExecutableHint {
    /// Related command name.
    pub command: String,
    /// Paths found for the related command.
    pub paths: Vec<String>,
}

/// Result of command resolution.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ResolutionReport {
    /// Requested command.
    pub command: String,
    /// Exact executable candidates.
    pub candidates: Vec<ResolutionCandidate>,
    /// PATH directories searched, in order.
    pub searched_directories: Vec<String>,
    /// Advisory related executable hints.
    pub related_hints: Vec<RelatedExecutableHint>,
}
