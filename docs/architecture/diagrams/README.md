# Architecture diagrams

This pack is the visual design reference for CorexPM. Diagrams describe target
architecture; the [roadmap](../../roadmap/ROADMAP.md) remains the source of
truth for implementation status.

| Diagram | Design question |
| --- | --- |
| [System context](01-system-context.md) | Who and what interacts with CorexPM? |
| [Container architecture](02-container-architecture.md) | What are the main runtime components? |
| [Install sequence](03-install-sequence.md) | What happens during `corexpm install`? |
| [CAS storage](04-cas-storage.md) | How is one package reused by many projects? |
| [Isolated layout](05-isolated-layout.md) | How does strict `node_modules` linking work? |
| [Disk reduction](06-disk-reduction.md) | Where are physical bytes eliminated? |
| [Resolver](07-resolver.md) | How does a manifest become a deterministic graph? |
| [Corex Guard](08-security.md) | How are untrusted scripts controlled? |
| [Transactions](09-transaction-concurrency.md) | How are crashes and concurrent installs handled? |
| [Workspaces](10-workspace.md) | How are monorepo tasks ordered? |
| [Crate dependencies](11-crate-dependencies.md) | Which Rust crates may depend on which? |
| [Delivery roadmap](12-roadmap.md) | In what order is the architecture implemented? |

The implementation rules behind the disk diagrams are specified in the
[node_modules size-reduction plan](../node-modules-size-reduction.md).

