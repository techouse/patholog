use std::path::PathBuf;

use clap::{Args, Parser, Subcommand};

use crate::model::{ExitCode, PlatformMode};

/// Runtime inputs injected into the CLI.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CommandContext {
    /// PATH value to parse.
    pub path_value: String,
    /// PATHEXT value used for Windows lookup modeling.
    pub pathext: Option<String>,
    /// Current working directory used for empty PATH entries.
    pub cwd: PathBuf,
    /// Home directory used by read-only profile scanning.
    pub home_dir: Option<PathBuf>,
}

impl CommandContext {
    /// Builds a command context from the process environment.
    #[must_use]
    pub fn from_env() -> Self {
        Self {
            path_value: std::env::var("PATH").unwrap_or_default(),
            pathext: std::env::var("PATHEXT").ok(),
            cwd: std::env::current_dir().unwrap_or_else(|_error| PathBuf::from(".")),
            home_dir: home_dir_from_env(),
        }
    }
}

fn home_dir_from_env() -> Option<PathBuf> {
    std::env::var_os("HOME")
        .filter(|value| !value.as_os_str().is_empty())
        .or_else(|| std::env::var_os("USERPROFILE").filter(|value| !value.as_os_str().is_empty()))
        .map(PathBuf::from)
}

/// Result produced by the library CLI entry point.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CliResult {
    /// Process exit status.
    pub exit_code: ExitCode,
    /// UTF-8 stdout.
    pub stdout: String,
    /// UTF-8 stderr.
    pub stderr: String,
}

impl CliResult {
    pub(super) fn success(stdout: String) -> Self {
        Self {
            exit_code: ExitCode::Success,
            stdout,
            stderr: String::new(),
        }
    }

    pub(super) fn error(message: impl Into<String>) -> Self {
        Self {
            exit_code: ExitCode::GeneralError,
            stdout: String::new(),
            stderr: format!("patholog: {}\n", message.into()),
        }
    }

    pub(super) fn clap_error(stderr: String) -> Self {
        Self {
            exit_code: ExitCode::GeneralError,
            stdout: String::new(),
            stderr,
        }
    }
}

#[derive(Parser)]
#[command(name = "patholog", version, disable_help_subcommand = true)]
pub(super) struct Cli {
    #[command(subcommand)]
    pub(super) command: Command,
}

#[derive(Subcommand)]
pub(super) enum Command {
    Print(CommonOptions),
    Doctor(DoctorOptions),
    Why(ResolutionOptions),
    Conflicts(ResolutionOptions),
    Clean(CleanOptions),
    Scan(ScanOptions),
}

#[derive(Args)]
pub(super) struct CommonOptions {
    #[arg(long, value_enum, default_value_t = PlatformMode::Auto)]
    pub(super) platform: PlatformMode,
    #[arg(long)]
    pub(super) json: bool,
}

#[derive(Args)]
pub(super) struct DoctorOptions {
    #[command(flatten)]
    pub(super) common: CommonOptions,
    #[arg(long, default_value = "", value_name = "KINDS")]
    pub(super) fail_on: String,
    #[arg(long, value_name = "COMMAND")]
    pub(super) command: Option<String>,
}

#[derive(Args)]
pub(super) struct ResolutionOptions {
    pub(super) command: String,
    #[command(flatten)]
    pub(super) common: CommonOptions,
}

#[derive(Args)]
pub(super) struct CleanOptions {
    #[arg(long, value_enum, default_value_t = PlatformMode::Auto)]
    pub(super) platform: PlatformMode,
    #[arg(long)]
    pub(super) stdout: bool,
}

#[derive(Args)]
pub(super) struct ScanOptions {
    #[command(flatten)]
    pub(super) common: CommonOptions,
    #[arg(long, value_name = "DIR")]
    pub(super) home: Option<PathBuf>,
}
