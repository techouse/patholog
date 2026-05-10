use std::fs::{self, File, OpenOptions, Permissions};
use std::io::Write;
use std::ops::Range;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::clean::{clean_with_policy, format_clean_export};
use crate::model::{ApplyAction, ApplyOutcome, ApplyPlan, PathVariable, PlatformMode, ShellKind};
use crate::platform::resolve_platform_rules;
use crate::policy::PathPolicy;

pub(crate) const START_MARKER: &str = "# >>> patholog PATH >>>";
pub(crate) const END_MARKER: &str = "# <<< patholog PATH <<<";

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ApplyPlanOptions<'a> {
    pub(crate) path_value: &'a str,
    pub(crate) platform_mode: PlatformMode,
    pub(crate) pathext: Option<&'a str>,
    pub(crate) shell: ShellKind,
    pub(crate) home_dir: Option<&'a Path>,
    pub(crate) user_profile_dir: Option<&'a Path>,
    pub(crate) profile: Option<&'a Path>,
    pub(crate) policy: PathPolicy,
}

pub(crate) fn plan_apply(options: &ApplyPlanOptions<'_>) -> Result<ApplyPlan, String> {
    let profile_path = target_profile(options)?;
    reject_symlink_profile(&profile_path)?;
    let cleaned = clean_with_policy(
        options.path_value,
        options.platform_mode,
        options.pathext,
        PathVariable::Path,
        &options.policy,
    );
    let cleaned_path = cleaned.raw_path();
    let planned_block = managed_block(&format_clean_export(
        &cleaned,
        options.shell,
        PathVariable::Path,
    ));

    match fs::metadata(&profile_path) {
        Ok(metadata) if !metadata.is_file() => Err(format!(
            "apply target profile is not a file: {}",
            profile_path.display()
        )),
        Ok(_metadata) => {
            plan_existing_profile(options.shell, &profile_path, planned_block, cleaned_path)
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(ApplyPlan {
            shell: options.shell,
            profile_path: profile_path.display().to_string(),
            action: ApplyAction::CreateProfile,
            existing_block: None,
            planned_block,
            cleaned_path,
            would_write: false,
        }),
        Err(error) => Err(format!(
            "apply target profile is not readable: {} ({error})",
            profile_path.display()
        )),
    }
}

pub(crate) fn write_apply_plan(
    mut plan: ApplyPlan,
    create_backup: bool,
) -> Result<ApplyOutcome, String> {
    let profile_path = PathBuf::from(&plan.profile_path);
    reject_symlink_profile(&profile_path)?;
    let existing_permissions = match plan.action {
        ApplyAction::CreateProfile => None,
        ApplyAction::AppendBlock | ApplyAction::ReplaceBlock => {
            Some(profile_permissions(&profile_path)?)
        }
    };
    let content = planned_profile_content(&profile_path, &plan)?;
    let backup_path = match (create_backup, plan.action) {
        (true, ApplyAction::AppendBlock | ApplyAction::ReplaceBlock) => {
            Some(create_profile_backup(&profile_path)?)
        }
        _ => None,
    };

    let write_mode = match plan.action {
        ApplyAction::CreateProfile => WriteMode::CreateNew,
        ApplyAction::AppendBlock | ApplyAction::ReplaceBlock => WriteMode::ReplaceExisting,
    };
    write_profile_atomically(&profile_path, &content, existing_permissions, write_mode)?;
    plan.would_write = true;

    let backup_created = backup_path.is_some();
    Ok(ApplyOutcome {
        plan,
        wrote: true,
        backup_path: backup_path.map(|path| path.display().to_string()),
        backup_created,
    })
}

fn target_profile(options: &ApplyPlanOptions<'_>) -> Result<PathBuf, String> {
    if let Some(profile) = options.profile {
        return Ok(profile.to_path_buf());
    }

    let Some(home) = default_home(options) else {
        return Err(missing_home_message(options).to_owned());
    };

    Ok(match options.shell {
        ShellKind::Bash => home.join(".bashrc"),
        ShellKind::Fish => home.join(".config/fish/config.fish"),
        ShellKind::Pwsh => pwsh_profile(home, options.platform_mode),
        ShellKind::Zsh => home.join(".zshrc"),
    })
}

fn default_home<'a>(options: &'a ApplyPlanOptions<'_>) -> Option<&'a Path> {
    match (
        options.shell,
        resolve_platform_rules(options.platform_mode, None).mode,
    ) {
        (ShellKind::Pwsh, PlatformMode::Windows) => options.user_profile_dir,
        _ => options.home_dir,
    }
}

fn missing_home_message(options: &ApplyPlanOptions<'_>) -> &'static str {
    match (
        options.shell,
        resolve_platform_rules(options.platform_mode, None).mode,
    ) {
        (ShellKind::Pwsh, PlatformMode::Windows) => {
            "apply requires a home directory; set USERPROFILE, pass --home, or pass --profile"
        }
        _ => "apply requires a home directory; set HOME, pass --home, or pass --profile",
    }
}

fn pwsh_profile(home: &Path, platform_mode: PlatformMode) -> PathBuf {
    match resolve_platform_rules(platform_mode, None).mode {
        PlatformMode::Windows => home.join("Documents/PowerShell/Microsoft.PowerShell_profile.ps1"),
        PlatformMode::Auto | PlatformMode::Posix => {
            home.join(".config/powershell/Microsoft.PowerShell_profile.ps1")
        }
    }
}

fn plan_existing_profile(
    shell: ShellKind,
    profile_path: &Path,
    planned_block: String,
    cleaned_path: String,
) -> Result<ApplyPlan, String> {
    let content = fs::read_to_string(profile_path).map_err(|error| {
        format!(
            "apply target profile is not readable: {} ({error})",
            profile_path.display()
        )
    })?;
    let existing_block = existing_managed_block(&content)?;
    let action = if existing_block.is_some() {
        ApplyAction::ReplaceBlock
    } else {
        ApplyAction::AppendBlock
    };

    Ok(ApplyPlan {
        shell,
        profile_path: profile_path.display().to_string(),
        action,
        existing_block,
        planned_block,
        cleaned_path,
        would_write: false,
    })
}

pub(crate) fn managed_block(snippet: &str) -> String {
    format!("{START_MARKER}\n{snippet}\n{END_MARKER}")
}

pub(crate) fn existing_managed_block(content: &str) -> Result<Option<String>, String> {
    existing_managed_block_span(content).map(|span| span.map(|span| content[span].to_owned()))
}

pub(crate) fn existing_managed_block_span(content: &str) -> Result<Option<Range<usize>>, String> {
    let starts = marker_offsets(content, START_MARKER);
    let ends = marker_offsets(content, END_MARKER);
    match (starts.as_slice(), ends.as_slice()) {
        ([], []) => Ok(None),
        ([start], [end]) if start < end => Ok(Some(existing_block_span(content, *start, *end))),
        ([start], [end]) if start >= end => {
            Err("apply target profile contains a malformed patholog managed block".to_owned())
        }
        _ => Err(
            "apply target profile contains duplicate or malformed patholog managed blocks"
                .to_owned(),
        ),
    }
}

pub(crate) fn appended_profile_content(existing: &str, planned_block: &str) -> String {
    let line_ending = line_ending_for(existing);
    let planned_block = planned_block_with_line_ending(planned_block, line_ending);
    let mut content =
        String::with_capacity(existing.len() + planned_block.len() + 2 * line_ending.len());
    content.push_str(existing);
    if !content.is_empty() {
        if !content.ends_with(line_ending) {
            content.push_str(line_ending);
        }
        content.push_str(line_ending);
    }
    content.push_str(&planned_block);
    content.push_str(line_ending);
    content
}

fn line_ending_for(content: &str) -> &'static str {
    if content.contains("\r\n") {
        "\r\n"
    } else {
        "\n"
    }
}

fn planned_block_with_line_ending(block: &str, line_ending: &str) -> String {
    if line_ending == "\n" {
        return block.to_owned();
    }
    block.replace('\n', line_ending)
}

pub(crate) fn replaced_profile_content(
    existing: &str,
    planned_block: &str,
) -> Result<String, String> {
    let Some(span) = existing_managed_block_span(existing)? else {
        return Err("apply target profile changed before write; rerun apply".to_owned());
    };
    let line_ending = line_ending_for(existing);
    let planned_block = planned_block_with_line_ending(planned_block, line_ending);
    let suffix = &existing[span.end..];
    let mut content = String::with_capacity(existing.len() + planned_block.len());
    content.push_str(&existing[..span.start]);
    content.push_str(&planned_block);
    if line_ending == "\r\n" && suffix.starts_with('\n') {
        content.push('\r');
    }
    content.push_str(suffix);
    Ok(content)
}

fn backup_path_candidate(profile_path: &Path, seconds: u64, suffix: usize) -> PathBuf {
    let parent = profile_parent(profile_path);
    let file_name = profile_path
        .file_name()
        .map(|name| name.to_string_lossy())
        .unwrap_or_else(|| "profile".into());
    let base_name = format!("{file_name}.patholog-backup.{seconds}");
    if suffix == 0 {
        parent.join(base_name)
    } else {
        parent.join(format!("{base_name}.{suffix}"))
    }
}

fn marker_offsets(content: &str, marker: &str) -> Vec<usize> {
    let mut offsets = Vec::new();
    let mut offset = 0;
    for segment in content.split_inclusive('\n') {
        let without_newline = segment.strip_suffix('\n').unwrap_or(segment);
        let line = without_newline
            .strip_suffix('\r')
            .unwrap_or(without_newline);
        if line == marker {
            offsets.push(offset);
        }
        offset += segment.len();
    }
    offsets
}

fn existing_block_span(content: &str, start: usize, end: usize) -> Range<usize> {
    let after_end_marker = end + END_MARKER.len();
    let block_end = content[after_end_marker..]
        .find('\n')
        .map_or(after_end_marker, |newline| after_end_marker + newline);
    start..block_end
}

fn planned_profile_content(profile_path: &Path, plan: &ApplyPlan) -> Result<String, String> {
    match plan.action {
        ApplyAction::CreateProfile => {
            if profile_path.exists() {
                return Err("apply target profile changed before write; rerun apply".to_owned());
            }
            Ok(format!("{}\n", plan.planned_block))
        }
        ApplyAction::AppendBlock => {
            let existing = read_profile_content(profile_path)?;
            if existing_managed_block(&existing)?.is_some() {
                return Err("apply target profile changed before write; rerun apply".to_owned());
            }
            Ok(appended_profile_content(&existing, &plan.planned_block))
        }
        ApplyAction::ReplaceBlock => {
            let existing = read_profile_content(profile_path)?;
            if existing_managed_block(&existing)? != plan.existing_block {
                return Err("apply target profile changed before write; rerun apply".to_owned());
            }
            replaced_profile_content(&existing, &plan.planned_block)
        }
    }
}

fn read_profile_content(profile_path: &Path) -> Result<String, String> {
    fs::read_to_string(profile_path).map_err(|error| {
        format!(
            "apply target profile is not readable: {} ({error})",
            profile_path.display()
        )
    })
}

fn profile_permissions(profile_path: &Path) -> Result<Permissions, String> {
    fs::metadata(profile_path)
        .map(|metadata| metadata.permissions())
        .map_err(|error| {
            format!(
                "apply target profile is not readable: {} ({error})",
                profile_path.display()
            )
        })
}

fn create_profile_backup(profile_path: &Path) -> Result<PathBuf, String> {
    create_profile_backup_for_seconds(profile_path, current_unix_seconds())
}

fn create_profile_backup_for_seconds(profile_path: &Path, seconds: u64) -> Result<PathBuf, String> {
    let mut suffix = 0;
    loop {
        let backup_path = backup_path_candidate(profile_path, seconds, suffix);
        match copy_profile_backup(profile_path, &backup_path) {
            Ok(()) => return Ok(backup_path),
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {
                suffix += 1;
            }
            Err(error) => {
                return Err(format!(
                    "apply backup failed for {}: {} ({error})",
                    profile_path.display(),
                    backup_path.display()
                ));
            }
        }
    }
}

fn copy_profile_backup(profile_path: &Path, backup_path: &Path) -> std::io::Result<()> {
    let mut source = File::open(profile_path)?;
    let source_permissions = source.metadata()?.permissions();
    let mut options = OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::{OpenOptionsExt, PermissionsExt};

        options.mode(source_permissions.mode());
    }
    let mut destination = options.open(backup_path)?;
    #[cfg(unix)]
    destination.set_permissions(source_permissions)?;
    std::io::copy(&mut source, &mut destination)?;
    destination.flush()?;
    #[cfg(not(unix))]
    destination.set_permissions(source_permissions)?;
    Ok(())
}

