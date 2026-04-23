use std::path::Path;

use crate::model::{PathEntry, PlatformMode};
use crate::normalize::comparison_key;
use crate::platform::resolve_platform_rules;

pub(crate) fn parse_path(
    path_value: &str,
    platform_mode: PlatformMode,
    pathext: Option<&str>,
) -> Vec<PathEntry> {
    let rules = resolve_platform_rules(platform_mode, pathext);

    path_value
        .split(rules.separator)
        .enumerate()
        .map(|(zero_based_index, raw_entry)| {
            let index = zero_based_index + 1;
            if raw_entry.is_empty() {
                return PathEntry {
                    index,
                    raw: String::new(),
                    display: String::new(),
                    comparison_key: String::new(),
                    exists: false,
                    is_dir: false,
                    is_empty: true,
                };
            }

            let path = Path::new(raw_entry);
            let exists = path.exists();
            PathEntry {
                index,
                raw: raw_entry.to_owned(),
                display: raw_entry.to_owned(),
                comparison_key: comparison_key(raw_entry, &rules),
                exists,
                is_dir: exists && path.is_dir(),
                is_empty: false,
            }
        })
        .collect()
}

#[cfg(test)]
#[path = "path_env/tests.rs"]
mod tests;
