use crate::model::{PlatformMode, PlatformRules};

const DEFAULT_WINDOWS_PATHEXT: &[&str] = &[".EXE", ".CMD", ".BAT"];

pub(crate) fn resolve_platform_rules(mode: PlatformMode, pathext: Option<&str>) -> PlatformRules {
    let resolved_mode = match mode {
        PlatformMode::Auto if cfg!(windows) => PlatformMode::Windows,
        PlatformMode::Auto => PlatformMode::Posix,
        explicit => explicit,
    };

    match resolved_mode {
        PlatformMode::Windows => PlatformRules {
            mode: PlatformMode::Windows,
            separator: ';',
            case_sensitive: false,
            pathext: parse_pathext(pathext),
        },
        PlatformMode::Auto | PlatformMode::Posix => PlatformRules {
            mode: PlatformMode::Posix,
            separator: ':',
            case_sensitive: true,
            pathext: Vec::new(),
        },
    }
}

pub(crate) fn parse_pathext(value: Option<&str>) -> Vec<String> {
    let Some(value) = value else {
        return default_windows_pathext();
    };
    if value.is_empty() {
        return default_windows_pathext();
    }

    let mut extensions = Vec::new();
    for raw_extension in value.split(';') {
        let extension = raw_extension.trim();
        if extension.is_empty() {
            continue;
        }
        let extension = if extension.starts_with('.') {
            extension.to_owned()
        } else {
            format!(".{extension}")
        }
        .to_uppercase();

        if !extensions.contains(&extension) {
            extensions.push(extension);
        }
    }

    if extensions.is_empty() {
        default_windows_pathext()
    } else {
        extensions
    }
}

fn default_windows_pathext() -> Vec<String> {
    DEFAULT_WINDOWS_PATHEXT
        .iter()
        .map(|extension| (*extension).to_owned())
        .collect()
}

#[cfg(test)]
#[path = "platform/tests.rs"]
mod tests;
