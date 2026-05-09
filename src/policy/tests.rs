use crate::model::{PathVariable, PlatformMode, PresetKind};
use crate::path_env::parse_path;

use super::PathPolicy;

#[test]
fn explicit_drop_matches_after_platform_normalization() {
    let policy = PathPolicy::new(&[r"C:\Tools\Bin".to_owned()], &[], PathVariable::Path)
        .compile(PlatformMode::Windows, None);
    let entries = parse_path(r"c:/tools/bin;C:\Other", PlatformMode::Windows, None);

    assert!(policy.matches_entry(&entries[0]));
    assert!(!policy.matches_entry(&entries[1]));
}

#[test]
fn fink_preset_adds_drop_rules_once() {
    let policy = PathPolicy::new(
        &["/sw/bin".to_owned()],
        &[PresetKind::Fink, PresetKind::Fink],
        PathVariable::Path,
    )
    .compile(PlatformMode::Posix, None);
    let entries = parse_path(
        "/sw/bin:/sw/sbin:/sw/share/man:/usr/bin",
        PlatformMode::Posix,
        None,
    );

    assert!(policy.matches_entry(&entries[0]));
    assert!(policy.matches_entry(&entries[1]));
    assert!(!policy.matches_entry(&entries[2]));
    assert!(!policy.matches_entry(&entries[3]));
}

#[test]
fn fink_preset_uses_manpath_drop_rules_for_manpath() {
    let policy = PathPolicy::new(&[], &[PresetKind::Fink], PathVariable::Manpath)
        .compile(PlatformMode::Posix, None);
    let entries = parse_path(
        "/sw/bin:/sw/sbin:/sw/share/man:/usr/share/man",
        PlatformMode::Posix,
        None,
    );

    assert!(!policy.matches_entry(&entries[0]));
    assert!(!policy.matches_entry(&entries[1]));
    assert!(policy.matches_entry(&entries[2]));
    assert!(!policy.matches_entry(&entries[3]));
}

#[test]
fn non_fink_presets_are_kept_as_deduped_ordering_rules() {
    let policy = PathPolicy::new(
        &[],
        &[
            PresetKind::Cargo,
            PresetKind::Homebrew,
            PresetKind::Cargo,
            PresetKind::Pyenv,
        ],
        PathVariable::Path,
    );

    assert_eq!(
        policy.ordering_presets(),
        &[PresetKind::Cargo, PresetKind::Homebrew, PresetKind::Pyenv]
    );
}
