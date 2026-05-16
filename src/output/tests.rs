use std::collections::BTreeMap;

use crate::model::{
    ApplyAction, ApplyOutcome, ApplyPlan, Diagnostic, DoctorReport, HealthReport, HealthSeverity,
    IssueKind, PathEntry, PathMutation, PathVariable, RelatedExecutableHint, ResolutionCandidate,
    ResolutionReport, ShellKind, ShellProfile, ShellProfileScanReport, WhyNotReport,
};

use super::human::{
    format_apply_outcome, format_apply_plan, format_conflicts, format_doctor, format_health,
    format_print, format_shell_profile_scan, format_why, format_why_not,
};
use super::json::{
    apply_outcome_to_json, apply_plan_to_json, doctor_to_json, dumps_json, health_to_json,
    resolution_to_json, shell_profile_scan_to_json, why_not_to_json,
};

#[test]
fn format_doctor_groups_diagnostics_in_contract_order() {
    let report = DoctorReport {
        variable: PathVariable::Path,
        entries: vec![entry(1, "/a"), entry(2, "")],
        diagnostics: vec![
            Diagnostic {
                kind: IssueKind::Empty,
                message: "entry 2 is empty".to_owned(),
                entry_index: Some(2),
                entry_value: Some(String::new()),
                related_indexes: Vec::new(),
            },
            Diagnostic {
                kind: IssueKind::Duplicate,
                message: "/a appears 2 times".to_owned(),
                entry_index: Some(1),
                entry_value: Some("/a".to_owned()),
                related_indexes: vec![1, 3],
            },
        ],
    };

    assert_eq!(
        format_doctor(&report),
        "PATH entries: 2\n\nDuplicates:\n  /a (2x; entries 1, 3)\n\nEmpty entries:\n  2  <empty>\n"
    );
}

#[test]
fn format_doctor_reports_no_issues() {
    let report = DoctorReport {
        variable: PathVariable::Path,
        entries: vec![entry(1, "/ok")],
        diagnostics: Vec::new(),
    };

    assert_eq!(
        format_doctor(&report),
        "PATH entries: 1\n\nNo issues found.\n"
    );
}

#[test]
fn format_doctor_renders_ordering_messages() {
    let report = DoctorReport {
        variable: PathVariable::Path,
        entries: Vec::new(),
        diagnostics: vec![Diagnostic {
            kind: IssueKind::SuspiciousOrder,
            message: "/bin appears before /home/me/.cargo/bin".to_owned(),
            entry_index: Some(1),
            entry_value: Some("/bin".to_owned()),
            related_indexes: vec![1, 2],
        }],
    };

    assert_eq!(
        format_doctor(&report),
        "PATH entries: 0\n\nOrdering warnings:\n  /bin appears before /home/me/.cargo/bin\n"
    );
}

#[test]
fn format_doctor_renders_shadowed_command_messages() {
    let report = DoctorReport {
        variable: PathVariable::Path,
        entries: Vec::new(),
        diagnostics: vec![Diagnostic {
            kind: IssueKind::ShadowedCommand,
            message: "tool at /b/tool is shadowed by /a/tool".to_owned(),
            entry_index: Some(2),
            entry_value: Some("/b/tool".to_owned()),
            related_indexes: vec![1, 2],
        }],
    };

    assert_eq!(
        format_doctor(&report),
        "PATH entries: 0\n\nShadowed commands:\n  tool at /b/tool is shadowed by /a/tool\n"
    );
}

#[test]
fn format_doctor_renders_unwanted_entries_and_variable_name() {
    let report = DoctorReport {
        variable: PathVariable::Manpath,
        entries: vec![entry(1, "/sw/share/man")],
        diagnostics: vec![Diagnostic {
            kind: IssueKind::Unwanted,
            message: "/sw/share/man is marked for removal".to_owned(),
            entry_index: Some(1),
            entry_value: Some("/sw/share/man".to_owned()),
            related_indexes: vec![1],
        }],
    };

    assert_eq!(
        format_doctor(&report),
        "MANPATH entries: 1\n\nUnwanted entries:\n  1  /sw/share/man\n"
    );
}

