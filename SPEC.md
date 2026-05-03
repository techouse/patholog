# patholog — SPECIFICATION (Rust CLI)

## 1. Purpose

`patholog` is a CLI tool for **diagnosing, explaining, and safely repairing PATH issues** across macOS, Linux, and Windows.

It is intended for developers and operators who need to answer questions like:

* Why is the wrong binary running?
* Why is this tool not found?
* Which PATH entries are dead or duplicated?
* Why does one machine behave differently from another?

`patholog` should be **read-first and fix-second**. The initial release must prioritise inspection and explanation over automatic mutation of shell config files.

---

## 1.1 Python Parity Milestone (v0.1)

The first Rust milestone is a strict port of the Python rapid prototype, not a product superset.

Authoritative parity sources:

* behaviour contract: `/Users/klemen/Work/patholog.py/BEHAVIOUR.md`
* porting notes: `/Users/klemen/Work/patholog.py/PORTING_TO_RUST.md`
* human and JSON golden fixtures: `tests/fixtures/golden`, vendored from `/Users/klemen/Work/patholog.py/tests/fixtures/golden`

The Rust implementation must pass this parity contract before adding Rust-only features.

v0.1 parity commands:

```bash
patholog print [--json] [--platform auto|posix|windows]
patholog doctor [--json] [--platform auto|posix|windows] [--fail-on=missing,duplicate,...]
patholog why <command> [--json] [--platform auto|posix|windows]
patholog conflicts <command> [--json] [--platform auto|posix|windows]
patholog clean --stdout [--platform auto|posix|windows]
```

The broader product ideas in this document remain valid, but these are explicitly post-parity unless deliberately
promoted later:

* `clean --export`
* inode or canonical-file duplicate analysis
* shell profile mutation or `apply`

---

## 1.2 Read-Only Diagnostics Milestone (v0.2)

v0.2 expands beyond Python parity while preserving the read-first safety model. It should add diagnostics that inspect
current state and startup configuration without mutating files or environment variables.

v0.2 read-only commands and flags:

```bash
patholog doctor [--json] [--platform auto|posix|windows] [--fail-on=missing,duplicate,...]
patholog doctor --command <command> [--json] [--platform auto|posix|windows]
patholog scan [--json] [--platform auto|posix|windows] [--home <dir>]
```

v0.2 read-only diagnostics:

* unreadable PATH directories
* command candidates shadowed by earlier PATH entries via `doctor --command`
* shell startup profile scanning for likely PATH mutations via `scan`

Still post-v0.2 unless deliberately promoted:

* automatic shell profile editing
* `apply`
* inode or canonical-file duplicate analysis
* long-running `watch`
* package-manager install hints

---

## 1.3 Safe Repair Output Milestone (v0.3)

v0.3 adds shell-ready repair output while preserving the read-only safety model. It may generate shell snippets and
completion scripts on stdout, but it must not install, source, or edit anything.

v0.3 safe-output commands and flags:

```bash
patholog clean --export --shell zsh|bash|fish|pwsh [--platform auto|posix|windows]
patholog completions zsh|bash|fish|pwsh
```

v0.3 still defers:

* automatic shell profile editing
* `apply`
* inode or canonical-file duplicate analysis
* long-running `watch`
* package-manager install hints

---

## 2. Product Goals

### Primary goals

* Explain command resolution clearly
* Detect common PATH problems
* Produce deterministic, scriptable output
* Work cross-platform
* Be safe by default
* Be useful both interactively and in CI

### Non-goals

* Not a shell
* Not a PATH generator from scratch
* Not a dotfile manager
* Not a package manager
* Not a general filesystem path manipulation library

---

## 3. Core Use Cases

### 3.1 Diagnose PATH health

```bash
patholog doctor
```

Should report:

* duplicate PATH entries
* non-existent directories
* empty entries
* suspicious ordering
* unreadable directories where relevant

---

### 3.2 Explain why a command resolves the way it does

```bash
patholog why python
patholog why cargo
patholog why node
```

Should explain:

* which executable wins
* where alternatives exist
* why the winning one was selected
* relevant PATH ordering

---

### 3.3 Show all candidates for a command

```bash
patholog conflicts python
patholog conflicts git
```

Should list all matching executables found in PATH order.

---

### 3.4 Produce a cleaned PATH

```bash
patholog clean --stdout
```

Should emit a deduplicated, sanitised PATH string without mutating user files.

---

### 3.5 Diagnose command-specific shadowing

```bash
patholog doctor --command python
```

Should report command candidates that exist but lose to earlier PATH entries.

---

### 3.6 Scan shell startup files read-only

```bash
patholog scan
```

Should inspect known shell startup profiles under the user's home directory and report likely PATH mutation lines without
editing those files.

---

## 4. Initial CLI Surface (v0.1)

