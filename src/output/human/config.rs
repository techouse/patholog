use crate::config::{ConfigPolicy, LoadedConfig};

use super::shared::finish_lines;

pub(crate) fn format_config_check(config: &LoadedConfig) -> String {
    finish_lines(vec![format!("Config OK: {}", config.path.display())])
}

pub(crate) fn format_config_print(config: &LoadedConfig) -> String {
    finish_lines(vec![
        format!("Config: {}", config.path.display()),
        format!("Version: {}", config.config.version),
        String::new(),
        "PATH:".to_owned(),
        format!("  drop: {}", format_drop_entries(&config.config.path)),
        format!("  preset: {}", format_presets(&config.config.path)),
        format!("  fail_on: {}", format_fail_on(&config.config.path)),
        String::new(),
        "MANPATH:".to_owned(),
        format!("  drop: {}", format_drop_entries(&config.config.manpath)),
        format!("  preset: {}", format_presets(&config.config.manpath)),
        format!("  fail_on: {}", format_fail_on(&config.config.manpath)),
    ])
}

fn format_drop_entries(policy: &ConfigPolicy) -> String {
    format_list(policy.drop_entries.iter().map(String::as_str))
}

fn format_presets(policy: &ConfigPolicy) -> String {
    format_list(policy.presets.iter().map(|preset| preset.as_str()))
}

fn format_fail_on(policy: &ConfigPolicy) -> String {
    format_list(policy.fail_on.iter().map(|kind| kind.as_str()))
}

fn format_list<'a>(items: impl Iterator<Item = &'a str>) -> String {
    let values = items.collect::<Vec<_>>();
    if values.is_empty() {
        "none".to_owned()
    } else {
        values.join(", ")
    }
}
