use crate::model::ShellKind;

/// Planned profile edit action for read-only apply dry-runs.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ApplyAction {
    /// The target profile does not exist and would be created by a future mutating apply.
    CreateProfile,
    /// The target profile exists without a patholog managed block.
    AppendBlock,
    /// The target profile has one complete patholog managed block.
    ReplaceBlock,
}

impl ApplyAction {
    /// Stable action string for human and JSON output.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::CreateProfile => "create_profile",
            Self::AppendBlock => "append_block",
            Self::ReplaceBlock => "replace_block",
        }
    }
}

/// Read-only plan describing what a future apply operation would write.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ApplyPlan {
    /// Shell syntax used by the planned block.
    pub shell: ShellKind,
    /// Target shell profile path.
    pub profile_path: String,
    /// Planned operation.
    pub action: ApplyAction,
    /// Existing managed block when replacing one.
    pub existing_block: Option<String>,
    /// Complete managed block that would be written by a future mutating apply.
    pub planned_block: String,
    /// Cleaned PATH value used to render the planned block.
    pub cleaned_path: String,
    /// Whether this run writes files. Always false in v0.4.
    pub would_write: bool,
}
