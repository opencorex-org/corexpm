# ADR 0003: Make isolated `node_modules` the default linker

- Status: accepted
- Date: 2026-08-17

## Context

CorexPM needs strict dependency visibility without demanding custom-loader
support from the whole JavaScript tool ecosystem. A virtual layout minimizes
disk use but creates a larger compatibility surface.

## Decision

V1 uses an isolated `node_modules` layout backed by Corex CAS. Only declared
direct dependencies are exposed at a package's resolution boundary. A
loader-based virtual mode is deferred to a future major version.

## Consequences

Most Node tooling sees familiar paths while phantom dependencies fail by
default. The linker must solve difficult peer-context, symlink, Windows, binary,
and native-build cases. Disk use is higher than a fully virtual layout but far
lower than repeated physical package copies where linking is available.

