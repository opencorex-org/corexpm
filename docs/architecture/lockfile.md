# `corex.lock` specification goals

`corex.lock` records a complete resolved graph and the information required to
verify and reconstruct an install. The v1 format will be human-readable,
deterministic, merge-aware, and versioned.

## Required properties

- stable ordering independent of concurrency and platform enumeration;
- explicit schema version;
- original requirements plus exact resolutions;
- registry identity, tarball location policy, and integrity;
- dependency kinds, peer contexts, optional/platform decisions;
- workspace links represented portably;
- no credentials, absolute user paths, timestamps, or machine-specific noise;
- atomic writes and useful parse diagnostics; and
- forward-version rejection with upgrade guidance.

## Conceptual example

```yaml
lockfileVersion: 1
importers:
  .:
    dependencies:
      react: ^19.1.0
packages:
  react@19.1.0:
    resolution:
      registry: npm
      integrity: sha512-EXAMPLE
    dependencies: {}
```

This example is illustrative, not a frozen grammar. An RFC must choose the
encoding and canonicalization rules before 0.5.

## Frozen installs

`corexpm install --frozen` and `corexpm ci` fail if relevant manifest or
workspace requirements do not match the lockfile, if a required resolution is
missing, or if integrity data is unusable. Frozen mode never updates the
lockfile implicitly.

