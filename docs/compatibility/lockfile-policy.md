# CorexPM Lockfile Support & Versioning Policy

This document defines the stability, format, and compatibility guarantees for `corex.lock.json`.

## Core Lockfile Guarantees

1. **Deterministic Canonical Formatting**:
   - `corex.lock.json` is formatted with 2-space indented JSON and a trailing newline.
   - Map keys (importers, packages, dependencies) are sorted alphabetically to guarantee identical byte output regardless of OS or execution concurrency.

2. **Schema Versioning & Forward Compatibility**:
   - Current supported schema version is `lockfileVersion: 1`.
   - CorexPM `1.x` releases will accept and read `lockfileVersion: 1`.
   - Newer lockfile schema versions exceeding the maximum supported version fail safely with diagnostic `CXLOCK0002` and actionable upgrade guidance.

3. **Frozen Install Invariant**:
   - Running `corexpm install --frozen` or `corexpm ci` strictly verifies that `package.json` requirements match `corex.lock.json`.
   - Frozen mode **never** mutates lockfile contents or fetches new un-locked package resolutions.

4. **Foreign Lockfile Non-Destruction Policy**:
   - CorexPM migration commands (`corexpm migrate`) read `package-lock.json`, `pnpm-lock.yaml`, `yarn.lock`, and `bun.lock` to generate `corex.lock.json`.
   - Foreign source lockfiles are treated as **read-only** inputs and are **never** mutated or automatically deleted.
