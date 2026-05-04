use clap::ValueEnum;

/// Shell syntax variants used for generated stdout snippets.
#[derive(Clone, Copy, Debug, Eq, PartialEq, ValueEnum)]
pub enum ShellKind {
    /// Bourne Again Shell syntax.
    Bash,
    /// fish shell syntax.
    Fish,
    /// PowerShell syntax.
    Pwsh,
    /// Z shell syntax.
    Zsh,
}

impl ShellKind {
    /// Stable shell string for human and JSON output.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Bash => "bash",
            Self::Fish => "fish",
            Self::Pwsh => "pwsh",
            Self::Zsh => "zsh",
        }
    }
}
