# Changelog

All notable changes to CorexPM will be documented here. The project follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/) and adheres to [Semantic Versioning](https://semver.org/).

## [1.0.0] - 2026-08-29

### Added

- **Immutable Content-Addressed Store (CAS)** (`corex-store`): Package-level content storage under `~/.corex/store/v1/packages/sha256/` with atomic staging commits, validation reports, physical metrics, and garbage collection (`corexpm store status`, `corexpm store prune`).
- **Isolated Node Modules Materializer** (`corex-linker`): Isolated symlinked `node_modules` materialization with dual CJS/ESM support and binary linking under `node_modules/.bin`.
- **Deterministic Lockfile Engine** (`corex-lockfile`): Canonical, versioned `corex.lock.json` parser, canonical 2-space JSON serializer, validation, frozen install enforcement (`corexpm install --frozen`, `corexpm ci`).
- **Foreign Lockfile Importers**: Non-destructive lockfile migration for npm (`package-lock.json`), pnpm (`pnpm-lock.yaml`), Yarn (`yarn.lock`), and Bun (`bun.lock`) via `corexpm migrate`. Foreign source lockfiles are preserved untouched.
- **Security & Integrity Engine** (`corex-security`): SHA-512 archive integrity verification, tarball path traversal rejection, build provenance attestation generator/verifier, and OS capability enforcement evaluation.
- **Corex Guard Lifecycle Policy** (`corex-policy` & `corex-scripts`): Opt-in policy engine for package lifecycle scripts (`preinstall`, `postinstall`), secret redaction, and policy trust management (`corexpm trust approve <pkg>`, `corexpm trust deny <pkg>`).
- **Advisory Security Auditor** (`corex-audit`): Vulnerability graph auditing with severity filtering (`corexpm audit --severity=high`).
- **Monorepo Workspace Intelligence** (`corex-workspace`): Workspace discovery, dependency wave graph analysis, filtering, and concurrency scheduling (`corexpm workspace list`, `corexpm run build --all`).
- **NPM Package Distribution & Node.js SDK** (`packages/corexpm`): Cross-platform Node binary wrapper (`npx corexpm`, `npm i -g corexpm`) and JavaScript/TypeScript SDK API (`require("corexpm")`).
- **Universal Installers**: POSIX shell installer (`scripts/install.sh`) and PowerShell installer (`scripts/install.ps1`).
- **Release Tooling**: Release automation script (`scripts/release.sh`), SHA-256 checksum generation (`SHA256SUMS`), and GitHub NPM release workflows.
