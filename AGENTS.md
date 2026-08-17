# CorexPM agent guide

This file applies to the entire repository. More specific `AGENTS.md` files may
be added inside subdirectories later; when present, the nearest file takes
precedence for that subtree.

## Mission

CorexPM is an experimental native package manager for JavaScript and TypeScript
projects. Its product promise is:

> Download once. Store once. Use everywhere.

The intended v1 combines npm ecosystem compatibility, an immutable global
content-addressed package store, strict dependency isolation, deterministic
installs, and explicit lifecycle-script trust.

CorexPM is currently a `0.1.0-dev` bootstrap. The repository contains a working
Rust workspace and design specifications, but package resolution, fetching,
storage, and installation are not implemented. Never describe a planned
feature as working functionality.

## Read before changing code

Start with:

1. `README.md`
2. `docs/product/scope.md`
3. `docs/architecture/overview.md`
4. `docs/architecture/diagrams/README.md`
5. `docs/roadmap/ROADMAP.md`
6. the specification and ADRs relevant to the task

For storage or linker work, also read
`docs/architecture/node-modules-size-reduction.md` and
`docs/security/threat-model.md`.

## Repository map

- `crates/` — Rust implementation of the native CLI and core services.
- `docs/` — product intent, specifications, ADRs, RFCs, and roadmap.
- `tests/` — future cross-crate, compatibility, and adversarial fixtures.
- `benchmarks/` — reproducible performance scenarios and raw evidence.
- `fuzz/` — future parser, resolver, and archive fuzz targets.
- `packages/` — future TypeScript SDK and Node integration packages.
- `integrations/` — ecosystem integration contracts and fixtures.
- `apps/` — future documentation, playground, and benchmark applications.
- `examples/` — minimal supported JavaScript project examples.
- `scripts/` — repository automation that does not belong in Cargo or CI.

## Current Rust crates

- `corex-cli` owns argument handling, command dispatch, and terminal output.
- `corex-core` owns use-case orchestration and service composition.
- `corex-config` owns validated configuration types and precedence.
- `corex-manifest` owns npm manifest domain types and parsing.
- `corex-errors` owns stable user-facing diagnostics and error codes.

Future crate allocations are documented in
`docs/architecture/crate-map.md`. Do not create every planned crate in advance.
Extract a crate only when its roadmap milestone provides a stable
responsibility, owned data, interface, failure model, and tests.

## Architecture boundaries

- The CLI may depend on `corex-core`; domain crates must not depend on the CLI.
- Domain crates return structured values and diagnostics; they do not print.
- Network, filesystem, clock, process, and environment effects belong behind
  narrow interfaces so tests can replace them.
- Use typed identifiers such as package, version, registry, graph node, and
  integrity IDs instead of passing ambiguous strings through the system.
- Concurrent completion order must never affect graph identity, lockfile bytes,
  project layout, or machine-readable output.
- Persistent formats require schema versions, deterministic serialization,
  forward-version handling, and migration consideration.
- Prefer a small end-to-end vertical slice over broad placeholder modules.
- Keep Node.js integration outside the native core. CorexPM must start without
  requiring Node.js to be installed.

## Security invariants

The following rules are non-negotiable unless an accepted security RFC and ADR
explicitly replace them:

- Verify downloaded integrity before committing or executing package content.
- Treat registry metadata, archives, manifests, lockfiles, scripts, and local
  writable state as untrusted input.
- Safely extract archives without path traversal, link escape, special-file, or
  resource-exhaustion vulnerabilities.
- Once committed, a global CAS object is immutable.
- Dependency lifecycle scripts are denied unless effective policy permits them.
- Never mutate shared package source during native or lifecycle builds; use a
  writable project overlay or separately keyed build artifact.
- Redact registry credentials, authentication headers, tokens, and sensitive
  paths from normal diagnostics and logs.
- Use unique staging paths, narrow locks, validation, and atomic activation for
  project and store mutations.
