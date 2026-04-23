mod duplicates;
mod ordering;
mod state;

use crate::model::{DoctorReport, PlatformMode};
use crate::path_env::parse_path;

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

#[cfg(test)]
#[path = "doctor/tests.rs"]
mod tests;
