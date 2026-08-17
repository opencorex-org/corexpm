# Corex CAS storage

```mermaid
flowchart TB
    Tar["Registry tarball stream"] --> Integrity["Verify registry integrity"]
    Integrity --> Safe["Safe extraction and normalization"]
    Safe --> Hash["Calculate canonical SHA-256 content key"]
    Hash --> Temp["Unique temporary object"]
    Temp --> Validate["Validate files, links, metadata, limits"]
    Validate --> Rename["Atomic rename under object lock"]

    subgraph Global["~/.corex/store/v1"]
        Obj["packages/sha256/ab/abcdef..."]
        Index["Rebuildable metadata index"]
    end

    Rename --> Obj
    Rename --> Index

    Obj --> A["Project A links"]
    Obj --> B["Project B links"]
    Obj --> C["Project C links"]

    Obj -.->|"source is never mutated"| OverlayA["Project/native build overlay A"]
    Obj -.->|"source is never mutated"| OverlayB["Project/native build overlay B"]
```

```mermaid
flowchart LR
    Name["react@19.1.0"] --> RegistryHash["Registry integrity: sha512-..."]
    RegistryHash --> Canonical["Canonical package hash: sha256-abcdef..."]
    Canonical --> Object["One physical immutable package object"]
    Object --> Ref1["project-a reference"]
    Object --> Ref2["project-b reference"]
    Object --> Ref3["project-c reference"]
```

Registry integrity authenticates the downloaded artifact expected by metadata.
The canonical content key identifies CorexPM's normalized immutable object.
Keeping both avoids conflating transport verification with local identity.

