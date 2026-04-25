#![forbid(unsafe_code)]

use std::path::PathBuf;

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
    let context = CommandContext {
        path_value: case.path,
        pathext: case.pathext,
        cwd: PathBuf::from("."),
        home_dir: None,
        user_profile_dir: None,
    };

    for args in cli_args(&case.command_mode, platform, case.json) {
        let result = run(args, context.clone());
        assert!(result.stdout.len() <= MAX_OUTPUT_LEN);
        assert!(result.stderr.len() <= MAX_OUTPUT_LEN);
    }
}

fn path_case_from_input(data: &[u8]) -> PathCase {
    let mut case = parse_json_case::<PathCase>(data).unwrap_or_else(|| PathCase {
        path: String::from_utf8_lossy(truncate_slice(data, MAX_INPUT_LEN)).into_owned(),
        platform: "posix".to_owned(),
        pathext: None,
        json: false,
        command_mode: None,
    });
    case.path = truncate_string(case.path, MAX_INPUT_LEN);
    if let Some(pathext) = case.pathext.take() {
        case.pathext = Some(truncate_string(pathext, MAX_INPUT_LEN));
    }
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

fn cli_args(command_mode: &Option<String>, platform: &str, json: bool) -> Vec<Vec<String>> {
    match command_mode.as_deref() {
        Some("print") => vec![print_args(platform, json)],
        Some("doctor") => vec![doctor_args(platform, json)],
        Some("clean") => vec![clean_args(platform)],
        _ => vec![
            print_args(platform, json),
            doctor_args(platform, json),
            clean_args(platform),
        ],
    }
}

fn print_args(platform: &str, json: bool) -> Vec<String> {
    let mut args = vec![
        "print".to_owned(),
        "--platform".to_owned(),
        platform.to_owned(),
    ];
    if json {
        args.push("--json".to_owned());
    }
    args
}

fn doctor_args(platform: &str, json: bool) -> Vec<String> {
    let mut args = vec![
        "doctor".to_owned(),
        "--platform".to_owned(),
        platform.to_owned(),
    ];
    if json {
        args.push("--json".to_owned());
    }
    args
}

fn clean_args(platform: &str) -> Vec<String> {
    vec![
        "clean".to_owned(),
        "--stdout".to_owned(),
        "--platform".to_owned(),
        platform.to_owned(),
    ]
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
