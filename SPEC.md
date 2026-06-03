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
* mutating `apply`
* inode or canonical-file duplicate analysis
* long-running `watch`
* package-manager install hints

---

## 1.4 Apply Dry-Run Milestone (v0.4)

v0.4 plans shell profile repairs without writing files. It defines the managed-block contract for a future mutating
`apply`, but every v0.4 apply path is read-only.

v0.4 safe planning command:

```bash
patholog apply --dry-run --shell zsh|bash|fish|pwsh [--json] [--platform auto|posix|windows] [--home <dir>] [--profile <file>]
```

v0.4 still defers:

* automatic shell profile editing
* mutating `apply`
* inode or canonical-file duplicate analysis
* long-running `watch`
* package-manager install hints

---

## 1.5 Fixpath-Inspired Policy Milestones (v0.5.x)

v0.5.x adds opt-in cleanup policy inspired by old PATH repair scripts while keeping the read-first safety model.

v0.5.x additive commands and flags:

```bash
patholog print [--var path|manpath]
patholog doctor [--var path|manpath] [--drop <entry>] [--preset homebrew|cargo|pyenv|fink]
patholog clean --stdout [--var path|manpath] [--drop <entry>] [--preset homebrew|cargo|pyenv|fink]
patholog clean --export --shell zsh|bash|fish|pwsh [--var path|manpath] [--drop <entry>] [--preset homebrew|cargo|pyenv|fink]
patholog apply --dry-run --shell zsh|bash|fish|pwsh [--drop <entry>] [--preset homebrew|cargo|pyenv|fink]
```

v0.5.x still defers:

* config-file policy
* arbitrary path-like variables beyond `PATH` and `MANPATH`
* automatic reordering
* mutating `apply`

---

## 1.6 Safe Mutating Apply Milestone (v0.6)

v0.6 allows `apply` to write the same patholog-managed block produced by dry-run planning. Mutation is explicitly gated
by `--yes`, backs up existing profiles by default, and does not rewrite arbitrary PATH lines.

v0.6 apply commands and flags:

```bash
patholog apply --dry-run --shell zsh|bash|fish|pwsh [--json] [--platform auto|posix|windows] [--home <dir>] [--profile <file>] [--drop <entry>] [--preset homebrew|cargo|pyenv|fink]
patholog apply --shell zsh|bash|fish|pwsh --yes [--no-backup] [--json] [--platform auto|posix|windows] [--home <dir>] [--profile <file>] [--drop <entry>] [--preset homebrew|cargo|pyenv|fink]
```

v0.6 still defers:

* config-file policy
* arbitrary path-like variables beyond `PATH` and `MANPATH`
* automatic reordering
* arbitrary shell profile PATH-line rewriting
* long-running `watch`

---

## 1.7 Declarative Policy Config Milestones (v0.7.x)

v0.7.x adds explicit TOML config-file policy without expanding mutation scope. Config files provide defaults for
existing `--drop`, `--preset`, and `doctor --fail-on` behavior.

v0.7.x config commands and flags:

```bash
patholog doctor [--config <file|auto>]
patholog clean --stdout|--export [--config <file|auto>]
patholog apply --dry-run|--yes --shell zsh|bash|fish|pwsh [--config <file|auto>]
patholog config check --config <file|auto>
patholog config print --config <file|auto> [--json]
```

v0.7.x still defers:

* arbitrary path-like variables beyond `PATH` and `MANPATH`
* automatic reordering
* arbitrary shell profile PATH-line rewriting
* user-global config discovery
* long-running `watch`

---

## 1.8 Missing Command Analysis Milestone (v0.8)

v0.8 adds read-only missing-command analysis. It explains why an exact command is not available by combining command
resolution, related executable hints, PATH health diagnostics, and safe advisory next checks.

v0.8 command:

```bash
patholog why-not <command> [--json] [--platform auto|posix|windows]
```

v0.8 still defers:

* arbitrary path-like variables beyond `PATH` and `MANPATH`
* automatic reordering
* arbitrary shell profile PATH-line rewriting
* user-global config discovery
* package manager integration
* long-running `watch`

---

## 1.9 Machine-Readable Health Score Milestone (v0.9)

v0.9 adds a read-only health summary command. It converts existing `doctor` diagnostics into a compact deterministic
score for PATH or MANPATH without changing the detailed `doctor` report.

v0.9 command:

```bash
patholog health [--json] [--platform auto|posix|windows] [--var path|manpath] [--drop <entry>] [--preset homebrew|cargo|pyenv|fink] [--config <file|auto>]
```

v0.9 still defers:

* arbitrary path-like variables beyond `PATH` and `MANPATH`
* automatic reordering
* arbitrary shell profile PATH-line rewriting
* user-global config discovery
* package manager integration
* daemon/background `watch`

