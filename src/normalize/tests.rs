use crate::model::{PathEntry, PlatformMode};
use crate::platform::resolve_platform_rules;

use super::{comparison_key, first_unique_entries};

#[test]
fn posix_comparison_keys_are_case_sensitive() {
    let rules = resolve_platform_rules(PlatformMode::Posix, None);

    assert_ne!(
        comparison_key("/tmp/Tools", &rules),
        comparison_key("/tmp/TOOLS", &rules)
    );
}

#[test]
fn windows_comparison_keys_normalize_and_case_fold() {
    let rules = resolve_platform_rules(PlatformMode::Windows, None);

    assert_eq!(comparison_key(r"C:\Tools\..\Tools", &rules), r"c:\tools");
}

#[test]
fn posix_comparison_keys_collapse_dot_and_dotdot_conservatively() {
    let rules = resolve_platform_rules(PlatformMode::Posix, None);

    assert_eq!(comparison_key("", &rules), "");
    assert_eq!(comparison_key("/a/../", &rules), "/");
    assert_eq!(comparison_key("./", &rules), ".");
    assert_eq!(comparison_key("a/../..", &rules), "..");
}

#[test]
fn windows_comparison_keys_cover_prefix_and_root_variants() {
    let rules = resolve_platform_rules(PlatformMode::Windows, None);

    assert_eq!(comparison_key(r"C:\", &rules), r"c:\");
    assert_eq!(comparison_key(r"C:tools", &rules), "c:tools");
    assert_eq!(comparison_key(r"C:foo\..", &rules), "c:");
    assert_eq!(comparison_key("/", &rules), r"\");
    assert_eq!(comparison_key(r"foo\..", &rules), ".");
    assert_eq!(comparison_key(r"..\foo", &rules), r"..\foo");
    assert_eq!(comparison_key("c", &rules), "c");
}

#[test]
fn first_unique_entries_removes_empty_and_later_duplicates() {
    let entries = vec![
        entry(1, "a", "a", false),
        entry(2, "", "", true),
        entry(3, "a", "a", false),
        entry(4, "b", "b", false),
    ];

    assert_eq!(
        first_unique_entries(&entries)
            .iter()
            .map(|entry| entry.raw.as_str())
            .collect::<Vec<_>>(),
        ["a", "b"]
    );
}

fn entry(index: usize, raw: &str, comparison_key: &str, is_empty: bool) -> PathEntry {
    PathEntry {
        index,
        raw: raw.to_owned(),
        display: raw.to_owned(),
        comparison_key: comparison_key.to_owned(),
        exists: false,
        is_dir: false,
        is_empty,
    }
}