### Commands

```bash
patholog doctor
patholog why <command>
patholog conflicts <command>
patholog clean --stdout
patholog print
```

### Optional flags for v0.1

```bash
patholog doctor --json
patholog doctor --fail-on=missing,duplicate
patholog print --json
patholog why <command> --json
patholog conflicts <command> --json
```

---

## 4.1 v0.2 Read-Only CLI Additions

```bash
patholog doctor --command <command>
patholog doctor --fail-on=unreadable,shadowed_command
patholog scan [--json] [--platform auto|posix|windows] [--home <dir>]
```

`scan --home` exists for deterministic inspection of an alternate home directory and must remain read-only.

---

## 4.2 v0.3 Safe Repair Output Additions

```bash
patholog clean --export --shell zsh|bash|fish|pwsh [--platform auto|posix|windows]
patholog completions zsh|bash|fish|pwsh
```

`clean --export` and `completions` write generated text to stdout only.

---

## 5. Command Semantics

## 5.1 `print`

```bash
patholog print
```

Outputs the current PATH as parsed by `patholog`.

Default output:

* one entry per line
* in effective search order

Example:

```text
1  /opt/homebrew/bin
2  /usr/local/bin
3  /usr/bin
4  /bin
5  /Users/k/.cargo/bin
```

### Behaviour

* preserve effective order
* show empty entries explicitly if present
* expand nothing by default beyond what the current process already sees

---

## 5.2 `doctor`

```bash
patholog doctor
```

Analyses the current PATH and emits diagnostics.

### Diagnostic categories

* duplicate entries
* empty entries
* non-existent directories
* non-directory entries
* unreadable directories
* suspicious ordering heuristics
* shadowed command candidates when `--command` is provided

Broader platform-specific oddities remain post-v0.2 diagnostics unless deliberately promoted.

### Example output

```text
PATH entries: 12

Duplicates:
  /usr/local/bin (2x)

Missing directories:
  /Users/k/old/bin

Empty entries:
  entry 7

Ordering warnings:
  /usr/bin appears before /opt/homebrew/bin
  this may cause system binaries to shadow Homebrew binaries
```

### Exit codes

* `0` if no selected issues found
* `1` for general runtime error
* `2` if issues were found and `--fail-on` matched them

### `--fail-on`

Comma-separated categories, for example:

```bash
patholog doctor --fail-on=missing,duplicate
```

Useful in CI and shell startup checks.

---

### `--command`

```bash
patholog doctor --command python
```

Runs normal PATH diagnostics and adds command-focused shadowing diagnostics. A shadowed command diagnostic means an
executable candidate was found later in PATH but an earlier candidate wins.

---

## 5.3 `why <command>`

```bash
patholog why python
```

Explains command resolution.

### Output must include

* resolved path, if found
* all candidates in PATH order
* reason winner was selected

### Example

```text
Command: python

Resolved to:
  /usr/bin/python

Why:
  entry 1 (/usr/bin) appears before all other matching PATH entries.

Other matches:
  2  /opt/homebrew/bin/python
  3  /Users/k/.pyenv/shims/python
```

On POSIX, lookup is exact. `patholog why python` does not treat `python3` as a PATH match for `python`. If exact lookup
fails, related executable names such as `python3` may appear only in a clearly labelled advisory section and must not
change the exit code.

### If not found

```text
Command: poetry

Not found in PATH.

Searched directories:
  1  /opt/homebrew/bin
  2  /usr/local/bin
  3  /usr/bin
```

Exit code:

* `0` if found
* `3` if not found
* `1` on runtime error

---

## 5.4 `conflicts <command>`

```bash
patholog conflicts python
```

Lists all matching executables found for a command.

### Behaviour

* search in PATH order
* include only executable matches according to platform rules
* identify the winning candidate
* defer inode or canonical-file duplicate analysis until after parity

### Example

```text
1* /usr/bin/python
2  /opt/homebrew/bin/python
3  /Users/k/.pyenv/shims/python
```

Where `*` marks the active winner.

Exit code:

* `0` if at least one match
* `3` if none
* `1` on runtime error

---

## 5.5 `clean`

```bash
patholog clean --stdout
```

Produces a cleaned PATH representation.

### Default cleaning rules in v0.1

* remove empty entries
* remove later duplicates after comparison-key normalisation
* preserve original winning order for first occurrences
* keep missing and non-directory entries

### Important

`clean` in v0.1 must **not** edit shell rc files by default.

### Output modes

* `--stdout`

  * print raw PATH string suitable for export

* `--export --shell zsh|bash|fish|pwsh`

  * print a shell-ready PATH assignment snippet
  * require an explicit shell to keep output deterministic
  * do not edit shell profiles or completion directories

### Example

```bash
patholog clean --stdout
```

