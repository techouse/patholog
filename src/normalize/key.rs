use crate::model::PlatformRules;

pub(crate) fn comparison_key(raw: &str, rules: &PlatformRules) -> String {
    if raw.is_empty() {
        return String::new();
    }

    if rules.case_sensitive {
        return normalize_posix(raw);
    }

    normalize_windows(raw).to_lowercase()
}

fn normalize_posix(raw: &str) -> String {
    let absolute = raw.starts_with('/');
    let mut parts = Vec::new();
    for part in raw.split('/') {
        match part {
            "" | "." => {}
            ".." => {
                if !parts.is_empty() && parts.last() != Some(&"..") {
                    parts.pop();
                } else if !absolute {
                    parts.push(part);
                }
            }
            _ => parts.push(part),
        }
    }

    let joined = parts.join("/");
    if absolute {
        if joined.is_empty() {
            "/".to_owned()
        } else {
            format!("/{joined}")
        }
    } else if joined.is_empty() {
        ".".to_owned()
    } else {
        joined
    }
}

fn normalize_windows(raw: &str) -> String {
    let path = raw.replace('/', "\\");
    let (prefix, rest) = windows_prefix(&path);
    let rooted = rest.starts_with('\\');
    let mut parts = Vec::new();
    for part in rest.split('\\') {
        match part {
            "" | "." => {}
            ".." => {
                if !parts.is_empty() && parts.last() != Some(&"..") {
                    parts.pop();
                } else if !rooted {
                    parts.push(part);
                }
            }
            _ => parts.push(part),
        }
    }

    let joined = parts.join("\\");
    match (prefix, rooted, joined.is_empty()) {
        (Some(prefix), true, true) => format!("{prefix}\\"),
        (Some(prefix), true, false) => format!("{prefix}\\{joined}"),
        (Some(prefix), false, true) => prefix.to_owned(),
        (Some(prefix), false, false) => format!("{prefix}{joined}"),
        (None, true, true) => "\\".to_owned(),
        (None, true, false) => format!("\\{joined}"),
        (None, false, true) => ".".to_owned(),
        (None, false, false) => joined,
    }
}

fn windows_prefix(path: &str) -> (Option<&str>, &str) {
    let mut chars = path.char_indices();
    let Some((_, first)) = chars.next() else {
        return (None, path);
    };
    let Some((second_index, second)) = chars.next() else {
        return (None, path);
    };
    let rest_index = second_index + second.len_utf8();

    if first.is_ascii_alphabetic() && second == ':' {
        return (path.get(..rest_index), path.get(rest_index..).unwrap_or(""));
    }

    (None, path)
}
