# Installer and linker specification

The v1 default is an isolated `node_modules` layout. Applications see ordinary
top-level package paths while CorexPM controls which dependencies each package
can resolve.

```text
node_modules/
  react -> .corex/react@19.1.0/node_modules/react
  .corex/
    react@19.1.0/
      node_modules/react -> <immutable store object>
```

Exact path encoding must account for registries, peer contexts, scopes,
platform differences, and Windows constraints. It will be specified before the
linker format is treated as stable.

## Transaction

1. Acquire a project install lock.
2. Resolve and validate the graph.
3. Ensure all required store objects exist.
4. Build a complete tree in a transaction-specific staging location.
5. Link declared direct and transitive dependencies according to graph edges.
6. Link package binaries with collision checks.
7. Evaluate Corex Guard policy and run only approved lifecycle scripts.
8. Validate the result, atomically activate it, and persist tiny project state.
9. Roll back staging data on failure without damaging the last good install.

## Compatibility

Symlink, hardlink, reflink, and copy behavior differs across filesystems and
platforms. The linker will select safe strategies from detected capabilities,
with correctness ahead of savings. Windows junction behavior, antivirus races,
case sensitivity, maximum path length, and read-only stores require dedicated
tests.

Virtual mode is explicitly post-v1 and cannot weaken isolated-mode quality.

