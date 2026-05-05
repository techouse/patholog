use crate::model::{PathEntry, PathVariable, PlatformMode, PresetKind};
use crate::normalize::comparison_key;
use crate::platform::resolve_platform_rules;

const FINK_DROP_ENTRIES: &[&str] = &["/sw/bin", "/sw/sbin", "/sw/share/man"];

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct PathPolicy {
    drop_entries: Vec<String>,
}

impl PathPolicy {
    pub(crate) fn new(
        explicit_drops: &[String],
        presets: &[PresetKind],
        _variable: PathVariable,
    ) -> Self {
        let mut drop_entries = Vec::new();
        for entry in explicit_drops {
            push_unique(&mut drop_entries, entry);
        }
        for preset in presets {
            if *preset == PresetKind::Fink {
                for entry in FINK_DROP_ENTRIES {
                    push_unique(&mut drop_entries, entry);
                }
            }
        }
        Self { drop_entries }
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.drop_entries.is_empty()
    }

    pub(crate) fn compile(
        &self,
        platform_mode: PlatformMode,
        pathext: Option<&str>,
    ) -> CompiledPathPolicy {
        let rules = resolve_platform_rules(platform_mode, pathext);
        let mut drop_keys = Vec::new();
        for entry in &self.drop_entries {
            let key = comparison_key(entry, &rules);
            if !drop_keys.contains(&key) {
                drop_keys.push(key);
            }
        }
        CompiledPathPolicy { drop_keys }
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct CompiledPathPolicy {
    drop_keys: Vec<String>,
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

#[cfg(test)]
#[path = "policy/tests.rs"]
mod tests;
