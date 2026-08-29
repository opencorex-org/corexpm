# npm compatibility strategy

npm compatibility is a tested contract, not an assumption. CorexPM initially
targets normal npm registry packages and Node's standard module resolution
through isolated `node_modules`.

## Feature matrix

| Area | V1 intent | Current status |
| --- | --- | --- |
| `dependencies` / `devDependencies` | required | **implemented (0.1–0.4)** |
| `optionalDependencies` | required | **implemented (0.2–0.4)** |
| `peerDependencies` and metadata | required | **implemented (0.2–0.4)** |
| `bin` and package scripts | required with policy | **implemented (0.6)** |
| `engines`, `os`, `cpu` | required | **implemented (0.2)** |
| npm workspaces | required | **implemented (0.7)** |
| npm registry auth/scopes | required | **implemented (0.2–0.3)** |
| aliases and dist-tags | required | **implemented (0.2)** |
| `file:` dependencies | required before 1.0 | unscheduled detail |
| Git/URL dependencies | compatibility target | unscheduled detail |
| npm / pnpm / Yarn / Bun lockfile import | migration target | **implemented (0.9)** |
| arbitrary npm CLI flags | not a goal | n/a |

## Test corpus

Compatibility tests use minimal fixtures for each manifest behavior, captured
real-world metadata served from a deterministic local registry, framework
smoke projects, native packages, workspace graphs, and platform-specific path
cases. Network-dependent tests are separated from hermetic required checks.

Every known deviation records affected package patterns, detection behavior,
workaround, owning issue, and target milestone. Unsupported behavior fails
with a specific diagnostic; CorexPM must not silently approximate npm semantics.

## Migration behavior

Foreign lockfiles (`package-lock.json`, `pnpm-lock.yaml`, `yarn.lock`, `bun.lock`) are detected and left **completely untouched**. Running `corexpm migrate` writes a canonical `corex.lock.json`, reports package import details, and explicitly preserves the original foreign lockfile.

