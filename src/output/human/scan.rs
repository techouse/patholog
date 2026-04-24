use crate::model::{ShellProfile, ShellProfileScanReport};

use super::shared::finish_lines;

pub(crate) fn format_shell_profile_scan(report: &ShellProfileScanReport) -> String {
    let mut lines = vec![format!("Shell profile scan: {}", report.home)];

    let profiles_with_mutations = report
        .profiles
        .iter()
        .filter(|profile| !profile.path_mutations.is_empty())
        .collect::<Vec<_>>();
    let unreadable_profiles = report
        .profiles
        .iter()
        .filter(|profile| profile.exists && profile.is_file && !profile.readable)
        .collect::<Vec<_>>();

    if profiles_with_mutations.is_empty() && unreadable_profiles.is_empty() {
        lines.push(String::new());
        lines.push("No PATH changes found in readable shell profiles.".to_owned());
        return finish_lines(lines);
    }

    if !profiles_with_mutations.is_empty() {
        lines.push(String::new());
        lines.push("PATH changes:".to_owned());
        for profile in profiles_with_mutations {
            append_profile_mutations(&mut lines, profile);
        }
    }

    if !unreadable_profiles.is_empty() {
        lines.push(String::new());
        lines.push("Unreadable profiles:".to_owned());
        lines.extend(
            unreadable_profiles
                .iter()
                .map(|profile| format!("  {} ({})", profile.path, profile.shell)),
        );
    }

    finish_lines(lines)
}

fn append_profile_mutations(lines: &mut Vec<String>, profile: &ShellProfile) {
    lines.push(format!("  {} ({})", profile.path, profile.shell));
    lines.extend(profile.path_mutations.iter().map(|mutation| {
        format!(
            "    line {}  {}  {}",
            mutation.line, mutation.kind, mutation.text
        )
    }));
}
