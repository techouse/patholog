use clap::ValueEnum;
use std::fmt;
use std::str::FromStr;

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
    /// Stable accepted strings used in CLI flags and config files.
    pub const SUPPORTED_VALUES: &'static str = "homebrew, cargo, pyenv, fink";

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

impl FromStr for PresetKind {
    type Err = ParsePresetKindError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "homebrew" => Ok(Self::Homebrew),
            "cargo" => Ok(Self::Cargo),
            "pyenv" => Ok(Self::Pyenv),
            "fink" => Ok(Self::Fink),
            _ => Err(ParsePresetKindError),
        }
    }
}

/// Error returned when parsing an unsupported path policy preset.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ParsePresetKindError;

impl fmt::Display for ParsePresetKindError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "expected one of: {}",
            PresetKind::SUPPORTED_VALUES
        )
    }
}

impl std::error::Error for ParsePresetKindError {}
