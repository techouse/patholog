# Changelog

All notable changes to patholog are documented here.

## 1.0.0 - 2026-06-06

- Released the frozen v1 CLI, JSON, exit-code, and config contract publicly.
- Added public package metadata, install documentation, and release instructions.
- Added envq-style release automation for crates.io publishing, GitHub release artifacts, signed macOS builds, docs
  publishing, dry-run validation, checksums, and artifact attestations.

## 1.0.0-rc.4 - 2026-06-03

- Cut the final private v1 release candidate before the `1.0.0` release decision.
- Aligned public README safety wording and SPEC contract notes with the managed-block `apply --yes` behavior.
- Kept runtime behavior, CLI surface, JSON contracts, config schema, and package policy unchanged.

## 1.0.0-rc.3 - 2026-06-03

- Updated release metadata for the private cleanup release candidate.
- Clarified release documentation around online pre-release checks, offline package verification, and expected package
  contents.
- Documented intentional residual coverage gaps while keeping the frozen v1 contract unchanged.

## 1.0.0-rc.2 - 2026-06-01

- Added coverage-focused tests for config, scan, why-not, profile scanning, apply helpers, and CLI routing branches.
- Documented rc.2 as test-only coverage hardening with the v1 contract unchanged.
- Kept crates.io publishing and public install instructions deferred until the final v1 release pass.

## 1.0.0-rc.1 - 2026-06-01

- Cut the private v1 release candidate with the v1 command, JSON, exit-code, and config contract frozen.
- Updated release documentation for the private v1 RC flow.
- Kept crates.io publishing and public install instructions deferred until the final v1 release pass.

## 0.9.5 - 2026-05-31

- Added packaged install smoke testing for private public-release dry runs.
- Added package metadata and package policy checks for v1 readiness.
- Extended `make v1-contract-check` with metadata and install-smoke verification.

## 0.9.4 - 2026-05-31

- Added `make v1-contract-check` for pre-v1 command, JSON, docs, and package audits.
- Strengthened CLI contract tests for command listing, JSON field shapes, and exit codes.
- Documented the v1 stability contract and public-release readiness checklist.

## 0.9.3 - 2026-05-16

- Added `make version-check` and `make release-check` for public-v1 readiness audits.
- Expanded crate-level library docs with an injected CLI context example.
- Documented expected package contents and intentionally excluded internal files before public release.

## 0.9.2 - 2026-05-16

- Added `make package-check-offline` for release package verification using the local Cargo cache.
- Updated release documentation to remove stale tag examples and clarify version checklist files.
- Simplified README development guidance around `make ci` and `make pre-release`.

## 0.9.1 - 2026-05-16

- Hardened `patholog health` tests with parsed JSON contract assertions and Windows separator coverage.
- Added explicit health scoring coverage for every diagnostic issue kind.
- Clarified that health scores are advisory and `doctor --fail-on` remains the CI gate.

## 0.9.0 - 2026-05-16

- Added `patholog health` for read-only machine-readable PATH and MANPATH health scoring.
- Added human and JSON health output with score, healthy status, issue counts, worst severity, and diagnostics.
- Reused doctor drop, preset, and config policy while keeping `doctor --fail-on` as the only diagnostic exit gate.

## 0.8.0 - 2026-05-13

- Added `patholog why-not <command>` for read-only missing-command analysis.
- Added human and JSON output combining exact lookup, related executable hints, PATH health diagnostics, and safe advisory next checks.
- Kept `why-not` PATH-only with no config policy, package-manager integration, reordering, or mutation.

## 0.7.2 - 2026-05-10

- Added `--config auto` discovery for `patholog.toml` and `.patholog.toml` in the current working directory.
- Added missing-auto errors for `config check` and `config print` while keeping operational commands unchanged when auto finds no config.

## 0.7.1 - 2026-05-10

- Added `patholog config check --config <file>` and `patholog config print --config <file> [--json]`.
- Added normalized human and JSON config inspection output.

## 0.7.0 - 2026-05-10

- Added explicit TOML config policy via `--config <file>` for `doctor`, `clean`, and `apply`.
- Added config support for existing `drop`, `preset`, and `fail_on` policy.
- Kept config files policy-only with no extra mutation scope.

## 0.6.0 - 2026-05-09

- Added gated mutating `apply --shell zsh|bash|fish|pwsh --yes` using the existing patholog-managed block.
- Added default backups for existing profile edits plus `--no-backup` for explicit opt-out.
- Added same-directory temporary writes for profile updates and parent directory creation for new profile targets.
- Added human and JSON output for mutating apply results, including write and backup fields.
- Kept `apply --dry-run` as the read-only planning mode.

## 0.5.2 - 2026-05-05

- Added `--preset homebrew|cargo|pyenv|fink` to `doctor`, `clean`, and `apply --dry-run`.
- Added fink preset drop rules for PATH (`/sw/bin`, `/sw/sbin`) and MANPATH (`/sw/share/man`).
- Added preset CLI and fuzz coverage while keeping presets read-only and non-reordering.

## 0.5.1 - 2026-05-05

- Added `--var path|manpath` to `print`, `doctor`, and `clean`.
- Added `MANPATH` support in injected CLI context and shell export snippets.
- Preserved empty MANPATH components during cleaning because common man implementations treat them as default-manpath placeholders.
- Kept `apply`, `scan`, `why`, `conflicts`, and `doctor --command` PATH-only.

## 0.5.0 - 2026-05-05

- Added repeatable `--drop <entry>` to `doctor`, `clean`, and `apply --dry-run`.
- Added `unwanted` diagnostics and `doctor --fail-on=unwanted`.
- Added policy-filtered cleaning before first-win deduplication.

## 0.4.0 - 2026-05-04

- Added read-only `apply --dry-run --shell zsh|bash|fish|pwsh` profile repair planning.
- Added human and JSON apply dry-run output with planned managed blocks.
- Added default interactive profile targeting plus `--home` and `--profile` overrides.
- Added safety checks for non-file, unreadable, malformed, and duplicate managed-block profiles.
- Added CLI, unit, and fuzz coverage for apply dry-run planning.

## 0.3.0 - 2026-05-03

- Added read-only `clean --export --shell zsh|bash|fish|pwsh` shell assignment snippets.
- Added `completions zsh|bash|fish|pwsh` for stdout-only shell completion generation.
- Added shell-specific quoting for POSIX shells, fish, and PowerShell export output.
- Added read-only fuzz corpus coverage for export snippets and completions.
- Added release documentation for private v0.x release checks and tag verification.

## 0.2.0 - 2026-04-25

- Added read-only `patholog scan` for shell startup profile PATH mutation discovery.
- Added `doctor --command <command>` shadowed executable diagnostics.
- Added unreadable PATH directory diagnostics.
- Added `unreadable` and `shadowed_command` issue kinds for `doctor --fail-on`.
- Kept v0.1 parity output stable for existing golden fixture scenarios.

## 0.1.0 - 2026-04-23

- Initial Rust release of `patholog`.
- Added strict v0.1 parity with the Python rapid prototype for `print`, `doctor`, `why`, `conflicts`, and `clean --stdout`.
- Added deterministic human and JSON output with vendored golden parity fixtures.
- Added conservative PATH parsing, duplicate detection, cleaning, doctor diagnostics, and command resolution for POSIX plus modeled Windows `PATHEXT` behavior.
- Added injected CLI context for deterministic tests without depending on the developer machine PATH.
- Added focused unit, integration, binary wrapper, and fuzzing coverage.
- Added release-maintainer checks, package exclusions, third-party license notices, and pre-release Makefile gates.
