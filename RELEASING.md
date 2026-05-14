# Releasing patholog

`patholog` is currently released from the private repository. Do not publish to crates.io or make the repository public
unless that is explicitly part of the release scope.

## Private v0.x Release Checklist

1. Confirm the version in `Cargo.toml`, `Cargo.lock`, `README.md` examples, CLI version tests, and `CHANGELOG.md`.
2. Run the full local gate:

   ```sh
   make pre-release
   ```

3. Commit the release changes.
4. Create an annotated release tag:

   ```sh
   version="v0.8.0"
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
cargo publish --dry-run --locked --allow-dirty
```

Do not run `cargo publish` until the repository visibility, README, security policy, and install instructions are ready
for public users.
