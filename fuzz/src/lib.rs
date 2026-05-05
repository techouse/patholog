#![forbid(unsafe_code)]

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};

use patholog::cli::{CommandContext, run};
use patholog::model::PlatformMode;
use serde::Deserialize;

const MAX_INPUT_LEN: usize = 4096;
const MAX_JSON_INPUT_LEN: usize = 8192;
const MAX_OUTPUT_LEN: usize = 1_048_576;

#[derive(Debug, Deserialize, Default)]
struct PathCase {
    #[serde(default)]
    path: String,
    #[serde(default)]
    platform: String,
    pathext: Option<String>,
    #[serde(default)]
    json: bool,
    command_mode: Option<String>,
    #[serde(default)]
    manpath: String,
    #[serde(default)]
    variable: String,
    #[serde(default)]
    drop_entries: Vec<String>,
    #[serde(default)]
    preset: String,
    #[serde(default)]
    shell: String,
    #[serde(default)]
    profile_mode: String,
}

pub fn run_path_clean_bytes(data: &[u8]) {
    let case = path_case_from_input(data);
    let platform_mode = platform_mode(&case.platform);
    let cleaned = patholog::fuzzing::clean_path(&case.path, platform_mode, case.pathext.as_deref());
    let cleaned_again =
        patholog::fuzzing::clean_path(&cleaned, platform_mode, case.pathext.as_deref());

    assert!(cleaned.len() <= MAX_OUTPUT_LEN);
    assert_eq!(cleaned_again, cleaned);
    assert_no_empty_entries(&cleaned, platform_mode);
    assert_cleaned_entries_preserve_order(&case.path, &cleaned, platform_mode);
}

pub fn run_path_parse_doctor_bytes(data: &[u8]) {
    let case = path_case_from_input(data);
    let platform_mode = platform_mode(&case.platform);
    let entries = patholog::fuzzing::parse_path(&case.path, platform_mode, case.pathext.as_deref());

    for (zero_based_index, entry) in entries.iter().enumerate() {
        assert_eq!(entry.index, zero_based_index + 1);
        assert_eq!(entry.is_empty, entry.raw.is_empty());
        assert_eq!(entry.display, entry.raw);
    }

    let print_json = patholog::fuzzing::print_json(&entries).expect("render print json");
    assert!(print_json.ends_with('\n'));
    assert!(print_json.len() <= MAX_OUTPUT_LEN);
    let _: serde_json::Value = serde_json::from_str(&print_json).expect("parse print json");

    let report =
        patholog::fuzzing::diagnose_path(&case.path, platform_mode, case.pathext.as_deref());
    let doctor_json = patholog::fuzzing::doctor_json(&report).expect("render doctor json");
    assert!(doctor_json.ends_with('\n'));
    assert!(doctor_json.len() <= MAX_OUTPUT_LEN);
    let _: serde_json::Value = serde_json::from_str(&doctor_json).expect("parse doctor json");
}

pub fn run_cli_read_only_bytes(data: &[u8]) {
    let case = path_case_from_input(data);
    let platform = platform_arg(&case.platform);
    let variable = variable_arg(&case.variable);
    let preset = preset_arg(&case.preset);
    let shell = shell_arg(&case.shell);
    let profile = prepare_apply_profile(data, &case.profile_mode);
    let context = CommandContext {
        path_value: case.path,
        manpath_value: case.manpath,
        pathext: case.pathext,
        cwd: PathBuf::from("."),
        home_dir: None,
        user_profile_dir: None,
    };

    let cli_context = CliArgContext {
        platform,
        variable,
        shell,
        profile: profile.as_ref().map(|profile| profile.profile.as_path()),
        drop_entries: &case.drop_entries,
        preset,
        json: case.json,
    };
    for args in cli_args(&case.command_mode, &cli_context) {
        let result = run(args, context.clone());
        assert!(result.stdout.len() <= MAX_OUTPUT_LEN);
        assert!(result.stderr.len() <= MAX_OUTPUT_LEN);
    }
    if let Some(profile) = profile {
        let _ignore = std::fs::remove_dir_all(profile.root);
    }
}

fn path_case_from_input(data: &[u8]) -> PathCase {
    let mut case = parse_json_case::<PathCase>(data).unwrap_or_else(|| PathCase {
        path: String::from_utf8_lossy(truncate_slice(data, MAX_INPUT_LEN)).into_owned(),
        platform: "posix".to_owned(),
        pathext: None,
        json: false,
        command_mode: None,
        manpath: String::new(),
        variable: String::new(),
        drop_entries: Vec::new(),
        preset: String::new(),
        shell: "bash".to_owned(),
        profile_mode: String::new(),
    });
    case.path = truncate_string(case.path, MAX_INPUT_LEN);
    if let Some(pathext) = case.pathext.take() {
        case.pathext = Some(truncate_string(pathext, MAX_INPUT_LEN));
    }
    case.manpath = truncate_string(case.manpath, MAX_INPUT_LEN);
    case.drop_entries = case
        .drop_entries
        .into_iter()
        .take(4)
        .map(|entry| truncate_string(entry, MAX_INPUT_LEN))
        .collect();
    case
}

