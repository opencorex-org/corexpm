# Contributing to CorexPM

CorexPM is an early-stage systems project. Small, testable changes with explicit
compatibility evidence are more valuable than broad rewrites.

## Before contributing

1. Read the [project scope](docs/product/scope.md),
   [architecture](docs/architecture/overview.md), and relevant ADRs.
2. Search existing issues and RFCs.
3. Open an issue before large implementation work.
4. Use an RFC for user-visible semantics, file formats, security boundaries,
   compatibility breaks, or cross-crate architecture.

## Development setup

```sh
rustup show
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo run -p corex-cli -- --help
```

See [the development guide](docs/contributing/development.md) for repository
conventions and validation expectations.

## Pull requests

- Keep each change focused and explain its user impact.
- Add tests for behavior and fixtures for ecosystem compatibility.
- Update specifications when behavior changes.
- Do not claim performance wins without reproducible benchmark data.
- Do not add telemetry, network destinations, or script capabilities implicitly.
- Use a conventional commit-style title, such as `feat(store): verify package
  integrity before commit`.

All contributions are made under the repository's MIT License and must follow
the [Code of Conduct](CODE_OF_CONDUCT.md).

