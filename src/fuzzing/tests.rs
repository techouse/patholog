use crate::model::{IssueKind, PlatformMode};

use super::{clean_path, diagnose_path, doctor_json, parse_path, print_json};

#[test]
fn fuzzing_wrappers_delegate_to_read_only_behaviors() {
    let entries = parse_path(".:.", PlatformMode::Posix, None);
    let report = diagnose_path(".:.", PlatformMode::Posix, None);

    assert_eq!(clean_path(".:.", PlatformMode::Posix, None), ".");
    assert_eq!(entries.len(), 2);
    assert!(
        report
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.kind == IssueKind::Duplicate)
    );
    assert!(
        print_json(&entries)
            .expect("render entries json")
            .ends_with('\n')
    );
    assert!(
        doctor_json(&report)
            .expect("render doctor json")
            .ends_with('\n')
    );
}
