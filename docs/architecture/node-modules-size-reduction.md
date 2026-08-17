# `node_modules` size-reduction plan

## Objective

Reduce physical disk consumption across projects without weakening npm
compatibility, dependency isolation, integrity, or crash safety. The v1 design
optimizes **cross-project reuse** first; it does not promise that every single
project's visible `node_modules` path count will be small.

## Storage accounting model

Let:

- `P(i)` be the physical bytes of unique immutable package object `i`;
- `R(i)` be the number of project references to object `i`;
- `L(p)` be link and virtual-tree metadata for project `p`;
- `B(k)` be platform-specific build artifact bytes for compatible build key
  `k`; and
- `C` be compressed registry/download cache bytes.

A copy-per-project layout approaches:

```text
traditional physical bytes = sum(P(i) * R(i))
```

The CorexPM package-level design approaches:

```text
Corex physical bytes = sum(P(i)) + sum(L(p)) + sum(B(k)) + C
```

Savings are measured from filesystem allocation, not calculated solely from
logical file length. Hardlinks, reflinks, sparse files, compression, filesystem
block size, and metadata can otherwise produce misleading results.

## V1 architecture

### 1. One immutable package object per canonical content identity

Registry tarballs are streamed, verified, safely extracted, normalized, and
committed to `~/.corex/store/v1/packages/sha256/<prefix>/<hash>`. If two package
resolutions produce identical canonical content, they share one physical
object even if several projects reference it.

The content key must account for every normalization rule. CorexPM keeps npm's
expected integrity separately so a local canonical identity never replaces
registry verification.

### 2. Project dependency trees contain references

Each project receives an isolated graph under `node_modules/.corex`. Package
payload paths use the safest supported link strategy:

1. symlink or directory junction when semantically correct;
2. reflink/clone when packages or tools require a materialized writable view;
3. hardlink only with protections that preserve CAS immutability; and
4. copy as a correctness fallback on unsupported filesystems.

The selected strategy is recorded in project state and surfaced by
`corexpm doctor`. A fallback may reduce savings but must not break the install.

### 3. Strict graph-based exposure

Only declared dependency edges are linked into each virtual package instance.
Strictness does not itself remove CAS bytes, but it avoids accidental reliance
on hoisted packages and makes a compact deterministic tree possible.

### 4. Shared caches without double-counting

Registry metadata and compressed tarballs live outside projects. The cache has
its own retention policy because keeping both an archive and extracted CAS
object trades offline speed for physical space. Configuration will provide
balanced, offline-heavy, and low-disk profiles; stats report cache separately.

### 5. Native build artifacts outside immutable source

Lifecycle/native builds never modify a CAS package. Reusable output is keyed by:

```text
package content hash
+ operating system and CPU
+ Node ABI/runtime
+ build flags and relevant environment
+ Corex build schema version
```

Only exactly compatible keys are reused. Project-specific or untrusted output
stays in a project overlay and is not claimed as globally reusable.

### 6. Reference-aware garbage collection

Removing a dependency drops a project reference, not the shared object
immediately. `corexpm store prune` computes reachable objects from registered
project state, active transactions, pins, and a grace period. It produces a
plan before deletion and never follows untrusted links outside store roots.

## Project layout target

```text
my-project/
  package.json
  corex.lock
  corex.toml
  .corex/
    state.bin          # manifest, lockfile, platform and layout hashes
    graph.bin          # compact resolved/link graph cache
  node_modules/
    react -> .corex/react@19.1.0/node_modules/react
    vite  -> .corex/vite@7.0.0/node_modules/vite
    .corex/
      react@19.1.0/
        node_modules/react -> ~/.corex/store/.../react
      vite@7.0.0/
        node_modules/vite -> ~/.corex/store/.../vite
        node_modules/rollup -> ../../rollup@.../node_modules/rollup
```

Actual encodings will avoid embedding unsafe characters and will include peer
context, registry identity, and platform identity where these change a package
instance.

## Measurement commands and output

Planned `corexpm store stats` fields:

```text
Unique package objects
Package references
Physical CAS bytes
Logical referenced package bytes
Project link/state bytes
Compressed cache bytes
Build artifact bytes
Unreferenced reclaimable bytes
Measured physical bytes avoided
Measured reuse ratio
Link strategy distribution
```

The primary metrics are:

```text
reuse ratio = logical referenced package bytes / physical CAS bytes
physical bytes avoided = max(0, logical referenced bytes - physical shared bytes)
```

Reports must label estimated values and filesystem limitations. They must not
claim that cache bytes are savings, or count the same hardlinked blocks as
separate physical allocations.

## Implementation phases

| Phase | Deliverable | Acceptance evidence |
| --- | --- | --- |
| 0.2 | canonical package identity design | deterministic normalization fixtures |
| 0.3 | immutable package CAS | two projects reuse one verified object |
| 0.3 | cache profiles and offline behavior | network/cache/store byte accounting |
| 0.4 | cross-platform isolated links | Linux/macOS/Windows physical-byte tests |
| 0.4 | tiny project state and fast reconciliation | unchanged install performs no payload writes |
| 0.6 | native build overlay/artifact cache | incompatible ABI outputs never collide |
| 0.8 | safe reference-aware pruning | crash/failure/concurrency and grace-period tests |
| post-1.0 | optional file/blob dedup experiment | benchmark proves net benefit before adoption |

## Benchmarks

Test small, medium, large, and monorepo fixtures in these states:

1. first cold project install;
2. second identical project install;
3. overlapping dependency graphs with different versions;
4. warm exact reinstall;
5. offline install on a new project;
6. native dependency install across matching and different ABIs;
7. removal followed by prune; and
8. filesystems where preferred linking is unavailable.

Record wall time, CPU, peak RAM, network bytes, payload writes, logical bytes,
allocated physical blocks, file count, link count, store/cache/build bytes, and
competitor versions. Publish raw runs and environment details.

## Risks and controls

| Risk | Control |
| --- | --- |
| lifecycle script mutates shared content | read-only CAS plus writable overlay |
| tools resolve real paths unexpectedly | framework compatibility fixtures and explicit fallbacks |
| Windows link permissions/path limits | junction-aware linker, encoded short paths, Windows CI |
| hardlinks allow accidental mutation | avoid or protect hardlinks; validate CAS health |
| cache duplicates CAS content indefinitely | separate accounting, configurable retention, safe pruning |
| reference index becomes stale | rebuildable index and conservative reachability scan |
| concurrent writers corrupt an object | hash-scoped lock, unique temp path, atomic rename |
| disk-full leaves broken project | staged tree, atomic activation, last-good rollback |
| cross-device rename/link fails | detect device boundaries and choose safe fallback |
| file-level dedup costs more metadata than it saves | defer until measured on real workloads |

## Non-goals

- Mutating package sources to remove documentation, types, licenses, or maps.
- Compressing files that Node.js must read directly in isolated mode.
- Deleting unused content automatically without a safe reachability model.
- Reporting a universal percentage reduction.
- Making virtual mode a prerequisite for v1 disk efficiency.

The architecture diagrams for this plan are [CAS storage](diagrams/04-cas-storage.md),
[isolated layout](diagrams/05-isolated-layout.md), and
[disk reduction](diagrams/06-disk-reduction.md).

