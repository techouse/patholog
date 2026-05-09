use clap::ValueEnum;

/// Supported path-like environment variables.
#[derive(Clone, Copy, Debug, Eq, PartialEq, ValueEnum)]
pub enum PathVariable {
    /// Executable search path.
    Path,
    /// Manual page search path.
    Manpath,
}

impl PathVariable {
    /// Stable CLI and JSON string.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Path => "path",
            Self::Manpath => "manpath",
        }
    }

    /// Environment variable name for POSIX shell output and process lookup.
    #[must_use]
    pub const fn env_name(self) -> &'static str {
        match self {
            Self::Path => "PATH",
            Self::Manpath => "MANPATH",
        }
    }

    /// PowerShell environment variable name.
    #[must_use]
    pub const fn powershell_env_name(self) -> &'static str {
        match self {
            Self::Path => "Path",
            Self::Manpath => "MANPATH",
        }
    }
}