Output:

```text
/opt/homebrew/bin:/usr/local/bin:/usr/bin:/bin:/Users/k/.cargo/bin
```

```bash
patholog clean --export --shell zsh
```

Output:

```text
export PATH='/opt/homebrew/bin:/usr/local/bin:/usr/bin:/bin:/Users/k/.cargo/bin'
```

Exit code:

* `0` on success
* `1` on runtime error

---

## 5.6 `completions`

```bash
patholog completions zsh
```

Generates shell completion scripts to stdout for `zsh`, `bash`, `fish`, or `pwsh`.

### Behaviour

* write completion script text to stdout
* do not install generated files
* do not edit shell startup profiles

Exit code:

* `0` on success
* `1` on runtime or usage error

---

## 5.7 `scan`

```bash
patholog scan
```

Scans known shell startup profiles under the user's home directory and reports likely PATH mutations.

### Behaviour

* read files only
* do not source, execute, or edit shell profiles
* ignore missing profiles in human output
* report unreadable existing profiles
* support `--home <dir>` for inspecting an alternate home directory

### Profile families

POSIX mode considers zsh, bash, `.profile`, and PowerShell Core profile paths under `$HOME`.

Windows mode considers the common PowerShell profile paths under `%USERPROFILE%`.

### Exit code

* `0` on successful scan
* `1` if no home directory is available and `--home` was not provided, or on runtime error

---

## 6. v0.1 Safety Model

### Read-only by default

All commands in v0.1 through v0.3 must be read-only except for printing proposed cleaned output.

### No automatic mutation

Do not modify:

* `~/.zshrc`
* `~/.bashrc`
* `~/.profile`
* PowerShell profile
* system env config

### Rationale

PATH mutation is risky and shell-specific. Inspection must come first.

---

## 7. Platform Model

## 7.1 Supported platforms

* macOS
* Linux
* Windows

## 7.2 Separator handling

* POSIX: `:`
* Windows: `;`

Use platform-native parsing by default.

## 7.3 Executable resolution

### POSIX

A candidate is executable if:

* file exists
* not a directory
* execute bit or equivalent access is present as applicable

### Windows

Resolution must account for:

* file extensions
* `PATHEXT`
* executable candidates such as `.exe`, `.cmd`, `.bat`

This logic must be documented and tested.

## 7.4 Path normalisation

Normalisation should be conservative.

Suggested rules:

* trim nothing except what platform parsing naturally excludes
* preserve original textual entry for display where possible
* canonicalise only for comparison when necessary
* support case-insensitive comparison on Windows
* support literal preservation for output

---

## 8. Diagnostic Taxonomy

Use explicit internal issue kinds.

v0.1 parity issue-kind strings:

* `duplicate`
* `empty`
* `missing`
* `not_directory`
* `suspicious_order`

v0.2 read-only issue-kind strings:

* `unreadable`
* `shadowed_command`

Suggested Rust enum:

```rust
enum IssueKind {
    Duplicate,
    Empty,
    Missing,
    NotDirectory,
    Unreadable,
    SuspiciousOrder,
    ShadowedCommand,
}
```

Unreadable and shadowed-command diagnostics are post-parity and must not change v0.1 golden output unless those
conditions are present.

---

## 9. Output Modes

### Human mode

Default output:

* concise
* readable
* ordered
* grouped by issue kind where relevant

### Structured mode

Support JSON for:

* `doctor`
* `print`
* `conflicts`
* `why`

This is important for CI and editor tooling.

Example:

```bash
patholog doctor --json
```

JSON should be stable enough for automation.

For v0.1 parity, JSON output must match the Python golden fixtures exactly: stable field names, sorted object keys,
2-space indentation, and a trailing newline.

---

## 10. Internal Architecture

Suggested crate/module layout:

```text
src/
  main.rs
  cli.rs
  path_env.rs
  normalize.rs
  doctor.rs
  resolve.rs
  clean.rs
  output/
    human.rs
    json.rs
  platform/
    mod.rs
    unix.rs
    windows.rs
  model.rs
  exit_codes.rs
```

### Responsibilities

#### `path_env.rs`

* read and parse PATH from environment
* split into ordered entries

#### `normalize.rs`

* comparison keys
* dedupe helpers
* case sensitivity rules

#### `doctor.rs`

* diagnostics engine
* issue detection

#### `resolve.rs`

* executable lookup
* why/conflicts logic

#### `clean.rs`

* cleaned PATH output generation

#### `platform/*`

* platform-specific executable and path handling

#### `model.rs`

Core data structures, such as:

