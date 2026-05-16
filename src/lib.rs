#![forbid(unsafe_code)]

//! PATH diagnostics and command resolution library used by the `patholog` CLI.
//!
//! The command-line interface is the primary product surface. The public Rust API
//! is intentionally small and is mainly useful for tests, fuzzing, and embedding
//! the CLI with an injected runtime context.
//!
//! # Example
//!
//! ```
//! use std::path::PathBuf;
//!
//! use patholog::cli::{CommandContext, run};
//! use patholog::model::ExitCode;
//!
//! let context = CommandContext {
//!     path_value: ".".to_owned(),
//!     manpath_value: String::new(),
//!     pathext: None,
//!     cwd: PathBuf::from("."),
//!     home_dir: None,
//!     user_profile_dir: None,
//! };
//!
//! let result = run(["print", "--platform", "posix"], context);
//!
//! assert_eq!(result.exit_code, ExitCode::Success);
//! assert_eq!(result.stdout, "1  .\n");
//! ```

/// Command-line parsing and execution entry points.
pub mod cli;
/// Public data model types returned by analysis and resolution operations.
pub mod model;

pub(crate) mod apply;
pub(crate) mod clean;
pub(crate) mod config;
pub(crate) mod doctor;
pub(crate) mod health;
pub(crate) mod normalize;
pub(crate) mod output;
pub(crate) mod path_env;
pub(crate) mod platform;
pub(crate) mod policy;
pub(crate) mod profile_scan;
pub(crate) mod resolve;
pub(crate) mod why_not;

#[cfg(feature = "fuzzing")]
/// Fuzzing-only entry points for internal read-only behavior.
pub mod fuzzing {
    use crate::clean;
    use crate::doctor;
    use crate::model::{DoctorReport, PathEntry, PlatformMode, ShellKind};
    use crate::output::json;
    use crate::path_env;

    /// Runs PATH cleaning with injected platform rules.
    #[must_use]
    pub fn clean_path(
        path_value: &str,
        platform_mode: PlatformMode,
        pathext: Option<&str>,
    ) -> String {
        clean::clean_path(path_value, platform_mode, pathext)
    }

    /// Runs PATH export formatting with injected platform and shell rules.
    #[must_use]
    pub fn clean_export(
        path_value: &str,
        platform_mode: PlatformMode,
        pathext: Option<&str>,
        shell: ShellKind,
    ) -> String {
        clean::clean_export(path_value, platform_mode, pathext, shell)
    }

    /// Parses a PATH value with injected platform rules.
    #[must_use]
    pub fn parse_path(
        path_value: &str,
        platform_mode: PlatformMode,
        pathext: Option<&str>,
    ) -> Vec<PathEntry> {
        path_env::parse_path(path_value, platform_mode, pathext)
    }

    /// Runs doctor diagnostics with injected platform rules.
    #[must_use]
    pub fn diagnose_path(
        path_value: &str,
        platform_mode: PlatformMode,
        pathext: Option<&str>,
    ) -> DoctorReport {
        doctor::diagnose_path(path_value, platform_mode, pathext)
    }

    /// Renders parsed PATH entries as deterministic JSON.
    ///
    /// # Errors
    ///
    /// Returns any JSON serialization error from `serde_json`.
    pub fn print_json(entries: &[PathEntry]) -> Result<String, serde_json::Error> {
        json::dumps_json(&json::entries_to_json(entries))
    }

    /// Renders doctor diagnostics as deterministic JSON.
    ///
    /// # Errors
    ///
    /// Returns any JSON serialization error from `serde_json`.
    pub fn doctor_json(report: &DoctorReport) -> Result<String, serde_json::Error> {
        json::dumps_json(&json::doctor_to_json(report))
    }

    #[cfg(test)]
    #[path = "tests.rs"]
    mod tests;
}