#[test]
fn format_health_reports_clean_status() {
    let report = health_report(
        PathVariable::Path,
        100,
        true,
        1,
        Vec::new(),
        HealthSeverity::None,
    );

    assert_eq!(
        format_health(&report),
        "PATH health: 100/100\nStatus: healthy\nEntries: 1\nIssues: 0\nWorst severity: none\n"
    );
}

#[test]
fn format_health_reports_warning_counts() {
    let report = health_report(
        PathVariable::Path,
        95,
        false,
        2,
        vec![(IssueKind::Duplicate, 1)],
        HealthSeverity::Warning,
    );

    assert_eq!(
        format_health(&report),
        "PATH health: 95/100\nStatus: issues found\nEntries: 2\nIssues: 1\nWorst severity: warning\n\nCounts:\n  duplicate  1\n"
    );
}

#[test]
fn format_health_reports_error_counts_before_warning_counts() {
    let report = health_report(
        PathVariable::Manpath,
        80,
        false,
        3,
        vec![(IssueKind::Duplicate, 1), (IssueKind::Missing, 1)],
        HealthSeverity::Error,
    );

    assert_eq!(
        format_health(&report),
        "MANPATH health: 80/100\nStatus: issues found\nEntries: 3\nIssues: 2\nWorst severity: error\n\nCounts:\n  missing  1\n  duplicate  1\n"
    );
}

#[test]
fn format_print_renders_empty_entries() {
    assert_eq!(format_print(&[entry(1, "")]), "1  <empty>\n");
}

#[test]
fn format_why_renders_single_match_without_other_matches() {
    let report = ResolutionReport {
        command: "tool".to_owned(),
        candidates: vec![candidate(1, "/bin", "/bin/tool", true)],
        searched_directories: vec!["/bin".to_owned()],
        related_hints: Vec::new(),
    };

    assert_eq!(
        format_why(&report),
        "Command: tool\n\nResolved to:\n  /bin/tool\n\nWhy:\n  entry 1 (/bin) appears before all other matching PATH entries.\n\nOther matches:\n  none\n"
    );
}

#[test]
fn format_why_renders_not_found_with_related_hints() {
    let report = ResolutionReport {
        command: "python".to_owned(),
        candidates: Vec::new(),
        searched_directories: vec!["/bin".to_owned()],
        related_hints: vec![RelatedExecutableHint {
            command: "python3".to_owned(),
            paths: vec!["/bin/python3".to_owned()],
        }],
    };

    assert_eq!(
        format_why(&report),
        "Command: python\n\nNot found in PATH.\n\nSearched directories:\n  1  /bin\n\nRelated executables, not PATH matches:\n  python3\n    /bin/python3\n"
    );
}

#[test]
fn format_conflicts_reports_no_matches() {
    let report = ResolutionReport {
        command: "tool".to_owned(),
        candidates: Vec::new(),
        searched_directories: Vec::new(),
        related_hints: Vec::new(),
    };

    assert_eq!(format_conflicts(&report), "No matches for tool\n");
}

#[test]
fn format_why_not_renders_found_command() {
    let report = WhyNotReport {
        command: "tool".to_owned(),
        candidates: vec![candidate(1, "/bin", "/bin/tool", true)],
        searched_directories: vec!["/bin".to_owned()],
        related_hints: Vec::new(),
        path_diagnostics: Vec::new(),
        advice: vec!["The exact command is already available from PATH.".to_owned()],
    };

    assert_eq!(
        format_why_not(&report),
        "Command: tool\n\nAvailable in PATH:\n  /bin/tool\n\nStatus:\n  The exact command is already available.\n"
    );
}

