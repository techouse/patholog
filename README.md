# patholog

Diagnose and fix PATH problems across macOS, Linux, and Windows.

[![Test](https://github.com/techouse/patholog/actions/workflows/test.yml/badge.svg)](https://github.com/techouse/patholog/actions/workflows/test.yml)

`patholog` explains why a command resolves to a particular executable, shows competing matches, diagnoses common PATH problems, scans shell startup files read-only, prints cleaned PATH proposals, and plans safe profile repairs without mutating shell configuration.

## Quick Examples

```sh
patholog doctor
patholog doctor --command python
patholog why python
patholog conflicts cargo
patholog scan
patholog clean --stdout
patholog clean --stdout --drop /sw/bin
patholog clean --export --var manpath --shell zsh
patholog clean --export --shell zsh
patholog apply --dry-run --shell zsh
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
patholog doctor [--json] [--platform auto|posix|windows] [--var path|manpath] [--drop <entry>] [--preset homebrew|cargo|pyenv|fink] [--fail-on=missing,duplicate,...] [--command <command>]
patholog why <command> [--json] [--platform auto|posix|windows]
patholog conflicts <command> [--json] [--platform auto|posix|windows]
patholog scan [--json] [--platform auto|posix|windows] [--home <dir>]
patholog clean --stdout [--platform auto|posix|windows] [--var path|manpath] [--drop <entry>] [--preset homebrew|cargo|pyenv|fink]
patholog clean --export --shell zsh|bash|fish|pwsh [--platform auto|posix|windows] [--var path|manpath] [--drop <entry>] [--preset homebrew|cargo|pyenv|fink]
patholog apply --dry-run --shell zsh|bash|fish|pwsh [--json] [--platform auto|posix|windows] [--home <dir>] [--profile <file>] [--drop <entry>] [--preset homebrew|cargo|pyenv|fink]
patholog completions zsh|bash|fish|pwsh
```

`patholog` does not mutate shell profiles, environment variables, or files.

## Read-Only Diagnostics

`doctor` reports duplicate, missing, non-directory, unreadable, empty, explicitly unwanted, and suspiciously ordered PATH entries. With `--command`, it also reports executable candidates shadowed by an earlier winner:

```sh
patholog doctor --command python
patholog doctor --drop /sw/bin --fail-on=unwanted
```

`--var manpath` switches `print`, `doctor`, and `clean` to `MANPATH`. Command resolution and profile planning stay PATH-only.

`scan` reads known shell startup profiles under the home directory and reports likely PATH mutation lines. It does not source or edit those files:

```sh
patholog scan
patholog scan --home /tmp/example-home
```

## Output Modes

Human output is the default. JSON output is available for `print`, `doctor`, `why`, `conflicts`, `scan`, and `apply --dry-run`:

```sh
patholog doctor --json
patholog why python --json
patholog scan --json
```

`clean --stdout` prints a raw PATH or MANPATH string suitable for review or manual export. `clean --export` prints a shell-ready assignment snippet for `zsh`, `bash`, `fish`, or `pwsh`:

```sh
patholog clean --export --shell zsh
patholog clean --export --var manpath --shell zsh
```

`completions` prints shell completion scripts to stdout:

```sh
patholog completions zsh
```

`apply --dry-run` plans a future shell profile edit without writing files:

```sh
patholog apply --dry-run --shell zsh
patholog apply --dry-run --shell zsh --json
patholog apply --dry-run --shell zsh --drop /sw/bin
```

`--preset fink` marks `/sw/bin`, `/sw/sbin`, and `/sw/share/man` as unwanted. `homebrew`, `cargo`, and `pyenv` presets enable ecosystem policy checks without automatically reordering entries.

## Exit Codes

- `0`: success
- `1`: usage or runtime error
- `2`: `doctor --fail-on` matched selected diagnostics
- `3`: command not found for `why` or `conflicts`

## Platform Model

- `auto` uses host platform rules.
- `posix` uses `:` separators and case-sensitive comparison keys.
- `windows` uses `;` separators, case-insensitive comparison keys, and PATHEXT modeling.

Windows command resolution differs from POSIX lookup because file extensions and `PATHEXT` affect which executable wins. `patholog --platform windows` models that behavior for tests and cross-platform inspection.

PATH values are handled as UTF-8 strings to preserve v0.1 parity. Symlinks, inode identity, and canonical-file identity are not analyzed.

## Safety

All commands are read-only except for writing output to stdout or stderr. `patholog` does not edit:

- `~/.zshrc`
- `~/.bashrc`
- `~/.profile`
- PowerShell profiles
- system environment configuration

`clean --stdout`, `clean --export`, `apply --dry-run`, and `completions` only print generated text. Applying or installing that output is a manual step.

## Development

```sh
cargo fmt --check
cargo clippy --all-targets --all-features --locked -- -D warnings
cargo test --all-targets --all-features --locked
make pre-release
```

Golden parity fixtures are vendored in `tests/fixtures/golden`.
