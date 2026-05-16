use serde::Serialize;
use std::fmt;
use std::str::FromStr;

/// Stable diagnostic issue kinds.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize)]
pub enum IssueKind {
    /// Duplicate PATH entries after conservative comparison-key normalization.
    Duplicate,
    /// Empty PATH entry.
    Empty,
    /// PATH entry that does not exist.
    Missing,
    /// PATH entry that exists but is not a directory.
    NotDirectory,
    /// PATH entry that exists as a directory but cannot be read.
    Unreadable,
    /// Heuristic warning about PATH entry ordering.
    SuspiciousOrder,
    /// A command candidate hidden by an earlier PATH entry.
    ShadowedCommand,
    /// PATH-like entry explicitly selected for removal.
    Unwanted,
}

impl IssueKind {
    /// Stable accepted strings used in CLI flags and config files.
    pub const SUPPORTED_VALUES: &'static str = "duplicate, empty, missing, not_directory, unreadable, suspicious_order, shadowed_command, unwanted";

    /// Stable string used in CLI flags and JSON output.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Duplicate => "duplicate",
            Self::Empty => "empty",
            Self::Missing => "missing",
            Self::NotDirectory => "not_directory",
            Self::Unreadable => "unreadable",
            Self::SuspiciousOrder => "suspicious_order",
            Self::ShadowedCommand => "shadowed_command",
            Self::Unwanted => "unwanted",
        }
    }
}

impl FromStr for IssueKind {
    type Err = ParseIssueKindError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "duplicate" => Ok(Self::Duplicate),
            "empty" => Ok(Self::Empty),
            "missing" => Ok(Self::Missing),
            "not_directory" => Ok(Self::NotDirectory),
            "unreadable" => Ok(Self::Unreadable),
            "suspicious_order" => Ok(Self::SuspiciousOrder),
            "shadowed_command" => Ok(Self::ShadowedCommand),
            "unwanted" => Ok(Self::Unwanted),
            _ => Err(ParseIssueKindError),
        }
    }
}

/// Error returned when parsing an unsupported diagnostic issue kind.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ParseIssueKindError;

impl fmt::Display for ParseIssueKindError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "expected one of: {}",
            IssueKind::SUPPORTED_VALUES
        )
    }
}

impl std::error::Error for ParseIssueKindError {}

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
