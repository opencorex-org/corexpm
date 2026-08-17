# Crate map

Crates are introduced incrementally. A directory name in this document is an
architectural allocation, not evidence that the feature exists.

| Crate | Responsibility | Target milestone |
| --- | --- | --- |
| `corex-cli` | argument parsing, output modes, command dispatch | 0.1 |
| `corex-core` | use-case orchestration and service composition | 0.1 |
| `corex-errors` | stable diagnostic codes and presentation model | 0.1 |
| `corex-config` | config discovery, precedence, validated settings | 0.1–0.2 |
| `corex-manifest` | npm `package.json` domain model and parser | 0.1–0.2 |
| `corex-semver` | npm-compatible range semantics | 0.2 |
| `corex-graph` | compact resolved dependency graph | 0.2 |
| `corex-resolver` | candidates, constraints, peers, optionals | 0.2 |
| `corex-registry` | registry abstraction and npm implementation | 0.2 |
| `corex-fetch` | concurrent streamed fetching and verification | 0.3 |
| `corex-cache` | conditional metadata and archive cache | 0.3 |
| `corex-store` | immutable CAS, indexes, locking, garbage collection | 0.3 |
| `corex-installer` | transaction planning and reconciliation | 0.4 |
| `corex-linker` | isolated `node_modules` materialization | 0.4 |
| `corex-lockfile` | deterministic `corex.lock` read/write/validation | 0.5 |
| `corex-policy` | trust and capability policy evaluation | 0.6 |
| `corex-scripts` | lifecycle and package script process control | 0.6 |
| `corex-security` | integrity and security orchestration | 0.6–0.8 |
| `corex-workspace` | discovery, graph, filtering, scheduling | 0.7 |
| `corex-audit` | advisory and policy reporting | 0.8 |
| `corex-plugin` | versioned sandboxed plugin contract | post-1.0 |

Before adding a crate, document its owned data, public interface, failure
model, and forbidden dependencies. Avoid an `every crate -> every crate`
graph; cross-cutting presentation belongs at the CLI boundary.

