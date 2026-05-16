# Releasing patholog

`patholog` is currently released from the private repository. Do not publish to crates.io or make the repository public
unless that is explicitly part of the release scope.

## Private v0.x Release Checklist

1. Confirm the version in `Cargo.toml`, `Cargo.lock`, `fuzz/Cargo.lock`, CLI version tests, binary wrapper version
   tests, and `CHANGELOG.md`.
2. Run the full local gate:

   ```sh
   make pre-release
   ```

   If network access is unavailable but the local Cargo cache is already populated, use this package verification
   fallback before tagging:

   ```sh
   make package-check-offline
   ```

3. Commit the release changes.
4. Create an annotated release tag:

   ```sh
   version="v0.9.3"
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

## Public Release Notes

Before the first public GitHub or crates.io release, re-check package contents with:

```sh
make package-list
make package-check-offline
cargo publish --dry-run --locked --allow-dirty
```

Expected package contents are the crate metadata, `README.md`, `CHANGELOG.md`, `LICENSE`, and source files needed to
build the library and binary. Internal project files such as integration tests, fuzz targets, scripts, release notes,
local agent instructions, and repository automation are intentionally excluded. `SECURITY.md` and generated
third-party license notices remain repository-level documents until a public release pass deliberately changes that
package policy.

Do not run `cargo publish` until the repository visibility, README, security policy, and install instructions are ready
for public users.
