use crate::model::IssueKind;

const SUPPORTED_ISSUE_KIND_VALUES: &str = "duplicate, empty, missing, not_directory, unreadable, suspicious_order, shadowed_command, unwanted";

pub(super) fn parse_fail_on(value: &str) -> Result<Vec<IssueKind>, String> {
    if value.is_empty() {
        return Ok(Vec::new());
    }

    let mut selected = Vec::new();
    for raw_kind in value.split(',') {
        let kind_value = raw_kind.trim();
        if kind_value.is_empty() {
            continue;
        }
        let Some(kind) = issue_kind_by_value(kind_value) else {
            return Err(format!(
                "unsupported issue kind {kind_value:?}; expected one of: {SUPPORTED_ISSUE_KIND_VALUES}"
            ));
        };
        if !selected.contains(&kind) {
            selected.push(kind);
        }
    }
    Ok(selected)
}

fn issue_kind_by_value(value: &str) -> Option<IssueKind> {
    match value {
        "duplicate" => Some(IssueKind::Duplicate),
        "empty" => Some(IssueKind::Empty),
        "missing" => Some(IssueKind::Missing),
        "not_directory" => Some(IssueKind::NotDirectory),
        "unreadable" => Some(IssueKind::Unreadable),
        "suspicious_order" => Some(IssueKind::SuspiciousOrder),
        "shadowed_command" => Some(IssueKind::ShadowedCommand),
        "unwanted" => Some(IssueKind::Unwanted),
        _ => None,
    }
}
