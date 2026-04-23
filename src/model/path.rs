/// A parsed PATH entry.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PathEntry {
    /// One-based entry index.
    pub index: usize,
    /// Original PATH entry text.
    pub raw: String,
    /// Display text; v0.1 preserves the raw value.
    pub display: String,
    /// Conservative comparison key for duplicate detection.
    pub comparison_key: String,
    /// Whether the entry exists on the native filesystem.
    pub exists: bool,
    /// Whether the entry exists and is a directory.
    pub is_dir: bool,
    /// Whether this is an empty PATH entry.
    pub is_empty: bool,
}
