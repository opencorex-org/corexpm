# Rust crate dependency design

```mermaid
flowchart TB
    CLI["corex-cli"] --> Core["corex-core"]
    CLI --> Errors["corex-errors"]

    Core --> Resolver["corex-resolver"]
    Core --> Registry["corex-registry"]
    Core --> Installer["corex-installer"]
    Core --> Lockfile["corex-lockfile"]
    Core --> Workspace["corex-workspace"]
    Core --> Security["corex-security"]
    Core --> Config["corex-config"]

    Resolver --> Graph["corex-graph"]
    Resolver --> Semver["corex-semver"]
    Resolver --> Manifest["corex-manifest"]
    Resolver --> Registry

    Registry --> Fetch["corex-fetch"]
    Registry --> Cache["corex-cache"]
    Fetch --> Cache

    Installer --> Store["corex-store"]
    Installer --> Linker["corex-linker"]
    Installer --> Scripts["corex-scripts"]
    Scripts --> Policy["corex-policy"]
    Security --> Policy
    Workspace --> Graph

    Resolver -.-> Errors
    Registry -.-> Errors
    Installer -.-> Errors
    Lockfile -.-> Errors
    Workspace -.-> Errors
    Security -.-> Errors
```

Solid arrows show domain dependency direction. Dotted arrows represent shared
diagnostic types. The CLI owns presentation; domain crates do not print or
depend back on orchestration. Crates are created only when their milestone has
stable responsibilities and tests.

