## Description

Describe the change, why it is needed, and the behavior it affects.

Fixes #(issue)

## Type of Change

Delete options that are not relevant.

- [ ] Bug fix
- [ ] New command or behavior
- [ ] Breaking behavior change
- [ ] Documentation or test update
- [ ] CI, packaging, or release change

## v1 Contract

- [ ] SPEC.md and README.md still match the implementation
- [ ] Command names and documented flags are unchanged, or the compatibility impact is explained
- [ ] JSON field names and documented exit codes are unchanged, or the compatibility impact is explained
- [ ] Config schema version `1` remains compatible
- [ ] Mutation remains limited to `apply --yes` and the managed profile block

## Testing

List the commands you ran.

```bash
make ci
make v1-contract-check
```

## Checklist

- [ ] I have performed a self-review
- [ ] I have kept the change scoped to patholog behavior or release tooling
- [ ] New and existing tests pass locally
- [ ] Documentation and tests were updated where needed
