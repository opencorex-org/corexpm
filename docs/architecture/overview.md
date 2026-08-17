# Architecture overview

CorexPM is a Rust workspace organized around narrow domain crates. The CLI
depends on an application orchestration layer; domain services depend on shared
models, not on the CLI.

## V1 flow

```text
package.json + corex.toml + corex.lock
                  |
                  v
          manifest/config loaders
                  |
                  v
 npm metadata -> resolver -> resolved dependency graph
                  |                    |
                  |                    v
                  |               lockfile writer
                  v
        fetch -> integrity -> immutable CAS
                                   |
                                   v
                           isolated tree planner
                                   |
                                   v
                           transactional linker
                                   |
                                   v
                         policy-approved scripts
```

## Dependency direction

```text
corex-cli -> corex-core
                 |
                 +-> resolver -> graph -> manifest/semver
                 +-> registry -> fetch -> cache
                 +-> installer -> store/linker/scripts
                 +-> lockfile
                 +-> workspace
                 +-> security/policy

all user-facing layers -> corex-errors
```

The workspace starts with only `corex-cli`, `corex-core`, `corex-config`,
`corex-manifest`, and `corex-errors`. New crates are extracted when a roadmap
milestone gives them a stable responsibility and tests.

## Invariants

- A CAS object is immutable after a successful atomic commit.
- A project installation is prepared separately and swapped atomically.
- Registry bytes are never executed before integrity verification.
- Undeclared dependencies are not exposed to consumers by default.
- Platform-specific build output never mutates shared source objects.
- Persistent output has deterministic ordering and versioned schemas.
- Credentials are held separately from project configuration and are redacted.

See the component specifications and accepted ADRs for detailed decisions.

The [architecture diagram pack](diagrams/README.md) provides system, sequence,
storage, resolver, security, transaction, workspace, crate, and roadmap views.
The [`node_modules` size-reduction plan](node-modules-size-reduction.md) defines
the disk model, link strategy, measurement rules, and phased implementation.