#[test]
fn format_why_not_renders_missing_context() {
    let report = WhyNotReport {
        command: "python".to_owned(),
        candidates: Vec::new(),
        searched_directories: vec!["/missing".to_owned(), "/bin".to_owned()],
        related_hints: vec![RelatedExecutableHint {
            command: "python3".to_owned(),
            paths: vec!["/bin/python3".to_owned()],
        }],
        path_diagnostics: vec![Diagnostic {
            kind: IssueKind::Missing,
            message: "/missing does not exist".to_owned(),
            entry_index: Some(1),
            entry_value: Some("/missing".to_owned()),
            related_indexes: Vec::new(),
        }],
        advice: vec![
            "Check that the command is installed and that its executable directory is present in PATH."
                .to_owned(),
        ],
    };

    assert_eq!(
        format_why_not(&report),
        "Command: python\n\nNot found in PATH.\n\nSearched directories:\n  1  /missing\n  2  /bin\n\nPATH diagnostics:\n  missing  1  /missing\n\nRelated executables, not PATH matches:\n  python3\n    /bin/python3\n\nAdvice:\n  Check that the command is installed and that its executable directory is present in PATH.\n"
    );
}

#[test]
fn dumps_json_uses_sorted_keys_pretty_indentation_and_trailing_newline() {
    let report = DoctorReport {
        variable: PathVariable::Path,
        entries: vec![entry(1, "/a")],
        diagnostics: Vec::new(),
    };

    let output = dumps_json(&doctor_to_json(&report)).expect("render json");

    assert!(output.ends_with('\n'));
    assert!(output.starts_with("{\n  \"diagnostics\": []"));
    assert!(output.contains("\"variable\": \"path\""));
}

#[test]
fn json_output_classifies_entry_kinds_and_missing_winner() {
    let report = DoctorReport {
        variable: PathVariable::Path,
        entries: vec![
            entry(1, ""),
            entry_with_state(2, "/missing", false, false, false),
            entry_with_state(3, "/file", true, false, false),
        ],
        diagnostics: Vec::new(),
    };
    let resolution = ResolutionReport {
        command: "missing".to_owned(),
        candidates: Vec::new(),
        searched_directories: vec!["/bin".to_owned()],
        related_hints: Vec::new(),
    };

    let doctor = dumps_json(&doctor_to_json(&report)).expect("render doctor json");
    let resolution = dumps_json(&resolution_to_json(&resolution)).expect("render resolution json");

    assert!(doctor.contains("\"kind\": \"empty\""));
    assert!(doctor.contains("\"kind\": \"missing\""));
    assert!(doctor.contains("\"kind\": \"not_directory\""));
    assert!(resolution.contains("\"winner\": null"));
}

#[test]
fn why_not_json_uses_resolution_and_diagnostic_shapes() {
    let report = WhyNotReport {
        command: "tool".to_owned(),
        candidates: vec![candidate(1, "/bin", "/bin/tool", true)],
        searched_directories: vec!["/bin".to_owned()],
        related_hints: vec![RelatedExecutableHint {
            command: "toolx".to_owned(),
            paths: vec!["/bin/toolx".to_owned()],
        }],
        path_diagnostics: vec![Diagnostic {
            kind: IssueKind::Empty,
            message: "entry 2 is empty".to_owned(),
            entry_index: Some(2),
            entry_value: Some(String::new()),
            related_indexes: Vec::new(),
        }],
        advice: vec!["The exact command is already available from PATH.".to_owned()],
    };

    let output = dumps_json(&why_not_to_json(&report)).expect("render why-not json");

    assert!(output.contains("\"found\": true"));
    assert!(output.contains("\"winner\": {"));
    assert!(output.contains("\"entry_index\": 1"));
    assert!(output.contains("\"path_diagnostics\": ["));
    assert!(output.contains("\"kind\": \"empty\""));
    assert!(output.contains("\"advice\": ["));
}

