use crate::model::PlatformMode;

use super::{parse_pathext, resolve_platform_rules};

#[test]
fn parse_pathext_uses_default_when_missing() {
    assert_eq!(parse_pathext(None), [".EXE", ".CMD", ".BAT"]);
}

#[test]
fn parse_pathext_uses_default_when_empty_or_only_separators() {
    assert_eq!(parse_pathext(Some("")), [".EXE", ".CMD", ".BAT"]);
    assert_eq!(parse_pathext(Some(" ; ; ")), [".EXE", ".CMD", ".BAT"]);
}

#[test]
fn parse_pathext_normalizes_extensions_and_removes_duplicates() {
    assert_eq!(parse_pathext(Some("EXE;.cmd;.EXE;;")), [".EXE", ".CMD"]);
}

#[test]
fn auto_rules_resolve_to_host_platform() {
    let rules = resolve_platform_rules(PlatformMode::Auto, None);

    if cfg!(windows) {
        assert_eq!(rules.mode, PlatformMode::Windows);
    } else {
        assert_eq!(rules.mode, PlatformMode::Posix);
    }
}

#[test]
fn windows_rules_use_semicolon_and_case_insensitive_keys() {
    let rules = resolve_platform_rules(PlatformMode::Windows, Some(".EXE"));

    assert_eq!(rules.separator, ';');
    assert!(!rules.case_sensitive);
    assert_eq!(rules.pathext, [".EXE"]);
}
