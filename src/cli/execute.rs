use std::path::Path;

use clap::CommandFactory;

use crate::clean::{clean_export, clean_path};
use crate::doctor::{diagnose_command_path, diagnose_path};
use crate::model::{ExitCode, PlatformMode, ShellKind};
use crate::output::human::{
    format_conflicts, format_doctor, format_print, format_shell_profile_scan, format_why,
};
use crate::output::json::{
    doctor_to_json, dumps_json, entries_to_json, resolution_to_json, shell_profile_scan_to_json,
};
use crate::path_env::parse_path;
use crate::platform::resolve_platform_rules;
use crate::profile_scan::scan_shell_profiles;
use crate::resolve::resolve_command;

use super::fail_on::parse_fail_on;
use super::types::{
    CleanOptions, Cli, CliResult, Command, CommandContext, CommonOptions, CompletionOptions,
    DoctorOptions, ResolutionOptions, ScanOptions,
};

pub(super) fn execute(cli: Cli, context: &CommandContext) -> CliResult {
    match cli.command {
        Command::Print(options) => run_print(options, context),
        Command::Doctor(options) => run_doctor(options, context),
        Command::Why(options) => run_why(options, context),
        Command::Conflicts(options) => run_conflicts(options, context),
        Command::Clean(options) => run_clean(options, context),
        Command::Scan(options) => run_scan(options, context),
        Command::Completions(options) => run_completions(options),
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
    let report = if let Some(command) = options.command.as_deref() {
        diagnose_command_path(
            &context.path_value,
            command,
            options.common.platform,
            context.pathext.as_deref(),
            &context.cwd,
        )
    } else {
        diagnose_path(
            &context.path_value,
            options.common.platform,
            context.pathext.as_deref(),
        )
    };
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
    if options.stdout && options.export {
        return CliResult::error("clean output modes --stdout and --export are mutually exclusive");
    }
    if !options.export && options.shell.is_some() {
        return CliResult::error("clean --shell requires --export");
    }
    if !options.stdout && !options.export {
        return CliResult::error("clean requires --stdout or --export");
    }
    if options.export {
        let Some(shell) = options.shell else {
            return CliResult::error("clean --export requires --shell");
        };
        return CliResult::success(format!(
            "{}\n",
            clean_export(
                &context.path_value,
                options.platform,
                context.pathext.as_deref(),
                shell,
            )
        ));
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

fn run_completions(options: CompletionOptions) -> CliResult {
    let mut command = Cli::command();
    let mut stdout = Vec::new();
    clap_complete::generate(
        completion_shell(options.shell),
        &mut command,
        "patholog",
        &mut stdout,
    );
    match String::from_utf8(stdout) {
        Ok(stdout) => CliResult::success(stdout),
        Err(error) => CliResult::error(error.to_string()),
    }
}

fn completion_shell(shell: ShellKind) -> clap_complete::Shell {
    match shell {
        ShellKind::Bash => clap_complete::Shell::Bash,
        ShellKind::Fish => clap_complete::Shell::Fish,
        ShellKind::Pwsh => clap_complete::Shell::PowerShell,
        ShellKind::Zsh => clap_complete::Shell::Zsh,
    }
}

fn run_scan(options: ScanOptions, context: &CommandContext) -> CliResult {
    let Some(home) = options
        .home
        .as_deref()
        .or_else(|| scan_home_dir(options.common.platform, context))
    else {
        return CliResult::error("scan requires a home directory; set HOME or pass --home");
    };

    let report = scan_shell_profiles(home, options.common.platform);
    if options.common.json {
        return json_result(shell_profile_scan_to_json(&report));
    }
    CliResult::success(format_shell_profile_scan(&report))
}

fn scan_home_dir(platform_mode: PlatformMode, context: &CommandContext) -> Option<&Path> {
    match resolve_platform_rules(platform_mode, None).mode {
        PlatformMode::Windows => context
            .user_profile_dir
            .as_deref()
            .or(context.home_dir.as_deref()),
        PlatformMode::Auto | PlatformMode::Posix => context
            .home_dir
            .as_deref()
            .or(context.user_profile_dir.as_deref()),
    }
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