#[test]
fn health_json_uses_summary_and_diagnostic_shapes() {
    let mut report = health_report(
        PathVariable::Path,
        85,
        false,
        2,
        vec![(IssueKind::Missing, 1)],
        HealthSeverity::Error,
    );
    report.diagnostics = vec![Diagnostic {
        kind: IssueKind::Missing,
        message: "/missing does not exist".to_owned(),
        entry_index: Some(1),
        entry_value: Some("/missing".to_owned()),
        related_indexes: Vec::new(),
    }];

    let output = dumps_json(&health_to_json(&report)).expect("render health json");
    let json = serde_json::from_str::<serde_json::Value>(&output).expect("parse health json");

    assert_eq!(json["variable"].as_str(), Some("path"));
    assert_eq!(json["score"].as_u64(), Some(85));
    assert_eq!(json["healthy"].as_bool(), Some(false));
    assert_eq!(json["entry_count"].as_u64(), Some(2));
    assert_eq!(json["issue_count"].as_u64(), Some(1));
    assert_eq!(json["worst_severity"].as_str(), Some("error"));
    assert_eq!(json["counts"]["missing"].as_u64(), Some(1));

    let diagnostics = json["diagnostics"].as_array().expect("diagnostics array");
    assert_eq!(diagnostics.len(), 1);
    assert_eq!(diagnostics[0]["kind"].as_str(), Some("missing"));
    assert_eq!(
        diagnostics[0]["message"].as_str(),
        Some("/missing does not exist")
    );
    assert_eq!(diagnostics[0]["entry_index"].as_u64(), Some(1));
    assert_eq!(diagnostics[0]["entry_value"].as_str(), Some("/missing"));
    assert!(
        diagnostics[0]["related_indexes"]
            .as_array()
            .expect("related indexes array")
            .is_empty()
    );
}

#[test]
fn doctor_json_includes_report_variable() {
    let report = DoctorReport {
        variable: PathVariable::Manpath,
        entries: vec![entry(1, "/usr/share/man")],
        diagnostics: Vec::new(),
    };

    let output = dumps_json(&doctor_to_json(&report)).expect("render doctor json");

    assert!(output.contains("\"variable\": \"manpath\""));
}

#[test]
fn doctor_json_includes_unwanted_issue_kind() {
    let report = DoctorReport {
        variable: PathVariable::Path,
        entries: vec![entry(1, "/sw/bin")],
        diagnostics: vec![Diagnostic {
            kind: IssueKind::Unwanted,
            message: "/sw/bin is marked for removal".to_owned(),
            entry_index: Some(1),
            entry_value: Some("/sw/bin".to_owned()),
            related_indexes: vec![1],
        }],
    };

    let output = dumps_json(&doctor_to_json(&report)).expect("render doctor json");

    assert!(output.contains("\"kind\": \"unwanted\""));
}

#[test]
fn format_shell_profile_scan_renders_path_mutations() {
    let report = ShellProfileScanReport {
        home: "/home/me".to_owned(),
        profiles: vec![ShellProfile {
            shell: "zsh",
            path: "/home/me/.zshrc".to_owned(),
            exists: true,
            is_file: true,
            readable: true,
            path_mutations: vec![PathMutation {
                line: 3,
                kind: "path_assignment",
                text: "export PATH=\"$HOME/bin:$PATH\"".to_owned(),
            }],
        }],
    };

    assert_eq!(
        format_shell_profile_scan(&report),
        "Shell profile scan: /home/me\n\nPATH changes:\n  /home/me/.zshrc (zsh)\n    line 3  path_assignment  export PATH=\"$HOME/bin:$PATH\"\n"
    );
}

#[test]
fn format_shell_profile_scan_does_not_report_non_files_as_unreadable() {
    let report = ShellProfileScanReport {
        home: "/home/me".to_owned(),
        profiles: vec![ShellProfile {
            shell: "zsh",
            path: "/home/me/.zshrc".to_owned(),
            exists: true,
            is_file: false,
            readable: false,
            path_mutations: Vec::new(),
        }],
    };

    assert_eq!(
        format_shell_profile_scan(&report),
        "Shell profile scan: /home/me\n\nNo PATH changes found in readable shell profiles.\n"
    );
}

#[test]
fn shell_profile_scan_json_includes_profile_state() {
    let report = ShellProfileScanReport {
        home: "/home/me".to_owned(),
        profiles: vec![ShellProfile {
            shell: "bash",
            path: "/home/me/.bashrc".to_owned(),
            exists: true,
            is_file: true,
            readable: false,
            path_mutations: Vec::new(),
        }],
    };

    let output = dumps_json(&shell_profile_scan_to_json(&report)).expect("render shell scan json");

    assert!(output.contains("\"path\": \"/home/me/.bashrc\""));
    assert!(output.contains("\"readable\": false"));
}

