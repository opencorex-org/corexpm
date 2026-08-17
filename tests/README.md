# Cross-project tests

The workspace keeps unit tests beside Rust code. This directory holds fixtures
and end-to-end suites that cross crate or process boundaries.

Planned groups:

```text
fixtures/       minimal package and registry datasets
integration/    multi-component command tests
compatibility/  npm behavior and real-world project cases
registry/       deterministic mock registry scenarios
security/       hostile archives, scripts, redaction, and policy
workspaces/     graph and scheduling cases
performance/    datasets shared by benchmark harnesses
```

Tests must redirect all project, cache, store, config, credential, and log paths
to temporary roots and must never touch the developer's real Corex data.

