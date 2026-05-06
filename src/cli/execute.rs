use std::path::Path;

use clap::CommandFactory;

use crate::apply::{ApplyPlanOptions, plan_apply};
use crate::clean::{clean_export_with_policy, clean_path_with_policy};
use crate::doctor::{diagnose_command_path_with_policy, diagnose_path_with_policy};
use crate::model::{ExitCode, PathVariable, PlatformMode, PresetKind, ShellKind};
use crate::output::human::{
    format_apply_plan, format_conflicts, format_doctor, format_print, format_shell_profile_scan,
    format_why,
};
use crate::output::json::{
    apply_plan_to_json, doctor_to_json, dumps_json, entries_to_json, resolution_to_json,
    shell_profile_scan_to_json,
};
use crate::path_env::parse_path;
use crate::platform::resolve_platform_rules;
use crate::policy::PathPolicy;
use crate::profile_scan::scan_shell_profiles;
use crate::resolve::resolve_command;

use super::fail_on::parse_fail_on;
use super::types::{
    ApplyOptions, CleanOptions, Cli, CliResult, Command, CommandContext, CompletionOptions,
    DoctorOptions, PrintOptions, ResolutionOptions, ScanOptions,
};

pub(super) fn execute(cli: Cli, context: &CommandContext) -> CliResult {
    match cli.command {
        Command::Print(options) => run_print(options, context),
        Command::Doctor(options) => run_doctor(options, context),
        Command::Why(options) => run_why(options, context),
        Command::Conflicts(options) => run_conflicts(options, context),
        Command::Clean(options) => run_clean(options, context),
        Command::Apply(options) => run_apply(options, context),
        Command::Scan(options) => run_scan(options, context),
        Command::Completions(options) => run_completions(options),
    }
}

fn run_print(options: PrintOptions, context: &CommandContext) -> CliResult {
    let entries = parse_path(
        variable_value(context, options.variable),
        options.common.platform,
        context.pathext.as_deref(),
    );
    if options.common.json {
        return json_result(entries_to_json(&entries));
    }
    CliResult::success(format_print(&entries))
}

fn run_doctor(options: DoctorOptions, context: &CommandContext) -> CliResult {
    if options.command.is_some() && options.variable != PathVariable::Path {
        return CliResult::error("doctor --command only supports --var path");
    }
    let policy = path_policy(&options.drop_entries, &options.presets, options.variable);
    let report = if let Some(command) = options.command.as_deref() {
        diagnose_command_path_with_policy(
            &context.path_value,
            command,
            options.common.platform,
            context.pathext.as_deref(),
            &context.cwd,
            &policy,
        )
    } else {
        diagnose_path_with_policy(
            variable_value(context, options.variable),
            options.common.platform,
            context.pathext.as_deref(),
            options.variable,
            &policy,
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
        let policy = path_policy(&options.drop_entries, &options.presets, options.variable);
        return CliResult::success(format!(
            "{}\n",
            clean_export_with_policy(
                variable_value(context, options.variable),
                options.platform,
                context.pathext.as_deref(),
                shell,
                options.variable,
                &policy,
            )
        ));
    }
    let policy = path_policy(&options.drop_entries, &options.presets, options.variable);
    CliResult::success(format!(
        "{}\n",
        clean_path_with_policy(
            variable_value(context, options.variable),
            options.platform,
            context.pathext.as_deref(),
            options.variable,
            &policy,
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

fn run_apply(options: ApplyOptions, context: &CommandContext) -> CliResult {
    if !options.dry_run {
        return CliResult::error("apply requires --dry-run in v0.5.x");
    }
    let Some(shell) = options.shell else {
        return CliResult::error("apply requires --shell");
    };
    let policy = path_policy(&options.drop_entries, &options.presets, PathVariable::Path);

    let home_dir = options
        .home
        .as_deref()
        .or_else(|| apply_home_dir(options.common.platform, context));
    let user_profile_dir = options
        .home
        .as_deref()
        .or_else(|| apply_user_profile_dir(options.common.platform, context));
    let plan = match plan_apply(&ApplyPlanOptions {
        path_value: &context.path_value,
        platform_mode: options.common.platform,
        pathext: context.pathext.as_deref(),
        shell,
        home_dir,
        user_profile_dir,
        profile: options.profile.as_deref(),
        policy,
    }) {
        Ok(plan) => plan,
        Err(message) => return CliResult::error(message),
    };
    if options.common.json {
        return json_result(apply_plan_to_json(&plan));
    }
    CliResult::success(format_apply_plan(&plan))
}

fn variable_value(context: &CommandContext, variable: PathVariable) -> &str {
    match variable {
        PathVariable::Path => &context.path_value,
        PathVariable::Manpath => &context.manpath_value,
    }
}

fn path_policy(
    drop_entries: &[String],
    presets: &[PresetKind],
    variable: PathVariable,
) -> PathPolicy {
    PathPolicy::new(drop_entries, presets, variable)
}

fn apply_home_dir(platform_mode: PlatformMode, context: &CommandContext) -> Option<&Path> {
    match resolve_platform_rules(platform_mode, None).mode {
        PlatformMode::Windows => context.home_dir.as_deref(),
        PlatformMode::Auto | PlatformMode::Posix => context.home_dir.as_deref(),
    }
}

fn apply_user_profile_dir(platform_mode: PlatformMode, context: &CommandContext) -> Option<&Path> {
    match resolve_platform_rules(platform_mode, None).mode {
        PlatformMode::Windows => context.user_profile_dir.as_deref(),
        PlatformMode::Auto | PlatformMode::Posix => context.user_profile_dir.as_deref(),
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
