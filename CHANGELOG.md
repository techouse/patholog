# Changelog

All notable changes to patholog are documented here.

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
