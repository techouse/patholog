use std::path::Path;

use clap::CommandFactory;

use crate::apply::{ApplyPlanOptions, plan_apply_operation, write_apply_plan};
use crate::clean::{clean_export_with_policy, clean_path_with_policy};
use crate::config::{
    ConfigPolicy, LoadedConfig, load_optional_config, load_required_config, merge_drop_entries,
    merge_fail_on, merge_presets,
};
use crate::doctor::{diagnose_command_path_with_policy, diagnose_path_with_policy};
use crate::health::summarize_health;
use crate::model::{ExitCode, PathVariable, PlatformMode, PresetKind, ShellKind};
use crate::output::human::{
    format_apply_outcome, format_apply_plan, format_config_check, format_config_print,
    format_conflicts, format_doctor, format_health, format_print, format_shell_profile_scan,
    format_why, format_why_not,
};
use crate::output::json::{
    apply_outcome_to_json, apply_plan_to_json, config_to_json, doctor_to_json, dumps_json,
    entries_to_json, health_to_json, resolution_to_json, shell_profile_scan_to_json,
    why_not_to_json,
};
use crate::path_env::parse_path;
use crate::platform::resolve_platform_rules;
use crate::policy::PathPolicy;
use crate::profile_scan::scan_shell_profiles;
use crate::resolve::resolve_command;
use crate::why_not::{WhyNotOptions, analyze_why_not};

use super::fail_on::parse_fail_on;
use super::types::{
    ApplyOptions, CleanOptions, Cli, CliResult, Command, CommandContext, CompletionOptions,
    ConfigCheckOptions, ConfigCommand, ConfigOptions, ConfigPrintOptions, DoctorOptions,
    HealthOptions, PrintOptions, ResolutionOptions, ScanOptions,
};

