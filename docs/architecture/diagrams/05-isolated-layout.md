# Isolated `node_modules` layout

## Project view

```mermaid
flowchart TB
    App["my-app"] --> NM["node_modules"]
    NM --> ReactTop["react -> .corex/react@19.1.0/.../react"]
    NM --> ViteTop["vite -> .corex/vite@7.0.0/.../vite"]
    NM --> CX[".corex virtual package instances"]

    CX --> ReactInstance["react@19.1.0/node_modules/react"]
    CX --> ViteInstance["vite@7.0.0/node_modules/vite"]
    CX --> RollupEdge["vite@7.0.0/node_modules/rollup"]

    ReactInstance --> ReactCAS["Corex CAS: react content"]
    ViteInstance --> ViteCAS["Corex CAS: vite content"]
    RollupEdge --> RollupInstance["rollup package instance"]
    RollupInstance --> RollupCAS["Corex CAS: rollup content"]
```

## Dependency visibility

```mermaid
flowchart LR
    App["Application declares react + vite"]
    React["react declares no lodash"]
    Vite["vite declares rollup"]
    Rollup["rollup"]
    Lodash["lodash present elsewhere in store"]

    App -->|"allowed direct import"| React
    App -->|"allowed direct import"| Vite
    Vite -->|"allowed declared edge"| Rollup
    App -.->|"blocked: undeclared"| Rollup
    React -.->|"blocked: undeclared"| Lodash
```

Only graph edges become resolution paths. A package existing globally does not
make it importable, preventing phantom dependencies while retaining ordinary
Node.js-visible paths for declared dependencies.

