mod apply;
mod diagnostic;
mod doctor;
mod exit_code;
mod path;
mod path_variable;
mod platform;
mod preset;
mod resolution;
mod scan;
mod shell;

pub use apply::{ApplyAction, ApplyPlan};
pub use diagnostic::{Diagnostic, IssueKind};
pub use doctor::DoctorReport;
pub use exit_code::ExitCode;
pub use path::PathEntry;
pub use path_variable::PathVariable;
pub use platform::{PlatformMode, PlatformRules};
pub use preset::PresetKind;
pub use resolution::{RelatedExecutableHint, ResolutionCandidate, ResolutionReport};
pub use scan::{PathMutation, ShellProfile, ShellProfileScanReport};
pub use shell::ShellKind;

#[cfg(test)]
#[path = "model/tests.rs"]
mod tests;
