# patholog

Diagnose and fix PATH problems across macOS, Linux, and Windows.

[![Test](https://github.com/techouse/patholog/actions/workflows/test.yml/badge.svg)](https://github.com/techouse/patholog/actions/workflows/test.yml)

`patholog` explains why a command resolves to a particular executable, shows competing matches, diagnoses common PATH problems, and prints a cleaned PATH proposal. The v0.1 Rust crate is a strict port of the Python rapid prototype, with the public surface intentionally limited to read-only diagnostics and `clean --stdout`.

## Quick Examples

```sh
patholog doctor
patholog why python
patholog conflicts cargo
patholog clean --stdout
```

## Why It Exists

PATH problems are usually invisible until the wrong tool runs. `patholog` is built for cases like:

- `python`, `node`, or `cargo` resolving to an unexpected executable.
- duplicate PATH entries hiding the real search order.
- missing directories left behind by old installers.
- Cargo, Homebrew, pyenv, nvm, or system directories appearing in surprising order.
- shell config drift between machines.

The goal is to explain before changing anything.

## Command Surface

```sh
patholog print [--json] [--platform auto|posix|windows]
patholog doctor [--json] [--platform auto|posix|windows] [--fail-on=missing,duplicate,...]
patholog why <command> [--json] [--platform auto|posix|windows]
patholog conflicts <command> [--json] [--platform auto|posix|windows]
patholog clean --stdout [--platform auto|posix|windows]
```

`patholog` does not mutate shell profiles, environment variables, or files in v0.1.

## Output Modes

Human output is the default. JSON output is available for `print`, `doctor`, `why`, and `conflicts`:

```sh
patholog doctor --json
patholog why python --json
```

`clean --stdout` prints a raw PATH string suitable for review or manual export.

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

PATH values are handled as UTF-8 strings for v0.1 parity. Symlinks, inode identity, and canonical-file identity are not analyzed.

## Safety

All v0.1 commands are read-only except for writing output to stdout or stderr. `patholog` does not edit:

- `~/.zshrc`
- `~/.bashrc`
- `~/.profile`
- PowerShell profiles
- system environment configuration

`clean --stdout` only proposes a cleaned PATH string. Applying it is a manual step.

## Development

```sh
cargo fmt --check
cargo clippy --all-targets --all-features --locked -- -D warnings
cargo test --all-targets --all-features --locked
make pre-release
```

Golden parity fixtures are vendored in `tests/fixtures/golden`.
