use clap::ValueEnum;

/// Supported PATH parsing and command-resolution modes.
#[derive(Clone, Copy, Debug, Eq, PartialEq, ValueEnum)]
pub enum PlatformMode {
    /// Resolve to the host platform rules at runtime.
    Auto,
    /// POSIX `PATH` semantics.
    Posix,
    /// Windows `PATH` and `PATHEXT` semantics.
    Windows,
}

/// Concrete platform rules after resolving [`PlatformMode::Auto`].
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PlatformRules {
    /// Resolved platform mode.
    pub mode: PlatformMode,
    /// PATH separator for this platform.
    pub separator: char,
    /// Whether comparison keys are case-sensitive.
    pub case_sensitive: bool,
    /// Windows executable suffixes, empty for POSIX mode.
    pub pathext: Vec<String>,
}
