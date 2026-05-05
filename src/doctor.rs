mod duplicates;
mod ordering;
mod shadowed;
mod state;
mod unwanted;

use std::path::Path;

use crate::model::{DoctorReport, PathVariable, PlatformMode};
use crate::path_env::parse_path;
use crate::policy::PathPolicy;
use crate::resolve::resolve_command;

#[cfg(any(test, feature = "fuzzing"))]
pub(crate) fn diagnose_path(
    path_value: &str,
    platform_mode: PlatformMode,
    pathext: Option<&str>,
) -> DoctorReport {
    diagnose_path_with_policy(
        path_value,
        platform_mode,
        pathext,
        PathVariable::Path,
        &PathPolicy::default(),
    )
}

pub(crate) fn diagnose_path_with_policy(
    path_value: &str,
    platform_mode: PlatformMode,
    pathext: Option<&str>,
    variable: PathVariable,
    policy: &PathPolicy,
) -> DoctorReport {
    let entries = parse_path(path_value, platform_mode, pathext);
    let mut diagnostics = Vec::new();
    diagnostics.extend(duplicates::diagnostics(&entries));
    diagnostics.extend(state::diagnostics(&entries));
    diagnostics.extend(unwanted::diagnostics(
        &entries,
        policy,
        platform_mode,
        pathext,
    ));
    if variable == PathVariable::Path {
        diagnostics.extend(ordering::diagnostics(&entries));
    }
    DoctorReport {
        variable,
        entries,
        diagnostics,
    }
}

pub(crate) fn diagnose_command_path_with_policy(
    path_value: &str,
    command: &str,
    platform_mode: PlatformMode,
    pathext: Option<&str>,
    cwd: &Path,
    policy: &PathPolicy,
) -> DoctorReport {
    let mut report = diagnose_path_with_policy(
        path_value,
        platform_mode,
        pathext,
        PathVariable::Path,
        policy,
    );
    let resolution = resolve_command(path_value, command, platform_mode, pathext, cwd, false);
    report
        .diagnostics
        .extend(shadowed::diagnostics(command, &resolution.candidates));
    report
}

#[cfg(test)]
#[path = "doctor/tests.rs"]
mod tests;
