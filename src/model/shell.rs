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
