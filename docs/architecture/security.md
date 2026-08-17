# Corex Guard architecture

CorexPM treats dependency lifecycle execution as a granted capability, not an
automatic side effect of fetching a package.

## Baseline behavior

- Verify archive integrity and extraction safety first.
- Deny dependency lifecycle scripts by default.
- In an interactive session, policy may allow a trust prompt.
- In CI or another non-interactive session, unresolved prompts become denials.
- Project scripts explicitly invoked by the user are distinct from dependency
  install scripts, but still inherit environment and secret-redaction rules.

## Proposed project policy

```toml
[scripts]
default = "deny"

[trusted]
packages = ["esbuild", "sharp"]

[network]
default = "deny"

[filesystem]
outside_project = "deny"
```

The syntax is provisional. A lockfile must record outcomes that affect the
installed graph or artifacts, while personal trust decisions must not leak
secrets into the repository.

## Capability model

Long-term capability categories include process spawning, network access,
project filesystem writes, writes outside the project, native execution, and
environment access. Platform sandbox guarantees vary; the CLI must distinguish
enforced controls from advisory/audited controls.

Native build output is keyed by package content, OS, CPU, Node ABI, and build
inputs and stored separately from immutable package source.

## UX requirements

`corexpm permissions` explains effective policy per package. Denials name the
package, lifecycle hook, requested action, relevant policy source, and safe
next steps. Flags that bypass Guard must be explicit, noisy, and unsuitable as
silent defaults.

