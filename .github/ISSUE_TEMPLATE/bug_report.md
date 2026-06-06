---
name: Bug report
about: Report incorrect PATH diagnostics, command resolution, profile planning, CLI, or file I/O behavior.
title: ""
labels: bug
assignees: techouse
---

<!--
    Before filing, check README.md and SPEC.md for the documented v1 contract:
    command names, flags, exit codes, JSON fields, config schema version 1,
    and the rule that only apply --yes mutates files.
-->

## Problem Summary

<!--
Describe the bug clearly:
- the patholog command or Rust API path you used
- the PATH, MANPATH, PATHEXT, config, or profile input
- what you expected
- what happened instead
-->

## Reproduction

Prefer a minimal command plus temporary directories or a tiny config/profile fixture.

```bash
mkdir -p /tmp/patholog-repro/bin
patholog_bin="$(command -v patholog)"
PATH="/tmp/patholog-repro/bin:/missing" "$patholog_bin" doctor --json
```

## Expected Behavior

<!-- What should have happened? Include exact stdout/stderr and exit code when relevant. -->

## Actual Behavior

<!-- What happened instead? Include exact stdout/stderr and exit code. -->

## Contract Context

- [ ] Matches the README command documentation
- [ ] Matches the SPEC.md v1 contract
- [ ] Affects JSON field names or value shape
- [ ] Affects documented exit codes
- [ ] Affects the apply --yes mutation boundary

Relevant references:

- SPEC.md section:
- README.md section:
- Existing test or fixture:

## Inputs

- Command:
- PATH:
- MANPATH:
- PATHEXT:
- Config file:
- Shell/profile path:
- Platform:
- Rust version:

## Environment

```bash
patholog --version
rustc --version
cargo --version
uname -a
```

```text
```

## Additional Context

<!-- Add logs, release workflow links, comparison output, or anything else that helps reproduce the issue. -->
