# Copilot Project Instructions: patholog

Concise, project-specific guidance for AI coding agents working on this repo. Preserve the documented v1 CLI contract, JSON contracts, exit codes, config schema version `1`, and the safety boundary that only `apply --yes` mutates files.

## 1. Project Purpose And Architecture

- Library and binary: Rust implementation of the `patholog` CLI.
- Current crate-wide MSRV: Rust `1.88`.
- `src/main.rs` is a thin process wrapper. Core CLI execution is in `src/cli`.
- The CLI is the primary product surface. The public Rust API is intentionally small and mainly supports tests, fuzzing, and embedding the CLI with an injected `CommandContext`.
- Internal modules:
  - `src/path_env.rs`: PATH/MANPATH parsing.
  - `src/platform.rs`: platform selection and path separator behavior.
  - `src/normalize.rs`: comparison keys and deduplication.
  - `src/resolve.rs`: command lookup, candidates, PATHEXT modeling, and related hints.
  - `src/doctor.rs`: PATH diagnostics, ordering, duplicates, unwanted entries, and shadowed commands.
  - `src/health.rs`: health scoring over doctor diagnostics.
  - `src/why_not.rs`: missing-command analysis.
  - `src/clean.rs`: cleaned PATH/MANPATH rendering.
  - `src/profile_scan.rs`: read-only shell profile scanning.
  - `src/apply.rs`: managed shell profile block planning and writes.
  - `src/config.rs`: config schema version `1` loading and normalization.
  - `src/output/`: human and JSON formatting.
  - `src/model/`: public data model and exit code types.
- Parity fixtures live under `tests/fixtures/golden/`. Fuzz targets live under `fuzz/`.

## 2. Behavioral Oracles

- Source of truth order:
  1. `SPEC.md` v1 contract and command sections
  2. `README.md` user-facing command documentation
  3. contract/unit/parity tests in `src/**/tests.rs` and `tests/`
- Human output is user-facing and may evolve more freely than JSON.
- JSON field names and documented exit codes are public contract.

## 3. Key Invariants

- Keep all commands read-only except `apply --yes`.
- `apply --yes` may write only the patholog-managed shell profile block and should not rewrite arbitrary PATH lines outside that block.
- Do not add package-manager integration, global config discovery, automatic reordering, daemon/watch behavior, or broader mutation unless the spec is intentionally changed.
- `--config auto` searches only the current working directory for `patholog.toml`, then `.patholog.toml`.
- Config schema version remains `1`.
- `PATH` and `MANPATH` are the only supported path-like variables. Command resolution remains PATH-only.
- Platform-specific behavior must be explicit and deterministic: POSIX uses `:`, Windows uses `;`, case-insensitive comparison keys, and PATHEXT modeling.
- Preserve deterministic JSON output for script and dashboard consumers.

## 4. Developer Workflow

- Normal local check:
  - `make ci`
- Contract and release-readiness checks:
  - `make version-check`
  - `make release-check`
  - `make v1-contract-check`
- Full release gate before tagging:
  - `make pre-release`
- Focused Rust checks:
  - `cargo fmt --check`
  - `cargo test --locked`
  - `cargo test --doc --locked`
  - `cargo clippy --all-targets --all-features --locked -- -D warnings`
  - `cargo clippy --manifest-path fuzz/Cargo.toml --all-targets --locked -- -D warnings`

## 5. Testing Strategy

- Add focused unit tests for pure parsing, normalization, diagnostics, resolution, output, and apply-planning behavior.
- Add CLI contract tests for command routing, exit codes, completions, and parsed JSON top-level fields.
- Prefer deterministic fixtures with temp directories over permission, clock, race, or OS-specific failure simulations.
- For JSON tests, parse with `serde_json` and assert fields/types rather than relying only on substring checks.
- Keep fuzz harnesses read-only unless intentionally testing apply planning without writes.

## 6. Common Pitfalls

- Adding a command, flag, JSON field, or config schema change while intending only release/tooling work.
- Treating human output formatting as the stable machine contract.
- Applying config `fail_on` outside `doctor`.
- Letting `health` become a CI gate; `doctor --fail-on` owns diagnostic exit gating.
- Running package-manager commands or emitting package-manager install commands from `why-not`.
- Broadening `apply` so it rewrites unmanaged shell profile content.
- Making Windows PATH behavior depend on the host OS instead of injected platform mode in tests.

## 7. When Unsure

- Check `SPEC.md` first, then `README.md`, then existing tests.
- Prefer the smallest contract-preserving change over new abstractions.
- Document ambiguity instead of silently changing public behavior.

---
If these instructions conflict with measured behavior, tests, or the documented v1 contract, follow the measured/tested behavior and update the docs or tests explicitly.
