/// Process exit codes defined by the patholog parity contract.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ExitCode {
    /// Command completed successfully.
    Success = 0,
    /// Usage, runtime, or output failure.
    GeneralError = 1,
    /// `doctor --fail-on` matched selected diagnostics.
    DiagnosticsFound = 2,
    /// `why` or `conflicts` found no executable candidates.
    CommandNotFound = 3,
}

impl ExitCode {
    /// Numeric process status code.
    #[must_use]
    pub const fn code(self) -> i32 {
        self as i32
    }
}
