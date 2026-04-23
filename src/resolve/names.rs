use crate::model::{PlatformMode, PlatformRules};

pub(super) fn candidate_names(command: &str, rules: &PlatformRules) -> Vec<String> {
    if rules.mode == PlatformMode::Posix {
        return vec![command.to_owned()];
    }

    if let Some(suffix) = command_suffix(command)
        && rules
            .pathext
            .iter()
            .any(|extension| suffix.eq_ignore_ascii_case(extension))
    {
        return vec![command.to_owned()];
    }

    rules
        .pathext
        .iter()
        .map(|extension| format!("{command}{extension}"))
        .collect()
}

fn command_suffix(command: &str) -> Option<&str> {
    let last_separator = command
        .rfind(['/', '\\'])
        .map_or(0, |separator_index| separator_index + 1);
    command
        .get(last_separator..)?
        .rfind('.')
        .and_then(|dot_index| command.get(last_separator + dot_index..))
}
