# Disk-reduction architecture

## Traditional repeated materialization

```mermaid
flowchart TB
    subgraph A["Project A: physical dependency payloads"]
        AR["React 5 MB"]
        AV["Vite 15 MB"]
        AT["TypeScript 25 MB"]
    end
    subgraph B["Project B: physical dependency payloads"]
        BR["React 5 MB"]
        BV["Vite 15 MB"]
        BT["TypeScript 25 MB"]
    end
    subgraph C["Project C: physical dependency payloads"]
        CR["React 5 MB"]
        CV["Vite 15 MB"]
        CT["TypeScript 25 MB"]
    end
```

Illustrative physical package payload: `3 × 45 MB = 135 MB`.

## CorexPM shared package representation

```mermaid
flowchart TB
    subgraph Store["Global Corex CAS: physical payloads"]
        R["React 5 MB"]
        V["Vite 15 MB"]
        T["TypeScript 25 MB"]
    end

    A["Project A: links + tiny graph state"] --> R
    A --> V
    A --> T
    B["Project B: links + tiny graph state"] --> R
    B --> V
    B --> T
    C["Project C: links + tiny graph state"] --> R
    C --> V
    C --> T
```

Illustrative physical package payload: `45 MB + link/state overhead`. The
example explains the mechanism; CorexPM reports measured filesystem allocation
rather than promising a fixed percentage.

## Reduction layers

```mermaid
flowchart LR
    L1["1. Package-level global CAS"] --> L2["2. Project links, not copies"]
    L2 --> L3["3. Shared compressed download cache"]
    L3 --> L4["4. Tiny project graph/state"]
    L4 --> L5["5. Separate keyed native build artifacts"]
    L5 --> L6["6. Reference-aware garbage collection"]
    L6 -.-> L7["Later: measured file/blob deduplication"]

    L1 --> Win1["Largest v1 saving"]
    L2 --> Win1
    L3 --> Win2["Avoid repeated network/cache bytes"]
    L4 --> Win3["Fast reconciliation"]
    L5 --> Win4["Reuse only ABI-compatible builds"]
    L6 --> Win5["Bound long-term store growth"]
```

See the [size-reduction plan](../node-modules-size-reduction.md) for formulas,
measurement, implementation phases, and failure handling.