fn parse_json_case<'a, T>(data: &'a [u8]) -> Option<T>
where
    T: Deserialize<'a>,
{
    serde_json::from_slice(truncate_slice(data, MAX_JSON_INPUT_LEN)).ok()
}

fn platform_mode(value: &str) -> PlatformMode {
    match value.to_ascii_lowercase().as_str() {
        "auto" => PlatformMode::Auto,
        "windows" => PlatformMode::Windows,
        _ => PlatformMode::Posix,
    }
}

fn platform_arg(value: &str) -> &'static str {
    match value.to_ascii_lowercase().as_str() {
        "auto" => "auto",
        "windows" => "windows",
        _ => "posix",
    }
}

fn shell_arg(value: &str) -> &'static str {
    match value.to_ascii_lowercase().as_str() {
        "fish" => "fish",
        "pwsh" | "powershell" => "pwsh",
        "zsh" => "zsh",
        _ => "bash",
    }
}

fn variable_arg(value: &str) -> &'static str {
    match value.to_ascii_lowercase().as_str() {
        "manpath" => "manpath",
        _ => "path",
    }
}

fn preset_arg(value: &str) -> Option<&'static str> {
    match value.to_ascii_lowercase().as_str() {
        "homebrew" => Some("homebrew"),
        "cargo" => Some("cargo"),
        "pyenv" => Some("pyenv"),
        "fink" => Some("fink"),
        _ => None,
    }
}

struct FuzzProfile {
    root: PathBuf,
    profile: PathBuf,
}

fn prepare_apply_profile(data: &[u8], profile_mode: &str) -> Option<FuzzProfile> {
    let mut hasher = DefaultHasher::new();
    data.hash(&mut hasher);
    let root = std::env::temp_dir().join(format!(
        "patholog-fuzz-{}-{}",
        std::process::id(),
        hasher.finish()
    ));
    let profile = root.join("profile");
    let _ignore = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(&root).ok()?;
    match profile_mode {
        "append" => {
            std::fs::write(&profile, "export PATH=\"$HOME/bin:$PATH\"\n").ok()?;
        }
        "replace" => {
            std::fs::write(
                &profile,
                "# >>> patholog PATH >>>\nexport PATH='/old'\n# <<< patholog PATH <<<\n",
            )
            .ok()?;
        }
        "malformed" => {
            std::fs::write(&profile, "# >>> patholog PATH >>>\nexport PATH='/old'\n").ok()?;
        }
        "duplicate" => {
            std::fs::write(
                &profile,
                "# >>> patholog PATH >>>\nexport PATH='/old'\n# <<< patholog PATH <<<\n# >>> patholog PATH >>>\nexport PATH='/old'\n# <<< patholog PATH <<<\n",
            )
            .ok()?;
        }
        "non_file" => {
            std::fs::create_dir(&profile).ok()?;
        }
        _ => {}
    }
    Some(FuzzProfile { root, profile })
}

fn platform_separator(platform_mode: PlatformMode) -> char {
    match platform_mode {
        PlatformMode::Windows => ';',
        PlatformMode::Auto if cfg!(windows) => ';',
        PlatformMode::Auto | PlatformMode::Posix => ':',
    }
}

fn assert_no_empty_entries(cleaned: &str, platform_mode: PlatformMode) {
    if cleaned.is_empty() {
        return;
    }
    assert!(
        cleaned
            .split(platform_separator(platform_mode))
            .all(|entry| !entry.is_empty())
    );
}

fn assert_cleaned_entries_preserve_order(
    original: &str,
    cleaned: &str,
    platform_mode: PlatformMode,
) {
    let separator = platform_separator(platform_mode);
    let mut original_entries = original.split(separator).filter(|entry| !entry.is_empty());
    for cleaned_entry in cleaned.split(separator).filter(|entry| !entry.is_empty()) {
        assert!(original_entries.any(|original_entry| original_entry == cleaned_entry));
    }
}

struct CliArgContext<'a> {
    platform: &'a str,
    variable: &'a str,
    shell: &'a str,
    profile: Option<&'a Path>,
    drop_entries: &'a [String],
    preset: Option<&'a str>,
    json: bool,
}

