use std::fs;
use std::path::{Path, PathBuf};

use crate::model::{PlatformMode, PlatformRules};

pub(super) fn find_executable(
    directory: &Path,
    candidate_name: &str,
    rules: &PlatformRules,
) -> Option<PathBuf> {
    if rules.mode == PlatformMode::Windows {
        return find_windows_executable(directory, candidate_name);
    }

    let candidate = directory.join(candidate_name);
    is_posix_executable(&candidate).then_some(candidate)
}

fn is_posix_executable(candidate: &Path) -> bool {
    let Ok(metadata) = fs::metadata(candidate) else {
        return false;
    };
    if !metadata.is_file() {
        return false;
    }
    has_execute_permission(&metadata)
}

#[cfg(unix)]
fn has_execute_permission(metadata: &fs::Metadata) -> bool {
    use std::os::unix::fs::PermissionsExt;

    metadata.permissions().mode() & 0o111 != 0
}

#[cfg(not(unix))]
fn has_execute_permission(_metadata: &fs::Metadata) -> bool {
    true
}

fn find_windows_executable(directory: &Path, candidate_name: &str) -> Option<PathBuf> {
    let read_dir = match fs::read_dir(directory) {
        Ok(read_dir) => read_dir,
        Err(_error) => {
            let direct_candidate = directory.join(candidate_name);
            return direct_candidate.is_file().then_some(direct_candidate);
        }
    };

    let children = read_dir
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .collect::<Vec<_>>();

    for child in &children {
        if child.is_file() && file_name_matches(child, candidate_name, false) {
            return Some(child.clone());
        }
    }

    for child in &children {
        if child.is_file() && file_name_matches(child, candidate_name, true) {
            return Some(child.clone());
        }
    }

    None
}

fn file_name_matches(path: &Path, candidate_name: &str, ignore_case: bool) -> bool {
    let Some(file_name) = path.file_name().and_then(|name| name.to_str()) else {
        return false;
    };
    if ignore_case {
        file_name.eq_ignore_ascii_case(candidate_name)
    } else {
        file_name == candidate_name
    }
}
