# Changelog

All notable changes to patholog are documented here.

## 0.1.0 - 2026-04-23

- Initial Rust release of `patholog`.
- Added strict v0.1 parity with the Python rapid prototype for `print`, `doctor`, `why`, `conflicts`, and `clean --stdout`.
- Added deterministic human and JSON output with vendored golden parity fixtures.
- Added conservative PATH parsing, duplicate detection, cleaning, doctor diagnostics, and command resolution for POSIX plus modeled Windows `PATHEXT` behavior.
- Added injected CLI context for deterministic tests without depending on the developer machine PATH.
- Added focused unit, integration, binary wrapper, and fuzzing coverage.
- Added release-maintainer checks, package exclusions, third-party license notices, and pre-release Makefile gates.
