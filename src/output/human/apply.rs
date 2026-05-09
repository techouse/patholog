use crate::model::{ApplyAction, ApplyOutcome, ApplyPlan};

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

pub(crate) fn format_apply_outcome(outcome: &ApplyOutcome) -> String {
    let mut lines = vec![
        format!("Apply: {}", outcome.plan.shell.as_str()),
        String::new(),
        "Target profile:".to_owned(),
        format!("  {}", outcome.plan.profile_path),
        "Action:".to_owned(),
        format!("  {}", outcome.plan.action.as_str()),
        "Wrote:".to_owned(),
        format!("  {}", outcome.wrote),
        "Backup:".to_owned(),
        backup_line(outcome),
        String::new(),
        "Cleaned PATH:".to_owned(),
        format!("  {}", outcome.plan.cleaned_path),
        String::new(),
        "Written block:".to_owned(),
    ];
    lines.extend(outcome.plan.planned_block.lines().map(str::to_owned));
    finish_lines(lines)
}

fn backup_line(outcome: &ApplyOutcome) -> String {
    match (&outcome.backup_path, outcome.backup_created) {
        (Some(path), true) => format!("  {path}"),
        (None, false) if outcome.plan.action == ApplyAction::CreateProfile => "  none".to_owned(),
        (None, false) => "  disabled".to_owned(),
        _ => "  disabled".to_owned(),
    }
}
