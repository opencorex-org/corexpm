# Corex Guard security architecture

```mermaid
flowchart TB
    Package["Untrusted dependency package"] --> Integrity{"Integrity valid?"}
    Integrity -->|"No"| Reject["Reject and quarantine temporary data"]
    Integrity -->|"Yes"| Extract{"Archive safe to extract?"}
    Extract -->|"No"| Reject
    Extract -->|"Yes"| Scripts{"Lifecycle scripts declared?"}
    Scripts -->|"No"| Link["Link immutable package"]
    Scripts -->|"Yes"| Policy["Evaluate project + user + CI policy"]

    Policy --> Decision{"Effective decision"}
    Decision -->|"Deny"| Denied["Structured denial; no execution"]
    Decision -->|"Prompt in interactive mode"| Prompt["Explain package, hook, requested capabilities"]
    Decision -->|"Allow"| Sandbox["Restricted runner / audited boundary"]
    Prompt -->|"Deny"| Denied
    Prompt -->|"Allow once or trust"| Sandbox

    Sandbox --> Overlay["Writable project/build overlay"]
    Sandbox -.->|"never writable"| CAS["Global immutable CAS"]
    Overlay --> Artifact["Artifact keyed by package + OS + CPU + Node ABI + flags"]
    Artifact --> Link
```

```mermaid
flowchart LR
    ProjectPolicy["Project policy"] --> Effective["Effective capability decision"]
    UserTrust["User trust store"] --> Effective
    CommandFlags["Explicit command flags"] --> Effective
    Environment["Interactive vs CI"] --> Effective
    PackageRequest["Package lifecycle request"] --> Effective

    Effective --> Process["process.spawn"]
    Effective --> Network["network.access"]
    Effective --> ProjectFS["filesystem.project.write"]
    Effective --> OutsideFS["filesystem.outside_project"]
    Effective --> Native["native.execute"]
    Effective --> Env["environment.read"]
```

The CLI must label each capability as enforced, detected, or advisory on the
current platform; policy text must never imply a sandbox guarantee that the OS
cannot provide.

