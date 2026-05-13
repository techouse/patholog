use std::path::PathBuf;

use clap::{Args, Parser, Subcommand};

use crate::model::{ExitCode, PathVariable, PlatformMode, PresetKind, ShellKind};

/// Runtime inputs injected into the CLI.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CommandContext {
    /// PATH value to parse.
    pub path_value: String,
    /// MANPATH value to parse.
    pub manpath_value: String,
    /// PATHEXT value used for Windows lookup modeling.
    pub pathext: Option<String>,
    /// Current working directory used for empty PATH entries.
    pub cwd: PathBuf,
    /// HOME directory used by read-only profile scanning.
    pub home_dir: Option<PathBuf>,
    /// USERPROFILE directory used by Windows profile scanning.
    pub user_profile_dir: Option<PathBuf>,
}

impl CommandContext {
    /// Builds a command context from the process environment.
    #[must_use]
    pub fn from_env() -> Self {
        Self {
            path_value: std::env::var("PATH").unwrap_or_default(),
            manpath_value: std::env::var("MANPATH").unwrap_or_default(),
            pathext: std::env::var("PATHEXT").ok(),
            cwd: std::env::current_dir().unwrap_or_else(|_error| PathBuf::from(".")),
            home_dir: env_path("HOME"),
            user_profile_dir: env_path("USERPROFILE"),
        }
    }
}

fn env_path(name: &str) -> Option<PathBuf> {
    std::env::var_os(name)
        .filter(|value| !value.as_os_str().is_empty())
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
    Print(PrintOptions),
    Doctor(DoctorOptions),
    Why(ResolutionOptions),
    Conflicts(ResolutionOptions),
    Clean(CleanOptions),
    Apply(ApplyOptions),
    Scan(ScanOptions),
    Config(ConfigOptions),
    Completions(CompletionOptions),
}

#[derive(Args)]
pub(super) struct CommonOptions {
    #[arg(long, value_enum, default_value_t = PlatformMode::Auto)]
    pub(super) platform: PlatformMode,
    #[arg(long)]
    pub(super) json: bool,
}

#[derive(Args)]
pub(super) struct PrintOptions {
    #[command(flatten)]
    pub(super) common: CommonOptions,
    #[arg(long = "var", value_enum, default_value_t = PathVariable::Path)]
    pub(super) variable: PathVariable,
}

#[derive(Args)]
pub(super) struct DoctorOptions {
    #[command(flatten)]
    pub(super) common: CommonOptions,
    #[arg(long = "var", value_enum, default_value_t = PathVariable::Path)]
    pub(super) variable: PathVariable,
    #[arg(long, value_name = "KINDS")]
    pub(super) fail_on: Option<String>,
    #[arg(long, value_name = "COMMAND")]
    pub(super) command: Option<String>,
    #[arg(long = "drop", value_name = "ENTRY")]
    pub(super) drop_entries: Vec<String>,
    #[arg(long = "preset", value_enum)]
    pub(super) presets: Vec<PresetKind>,
    #[arg(long, value_name = "FILE")]
    pub(super) config: Option<PathBuf>,
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
    #[arg(long = "var", value_enum, default_value_t = PathVariable::Path)]
    pub(super) variable: PathVariable,
    #[arg(long)]
    pub(super) stdout: bool,
    #[arg(long)]
    pub(super) export: bool,
    #[arg(long, value_enum)]
    pub(super) shell: Option<ShellKind>,
    #[arg(long = "drop", value_name = "ENTRY")]
    pub(super) drop_entries: Vec<String>,
    #[arg(long = "preset", value_enum)]
    pub(super) presets: Vec<PresetKind>,
    #[arg(long, value_name = "FILE")]
    pub(super) config: Option<PathBuf>,
}

#[derive(Args)]
pub(super) struct ScanOptions {
    #[command(flatten)]
    pub(super) common: CommonOptions,
    #[arg(long, value_name = "DIR")]
    pub(super) home: Option<PathBuf>,
}

#[derive(Args)]
pub(super) struct ApplyOptions {
    #[command(flatten)]
    pub(super) common: CommonOptions,
    #[arg(long)]
    pub(super) dry_run: bool,
    #[arg(long)]
    pub(super) yes: bool,
    #[arg(long)]
    pub(super) no_backup: bool,
    #[arg(long, value_enum)]
    pub(super) shell: Option<ShellKind>,
    #[arg(long, value_name = "DIR")]
    pub(super) home: Option<PathBuf>,
    #[arg(long, value_name = "FILE")]
    pub(super) profile: Option<PathBuf>,
    #[arg(long = "drop", value_name = "ENTRY")]
    pub(super) drop_entries: Vec<String>,
    #[arg(long = "preset", value_enum)]
    pub(super) presets: Vec<PresetKind>,
    #[arg(long, value_name = "FILE")]
    pub(super) config: Option<PathBuf>,
}

#[derive(Args)]
pub(super) struct ConfigOptions {
    #[command(subcommand)]
    pub(super) command: ConfigCommand,
}

#[derive(Subcommand)]
pub(super) enum ConfigCommand {
    Check(ConfigCheckOptions),
    Print(ConfigPrintOptions),
}

#[derive(Args)]
pub(super) struct ConfigCheckOptions {
    #[arg(long, value_name = "FILE")]
    pub(super) config: PathBuf,
}

#[derive(Args)]
pub(super) struct ConfigPrintOptions {
    #[arg(long, value_name = "FILE")]
    pub(super) config: PathBuf,
    #[arg(long)]
    pub(super) json: bool,
}

#[derive(Args)]
pub(super) struct CompletionOptions {
    #[arg(value_enum)]
    pub(super) shell: ShellKind,
}
