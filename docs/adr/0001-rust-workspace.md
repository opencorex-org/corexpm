# ADR 0001: Use a Rust workspace for the native core

- Status: accepted
- Date: 2026-08-17

## Context

CorexPM performs concurrent network I/O, archive extraction, hashing,
filesystem transactions, graph processing, and cross-platform distribution.
It should run without Node.js already being installed.

## Decision

Implement the CLI and performance/security-sensitive core as a Rust workspace.
Use TypeScript later for Node ecosystem integrations, SDKs, documentation, and
editor tooling. Keep important behavior behind project-owned interfaces.

## Consequences

The project gains a single native binary, strong memory safety, explicit error
handling, and fine filesystem control. It also assumes Rust expertise and must
manage compilation time and dependency growth. npm compatibility still needs
JavaScript fixtures and integration testing.

## Alternatives

TypeScript would simplify ecosystem prototyping but require a runtime and make
low-level filesystem control harder. Go would also provide an excellent native
binary, but Rust's ownership and type system better fit the intended immutable
store and transaction invariants.

