# Dependency resolver specification

## Objective

Given a root manifest, workspace manifests, registry metadata, platform, and
optional existing lockfile, produce one deterministic dependency graph or a
structured explanation of why no valid graph exists.

## Inputs

- dependency, development, optional, and peer requirements;
- npm dist-tags and version metadata;
- OS, CPU, runtime version, and Node ABI where relevant;
- workspace packages and workspace protocol requirements;
- overrides/resolutions once specified; and
- an existing lockfile as a reproducibility preference, never unchecked truth.

## Model

Internal code uses identifiers (`PackageId`, `VersionId`, `NodeId`,
`RegistryId`, `IntegrityHash`) instead of repeating strings. Graph nodes
represent package instances, including peer context when it changes runtime
identity. Edges record dependency kind and original requirement.

Candidate ordering MUST be deterministic. Registry response order, concurrent
completion order, and local filesystem enumeration MUST NOT affect the result.

## Resolution order

1. Validate manifests and configuration.
2. Reuse compatible locked candidates where possible.
3. Enumerate versions matching registry, platform, and engine constraints.
4. Add transitive and optional constraints.
5. Resolve peer environments and report conflicts with dependency paths.
6. Produce a normalized graph with stable node and edge ordering.
7. Validate graph invariants before lockfile or installation planning.

Optional packages may be omitted only for a recorded supported reason. Peer
conflicts must not silently choose an invalid environment. Error diagnostics
should include the shortest useful requirement paths and an actionable command
such as `corexpm why` where appropriate.

## Initial exclusions

Git dependencies, exotic registry extensions, overrides, and legacy npm edge
cases are staged behind the compatibility matrix. Unsupported syntax must fail
clearly rather than resolve approximately.

