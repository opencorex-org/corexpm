# Development guide

## Prerequisites

- stable Rust matching `rust-toolchain.toml`;
- Git; and
- Node.js only when working on compatibility fixtures or TypeScript packages.

## Build and check

```sh
cargo build --workspace
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

Keep default tests hermetic. Registry tests use a local mock server and store
tests override the Corex root with a fresh temporary directory. Tests must not
read or mutate the developer's real `~/.corex` directory.

## Code boundaries

- Domain crates do not format terminal output.
- Filesystem and network effects sit behind narrow interfaces.
- User-facing failures use stable `corex-errors` diagnostics.
- Persistent formats include a version and deterministic serialization rules.
- `unsafe` Rust is forbidden workspace-wide unless a future ADR changes policy.
- New dependencies require license, maintenance, security, and necessity review.

## Documentation changes

Use an ADR to record an accepted implementation decision and an RFC to propose
broad user-visible behavior. Update the compatibility matrix when adding or
deferring npm behavior. Update the threat model for new trust boundaries.