#[test]
fn format_apply_plan_renders_dry_run_details() {
    let plan = apply_plan();

    assert_eq!(
        format_apply_plan(&plan),
        "Apply dry-run: zsh\n\nTarget profile:\n  /home/me/.zshrc\nAction:\n  append_block\nWould write:\n  false\n\nCleaned PATH:\n  /a:/b\n\nPlanned block:\n# >>> patholog PATH >>>\nexport PATH='/a:/b'\n# <<< patholog PATH <<<\n"
    );
}

#[test]
fn apply_plan_json_uses_stable_fields() {
    let output = dumps_json(&apply_plan_to_json(&apply_plan())).expect("render apply json");

    assert!(output.contains("\"action\": \"append_block\""));
    assert!(output.contains("\"existing_block\": null"));
    assert!(output.contains("\"would_write\": false"));
}

#[test]
fn format_apply_outcome_renders_write_details() {
    let outcome = apply_outcome();

    assert_eq!(
        format_apply_outcome(&outcome),
        "Apply: zsh\n\nTarget profile:\n  /home/me/.zshrc\nAction:\n  append_block\nWrote:\n  true\nBackup:\n  /home/me/.zshrc.patholog-backup.123\n\nCleaned PATH:\n  /a:/b\n\nWritten block:\n# >>> patholog PATH >>>\nexport PATH='/a:/b'\n# <<< patholog PATH <<<\n"
    );
}

#[test]
fn apply_outcome_json_adds_write_fields() {
    let output = dumps_json(&apply_outcome_to_json(&apply_outcome())).expect("render apply json");

    assert!(output.contains("\"would_write\": true"));
    assert!(output.contains("\"wrote\": true"));
    assert!(output.contains("\"backup_created\": true"));
    assert!(output.contains("\"backup_path\": \"/home/me/.zshrc.patholog-backup.123\""));
}

fn entry(index: usize, raw: &str) -> PathEntry {
    entry_with_state(index, raw, !raw.is_empty(), !raw.is_empty(), raw.is_empty())
}

fn entry_with_state(
    index: usize,
    raw: &str,
    exists: bool,
    is_dir: bool,
    is_empty: bool,
) -> PathEntry {
    PathEntry {
        index,
        raw: raw.to_owned(),
        display: raw.to_owned(),
        comparison_key: raw.to_owned(),
        exists,
        is_dir,
        is_empty,
    }
}

fn candidate(entry_index: usize, directory: &str, path: &str, wins: bool) -> ResolutionCandidate {
    ResolutionCandidate {
        entry_index,
        directory: directory.to_owned(),
        path: path.to_owned(),
        wins,
    }
}

fn health_report(
    variable: PathVariable,
    score: u8,
    healthy: bool,
    entry_count: usize,
    counts: Vec<(IssueKind, usize)>,
    worst_severity: HealthSeverity,
) -> HealthReport {
    let counts = counts.into_iter().collect::<BTreeMap<_, _>>();
    HealthReport {
        variable,
        score,
        healthy,
        entry_count,
        issue_count: counts.values().sum(),
        worst_severity,
        counts,
        diagnostics: Vec::new(),
    }
}

fn apply_plan() -> ApplyPlan {
    ApplyPlan {
        shell: ShellKind::Zsh,
        profile_path: "/home/me/.zshrc".to_owned(),
        action: ApplyAction::AppendBlock,
        existing_block: None,
        planned_block: "# >>> patholog PATH >>>\nexport PATH='/a:/b'\n# <<< patholog PATH <<<"
            .to_owned(),
        cleaned_path: "/a:/b".to_owned(),
        would_write: false,
    }
}

fn apply_outcome() -> ApplyOutcome {
    let mut plan = apply_plan();
    plan.would_write = true;
    ApplyOutcome {
        plan,
        wrote: true,
        backup_path: Some("/home/me/.zshrc.patholog-backup.123".to_owned()),
        backup_created: true,
    }
}
