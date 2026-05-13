use std::path::Path;

use crate::model::{IssueKind, PathVariable, PlatformMode, PresetKind};
use crate::path_env::parse_path;
use crate::policy::PathPolicy;

use super::{
    load_optional_config, load_required_config, merge_drop_entries, merge_fail_on, merge_presets,
};

#[test]
fn load_required_config_accepts_minimal_config() {
    let directory = tempfile::tempdir().expect("create tempdir");
    let config_path = write_config(directory.path(), "version = 1\n");

    let config = load_required_config(&config_path, directory.path()).expect("load config");

    assert_eq!(config.config.version, 1);
    assert!(config.config.path.drop_entries.is_empty());
    assert!(config.config.manpath.presets.is_empty());
}

#[test]
fn load_required_config_parses_path_and_manpath_policy() {
    let directory = tempfile::tempdir().expect("create tempdir");
    let config_path = write_config(
        directory.path(),
        r#"
version = 1

[path]
drop = ["/drop", "/drop"]
preset = ["homebrew", "cargo"]
fail_on = ["duplicate", "unwanted"]

[manpath]
drop = ["/man"]
preset = ["fink"]
fail_on = ["missing"]
"#,
    );

    let config = load_required_config(&config_path, directory.path()).expect("load config");

    assert_eq!(config.config.path.drop_entries, ["/drop"]);
    assert_eq!(
        config.config.path.presets,
        [PresetKind::Homebrew, PresetKind::Cargo]
    );
    assert_eq!(
        config.config.path.fail_on,
        [IssueKind::Duplicate, IssueKind::Unwanted]
    );
    assert_eq!(config.config.manpath.drop_entries, ["/man"]);
    assert_eq!(config.config.manpath.presets, [PresetKind::Fink]);
    assert_eq!(config.config.manpath.fail_on, [IssueKind::Missing]);
}

#[test]
fn load_required_config_rejects_unknown_fields() {
    let directory = tempfile::tempdir().expect("create tempdir");
    let config_path = write_config(directory.path(), "version = 1\nunknown = true\n");

    let error = load_required_config(&config_path, directory.path()).expect_err("load should fail");

    assert!(error.contains("config file is invalid"));
    assert!(error.contains("unknown"));
}

#[test]
fn load_required_config_rejects_unsupported_version() {
    let directory = tempfile::tempdir().expect("create tempdir");
    let config_path = write_config(directory.path(), "version = 2\n");

    let error = load_required_config(&config_path, directory.path()).expect_err("load should fail");

    assert!(error.contains("unsupported version 2"));
}

#[test]
fn load_required_config_rejects_invalid_preset() {
    let directory = tempfile::tempdir().expect("create tempdir");
    let config_path = write_config(
        directory.path(),
        "version = 1\n[path]\npreset = [\"unknown\"]\n",
    );

    let error = load_required_config(&config_path, directory.path()).expect_err("load should fail");

    assert!(error.contains("unsupported preset"));
}

#[test]
fn load_required_config_rejects_invalid_fail_on() {
    let directory = tempfile::tempdir().expect("create tempdir");
    let config_path = write_config(
        directory.path(),
        "version = 1\n[path]\nfail_on = [\"unknown\"]\n",
    );

    let error = load_required_config(&config_path, directory.path()).expect_err("load should fail");

    assert!(error.contains("unsupported issue kind"));
}

#[test]
fn merge_helpers_append_cli_values_after_config_values() {
    let directory = tempfile::tempdir().expect("create tempdir");
    let config_path = write_config(
        directory.path(),
        "version = 1\n[path]\ndrop = [\"/config\"]\npreset = [\"cargo\"]\nfail_on = [\"duplicate\"]\n",
    );
    let config = load_required_config(&config_path, directory.path()).expect("load config");
    let policy = config.policy_for(PathVariable::Path);

    assert_eq!(
        merge_drop_entries(Some(policy), &[String::from("/cli")]),
        ["/config", "/cli"]
    );
    assert_eq!(
        merge_presets(Some(policy), &[PresetKind::Pyenv]),
        [PresetKind::Cargo, PresetKind::Pyenv]
    );
    assert_eq!(
        merge_fail_on(Some(policy), &[IssueKind::Unwanted]),
        [IssueKind::Duplicate, IssueKind::Unwanted]
    );
}

