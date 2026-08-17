# Workspace architecture

CorexPM discovers workspace members from explicit, deterministic glob patterns
in `corex.toml` and supported `package.json` workspace syntax.

## Graph

Workspace packages are nodes; local dependency relationships are directed
edges. The graph supports:

- topological recursive execution;
- cycle detection and clear diagnostics;
- include/exclude filters;
- dependency and dependent expansion;
- affected-package calculation from changed paths; and
- bounded parallel scheduling of independent tasks.

Commands must produce a stable order when tasks are otherwise independent.
Failure policy (`fail-fast` versus continuing independent branches) must be
explicit and machine-readable output must preserve per-package results.

## Initial command surface

```text
corexpm workspace list
corexpm run test --all
corexpm run build --workspace @app/web
corexpm changed
corexpm run test --changed
corexpm run build --affected
```

Remote task caching and general build-system features belong to a future
CorexBuild product and are not part of the package-manager v1.

