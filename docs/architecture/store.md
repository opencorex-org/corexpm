# Corex content-addressed store

## Goals

Store each verified immutable package representation once, safely reuse it
across projects, support concurrent processes, and measure actual physical
savings without corrupting shared content.

## Proposed layout

```text
~/.corex/
  store/v1/
    packages/sha256/ab/abcdef.../
    indexes/
    temp/
  cache/
    registry/
    tarballs/
  builds/
  security/
  logs/
```

The final key format will be frozen by an ADR. Registry integrity and Corex's
canonical content hash are distinct values and both must be retained.

## Write transaction

1. Stream registry content through the expected-integrity verifier.
2. Extract into a process-unique temporary directory.
3. Reject path traversal, unsafe links, special files, and platform-invalid
   paths.
4. Normalize the package representation and calculate its CAS key.
5. Validate metadata and file boundaries.
6. Atomically rename into place while holding the narrowest necessary lock.
7. If another process won the race, validate and reuse the committed object.

Committed objects are read-only by contract. Lifecycle scripts operate on a
project overlay or platform artifact directory, never the shared source.

## References and garbage collection

Project state records references to CAS keys. Removal makes unreferenced keys
eligible for garbage collection but does not delete them immediately.
`corexpm store prune` will use a grace period, active transaction markers, and
a repairable index. The filesystem remains authoritative enough to rebuild
metadata after a crash.

## Metrics

`corexpm store stats` should report unique package versions, logical referenced
bytes, physical bytes, reusable objects, and measured savings. It must not use
a marketing estimate where filesystem allocation can be measured.