#[test]
fn fink_preset_remains_variable_specific_after_config_merge() {
    let directory = tempfile::tempdir().expect("create tempdir");
    let config_path = write_config(
        directory.path(),
        "version = 1\n[path]\npreset = [\"fink\"]\n[manpath]\npreset = [\"fink\"]\n",
    );
    let config = load_required_config(&config_path, directory.path()).expect("load config");

    let path_policy = PathPolicy::new(
        &merge_drop_entries(Some(config.policy_for(PathVariable::Path)), &[]),
        &merge_presets(Some(config.policy_for(PathVariable::Path)), &[]),
        PathVariable::Path,
    )
    .compile(PlatformMode::Posix, None);
    let manpath_policy = PathPolicy::new(
        &merge_drop_entries(Some(config.policy_for(PathVariable::Manpath)), &[]),
        &merge_presets(Some(config.policy_for(PathVariable::Manpath)), &[]),
        PathVariable::Manpath,
    )
    .compile(PlatformMode::Posix, None);
    let path_entries = parse_path("/sw/bin:/sw/share/man", PlatformMode::Posix, None);
    let manpath_entries = parse_path("/sw/bin:/sw/share/man", PlatformMode::Posix, None);

    assert!(path_policy.matches_entry(&path_entries[0]));
    assert!(!path_policy.matches_entry(&path_entries[1]));
    assert!(!manpath_policy.matches_entry(&manpath_entries[0]));
    assert!(manpath_policy.matches_entry(&manpath_entries[1]));
}

#[test]
fn load_optional_config_auto_returns_none_when_not_found() {
    let directory = tempfile::tempdir().expect("create tempdir");

    let config = load_optional_config(Some(Path::new("auto")), directory.path())
        .expect("load optional config");

    assert!(config.is_none());
}

#[test]
fn load_required_config_auto_discovers_patholog_toml() {
    let directory = tempfile::tempdir().expect("create tempdir");
    write_config(directory.path(), "version = 1\n");

    let config = load_required_config(Path::new("auto"), directory.path()).expect("load config");

    assert_eq!(config.path, directory.path().join("patholog.toml"));
}

#[test]
fn load_required_config_auto_skips_directory_candidate() {
    let directory = tempfile::tempdir().expect("create tempdir");
    std::fs::create_dir(directory.path().join("patholog.toml")).expect("create config dir");
    let dot_config_path = directory.path().join(".patholog.toml");
    std::fs::write(&dot_config_path, "version = 1\n").expect("write dot config");

    let config = load_required_config(Path::new("auto"), directory.path()).expect("load config");

    assert_eq!(config.path, dot_config_path);
}

#[test]
fn load_optional_config_auto_ignores_directory_candidates() {
    let directory = tempfile::tempdir().expect("create tempdir");
    std::fs::create_dir(directory.path().join("patholog.toml")).expect("create config dir");
    std::fs::create_dir(directory.path().join(".patholog.toml")).expect("create dot config dir");

    let config = load_optional_config(Some(Path::new("auto")), directory.path())
        .expect("load optional config");

    assert!(config.is_none());
}

#[test]
fn load_required_config_auto_errors_when_not_found() {
    let directory = tempfile::tempdir().expect("create tempdir");

    let error =
        load_required_config(Path::new("auto"), directory.path()).expect_err("load should fail");

    assert!(error.contains("config auto did not find"));
}

fn write_config(directory: &Path, content: &str) -> std::path::PathBuf {
    let config_path = directory.join("patholog.toml");
    std::fs::write(&config_path, content).expect("write config");
    config_path
}
