use crate::model::ApplyPlan;

use super::shared::finish_lines;

pub(crate) fn format_apply_plan(plan: &ApplyPlan) -> String {
    let mut lines = vec![
        format!("Apply dry-run: {}", plan.shell.as_str()),
        String::new(),
        "Target profile:".to_owned(),
        format!("  {}", plan.profile_path),
        "Action:".to_owned(),
        format!("  {}", plan.action.as_str()),
        "Would write:".to_owned(),
        format!("  {}", plan.would_write),
        String::new(),
        "Cleaned PATH:".to_owned(),
        format!("  {}", plan.cleaned_path),
        String::new(),
        "Planned block:".to_owned(),
    ];
    lines.extend(plan.planned_block.lines().map(str::to_owned));
    finish_lines(lines)
}
