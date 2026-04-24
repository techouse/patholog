/// A PATH-affecting line found in a shell profile.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PathMutation {
    /// One-based line number.
    pub line: usize,
    /// Stable mutation classification.
    pub kind: &'static str,
    /// Trimmed source line, capped for display.
    pub text: String,
}

/// Shell startup profile considered by the read-only scanner.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ShellProfile {
    /// Shell or profile family.
    pub shell: &'static str,
    /// Profile path display value.
    pub path: String,
    /// Whether the profile exists.
    pub exists: bool,
    /// Whether the existing profile is a regular file.
    pub is_file: bool,
    /// Whether the profile content could be read.
    pub readable: bool,
    /// PATH-affecting lines found in the profile.
    pub path_mutations: Vec<PathMutation>,
}

/// Read-only shell startup profile scan report.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ShellProfileScanReport {
    /// Home directory used to build known profile paths.
    pub home: String,
    /// Known profiles considered by the scanner.
    pub profiles: Vec<ShellProfile>,
}