pub(super) fn execute(cli: Cli, context: &CommandContext) -> CliResult {
    match cli.command {
        Command::Print(options) => run_print(options, context),
        Command::Doctor(options) => run_doctor(options, context),
        Command::Health(options) => run_health(options, context),
        Command::Why(options) => run_why(options, context),
        Command::WhyNot(options) => run_why_not(options, context),
        Command::Conflicts(options) => run_conflicts(options, context),
        Command::Clean(options) => run_clean(options, context),
        Command::Apply(options) => run_apply(options, context),
        Command::Scan(options) => run_scan(options, context),
        Command::Config(options) => run_config(options, context),
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
    let config = match load_optional_config(options.config.as_deref(), &context.cwd) {
        Ok(config) => config,
        Err(message) => return CliResult::error(message),
    };
    let config_policy = config_policy(config.as_ref(), options.variable);
    let policy = path_policy(
        config_policy,
        &options.drop_entries,
        &options.presets,
        options.variable,
    );
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
    let cli_fail_on = match parse_fail_on(options.fail_on.as_deref().unwrap_or("")) {
        Ok(cli_fail_on) => cli_fail_on,
        Err(message) => return CliResult::error(message),
    };
    let selected_issue_kinds = merge_fail_on(config_policy, &cli_fail_on);
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

fn run_health(options: HealthOptions, context: &CommandContext) -> CliResult {
    let config = match load_optional_config(options.config.as_deref(), &context.cwd) {
        Ok(config) => config,
        Err(message) => return CliResult::error(message),
    };
    let config_policy = config_policy(config.as_ref(), options.variable);
    let policy = path_policy(
        config_policy,
        &options.drop_entries,
        &options.presets,
        options.variable,
    );
    let doctor_report = diagnose_path_with_policy(
        variable_value(context, options.variable),
        options.common.platform,
        context.pathext.as_deref(),
        options.variable,
        &policy,
    );
    let report = summarize_health(doctor_report);
    if options.common.json {
        return json_result(health_to_json(&report));
    }
    CliResult::success(format_health(&report))
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

fn run_why_not(options: ResolutionOptions, context: &CommandContext) -> CliResult {
    let report = analyze_why_not(&WhyNotOptions {
        path_value: &context.path_value,
        command: &options.command,
        platform_mode: options.common.platform,
        pathext: context.pathext.as_deref(),
        cwd: &context.cwd,
        home_dir: context.home_dir.as_deref(),
        user_profile_dir: context.user_profile_dir.as_deref(),
    });
    let stdout = if options.common.json {
        match dumps_json(&why_not_to_json(&report)) {
            Ok(stdout) => stdout,
            Err(error) => return CliResult::error(error.to_string()),
        }
    } else {
        format_why_not(&report)
    };
    resolution_result(stdout, !report.found())
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
    let config = match load_optional_config(options.config.as_deref(), &context.cwd) {
        Ok(config) => config,
        Err(message) => return CliResult::error(message),
    };
    let config_policy = config_policy(config.as_ref(), options.variable);
    if options.export {
        let Some(shell) = options.shell else {
            return CliResult::error("clean --export requires --shell");
        };
        let policy = path_policy(
            config_policy,
            &options.drop_entries,
            &options.presets,
            options.variable,
        );
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
    let policy = path_policy(
        config_policy,
        &options.drop_entries,
        &options.presets,
        options.variable,
    );
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
    if options.dry_run && options.yes {
        return CliResult::error("apply cannot use --dry-run with --yes");
    }
    if options.no_backup && !options.yes {
        return CliResult::error("apply --no-backup requires --yes");
    }
    if !options.dry_run && !options.yes {
        return CliResult::error("apply requires --dry-run or --yes");
    }
    let Some(shell) = options.shell else {
        return CliResult::error("apply requires --shell");
    };
    let config = match load_optional_config(options.config.as_deref(), &context.cwd) {
        Ok(config) => config,
        Err(message) => return CliResult::error(message),
    };
    let config_policy = config_policy(config.as_ref(), PathVariable::Path);
    let policy = path_policy(
        config_policy,
        &options.drop_entries,
        &options.presets,
        PathVariable::Path,
    );

    let home_dir = options
        .home
        .as_deref()
        .or_else(|| apply_home_dir(options.common.platform, context));
    let user_profile_dir = options
        .home
        .as_deref()
        .or_else(|| apply_user_profile_dir(options.common.platform, context));
    let planned = match plan_apply_operation(&ApplyPlanOptions {
        path_value: &context.path_value,
        platform_mode: options.common.platform,
        pathext: context.pathext.as_deref(),
        shell,
        home_dir,
        user_profile_dir,
        profile: options.profile.as_deref(),
        policy,
    }) {
        Ok(planned) => planned,
        Err(message) => return CliResult::error(message),
    };
    if options.common.json {
        if options.dry_run {
            return json_result(apply_plan_to_json(&planned.plan));
        }
        let outcome = match write_apply_plan(planned, !options.no_backup) {
            Ok(outcome) => outcome,
            Err(message) => return CliResult::error(message),
        };
        return json_result(apply_outcome_to_json(&outcome));
    }
    if options.dry_run {
        return CliResult::success(format_apply_plan(&planned.plan));
    }
    match write_apply_plan(planned, !options.no_backup) {
        Ok(outcome) => CliResult::success(format_apply_outcome(&outcome)),
        Err(message) => CliResult::error(message),
    }
}

fn variable_value(context: &CommandContext, variable: PathVariable) -> &str {
    match variable {
        PathVariable::Path => &context.path_value,
        PathVariable::Manpath => &context.manpath_value,
    }
}

fn path_policy(
    config_policy: Option<&ConfigPolicy>,
    drop_entries: &[String],
    presets: &[PresetKind],
    variable: PathVariable,
) -> PathPolicy {
    let drop_entries = merge_drop_entries(config_policy, drop_entries);
    let presets = merge_presets(config_policy, presets);
    PathPolicy::new(&drop_entries, &presets, variable)
}

fn config_policy(config: Option<&LoadedConfig>, variable: PathVariable) -> Option<&ConfigPolicy> {
    config.map(|config| config.policy_for(variable))
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

fn run_config(options: ConfigOptions, context: &CommandContext) -> CliResult {
    match options.command {
        ConfigCommand::Check(options) => run_config_check(options, context),
        ConfigCommand::Print(options) => run_config_print(options, context),
    }
}

fn run_config_check(options: ConfigCheckOptions, context: &CommandContext) -> CliResult {
    match load_required_config(&options.config, &context.cwd) {
        Ok(config) => CliResult::success(format_config_check(&config)),
        Err(message) => CliResult::error(message),
    }
}

fn run_config_print(options: ConfigPrintOptions, context: &CommandContext) -> CliResult {
    let config = match load_required_config(&options.config, &context.cwd) {
        Ok(config) => config,
        Err(message) => return CliResult::error(message),
    };
    if options.json {
        return json_result(config_to_json(&config));
    }
    CliResult::success(format_config_print(&config))
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
