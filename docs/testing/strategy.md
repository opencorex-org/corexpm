# Testing strategy

CorexPM tests correctness under concurrency, crashes, hostile inputs, and
ecosystem edge cases—not only happy-path commands.

## Test layers

- **Unit:** domain types, canonicalization, semver, graph invariants, policy.
- **Property:** deterministic ordering, parser round trips, graph invariants.
- **Integration:** resolver/fetch/store/linker transactions using temporary
  project and Corex roots.
- **Compatibility:** npm behaviors, frameworks, native packages, workspaces.
- **Adversarial:** malicious archives, symlinks, corrupted caches, redaction.
- **Fault injection:** interrupted streams, disk-full writes, lock contention,
  process termination, failed atomic activation.
- **Fuzz:** manifests, lockfiles, semver, resolver inputs, archive metadata.
- **Performance:** cold, warm, offline, reconciliation, disk, RAM, and files.

## Determinism

Tests randomize registry response order, filesystem enumeration order, task
completion, and hash-map seeds where possible. Equivalent logical input must
produce the same graph and lockfile bytes.

## Platform tiers

Tier policy will be accepted before 0.4. CI must eventually cover Linux,
macOS, and Windows plus case-sensitive/insensitive filesystems and supported
architectures. Platform exclusions require user-facing detection and docs.

## Benchmark integrity

Reports capture hardware, OS, filesystem, runtime, competitor versions, cache
state, commands, samples, dispersion, network conditions, logical/physical
bytes, and raw results. CorexPM publishes reproducible scripts and avoids fixed
disk-saving or speed claims.

