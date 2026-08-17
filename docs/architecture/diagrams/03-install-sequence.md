# Install sequence

```mermaid
sequenceDiagram
    autonumber
    actor User
    participant CLI as CorexPM CLI
    participant Lock as Lockfile/Resolver
    participant Registry as npm Registry
    participant Store as Corex CAS
    participant Linker as Transactional Linker
    participant Guard as Corex Guard
    participant FS as Project Filesystem

    User->>CLI: corexpm install
    CLI->>FS: read package.json, corex.toml, corex.lock
    CLI->>Lock: validate or resolve dependency graph
    Lock-->>CLI: deterministic resolved graph

    loop each unique package object with bounded concurrency
        CLI->>Store: contains(content identity)?
        alt verified store hit
            Store-->>CLI: immutable object path
        else store miss
            CLI->>Registry: fetch metadata/tarball stream
            Registry-->>CLI: untrusted bytes + expected integrity
            CLI->>CLI: stream, hash, safely extract, verify
            CLI->>Store: atomic immutable commit
            Store-->>CLI: immutable object path
        end
    end

    CLI->>Linker: plan isolated dependency tree
    Linker->>FS: build complete staging tree
    Linker->>Guard: evaluate dependency lifecycle requests
    alt policy approved
        Guard->>FS: run in writable project/build overlay
    else denied or unresolved in CI
        Guard-->>CLI: structured security diagnostic
    end
    Linker->>FS: validate and atomically activate tree
    CLI->>FS: atomically write lockfile/state when allowed
    CLI-->>User: timings, cache/store hits, disk bytes, result
```

Package fetches are concurrent, but graph ordering, lockfile bytes, and final
layout remain deterministic. Frozen mode stops before mutation if inputs do not
match the lockfile.