---

## 1.10 v1.0.0 Release Candidate Audit (v0.9.4)

v0.9.4 is a pre-v1 audit patch. It should not add runtime behavior or expand the CLI; it should prove that the
current public surface is documented and covered by contract tests.

Surfaces intended to be stable for v1:

* command names and documented flags
* exit codes `0`, `1`, `2`, and `3`
* JSON field names and existing diagnostic/candidate/config shapes
* config schema version `1`
* the boundary that only `apply --yes` mutates files

Human output is intended for interactive use and may evolve more freely than JSON. v1 still excludes `watch`,
package-manager hints, user-global config discovery, automatic reordering, and mutation outside the managed apply
block.

---

## 1.11 Public Release Dry Run (v0.9.5)

v0.9.5 is the final private pre-RC packaging dry run. It should not add runtime behavior or expand the CLI; it should
verify that the packaged crate can be installed into an isolated temporary root and run as a normal binary.

v0.9.5 checks:

* package metadata and package exclusion policy are explicit
* `SECURITY.md` and generated third-party license notices remain repository-level for v1.0.0
* README keeps pre-v1/private-release positioning and no public crates.io install instructions
* packaged install smoke verifies `patholog --version` and `patholog health --json`

---

## 1.12 Private v1 Release Candidate (v1.0.0-rc.1)

`v1.0.0-rc.1` is the private v1 release candidate. It should not add runtime behavior or expand the CLI; it freezes the
documented v1 contract for final validation before `v1.0.0`.

RC contract boundaries:

* stable: command names, documented flags, exit codes, JSON field names, config schema version `1`, and `apply --yes`
  as the only mutating command
* flexible: human output may evolve more freely than JSON
* excluded: `watch`, new health flags, package-manager hints, user-global config discovery, automatic reordering, and
  mutation outside the managed apply block

Any post-RC change should be a release-blocking bug fix that preserves this contract, or a documentation clarification
for an ambiguity found during RC validation.

---

## 1.13 Coverage Hardening Release Candidate (v1.0.0-rc.2)

`v1.0.0-rc.2` is a private coverage-hardening release candidate. It should add tests and release metadata only; it
must not add runtime behavior, expand the CLI, change JSON contracts, change config schema version `1`, or broaden
mutation beyond `apply --yes`.

The goal is to raise meaningful coverage where tests remain deterministic. Coverage gaps caused by impractical
platform, permission, clock, or low-level I/O failure paths may remain uncovered rather than forcing brittle tests.

---

## 1.14 Release Cleanup Release Candidate (v1.0.0-rc.3)

`v1.0.0-rc.3` is a private cleanup release candidate. It should not add runtime behavior, expand the CLI, change JSON
contracts, change config schema version `1`, or broaden mutation beyond `apply --yes`.

The v1 contract remains frozen. rc.3 may clarify release documentation, package expectations, and coverage policy, but
it must not move stable command names, documented flags, exit codes, JSON field names, or read-only versus mutating
command boundaries.

Coverage should remain meaningful rather than absolute. 100% coverage is not a v1 release gate. Intentional residual
coverage gaps may remain for JSON serialization failure branches, completion UTF-8 failure branches, low-level
temporary-file/write/persist failures, true OS read errors, and platform-specific filesystem behavior that would require
brittle tests.

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

### 3.3 Explain why a command is not available

```bash
patholog why-not poetry
patholog why-not python
```

Should explain:

* whether the exact command is already available
* which PATH directories were searched
* related executable names that are present
* PATH health issues relevant to lookup
* safe next checks without invoking package managers or editing files

---

### 3.4 Show all candidates for a command

```bash
patholog conflicts python
patholog conflicts git
```

Should list all matching executables found in PATH order.

---

### 3.5 Produce a cleaned PATH

```bash
patholog clean --stdout
```

Should emit a deduplicated, sanitised PATH string without mutating user files.

---

### 3.6 Diagnose command-specific shadowing

```bash
patholog doctor --command python
```

Should report command candidates that exist but lose to earlier PATH entries.

---

### 3.7 Scan shell startup files read-only

```bash
patholog scan
```

Should inspect known shell startup profiles under the user's home directory and report likely PATH mutation lines without
editing those files.

---

### 3.8 Summarize PATH or MANPATH health for automation

```bash
patholog health --json
patholog health --var manpath
```

Should report a compact score, issue counts, worst severity, and the diagnostics used to compute the score.

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

## 4.3 v0.4 Apply Dry-Run Addition

```bash
patholog apply --dry-run --shell zsh|bash|fish|pwsh [--json] [--platform auto|posix|windows] [--home <dir>] [--profile <file>]
```

