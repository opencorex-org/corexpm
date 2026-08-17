# ADR 0002: Begin with a package-level immutable CAS

- Status: accepted
- Date: 2026-08-17

## Context

CorexPM's primary product promise requires reusing identical package content
across projects. File-level deduplication can save more space but greatly
increases metadata, materialization, and recovery complexity.

## Decision

V1 stores canonical immutable package objects keyed by cryptographic content
identity. Registry integrity remains separately recorded. File-level blob
deduplication is deferred until measurements justify it.

## Consequences

Projects can reuse package payloads through links while the store retains a
simple inspectable filesystem representation. Lifecycle and native builds need
writable overlays. Garbage collection operates on package objects and project
references. Some identical files across different packages remain duplicated.

