use serde::Serialize;

/// Stable diagnostic issue kinds.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
pub enum IssueKind {
    /// Duplicate PATH entries after conservative comparison-key normalization.
    Duplicate,
    /// Empty PATH entry.
    Empty,
    /// PATH entry that does not exist.
    Missing,
    /// PATH entry that exists but is not a directory.
    NotDirectory,
    /// Heuristic warning about PATH entry ordering.
    SuspiciousOrder,
}

impl IssueKind {
    /// Stable string used in CLI flags and JSON output.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Duplicate => "duplicate",
            Self::Empty => "empty",
            Self::Missing => "missing",
            Self::NotDirectory => "not_directory",
            Self::SuspiciousOrder => "suspicious_order",
        }
    }
}

/// A diagnostic produced by `patholog doctor`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Diagnostic {
    /// Stable diagnostic kind.
    pub kind: IssueKind,
    /// Human-readable diagnostic message.
    pub message: String,
    /// Primary PATH entry index, if applicable.
    pub entry_index: Option<usize>,
    /// Primary PATH entry value, if applicable.
    pub entry_value: Option<String>,
    /// Related PATH entry indexes.
    pub related_indexes: Vec<usize>,
}