fn write_profile_atomically(
    profile_path: &Path,
    content: &str,
    permissions: Option<Permissions>,
    mode: WriteMode,
) -> Result<(), String> {
    let parent = profile_parent(profile_path);
    fs::create_dir_all(parent).map_err(|error| {
        format!(
            "apply target profile parent is not writable: {} ({error})",
            parent.display()
        )
    })?;

    let mut temp_file = tempfile::Builder::new()
        .prefix(".patholog.")
        .tempfile_in(parent)
        .map_err(|error| {
            format!(
                "apply could not create temporary file in {} ({error})",
                parent.display()
            )
        })?;
    temp_file
        .write_all(content.as_bytes())
        .map_err(|error| format!("apply could not write temporary profile ({error})"))?;
    temp_file
        .flush()
        .map_err(|error| format!("apply could not flush temporary profile ({error})"))?;
    if let Some(permissions) = permissions {
        temp_file
            .as_file()
            .set_permissions(permissions)
            .map_err(|error| {
                format!("apply could not set temporary profile permissions ({error})")
            })?;
    }
    if mode == WriteMode::ReplaceExisting {
        reject_symlink_profile(profile_path)?;
    }
    match mode {
        WriteMode::CreateNew => temp_file.persist_noclobber(profile_path),
        WriteMode::ReplaceExisting => temp_file.persist(profile_path),
    }
    .map_err(|error| {
        format!(
            "apply could not write profile: {} ({})",
            profile_path.display(),
            error.error
        )
    })?;
    Ok(())
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum WriteMode {
    CreateNew,
    ReplaceExisting,
}

fn reject_symlink_profile(profile_path: &Path) -> Result<(), String> {
    match fs::symlink_metadata(profile_path) {
        Ok(metadata) if metadata.file_type().is_symlink() => Err(format!(
            "apply target profile is a symlink; use --profile with the resolved target path: {}",
            profile_path.display()
        )),
        Ok(_metadata) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(format!(
            "apply target profile is not readable: {} ({error})",
            profile_path.display()
        )),
    }
}

fn current_unix_seconds() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |duration| duration.as_secs())
}

fn profile_parent(profile_path: &Path) -> &Path {
    profile_path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."))
}

#[cfg(test)]
#[path = "apply/tests.rs"]
mod tests;
