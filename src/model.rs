mod diagnostic;
mod doctor;
mod exit_code;
mod path;
mod platform;
mod resolution;
mod scan;

pub use diagnostic::{Diagnostic, IssueKind};
pub use doctor::DoctorReport;
pub use exit_code::ExitCode;
pub use path::PathEntry;
pub use platform::{PlatformMode, PlatformRules};
pub use resolution::{RelatedExecutableHint, ResolutionCandidate, ResolutionReport};
pub use scan::{PathMutation, ShellProfile, ShellProfileScanReport};

#[cfg(test)]
#[path = "model/tests.rs"]
mod tests;
