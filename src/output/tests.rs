use crate::model::{
    Diagnostic, DoctorReport, IssueKind, PathEntry, RelatedExecutableHint, ResolutionCandidate,
    ResolutionReport,
};

use super::human::{format_conflicts, format_doctor, format_print, format_why};
use super::json::{doctor_to_json, dumps_json, resolution_to_json};

#[test]
fn format_doctor_groups_diagnostics_in_contract_order() {
    let report = DoctorReport {
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
fn dumps_json_uses_sorted_keys_pretty_indentation_and_trailing_newline() {
    let report = DoctorReport {
        entries: vec![entry(1, "/a")],
        diagnostics: Vec::new(),
    };

    let output = dumps_json(&doctor_to_json(&report)).expect("render json");

    assert!(output.ends_with('\n'));
    assert!(output.starts_with("{\n  \"diagnostics\": []"));
}

#[test]
fn json_output_classifies_entry_kinds_and_missing_winner() {
    let report = DoctorReport {
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
