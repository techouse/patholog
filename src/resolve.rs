use std::path::Path;

mod candidates;
mod filesystem;
mod hints;
mod names;

use crate::model::{PlatformMode, ResolutionReport};
use crate::path_env::parse_path;
use crate::platform::resolve_platform_rules;

use self::candidates::{find_candidates, searched_directory_display};
use self::hints::related_hints;

pub(crate) fn resolve_command(
    path_value: &str,
    command: &str,
    platform_mode: PlatformMode,
    pathext: Option<&str>,
    cwd: &Path,
    include_related_hints: bool,
) -> ResolutionReport {
    let rules = resolve_platform_rules(platform_mode, pathext);
    let entries = parse_path(path_value, platform_mode, pathext);
    let candidates = find_candidates(&entries, command, &rules, cwd);
    let related_hints = if include_related_hints && candidates.is_empty() {
        related_hints(&entries, command, &rules, cwd)
    } else {
        Vec::new()
    };
    let searched_directories = entries.iter().map(searched_directory_display).collect();

    ResolutionReport {
        command: command.to_owned(),
        candidates,
        searched_directories,
        related_hints,
    }
}

#[cfg(test)]
#[path = "resolve/tests.rs"]
mod tests;
