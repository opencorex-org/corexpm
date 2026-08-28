# CorexPM roadmap

This roadmap orders risk, not just features. Each milestone must leave behind
tests, specifications, and observable behavior. Dates are intentionally absent
until the contributor capacity and compatibility baseline are known.

## Phase 0 — foundation (`0.1.0-dev`, current)

Goal: establish buildable project boundaries and make design decisions
reviewable before installer code hardens them accidentally.

- [x] Rust workspace and dependency-free bootstrap CLI
- [x] product scope, architecture, security, testing, and governance docs
- [x] ADR/RFC process and automation templates
- [x] configuration discovery and precedence
- [x] `package.json` parser with fixtures and structured errors
- [x] machine-readable CLI output contract
- [x] supported platform tier policy

Exit: contributors can build and test locally; foundational open questions have
owners or RFCs; `corexpm --help`, `--version`, and `doctor` are reliable.

## Phase 1 — registry and resolver (`0.2`)

- [x] npm metadata client with caching, authentication redaction, and mock registry
- [x] npm-compatible semantic version range suite
- [x] typed dependency graph and deterministic candidate selection
- [x] transitive, development, optional, peer, OS, and CPU dependencies
- [x] `corexpm info`, resolution diagnostics, and initial `why`

Exit: a manifest resolves reproducibly to a verified in-memory graph across the
compatibility fixture corpus; no package content is installed yet.

## Phase 2 — Corex CAS (`0.3`)

- [x] streamed tarball fetch, expected-integrity verification, safe extraction
- [x] immutable package-level CAS with atomic commits and cross-process locks
- [x] registry metadata and tarball caches with offline/prefer-offline behavior
- [x] store validation, path, status, and stats commands
- [x] crash recovery and garbage-collection design validation

Exit: packages can be fetched once and safely reused across processes and
projects; corruption and unsafe archives fail without contaminating the store.

## Phase 3 — isolated installer (`0.4`)

- [x] isolated tree planner and strict dependency visibility
- [x] cross-platform link strategy and package binary links
- [x] transactional reconciliation and project state
- [x] install, add, remove, list, and initial run/exec command plumbing
- [x] cold, warm, offline, and reconciliation benchmarks

Exit: ordinary script-free npm projects run from an isolated install on tier-1
platforms, and repeat installs primarily reuse the global store.

## Phase 4 — deterministic lockfile (`0.5`)

- [x] accepted lockfile RFC and canonical serializer
- [x] graph/importer/peer/platform representation
- [x] atomic updates, merge-conflict diagnostics, and schema migration framework
- [x] `--frozen` and `corexpm ci`

Exit: equivalent inputs produce byte-identical lockfiles and clean machines
reproduce the same validated graph.

## Phase 5 — bins, scripts, and Corex Guard baseline (`0.6`)

- [x] npm lifecycle order and executable shim compatibility
- [x] deny-by-default dependency script policy
- [x] explicit trust decisions and non-interactive behavior
- [x] immutable-source build overlays and native artifact cache keys
- [x] `permissions` and trust explanation UX

Exit: supported lifecycle behavior never executes before integrity and policy
checks; common native/tooling packages work through explicit trust.

## Phase 6 — workspaces (`0.7`)

- workspace discovery and local protocol resolution
- topological recursive execution with bounded concurrency
- filtering, changed, and affected package calculations
- monorepo install and scheduler performance fixtures

Exit: representative monorepos install and execute deterministically with clear
cycle, failure, and filtering behavior.

## Phase 7 — security and audit expansion (`0.8`)

- advisory audit pipeline and policy reporting
- platform-specific capability enforcement experiments
- archive, lockfile, manifest, semver, and resolver fuzzing
- signed build provenance and security review readiness

Exit: threat-model mitigations are tested, enforcement limits are transparent,
and release artifacts have a verifiable supply chain.

## Phase 8 — migration and compatibility (`0.9`)

- npm, pnpm, Yarn, and Bun lockfile migration where formats permit
- no automatic deletion of foreign lockfiles
- broad framework/native/CLI compatibility matrix
- stable error-code catalog and migration guide
- performance and disk reports using published methods

Exit: target projects migrate predictably, incompatibilities are documented,
and release-candidate telemetry is not required to make compatibility claims.

## Phase 9 — stable release (`1.0`)

- stable CLI and config contract
- documented lockfile support policy
- release signing, checksums, installers, and rollback guidance
- production documentation and completed independent security review
- sustained compatibility and performance gates

Post-1.0 work may include registry federation, sandboxed WASM plugins, stronger
workspace intelligence, and blob-level deduplication. Corex Virtual is targeted
for a later major version only after isolated-mode compatibility is mature.

