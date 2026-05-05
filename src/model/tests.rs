use super::{ExitCode, IssueKind};

#[test]
fn exit_codes_match_contract() {
    assert_eq!(ExitCode::Success.code(), 0);
    assert_eq!(ExitCode::GeneralError.code(), 1);
    assert_eq!(ExitCode::DiagnosticsFound.code(), 2);
    assert_eq!(ExitCode::CommandNotFound.code(), 3);
}

#[test]
fn issue_kinds_match_cli_and_json_contract() {
    assert_eq!(IssueKind::Duplicate.as_str(), "duplicate");
    assert_eq!(IssueKind::Empty.as_str(), "empty");
    assert_eq!(IssueKind::Missing.as_str(), "missing");
    assert_eq!(IssueKind::NotDirectory.as_str(), "not_directory");
    assert_eq!(IssueKind::Unreadable.as_str(), "unreadable");
    assert_eq!(IssueKind::SuspiciousOrder.as_str(), "suspicious_order");
    assert_eq!(IssueKind::ShadowedCommand.as_str(), "shadowed_command");
    assert_eq!(IssueKind::Unwanted.as_str(), "unwanted");
}
