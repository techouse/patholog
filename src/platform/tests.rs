use crate::model::PlatformMode;

use super::{parse_pathext, resolve_platform_rules};

#[test]
fn parse_pathext_uses_default_when_missing() {
    assert_eq!(parse_pathext(None), [".EXE", ".CMD", ".BAT"]);
}

#[test]
fn parse_pathext_normalizes_extensions_and_removes_duplicates() {
    assert_eq!(parse_pathext(Some("EXE;.cmd;.EXE;;")), [".EXE", ".CMD"]);
}

#[test]
fn windows_rules_use_semicolon_and_case_insensitive_keys() {
    let rules = resolve_platform_rules(PlatformMode::Windows, Some(".EXE"));

    assert_eq!(rules.separator, ';');
    assert!(!rules.case_sensitive);
    assert_eq!(rules.pathext, [".EXE"]);
}
