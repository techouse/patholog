# Releasing patholog

`patholog` is currently released from the private repository. Do not publish to crates.io or make the repository public
unless that is explicitly part of the release scope.

## Private Release Checklist

1. Confirm the version in `Cargo.toml`, `Cargo.lock`, `fuzz/Cargo.lock`, and `CHANGELOG.md`. CLI and binary wrapper
   version tests should derive expectations from Cargo metadata rather than hardcoding release versions.
2. Run the full local gate. This requires network access for the online package and publish dry-run checks:

   ```sh
   make pre-release
   ```

   If network access is unavailable but the local Cargo cache is already populated, use this package verification
   fallback before tagging. It verifies package creation from the local cache, but it is not a full replacement for
   the online pre-release gate:

   ```sh
   make package-check-offline
   ```

3. Commit the release changes.
4. Create an annotated release tag:

   ```sh
   version="v1.0.0-rc.4"
   git tag -a "$version" -m "Release $version"
   ```

5. Verify the tag points at the intended commit:

   ```sh
   git tag --points-at HEAD
   git describe --tags --always --dirty --long
   ```

6. Push the commit and tag:

   ```sh
   git push origin main
   git push origin "$version"
   ```

7. Confirm GitHub Actions pass on the pushed commit.

## v1 Readiness Checklist

Before cutting a v1 release candidate, run:

```sh
make package-metadata-check
make install-smoke
make v1-contract-check
```

Confirm the package contents policy, `SECURITY.md` status, third-party license notice policy, README install claims,
repository visibility, and crates.io publish decision are all intentional. Do not add public install instructions or
publish to crates.io until those decisions are complete.

## Private v1 RC Checklist

Before tagging `v1.0.0-rc.4`, confirm the v1 contract remains frozen except for release-blocking bug fixes, then run:

```sh
make v1-contract-check
make pre-release
```

After tagging and pushing the RC commit, verify the tag points at the intended commit and confirm GitHub Actions pass.
Do not publish the RC to crates.io unless publishing is explicitly added to the release scope.

## Public Release Notes

Before the first public GitHub or crates.io release, re-check package contents with:

```sh
make package-list
make package-check-offline
cargo publish --dry-run --locked --allow-dirty
```

Expected package contents are the crate metadata, `README.md`, `CHANGELOG.md`, `LICENSE`, and source files needed to
build the library and binary. `Cargo.toml.orig` may appear in `cargo package --list`; it is Cargo-generated package
metadata and is expected. Internal project files such as integration tests, fuzz targets, scripts, release notes, local
agent instructions, and repository automation are intentionally excluded. `SECURITY.md` and generated third-party
license notices remain repository-level documents until a public release pass deliberately changes that package policy.

Do not run `cargo publish` until the repository visibility, README, security policy, and install instructions are ready
for public users.