`apply --dry-run` writes the planned profile action to stdout only.

---

## 4.4 v0.5.x Policy Additions

```bash
patholog print [--var path|manpath]
patholog doctor [--var path|manpath] [--drop <entry>] [--preset homebrew|cargo|pyenv|fink]
patholog clean --stdout [--var path|manpath] [--drop <entry>] [--preset homebrew|cargo|pyenv|fink]
patholog clean --export --shell zsh|bash|fish|pwsh [--var path|manpath] [--drop <entry>] [--preset homebrew|cargo|pyenv|fink]
patholog apply --dry-run --shell zsh|bash|fish|pwsh [--drop <entry>] [--preset homebrew|cargo|pyenv|fink]
```

`--drop` and `--preset` are opt-in cleanup policy. They do not mutate files or reorder entries.

---

## 4.5 v0.6 Safe Apply Addition

```bash
patholog apply --shell zsh|bash|fish|pwsh --yes [--no-backup] [--json] [--platform auto|posix|windows] [--home <dir>] [--profile <file>] [--drop <entry>] [--preset homebrew|cargo|pyenv|fink]
```

`apply --yes` writes only the patholog-managed PATH block. Existing profiles are backed up by default.

---

## 4.6 v0.7.x Config Additions

```bash
patholog doctor [--config <file|auto>]
patholog clean --stdout|--export [--config <file|auto>]
patholog apply --dry-run|--yes --shell zsh|bash|fish|pwsh [--config <file|auto>]
patholog config check --config <file|auto>
patholog config print --config <file|auto> [--json]
```

Config files are policy-only. They do not configure shell, profile path, backup behavior, output mode, platform, or
mutation consent.

---

## 4.7 v0.9 Health Score Addition

```bash
patholog health [--json] [--platform auto|posix|windows] [--var path|manpath] [--drop <entry>] [--preset homebrew|cargo|pyenv|fink] [--config <file|auto>]
```

`health` is a read-only summary of existing `doctor` diagnostics. It does not support `--command`, `--fail-on`, or any
mutation.

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

## 5.3 `health`

```bash
patholog health
patholog health --json
```

Summarizes PATH or MANPATH health using the same diagnostic engine and policy inputs as `doctor`.

### Output must include

* score from `0` to `100`
* healthy boolean
* entry and issue counts
* worst severity: `none`, `warning`, or `error`
* diagnostic counts by stable issue kind
* diagnostics using the existing diagnostic JSON shape

### Scoring

Start at `100`, subtract `15` for each `missing`, `not_directory`, `unreadable`, or `empty` diagnostic, subtract `5`
for each `duplicate`, `unwanted`, or `suspicious_order` diagnostic, and clamp at `0`.

The score is an advisory summary. Use `doctor --fail-on` for CI gating; `health` does not consume `fail_on` policy.

Exit code:

* `0` when health calculation succeeds, even if issues are found
* `1` on usage or runtime error

---

## 5.4 `why <command>`

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

## 5.5 `why-not <command>`

```bash
patholog why-not poetry
```

Explains missing-command cases without mutating files, reordering PATH, or invoking package managers.

### Output must include

* whether an exact executable candidate was found
* the winning exact candidate, if found
* searched directories when not found
* related executable hints, if any
* PATH health diagnostics relevant to lookup, such as empty, missing, non-directory, or unreadable entries
* ordered advisory next checks

### Example

```text
Command: poetry

Not found in PATH.

Searched directories:
  1  /opt/homebrew/bin
  2  /usr/local/bin
  3  /usr/bin

Advice:
  Check that the command is installed and that its executable directory is present in PATH.
```

JSON output must expose stable top-level fields:

* `command`
* `found`
* `winner`
* `candidates`
* `searched_directories`
* `related_hints`
* `path_diagnostics`
* `advice`

Exit code:

* `0` if an exact command candidate is found
* `3` if not found
* `1` on runtime error

---

## 5.6 `conflicts <command>`

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

## 5.7 `clean`

```bash
patholog clean --stdout
patholog clean --stdout --drop /sw/bin
patholog clean --export --var manpath --shell zsh
```

Produces a cleaned PATH representation.

### Default cleaning rules in v0.1

* remove empty entries
* remove later duplicates after comparison-key normalisation
* remove entries matched by explicit `--drop` or drop-style presets
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

* `--var path|manpath`

  * default to `path`
  * switch `print`, `doctor`, and `clean` to `MANPATH` when set to `manpath`
  * preserve empty MANPATH components because they can represent the system default manpath
  * keep `apply`, `why`, `conflicts`, and `scan` PATH-only

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

## 5.8 `completions`

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

