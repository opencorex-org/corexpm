# Milestone operating plan

Every roadmap milestone is split into five parallel workstreams:

| Workstream | Required output |
| --- | --- |
| Specification | accepted semantics, formats, invariants, and non-goals |
| Implementation | smallest end-to-end vertical slice behind stable boundaries |
| Compatibility | npm ecosystem fixtures and documented deviations |
| Safety | threat review, failure injection, recovery tests, redaction checks |
| Evidence | benchmarks, disk measurements, and release acceptance report |

An issue is ready when it names the behavior, owning crate, inputs/outputs,
failure cases, tests, and documentation impact. A milestone is complete only
when its exit criteria pass on supported platforms; a command merely existing
is not completion.

## First implementation backlog

1. Define CLI output and exit-code conventions.
2. Specify configuration sources and precedence.
3. Add JSON parsing and a complete minimal `package.json` model.
4. Build a filesystem fixture harness with isolated temporary roots.
5. Implement an HTTP abstraction and deterministic mock npm registry.
6. Establish package and graph identifier types.
7. Import npm semver conformance cases with license provenance.
8. Draft the `corex.lock` RFC before serializer implementation.
9. Create failure-injection tests for atomic store commits.
10. Establish benchmark datasets and competitor version capture.

