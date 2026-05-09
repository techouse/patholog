use clap::ValueEnum;

/// Built-in path policy presets.
#[derive(Clone, Copy, Debug, Eq, PartialEq, ValueEnum)]
pub enum PresetKind {
    /// Homebrew path ordering checks.
    Homebrew,
    /// Cargo user tool path checks.
    Cargo,
    /// pyenv shim path checks.
    Pyenv,
    /// Fink path removal rules.
    Fink,
}

impl PresetKind {
    /// Stable CLI string.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Homebrew => "homebrew",
            Self::Cargo => "cargo",
            Self::Pyenv => "pyenv",
            Self::Fink => "fink",
        }
    }
}
