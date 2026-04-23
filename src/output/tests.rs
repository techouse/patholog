use crate::model::{Diagnostic, DoctorReport, IssueKind, PathEntry};

use super::human::format_doctor;
use super::json::{doctor_to_json, dumps_json};

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
fn dumps_json_uses_sorted_keys_pretty_indentation_and_trailing_newline() {
    let report = DoctorReport {
        entries: vec![entry(1, "/a")],
        diagnostics: Vec::new(),
    };

    let output = dumps_json(&doctor_to_json(&report)).expect("render json");

    assert!(output.ends_with('\n'));
    assert!(output.starts_with("{\n  \"diagnostics\": []"));
}

fn entry(index: usize, raw: &str) -> PathEntry {
    PathEntry {
        index,
        raw: raw.to_owned(),
        display: raw.to_owned(),
        comparison_key: raw.to_owned(),
        exists: !raw.is_empty(),
        is_dir: !raw.is_empty(),
        is_empty: raw.is_empty(),
    }
}
