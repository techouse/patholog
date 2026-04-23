use crate::clean::clean_path;
use crate::doctor::diagnose_path;
use crate::model::ExitCode;
use crate::output::human::{format_conflicts, format_doctor, format_print, format_why};
use crate::output::json::{doctor_to_json, dumps_json, entries_to_json, resolution_to_json};
use crate::path_env::parse_path;
use crate::resolve::resolve_command;

use super::fail_on::parse_fail_on;
use super::types::{
    CleanOptions, Cli, CliResult, Command, CommandContext, CommonOptions, DoctorOptions,
    ResolutionOptions,
};

pub(super) fn execute(cli: Cli, context: &CommandContext) -> CliResult {
    match cli.command {
        Command::Print(options) => run_print(options, context),
        Command::Doctor(options) => run_doctor(options, context),
        Command::Why(options) => run_why(options, context),
        Command::Conflicts(options) => run_conflicts(options, context),
        Command::Clean(options) => run_clean(options, context),
    }
}

fn run_print(options: CommonOptions, context: &CommandContext) -> CliResult {
    let entries = parse_path(
        &context.path_value,
        options.platform,
        context.pathext.as_deref(),
    );
    if options.json {
        return json_result(entries_to_json(&entries));
    }
    CliResult::success(format_print(&entries))
}

fn run_doctor(options: DoctorOptions, context: &CommandContext) -> CliResult {
    let report = diagnose_path(
        &context.path_value,
        options.common.platform,
        context.pathext.as_deref(),
    );
    let selected_issue_kinds = match parse_fail_on(&options.fail_on) {
        Ok(selected_issue_kinds) => selected_issue_kinds,
        Err(message) => return CliResult::error(message),
    };
    let stdout = if options.common.json {
        match dumps_json(&doctor_to_json(&report)) {
            Ok(stdout) => stdout,
            Err(error) => return CliResult::error(error.to_string()),
        }
    } else {
        format_doctor(&report)
    };

    let has_selected_issue = !selected_issue_kinds.is_empty()
        && report
            .diagnostics
            .iter()
            .any(|diagnostic| selected_issue_kinds.contains(&diagnostic.kind));

    CliResult {
        exit_code: if has_selected_issue {
            ExitCode::DiagnosticsFound
        } else {
            ExitCode::Success
        },
        stdout,
        stderr: String::new(),
    }
}

fn run_why(options: ResolutionOptions, context: &CommandContext) -> CliResult {
    let report = resolve_command(
        &context.path_value,
        &options.command,
        options.common.platform,
        context.pathext.as_deref(),
        &context.cwd,
        true,
    );
    let stdout = if options.common.json {
        match dumps_json(&resolution_to_json(&report)) {
            Ok(stdout) => stdout,
            Err(error) => return CliResult::error(error.to_string()),
        }
    } else {
        format_why(&report)
    };
    resolution_result(stdout, report.candidates.is_empty())
}

fn run_conflicts(options: ResolutionOptions, context: &CommandContext) -> CliResult {
    let report = resolve_command(
        &context.path_value,
        &options.command,
        options.common.platform,
        context.pathext.as_deref(),
        &context.cwd,
        false,
    );
    let stdout = if options.common.json {
        match dumps_json(&resolution_to_json(&report)) {
            Ok(stdout) => stdout,
            Err(error) => return CliResult::error(error.to_string()),
        }
    } else {
        format_conflicts(&report)
    };
    resolution_result(stdout, report.candidates.is_empty())
}

fn run_clean(options: CleanOptions, context: &CommandContext) -> CliResult {
    if !options.stdout {
        return CliResult::error("clean requires --stdout");
    }
    CliResult::success(format!(
        "{}\n",
        clean_path(
            &context.path_value,
            options.platform,
            context.pathext.as_deref()
        )
    ))
}

fn json_result(value: serde_json::Value) -> CliResult {
    match dumps_json(&value) {
        Ok(stdout) => CliResult::success(stdout),
        Err(error) => CliResult::error(error.to_string()),
    }
}

fn resolution_result(stdout: String, not_found: bool) -> CliResult {
    CliResult {
        exit_code: if not_found {
            ExitCode::CommandNotFound
        } else {
            ExitCode::Success
        },
        stdout,
        stderr: String::new(),
    }
}
