# Scope and non-goals

## Version 1 scope

CorexPM v1 is a native client for the existing npm ecosystem. Its essential
deliverables are:

- npm manifest and registry protocol support;
- deterministic dependency resolution, including peers and optionals;
- immutable, integrity-verified global package storage;
- a strict but broadly compatible isolated `node_modules` linker;
- a deterministic, merge-friendly `corex.lock`;
- lifecycle script trust controls introduced with script support;
- frozen, offline, and prefer-offline installs;
- workspace discovery and recursive execution; and
- migration from common lockfile formats without deleting source files.

## Explicit non-goals for v1

- a JavaScript runtime, compiler, bundler, or transpiler;
- an OpenCorex package registry;
- remote build cache or cloud service;
- Node version management;
- a full IDE or editor extension;
- file-level cross-package deduplication;
- loader-based virtual installation;
- general monorepo build caching; and
- arbitrary in-process native plugins.

These exclusions protect the resolver, store, installer, compatibility, and
security work that must be correct before the ecosystem expands.

## Success criteria

The v1 release requires correctness against a published compatibility corpus,
transactional installs on supported platforms, no known high-severity security
issues, deterministic lockfile output, measurable reuse from the global store,
and documented performance results for cold, warm, offline, and reconciliation
scenarios.

