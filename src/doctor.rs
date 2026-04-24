mod duplicates;
mod ordering;
mod shadowed;
mod state;

use std::path::Path;

use crate::model::{DoctorReport, PlatformMode};
use crate::path_env::parse_path;
use crate::resolve::resolve_command;

pub(crate) fn diagnose_path(
    path_value: &str,
    platform_mode: PlatformMode,
    pathext: Option<&str>,
) -> DoctorReport {
    let entries = parse_path(path_value, platform_mode, pathext);
    let mut diagnostics = Vec::new();
    diagnostics.extend(duplicates::diagnostics(&entries));
    diagnostics.extend(state::diagnostics(&entries));
    diagnostics.extend(ordering::diagnostics(&entries));
    DoctorReport {
        entries,
        diagnostics,
    }
}

pub(crate) fn diagnose_command_path(
    path_value: &str,
    command: &str,
    platform_mode: PlatformMode,
    pathext: Option<&str>,
    cwd: &Path,
) -> DoctorReport {
    let mut report = diagnose_path(path_value, platform_mode, pathext);
    let resolution = resolve_command(path_value, command, platform_mode, pathext, cwd, false);
    report
        .diagnostics
        .extend(shadowed::diagnostics(command, &resolution.candidates));
    report
}

#[cfg(test)]
#[path = "doctor/tests.rs"]
mod tests;