```rust
struct PathEntry {
    index: usize,
    raw: String,
    display: String,
    comparison_key: String,
    exists: bool,
    is_dir: bool,
    is_empty: bool,
}

struct ResolutionCandidate {
    entry_index: usize,
    directory: String,
    path: String,
    wins: bool,
}

struct Diagnostic {
    kind: IssueKind,
    message: String,
    entry_index: Option<usize>,
    entry_value: Option<String>,
    related_indexes: Vec<usize>,
}
```

These v0.1 model shapes intentionally use UTF-8 strings to preserve Python parity. Broader `OsString`/`PathBuf`
handling is future hardening.

---

## 11. Exit Codes

| Code | Meaning                                      |
| ---- | -------------------------------------------- |
| 0    | Success                                      |
| 1    | Runtime or usage error                       |
| 2    | Diagnostic issues found matching `--fail-on` |
| 3    | Command not found                            |

Keep exit codes stable and documented.

---

## 12. Testing Strategy

## 12.1 Unit tests

Test:

* PATH splitting
* path normalisation
* duplicate detection
* case sensitivity behaviour
* PATHEXT logic
* executable detection

## 12.2 Fixture tests

Create platform-specific fixtures for:

* POSIX PATH strings
* Windows PATH strings
* command lookup scenarios
* duplicate entries
* missing entries
* empty entries

## 12.3 Golden tests

For human output:

* `print`
* `doctor`
* `why`
* `conflicts`

Store expected output snapshots.

For JSON output, compare exact output for `print`, `doctor`, `why`, `conflicts`, and the related-hint `why` not-found
case against `/Users/klemen/Work/patholog.py/tests/fixtures/golden`.

## 12.4 Integration tests

Use temporary directories to create:

* fake PATH trees
* fake commands
* command resolution scenarios

Examples:

* two `python` binaries in different dirs
* duplicate PATH entries
* missing dir plus working dir
* Windows-style `.cmd` vs `.exe`

---

## 13. Required Test Cases

Minimum coverage:

1. exact duplicate PATH entries
2. duplicate entries with platform-specific case differences
3. empty PATH entry
4. missing directory entry
5. non-directory PATH entry
6. command found once
7. command found multiple times
8. command not found
9. Homebrew path behind system path on macOS
10. Cargo bin after system dirs
11. Windows PATHEXT lookup
12. `clean --stdout` preserves first-win order
13. JSON output matches Python golden fixtures
14. `doctor --fail-on` returns exit code 2 correctly
15. unreadable directory diagnostics when the host can represent them
16. `doctor --command` reports shadowed candidates
17. `scan` reports PATH mutations in shell startup profiles

---

## 14. Documentation Requirements

README must include:

### 14.1 One-line pitch

> Diagnose and fix PATH problems across macOS, Linux and Windows.

### 14.2 Quick examples

```bash
patholog doctor
patholog why python
patholog conflicts cargo
patholog clean --stdout
```

### 14.3 Why it exists

Show common pain:

* wrong Python
* wrong Node
* duplicate PATH junk
* missing Cargo bin
* shell config drift

### 14.4 Platform notes

Explain:

* Windows command resolution differs
* shell config mutation is not performed
* `scan` reads startup profiles but does not source or edit them
* `clean` only proposes output

---

## 15. Explicit Non-Goals for v0.1 through v0.3

Do not implement yet:

* automatic shell profile editing
* PATH generation from declarative config
* daemon/background watcher
* TUI
* editor integration
* shell plugin manager
* package manager integration
* “fix everything” one-shot command that mutates files

---

## 16. Future Extensions

Possible post-v0.3 features:

* `apply --shell zsh|bash|pwsh`
* `why-not <command>` with install hints
* `watch` to detect PATH drift
* machine-readable health score

---

## 17. Design Philosophy

### Small, precise, safe

`patholog` should:

* explain before changing
* prefer explicitness over magic
* preserve user trust
* be predictable and scriptable

### Good default posture

A user must be able to run:

```bash
patholog doctor
```

on any machine without fear that it edits anything.

---

## 18. MVP Acceptance Criteria

v0.1 is successful if:

* `doctor` reliably detects obvious PATH issues
* `why` clearly explains command resolution
* `conflicts` shows all candidates in order
* `clean --stdout` produces a sane deduplicated PATH
* tests cover Unix and Windows semantics
* output is good enough to demo in 30 seconds

v0.1 is not successful if:

* it edits user shell files automatically
* Windows resolution is hand-waved
* output is vague or untrustworthy
* it becomes a general dotfile manager

---

## 19. Suggested Implementation Order

1. PATH parser
2. normalisation and comparison model
3. `print`
4. `doctor`
5. `conflicts`
6. `why`
7. `clean --stdout`
8. JSON output
9. tests and fixtures
10. docs and packaging

---

## 20. Final Note

The strongest angle for `patholog` is not “PATH cleaner”.

It is:

> **Explain why the wrong binary is running, then help fix PATH safely.**

That should remain the centre of the product.
