# System context

```mermaid
flowchart LR
    Developer["Developer"]
    CI["CI runner"]
    Node["Node.js and JavaScript tools"]
    Npm["npm-compatible registry"]
    Git["Git or local package sources (later)"]
    OS["Operating system and filesystem"]

    subgraph Corex["CorexPM trust boundary"]
        CLI["corexpm native CLI"]
        Guard["Corex Guard"]
        CAS["Global immutable CAS"]
        Project["Project dependency layout"]
    end

    Developer -->|"commands and policy"| CLI
    CI -->|"frozen, non-interactive commands"| CLI
    CLI -->|"metadata and tarballs"| Npm
    Git -.->|"future source support"| CLI
    CLI --> Guard
    CLI --> CAS
    CLI --> Project
    CAS -->|"verified package content"| Project
    Guard -->|"approved lifecycle execution"| Project
    Node -->|"resolves declared packages"| Project
    CLI <-->|"locks, links, atomic renames"| OS
```

The registry supplies untrusted bytes. CorexPM verifies and applies policy
before those bytes can influence execution. Node.js consumes the project
layout, but CorexPM itself does not require Node.js to start.

