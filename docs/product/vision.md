# Product vision

CorexPM is a native package manager for JavaScript and TypeScript projects with
one central promise: package content downloaded for one project should be
safely reusable by every compatible project on the machine.

## Positioning

CorexPM aims for:

- npm ecosystem compatibility;
- pnpm-class disk efficiency through shared immutable content;
- strict dependency isolation comparable to isolated/PnP-style layouts;
- native, concurrent execution with measured performance; and
- a stronger default security model for dependency lifecycle scripts.

CorexPM does not claim mature package managers are deficient, and it will not
claim superiority without published, reproducible evidence.

## Principles

1. Install once, reuse everywhere.
2. Preserve npm ecosystem compatibility.
3. Make dependency boundaries strict by default.
4. Verify content and evaluate policy before scripts execute.
5. Produce deterministic, reproducible installs.
6. Treat monorepos as a first-class use case.
7. Build performance-sensitive paths in native Rust.

## Product pillars

**Corex CAS** stores immutable package objects globally and links projects to
them. **Corex Graph** models dependency and workspace relationships. **Corex
Guard** controls install-script trust and capabilities. **Corex Virtual** is a
future optional loader-based installation mode.

The first release focuses on CAS, resolution, npm compatibility, isolated
installation, lockfiles, and basic Guard behavior. Graph automation and
Virtual mode follow only after compatibility is excellent.

