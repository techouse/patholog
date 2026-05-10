use crate::model::IssueKind;

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
        let kind = kind_value
            .parse::<IssueKind>()
            .map_err(|error| format!("unsupported issue kind {kind_value:?}; {error}"))?;
        if !selected.contains(&kind) {
            selected.push(kind);
        }
    }
    Ok(selected)
}
