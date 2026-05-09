use crate::model::{PathEntry, PathVariable, PlatformMode, PresetKind};
use crate::normalize::comparison_key;
use crate::platform::resolve_platform_rules;
use std::collections::HashSet;

const FINK_PATH_DROP_ENTRIES: &[&str] = &["/sw/bin", "/sw/sbin"];
const FINK_MANPATH_DROP_ENTRIES: &[&str] = &["/sw/share/man"];

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct PathPolicy {
    drop_entries: Vec<String>,
    ordering_presets: Vec<PresetKind>,
}

impl PathPolicy {
    pub(crate) fn new(
        explicit_drops: &[String],
        presets: &[PresetKind],
        variable: PathVariable,
    ) -> Self {
        let mut drop_entries = Vec::new();
        let mut ordering_presets = Vec::new();
        for entry in explicit_drops {
            push_unique(&mut drop_entries, entry);
        }
        for preset in presets {
            match preset {
                PresetKind::Fink => {
                    for entry in fink_drop_entries(variable) {
                        push_unique(&mut drop_entries, entry);
                    }
                }
                PresetKind::Homebrew | PresetKind::Cargo | PresetKind::Pyenv => {
                    push_unique_preset(&mut ordering_presets, *preset);
                }
            }
        }
        Self {
            drop_entries,
            ordering_presets,
        }
    }

    pub(crate) fn has_drop_entries(&self) -> bool {
        !self.drop_entries.is_empty()
    }

    pub(crate) fn ordering_presets(&self) -> &[PresetKind] {
        &self.ordering_presets
    }

    pub(crate) fn compile(
        &self,
        platform_mode: PlatformMode,
        pathext: Option<&str>,
    ) -> CompiledPathPolicy {
        let rules = resolve_platform_rules(platform_mode, pathext);
        let mut drop_keys = HashSet::with_capacity(self.drop_entries.len());
        for entry in &self.drop_entries {
            let key = comparison_key(entry, &rules);
            drop_keys.insert(key);
        }
        CompiledPathPolicy { drop_keys }
    }
}

fn fink_drop_entries(variable: PathVariable) -> &'static [&'static str] {
    match variable {
        PathVariable::Path => FINK_PATH_DROP_ENTRIES,
        PathVariable::Manpath => FINK_MANPATH_DROP_ENTRIES,
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct CompiledPathPolicy {
    drop_keys: HashSet<String>,
}

impl CompiledPathPolicy {
    pub(crate) fn matches_entry(&self, entry: &PathEntry) -> bool {
        !entry.is_empty && self.drop_keys.contains(&entry.comparison_key)
    }
}

fn push_unique(entries: &mut Vec<String>, entry: &str) {
    if entries.iter().any(|existing| existing == entry) {
        return;
    }
    entries.push(entry.to_owned());
}

fn push_unique_preset(presets: &mut Vec<PresetKind>, preset: PresetKind) {
    if presets.contains(&preset) {
        return;
    }
    presets.push(preset);
}

#[cfg(test)]
#[path = "policy/tests.rs"]
mod tests;