## 5.9 `scan`

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

## 5.10 `apply`

```bash
patholog apply --dry-run --shell zsh
patholog apply --dry-run --shell zsh --drop /sw/bin
patholog apply --shell zsh --yes
patholog apply --shell zsh --yes --no-backup
```

Plans or writes a shell profile edit using a patholog-managed block. Mutating apply requires `--yes`.

### Behaviour

* require exactly one of `--dry-run` or `--yes`
* require explicit `--shell zsh|bash|fish|pwsh`
* choose an interactive profile by default
* allow `--home <dir>` for deterministic default target selection
* allow `--profile <file>` to override the target profile
* allow `--drop <entry>` and drop-style presets for the PATH block
* report `create_profile`, `append_block`, or `replace_block`
* reject non-file, unreadable, malformed-block, or duplicate-block profiles
* create backups for existing profile writes unless `--no-backup` is passed
* never rewrite PATH lines outside the patholog managed block

Managed block markers:

```text
# >>> patholog PATH >>>
# <<< patholog PATH <<<
```

Exit code:

* `0` on successful dry-run planning or write
* `1` on usage, target-profile, or managed-block error

---

## 5.11 `--drop` and `--preset`

`--drop <entry>` is an exact opt-in cleanup rule for `doctor`, `health`, `clean`, and `apply`.

Behaviour:

* match after existing platform comparison-key normalisation
* do not use glob, regex, `~`, or environment expansion
* report `unwanted` diagnostics in `doctor` and `health`
* remove matching entries before first-win deduplication in `clean` and `apply`

`--preset homebrew|cargo|pyenv|fink` enables built-in policy. `homebrew`, `cargo`, and `pyenv` are diagnostic-only
ordering presets. `fink` marks `/sw/bin` and `/sw/sbin` as unwanted for PATH, and `/sw/share/man` as unwanted for MANPATH. Presets never reorder entries.

---

## 5.12 Config Files

```bash
patholog config check --config patholog.toml
patholog config print --config patholog.toml --json
patholog doctor --config patholog.toml
patholog health --config patholog.toml
patholog clean --stdout --config auto
```

Supported TOML schema:

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

Behaviour:

* reject unsupported `version` values and unknown keys
* apply `[path]` to PATH commands, `doctor --command`, `health`, and `apply`
* apply `[manpath]` to `doctor --var manpath`, `health --var manpath`, and `clean --var manpath`
* treat config values as defaults; repeated CLI flags append after config values
* apply config `drop` and `preset` to `health`, but keep `fail_on` doctor-only
* `--config auto` searches the current working directory for `patholog.toml`, then `.patholog.toml`
* operational commands ignore missing `--config auto`; `config check` and `config print` fail when auto finds nothing

---

## 6. v0.1 Safety Model

### Read-first by default

Commands inspect and explain before changing files. v0.6 mutation is limited to `apply --yes`.

### No automatic broad mutation

Do not modify these outside the explicit `apply --yes` managed-block path:

* `~/.zshrc`
* `~/.bashrc`
* `~/.profile`
* PowerShell profile
* system env config

### Rationale

PATH mutation is risky and shell-specific. Inspection must come first.
`apply --dry-run` shows the exact block before `apply --yes` writes it.

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
* `health`
* `print`
* `conflicts`
* `why`
* `why-not`

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
* `health`
* `why`
* `why-not`
* `conflicts`

Store expected output snapshots.

For JSON output, compare exact output for `print`, `doctor`, `why`, `conflicts`, and the related-hint `why` not-found
case against `/Users/klemen/Work/patholog.py/tests/fixtures/golden`.

For Rust-only output, include `why-not` human and JSON tests with exact lookup, related hints, PATH diagnostics, and
advice fields. Include `health` human and JSON tests for scoring, counts, severities, and diagnostics.

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
18. `health` reports deterministic score, counts, and severity
19. public command, JSON field, exit-code, and package contracts stay covered before v1

---

## 14. Documentation Requirements

README must include:

### 14.1 One-line pitch

> Diagnose and fix PATH problems across macOS, Linux and Windows.

### 14.2 Quick examples

```bash
patholog doctor
patholog why python
patholog why-not poetry
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

## 15. Explicit Non-Goals for v0.1 through v0.9

Do not implement yet:

* automatic shell profile editing outside the patholog managed block
* arbitrary path-like variables beyond `PATH` and `MANPATH`
* automatic PATH reordering
* daemon/background watcher
* user-global config discovery
* TUI
* editor integration
* shell plugin manager
* package manager integration
* “fix everything” one-shot command that mutates files

---

## 16. Future Extensions

Possible post-v0.9 features:

* `watch` to detect PATH drift

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
