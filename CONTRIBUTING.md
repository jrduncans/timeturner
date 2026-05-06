# Contributing

## Development

See [CLAUDE.md](CLAUDE.md) for build, test, lint, and format commands.

## Releasing

1. Bump the version in `Cargo.toml`.
2. Run `cargo build` to update `Cargo.lock`.
3. Commit both files:
   ```
   git commit -m "Bump version to vX.Y.Z"
   ```
4. Push to `main`.
5. Tag the commit and push the tag:
   ```
   git tag vX.Y.Z
   git push origin vX.Y.Z
   ```

Pushing the tag triggers the release workflow, which builds binaries for all targets (x86_64 Linux musl, x86_64 macOS, aarch64 macOS, universal macOS), publishes a GitHub release with zip archives and auto-generated release notes, and runs `cargo publish` to release to crates.io.

The workflow requires a `CARGO_REGISTRY_TOKEN` secret set in the repository settings with a crates.io API token that has publish permission for the `timeturner` crate.