fn cli_args(command_mode: &Option<String>, context: &CliArgContext<'_>) -> Vec<Vec<String>> {
    match command_mode.as_deref() {
        Some("print") => vec![print_args(context.platform, context.variable, context.json)],
        Some("doctor") => vec![doctor_args(
            context.platform,
            context.variable,
            context.drop_entries,
            context.preset,
            context.json,
        )],
        Some("clean") => vec![clean_args(
            context.platform,
            context.variable,
            context.drop_entries,
            context.preset,
        )],
        Some("clean_export") => vec![clean_export_args(
            context.platform,
            context.variable,
            context.shell,
            context.drop_entries,
            context.preset,
        )],
        Some("apply") => vec![apply_args(
            context.platform,
            context.shell,
            context.profile,
            context.drop_entries,
            context.preset,
            context.json,
        )],
        Some("completions") => vec![completions_args(context.shell)],
        _ => vec![
            print_args(context.platform, context.variable, context.json),
            doctor_args(
                context.platform,
                context.variable,
                context.drop_entries,
                context.preset,
                context.json,
            ),
            clean_args(
                context.platform,
                context.variable,
                context.drop_entries,
                context.preset,
            ),
            clean_export_args(
                context.platform,
                context.variable,
                context.shell,
                context.drop_entries,
                context.preset,
            ),
            apply_args(
                context.platform,
                context.shell,
                context.profile,
                context.drop_entries,
                context.preset,
                context.json,
            ),
            completions_args(context.shell),
        ],
    }
}

fn print_args(platform: &str, variable: &str, json: bool) -> Vec<String> {
    let mut args = vec![
        "print".to_owned(),
        "--platform".to_owned(),
        platform.to_owned(),
        "--var".to_owned(),
        variable.to_owned(),
    ];
    if json {
        args.push("--json".to_owned());
    }
    args
}

fn doctor_args(
    platform: &str,
    variable: &str,
    drop_entries: &[String],
    preset: Option<&str>,
    json: bool,
) -> Vec<String> {
    let mut args = vec![
        "doctor".to_owned(),
        "--platform".to_owned(),
        platform.to_owned(),
        "--var".to_owned(),
        variable.to_owned(),
    ];
    append_policy_args(&mut args, drop_entries, preset);
    if json {
        args.push("--json".to_owned());
    }
    args
}

fn clean_args(
    platform: &str,
    variable: &str,
    drop_entries: &[String],
    preset: Option<&str>,
) -> Vec<String> {
    let mut args = vec![
        "clean".to_owned(),
        "--stdout".to_owned(),
        "--platform".to_owned(),
        platform.to_owned(),
        "--var".to_owned(),
        variable.to_owned(),
    ];
    append_policy_args(&mut args, drop_entries, preset);
    args
}

fn clean_export_args(
    platform: &str,
    variable: &str,
    shell: &str,
    drop_entries: &[String],
    preset: Option<&str>,
) -> Vec<String> {
    let mut args = vec![
        "clean".to_owned(),
        "--export".to_owned(),
        "--shell".to_owned(),
        shell.to_owned(),
        "--platform".to_owned(),
        platform.to_owned(),
        "--var".to_owned(),
        variable.to_owned(),
    ];
    append_policy_args(&mut args, drop_entries, preset);
    args
}

fn completions_args(shell: &str) -> Vec<String> {
    vec!["completions".to_owned(), shell.to_owned()]
}

fn apply_args(
    platform: &str,
    shell: &str,
    profile: Option<&Path>,
    drop_entries: &[String],
    preset: Option<&str>,
    json: bool,
) -> Vec<String> {
    let mut args = vec![
        "apply".to_owned(),
        "--dry-run".to_owned(),
        "--shell".to_owned(),
        shell.to_owned(),
        "--platform".to_owned(),
        platform.to_owned(),
    ];
    if let Some(profile) = profile {
        args.push("--profile".to_owned());
        args.push(profile.display().to_string());
    }
    append_policy_args(&mut args, drop_entries, preset);
    if json {
        args.push("--json".to_owned());
    }
    args
}

fn append_policy_args(args: &mut Vec<String>, drop_entries: &[String], preset: Option<&str>) {
    for entry in drop_entries {
        args.push("--drop".to_owned());
        args.push(entry.to_owned());
    }
    if let Some(preset) = preset {
        args.push("--preset".to_owned());
        args.push(preset.to_owned());
    }
}

fn truncate_string(mut value: String, max_len: usize) -> String {
    if value.len() <= max_len {
        return value;
    }

    let mut end = max_len;
    while !value.is_char_boundary(end) {
        end -= 1;
    }
    value.truncate(end);
    value
}

fn truncate_slice(bytes: &[u8], max_len: usize) -> &[u8] {
    &bytes[..bytes.len().min(max_len)]
}