- Frozen installs fail before mutation when manifests and lockfiles disagree.
- Never claim a capability is sandbox-enforced when the platform can only
  detect or document it.

Update `docs/security/threat-model.md` whenever a change introduces a new
network source, credential source, execution path, plugin hook, persistent
format, or filesystem trust boundary.

## Disk-efficiency rules

- V1 uses package-level immutable content-addressed storage.
- Project layouts should reference shared payloads instead of copying them.
- Correctness determines whether the linker uses symlinks, junctions, reflinks,
  protected hardlinks, or copy fallback on a platform/filesystem.
- Keep caches, CAS objects, build artifacts, and project link/state bytes
  separately measurable.
- Report actual filesystem allocation where possible. Never promise a fixed
  disk-saving percentage.
- Defer file/blob-level deduplication until benchmarks show a net benefit.
- Garbage collection must be reference-aware, transaction-aware, conservative,
  and recoverable.

## Implementation conventions

- Rust edition and minimum version come from the workspace `Cargo.toml`.
- Unsafe Rust is forbidden workspace-wide by default.
- Reuse workspace package metadata and lint configuration.
- Keep public APIs documented and use `#[must_use]` where ignoring a value is
  likely a bug.
- User-facing failures use an allocated `corex-errors` family and provide an
  actionable explanation without exposing internals or secrets.
- Avoid panics for invalid user, registry, package, or filesystem input.
- Bound network concurrency, archive sizes, file counts, path depth, process
  execution, and memory use.
- Stream large network/archive content instead of loading it completely into
  memory.
- New third-party dependencies require a clear need plus maintenance, license,
  source, and security review. Keep behavior behind Corex-owned interfaces.

## Required validation

Run these checks for Rust changes:

```sh
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo build --workspace --locked
```

If `cargo` is not on the process `PATH` in the local Codex environment, use:

```sh
/Users/lahiruudayakumara/.cargo/bin/cargo
```

Additional expectations:

- Parser and resolver changes need unit, malformed-input, determinism, and
  compatibility cases.
- Store and linker changes need temporary isolated roots, concurrency tests,
  failure injection, crash-recovery coverage, and platform-specific cases.
- Security changes need adversarial tests and secret-redaction checks.
- Performance claims need reproducible benchmarks and raw environment details.
- Tests must never read or mutate the developer's real `~/.corex` directory.
- Default required tests should be hermetic and should not require the public
  npm registry.

## Documentation and decisions

- Implementation and documentation disagreements are bugs; update both in the
  same change or record the deliberate follow-up.
- Use an ADR for an accepted architecture decision that is expensive to
  rediscover.
- Use an RFC for lockfile/config formats, resolver semantics, security
  capabilities, CLI compatibility, plugin contracts, or cross-component
  behavior.
- Update `docs/compatibility/npm-compatibility.md` when npm behavior is added,
  deferred, or intentionally differs.
- Update `docs/roadmap/ROADMAP.md` only when milestone scope or status genuinely
  changes.
- Mermaid diagrams describe target architecture and must not imply target
  components already exist. Keep diagram labels consistent with specifications.
- Do not invent benchmark numbers, compatibility claims, release dates, or
  security guarantees.

## Git and change hygiene

- Preserve unrelated user changes in a dirty worktree.
- Keep commits focused by concern; avoid one bulk commit for unrelated work.
- Use conventional commit-style subjects, for example:
  `feat(store): verify package integrity before commit`.
- Do not commit generated `target/`, `node_modules/`, `.corex/`, credentials,
  logs, local IDE state, or benchmark output that contains machine-private data.
- Do not delete foreign lockfiles automatically. Migration commands must leave
  source lockfiles untouched until the user chooses otherwise.
- Do not push, publish, release, tag, or open a pull request unless explicitly
  requested.

## Definition of done

A change is complete when its behavior and non-goals are clear, architecture
boundaries remain clean, security invariants are preserved, relevant tests and
fixtures pass, persistent/user-visible behavior is documented, and claims are
supported by reproducible evidence.
