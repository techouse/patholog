use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};

use serde::Deserialize;

use crate::model::{IssueKind, PathVariable, PresetKind};

const CONFIG_VERSION: u64 = 1;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct LoadedConfig {
    pub(crate) path: PathBuf,
    pub(crate) config: PathologConfig,
}

impl LoadedConfig {
    pub(crate) fn policy_for(&self, variable: PathVariable) -> &ConfigPolicy {
        self.config.policy_for(variable)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct PathologConfig {
    pub(crate) version: u64,
    pub(crate) path: ConfigPolicy,
    pub(crate) manpath: ConfigPolicy,
}

impl PathologConfig {
    pub(crate) fn policy_for(&self, variable: PathVariable) -> &ConfigPolicy {
        match variable {
            PathVariable::Path => &self.path,
            PathVariable::Manpath => &self.manpath,
        }
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct ConfigPolicy {
    pub(crate) drop_entries: Vec<String>,
    pub(crate) presets: Vec<PresetKind>,
    pub(crate) fail_on: Vec<IssueKind>,
}

pub(crate) fn load_optional_config(
    config_arg: Option<&Path>,
    cwd: &Path,
) -> Result<Option<LoadedConfig>, String> {
    let Some(config_arg) = config_arg else {
        return Ok(None);
    };
    if is_auto_config(config_arg) {
        return discover_config(cwd)
            .map(|path| load_config_file(&path))
            .transpose();
    }
    load_config_file(config_arg).map(Some)
}

pub(crate) fn load_required_config(config_arg: &Path, cwd: &Path) -> Result<LoadedConfig, String> {
    if is_auto_config(config_arg) {
        let Some(path) = discover_config(cwd) else {
            return Err("config auto did not find patholog.toml or .patholog.toml".to_owned());
        };
        return load_config_file(&path);
    }
    load_config_file(config_arg)
}

pub(crate) fn merge_drop_entries(
    config_policy: Option<&ConfigPolicy>,
    cli_entries: &[String],
) -> Vec<String> {
    let mut entries = Vec::new();
    if let Some(config_policy) = config_policy {
        entries.extend(config_policy.drop_entries.iter().cloned());
    }
    entries.extend(cli_entries.iter().cloned());
    entries
}

pub(crate) fn merge_presets(
    config_policy: Option<&ConfigPolicy>,
    cli_presets: &[PresetKind],
) -> Vec<PresetKind> {
    let mut presets = Vec::new();
    if let Some(config_policy) = config_policy {
        presets.extend(config_policy.presets.iter().copied());
    }
    presets.extend(cli_presets.iter().copied());
    presets
}

pub(crate) fn merge_fail_on(
    config_policy: Option<&ConfigPolicy>,
    cli_fail_on: &[IssueKind],
) -> Vec<IssueKind> {
    let mut fail_on = Vec::new();
    if let Some(config_policy) = config_policy {
        for kind in &config_policy.fail_on {
            push_unique(&mut fail_on, *kind);
        }
    }
    for kind in cli_fail_on {
        push_unique(&mut fail_on, *kind);
    }
    fail_on
}

fn load_config_file(path: &Path) -> Result<LoadedConfig, String> {
    let content = fs::read_to_string(path)
        .map_err(|error| format!("config file is not readable: {} ({error})", path.display()))?;
    let raw = toml::from_str::<RawConfig>(&content)
        .map_err(|error| format!("config file is invalid: {} ({error})", path.display()))?;
    let config = PathologConfig::try_from(raw)?;
    Ok(LoadedConfig {
        path: path.to_path_buf(),
        config,
    })
}

fn discover_config(cwd: &Path) -> Option<PathBuf> {
    for name in ["patholog.toml", ".patholog.toml"] {
        let path = cwd.join(name);
        if path.exists() {
            return Some(path);
        }
    }
    None
}

fn is_auto_config(path: &Path) -> bool {
    path.as_os_str() == OsStr::new("auto")
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawConfig {
    version: u64,
    #[serde(default)]
    path: RawPolicy,
    #[serde(default)]
    manpath: RawPolicy,
}

impl TryFrom<RawConfig> for PathologConfig {
    type Error = String;

    fn try_from(raw: RawConfig) -> Result<Self, Self::Error> {
        if raw.version != CONFIG_VERSION {
            return Err(format!(
                "config file uses unsupported version {}; expected {CONFIG_VERSION}",
                raw.version
            ));
        }
        Ok(Self {
            version: raw.version,
            path: ConfigPolicy::try_from(raw.path)?,
            manpath: ConfigPolicy::try_from(raw.manpath)?,
        })
    }
}

#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawPolicy {
    #[serde(default, rename = "drop")]
    drop_entries: Vec<String>,
    #[serde(default, rename = "preset")]
    presets: Vec<String>,
    #[serde(default)]
    fail_on: Vec<String>,
}

impl TryFrom<RawPolicy> for ConfigPolicy {
    type Error = String;

    fn try_from(raw: RawPolicy) -> Result<Self, Self::Error> {
        let mut drop_entries = Vec::new();
        for entry in raw.drop_entries {
            push_unique_string(&mut drop_entries, entry);
        }

        let mut presets = Vec::new();
        for preset in raw.presets {
            let kind = preset
                .parse::<PresetKind>()
                .map_err(|error| format!("unsupported preset {preset:?}; {error}"))?;
            push_unique(&mut presets, kind);
        }

        let mut fail_on = Vec::new();
        for issue_kind in raw.fail_on {
            let kind = issue_kind
                .parse::<IssueKind>()
                .map_err(|error| format!("unsupported issue kind {issue_kind:?}; {error}"))?;
            push_unique(&mut fail_on, kind);
        }

        Ok(Self {
            drop_entries,
            presets,
            fail_on,
        })
    }
}

fn push_unique<T: Copy + Eq>(entries: &mut Vec<T>, entry: T) {
    if !entries.contains(&entry) {
        entries.push(entry);
    }
}

fn push_unique_string(entries: &mut Vec<String>, entry: String) {
    if !entries.contains(&entry) {
        entries.push(entry);
    }
}

#[cfg(test)]
#[path = "config/tests.rs"]
mod tests;
