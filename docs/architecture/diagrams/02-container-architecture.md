# Container architecture

```mermaid
flowchart TB
    User["CLI command"] --> CLI["CLI and output layer"]
    CLI --> Core["Core application orchestrator"]

    subgraph Resolve["Resolution plane"]
        Config["Config and manifest"]
        Registry["Registry metadata client"]
        Resolver["Deterministic resolver"]
        Graph["Dependency graph"]
        Lock["Lockfile engine"]
        Config --> Resolver
        Registry --> Resolver
        Resolver --> Graph
        Graph <--> Lock
    end

    subgraph Data["Package data plane"]
        Fetch["Streaming fetcher"]
        Verify["Integrity and archive verifier"]
        Cache["Metadata and tarball cache"]
        Store["Immutable package CAS"]
        Fetch --> Verify
        Fetch <--> Cache
        Verify --> Store
    end

    subgraph Materialize["Project materialization plane"]
        Planner["Isolated tree planner"]
        Linker["Transactional linker"]
        Bins["Executable shim linker"]
        State["Tiny project state"]
        Planner --> Linker
        Linker --> Bins
        Linker --> State
    end

    subgraph Execute["Execution and policy plane"]
        Policy["Corex Guard policy"]
        Scripts["Script runner"]
        Overlay["Writable build overlay"]
        Policy --> Scripts
        Scripts --> Overlay
    end

    Core --> Config
    Core --> Registry
    Core --> Fetch
    Core --> Planner
    Core --> Policy
    Graph --> Fetch
    Graph --> Planner
    Store --> Linker
    Linker --> Scripts
    Overlay --> State
```

Resolution describes exact package identities. The data plane obtains immutable
content. Materialization creates project references. Execution is a separate
policy-controlled stage and never mutates shared package source.

