use super::ExitCode;

#[test]
fn exit_codes_match_contract() {
    assert_eq!(ExitCode::Success.code(), 0);
    assert_eq!(ExitCode::GeneralError.code(), 1);
    assert_eq!(ExitCode::DiagnosticsFound.code(), 2);
    assert_eq!(ExitCode::CommandNotFound.code(), 3);
}
