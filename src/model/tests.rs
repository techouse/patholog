use super::{ExitCode, HealthSeverity, IssueKind, PresetKind};

#[test]
fn exit_codes_match_contract() {
    assert_eq!(ExitCode::Success.code(), 0);
    assert_eq!(ExitCode::GeneralError.code(), 1);
    assert_eq!(ExitCode::DiagnosticsFound.code(), 2);
    assert_eq!(ExitCode::CommandNotFound.code(), 3);
}

#[test]
fn issue_kinds_match_cli_and_json_contract() {
    assert_eq!(
        IssueKind::SUPPORTED_VALUES,
        "duplicate, empty, missing, not_directory, unreadable, suspicious_order, shadowed_command, unwanted"
    );
    assert_eq!(IssueKind::Duplicate.as_str(), "duplicate");
    assert_eq!(IssueKind::Empty.as_str(), "empty");
    assert_eq!(IssueKind::Missing.as_str(), "missing");
    assert_eq!(IssueKind::NotDirectory.as_str(), "not_directory");
    assert_eq!(IssueKind::Unreadable.as_str(), "unreadable");
    assert_eq!(IssueKind::SuspiciousOrder.as_str(), "suspicious_order");
    assert_eq!(IssueKind::ShadowedCommand.as_str(), "shadowed_command");
    assert_eq!(IssueKind::Unwanted.as_str(), "unwanted");
    assert_eq!("duplicate".parse::<IssueKind>(), Ok(IssueKind::Duplicate));
    assert!("unknown".parse::<IssueKind>().is_err());
}

#[test]
fn health_severities_match_json_contract() {
    assert_eq!(HealthSeverity::None.as_str(), "none");
    assert_eq!(HealthSeverity::Warning.as_str(), "warning");
    assert_eq!(HealthSeverity::Error.as_str(), "error");
}

#[test]
fn preset_kinds_match_cli_and_config_contract() {
    assert_eq!(PresetKind::SUPPORTED_VALUES, "homebrew, cargo, pyenv, fink");
    assert_eq!(PresetKind::Homebrew.as_str(), "homebrew");
    assert_eq!(PresetKind::Cargo.as_str(), "cargo");
    assert_eq!(PresetKind::Pyenv.as_str(), "pyenv");
    assert_eq!(PresetKind::Fink.as_str(), "fink");
    assert_eq!("cargo".parse::<PresetKind>(), Ok(PresetKind::Cargo));
    assert!("unknown".parse::<PresetKind>().is_err());
}
