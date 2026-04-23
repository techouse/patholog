use crate::model::PlatformMode;
use crate::normalize::first_unique_entries;
use crate::path_env::parse_path;
use crate::platform::resolve_platform_rules;

pub(crate) fn clean_path(
    path_value: &str,
    platform_mode: PlatformMode,
    pathext: Option<&str>,
) -> String {
    let rules = resolve_platform_rules(platform_mode, pathext);
    let entries = parse_path(path_value, platform_mode, pathext);
    first_unique_entries(&entries)
        .iter()
        .map(|entry| entry.raw.as_str())
        .collect::<Vec<_>>()
        .join(&rules.separator.to_string())
}

#[cfg(test)]
#[path = "clean/tests.rs"]
mod tests;
