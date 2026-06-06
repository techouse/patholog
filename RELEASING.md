# Releasing patholog

`patholog` releases are public from `v1.0.0` onward. Releases publish the crate to crates.io, create a GitHub release
with native binary artifacts, and publish library docs to GitHub Pages.

## Release Checklist

1. Confirm the version in `Cargo.toml`, `Cargo.lock`, `fuzz/Cargo.lock`, and `CHANGELOG.md`. CLI and binary wrapper
   version tests should derive expectations from Cargo metadata rather than hardcoding release versions.
2. Confirm public package metadata and install docs:

   ```sh
   make package-metadata-check
   ```

3. Run the full local gate. This requires network access for the online package and publish dry-run checks:

   ```sh
   make pre-release
   ```

   If network access is unavailable but the local Cargo cache is already populated, use this package verification
   fallback before tagging. It verifies package creation from the local cache, but it is not a full replacement for
   the online pre-release gate:

   ```sh
   make package-check-offline
   ```

4. Run the GitHub Actions `Release Dry Run` workflow before the final tag:
   - Use a version label such as `1.0.0-dry-run`.
   - Build Linux and Windows artifacts by default.
   - Run the signed macOS dry run only after release secrets and the `release` environment are configured.
5. Commit the release changes.
6. Create an annotated release tag:

   ```sh
   version="v1.0.0"
   git tag -a "$version" -m "Release $version"
   ```

7. Verify the tag points at the intended commit:

   ```sh
   git tag --points-at HEAD
   git describe --tags --always --dirty --long
   ```

8. Push the commit and tag:

   ```sh
   git push origin main
   git push origin "$version"
   ```

9. Confirm the `Release` workflow passes:
   - required tests pass
   - crates.io publish succeeds
   - Linux, Windows, and signed/notarized macOS artifacts are attached to the GitHub release
   - checksums and artifact attestations are present
   - GitHub Pages docs are published
10. After crates.io indexing finishes, verify:

    ```sh
    cargo install patholog
    patholog --version
    patholog health --json
    ```

## Required GitHub Configuration

The full release workflow expects:

- a `release` environment
- `CARGO_REGISTRY_TOKEN`
- `BUILD_CERTIFICATE_BASE64`
- `BUILD_CERTIFICATE_SHA1`
- `KEYCHAIN_PASSWORD`
- `P12_PASSWORD`
- `APPLE_ID`
- `NOTARYTOOL_PASSWORD`
- `TEAM_ID`
- `NOTARYTOOL_KEYCHAIN_PROFILE`
- optional coverage secrets used by `test.yml`: `CODECOV_TOKEN` and `CODACY_PROJECT_TOKEN`

## Package Contents Policy

The crates.io package should contain Cargo metadata, `README.md`, `CHANGELOG.md`, `LICENSE`, and source files needed to
build the library and binary. `Cargo.toml.orig` may appear in `cargo package --list`; it is Cargo-generated package
metadata and is expected.

Internal project files such as integration tests, fuzz targets, scripts, release notes, local agent instructions, and
repository automation are intentionally excluded from the crate package. `SECURITY.md` and generated third-party
license notices remain excluded from the crates.io package, but `THIRD-PARTY-LICENSES.md` is included in native binary
release archives and Linux OS packages.
