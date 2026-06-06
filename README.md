# patholog

Diagnose and fix PATH problems across macOS, Linux, and Windows.

[![Test](https://github.com/techouse/patholog/actions/workflows/test.yml/badge.svg)](https://github.com/techouse/patholog/actions/workflows/test.yml)

`patholog` explains why a command resolves to a particular executable, shows competing matches, diagnoses common PATH problems, scans shell startup files read-only, prints cleaned PATH proposals, and applies tightly scoped shell profile repairs through a managed block.

## Installation

Install the CLI from crates.io:

```sh
cargo install patholog
```

Prebuilt release archives are available from the
[GitHub releases page](https://github.com/techouse/patholog/releases). Archives include the `patholog` binary,
shell completions, the README, the project license, and third-party license notices.

## Quick Examples

```sh
patholog doctor
patholog health --json
patholog doctor --command python
patholog why python
patholog why-not poetry
patholog conflicts cargo
patholog scan
patholog clean --stdout
patholog clean --stdout --drop /sw/bin
patholog clean --export --var manpath --shell zsh
patholog clean --export --shell zsh
patholog apply --dry-run --shell zsh
patholog apply --shell zsh --yes
patholog config check --config patholog.toml
patholog completions zsh
```

## Why It Exists

PATH problems are usually invisible until the wrong tool runs. `patholog` is built for cases like:

- `python`, `node`, or `cargo` resolving to an unexpected executable.
- duplicate PATH entries hiding the real search order.
- missing directories left behind by old installers.
- unreadable PATH entries or shadowed command candidates.
- Cargo, Homebrew, pyenv, nvm, or system directories appearing in surprising order.
- shell config drift between machines.

The goal is to explain before changing anything.

## Command Surface

```sh
patholog print [--json] [--platform auto|posix|windows] [--var path|manpath]
patholog doctor [--json] [--platform auto|posix|windows] [--var path|manpath] [--drop <entry>] [--preset homebrew|cargo|pyenv|fink] [--fail-on=missing,duplicate,...] [--command <command>] [--config <file|auto>]
patholog health [--json] [--platform auto|posix|windows] [--var path|manpath] [--drop <entry>] [--preset homebrew|cargo|pyenv|fink] [--config <file|auto>]
patholog why <command> [--json] [--platform auto|posix|windows]
patholog why-not <command> [--json] [--platform auto|posix|windows]
patholog conflicts <command> [--json] [--platform auto|posix|windows]
patholog scan [--json] [--platform auto|posix|windows] [--home <dir>]
patholog clean --stdout [--platform auto|posix|windows] [--var path|manpath] [--drop <entry>] [--preset homebrew|cargo|pyenv|fink] [--config <file|auto>]
patholog clean --export --shell zsh|bash|fish|pwsh [--platform auto|posix|windows] [--var path|manpath] [--drop <entry>] [--preset homebrew|cargo|pyenv|fink] [--config <file|auto>]
patholog apply --dry-run --shell zsh|bash|fish|pwsh [--json] [--platform auto|posix|windows] [--home <dir>] [--profile <file>] [--drop <entry>] [--preset homebrew|cargo|pyenv|fink] [--config <file|auto>]
patholog apply --shell zsh|bash|fish|pwsh --yes [--no-backup] [--json] [--platform auto|posix|windows] [--home <dir>] [--profile <file>] [--drop <entry>] [--preset homebrew|cargo|pyenv|fink] [--config <file|auto>]
patholog config check --config <file|auto>
patholog config print --config <file|auto> [--json]
patholog completions zsh|bash|fish|pwsh
```

`patholog` mutates files only for `apply --yes`. Other commands do not edit shell profiles, environment variables, or files.

## Read-Only Diagnostics

`doctor` reports duplicate, missing, non-directory, unreadable, empty, explicitly unwanted, and suspiciously ordered PATH entries. With `--command`, it also reports executable candidates shadowed by an earlier winner:

```sh
patholog doctor --command python
patholog doctor --drop /sw/bin --fail-on=unwanted
```

`health` summarizes those diagnostics as an advisory deterministic score, issue counts, worst severity, and the diagnostics used to compute the score:

```sh
patholog health --json
patholog health --var manpath
```

Example JSON excerpt:

```json
{
  "score": 85,
  "healthy": false,
  "worst_severity": "error",
  "counts": {
    "missing": 1
  }
}
```

Use `doctor --fail-on` when you need a CI gate; `health` always exits `0` when the summary is calculated successfully.

`--var manpath` switches `print`, `doctor`, `health`, and `clean` to `MANPATH`. Command resolution and profile planning stay PATH-only. `clean --var manpath` preserves empty MANPATH components because common man implementations use them to include the system default manpath.

`scan` reads known shell startup profiles under the home directory and reports likely PATH mutation lines. It does not source or edit those files:

```sh
patholog scan
patholog scan --home /tmp/example-home
```

## Output Modes

Human output is the default. JSON output is available for `print`, `doctor`, `health`, `why`, `why-not`, `conflicts`, `scan`, and `apply`:

```sh
patholog doctor --json
patholog health --json
patholog why python --json
patholog why-not poetry --json
patholog scan --json
```

`why-not` explains missing-command cases by combining exact PATH lookup, related executable hints, PATH health diagnostics, and safe advisory next checks. It is read-only and does not run package managers or edit shell configuration.

`clean --stdout` prints a raw PATH or MANPATH string suitable for review or manual export. `clean --export` prints a shell-ready assignment snippet for `zsh`, `bash`, `fish`, or `pwsh`:

```sh
patholog clean --export --shell zsh
patholog clean --export --var manpath --shell zsh
```

`completions` prints shell completion scripts to stdout:

```sh
patholog completions zsh
```

`apply --dry-run` plans a shell profile edit without writing files. `apply --yes` writes the same managed block, creates backups for existing profiles by default, and never rewrites arbitrary PATH lines outside the patholog block:

```sh
patholog apply --dry-run --shell zsh
patholog apply --dry-run --shell zsh --json
patholog apply --dry-run --shell zsh --drop /sw/bin
patholog apply --shell zsh --yes
patholog apply --shell zsh --yes --no-backup
```

`--preset fink` marks `/sw/bin` and `/sw/sbin` as unwanted for PATH, and `/sw/share/man` as unwanted for MANPATH. `homebrew`, `cargo`, and `pyenv` presets enable ecosystem policy checks without automatically reordering entries.

## Config Files

`--config <file>` lets `doctor`, `health`, `clean`, and `apply` load declarative policy defaults. CLI flags append after config values:

```toml
version = 1

[path]
drop = ["/sw/bin"]
preset = ["homebrew", "cargo"]
fail_on = ["duplicate", "unwanted"]

[manpath]
drop = ["/sw/share/man"]
preset = ["fink"]
fail_on = ["duplicate"]
```

Use `patholog config check --config patholog.toml` to validate a file, or `patholog config print --config patholog.toml --json` to inspect normalized policy. `--config auto` searches only the current working directory for `patholog.toml`, then `.patholog.toml`.

`health` uses config `drop` and `preset` policy for scoring, but `fail_on` remains specific to `doctor`.

## Exit Codes

- `0`: success
- `1`: usage or runtime error
- `2`: `doctor --fail-on` matched selected diagnostics
- `3`: command not found for `why`, `why-not`, or `conflicts`

## Platform Model

- `auto` uses host platform rules.
- `posix` uses `:` separators and case-sensitive comparison keys.
- `windows` uses `;` separators, case-insensitive comparison keys, and PATHEXT modeling.

Windows command resolution differs from POSIX lookup because file extensions and `PATHEXT` affect which executable wins. `patholog --platform windows` models that behavior for tests and cross-platform inspection.

PATH values are handled as UTF-8 strings to preserve v0.1 parity. Symlinks, inode identity, and canonical-file identity are not analyzed.

## Safety

All commands are read-only except `apply --yes`, which writes only the patholog-managed block in the selected shell
profile. When the selected target is `~/.zshrc`, `~/.bashrc`, `~/.profile`, or a PowerShell profile, `patholog` may
create, append, or replace that managed block, but it does not rewrite arbitrary profile content outside the block or
edit system environment configuration.

`clean --stdout`, `clean --export`, `apply --dry-run`, and `completions` only print generated text. Mutating `apply` requires `--yes` and backs up existing profiles unless `--no-backup` is passed.

## Development

```sh
make ci
make install-smoke
make v1-contract-check
make pre-release
```

Use `make ci` for normal local checks, `make install-smoke` to test the packaged crate in a temporary install root,
`make v1-contract-check` for the v1 contract audit, and `make pre-release` as the full gate before tagging a
release.

Golden parity fixtures are vendored in `tests/fixtures/golden`.
