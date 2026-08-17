# CorexPM documentation

This directory is the source of truth for product intent and technical
decisions. If implementation and documentation disagree, treat the mismatch as
a bug and resolve it explicitly.

## Start here

- [Vision and principles](product/vision.md)
- [Scope and non-goals](product/scope.md)
- [Architecture overview](architecture/overview.md)
- [Architecture diagram pack](architecture/diagrams/README.md)
- [`node_modules` size-reduction plan](architecture/node-modules-size-reduction.md)
- [Roadmap](roadmap/ROADMAP.md)
- [Development guide](contributing/development.md)

## Specifications

- [Resolver](architecture/resolver.md)
- [Content-addressed store](architecture/store.md)
- [Installation and linker](architecture/installer.md)
- [Lockfile](architecture/lockfile.md)
- [Security and lifecycle policy](architecture/security.md)
- [Workspaces](architecture/workspaces.md)
- [Crate map](architecture/crate-map.md)
- [npm compatibility](compatibility/npm-compatibility.md)
- [Threat model](security/threat-model.md)
- [Testing strategy](testing/strategy.md)

## Decision process

- [Architecture Decision Records](adr/README.md)
- [Request for Comments](rfcs/README.md)
- [Release process](contributing/releases.md)

Specifications use normative words such as MUST, SHOULD, and MAY in their
usual RFC sense. Pre-1.0 documents can change, but accepted changes must be
captured in version control and persistent-format changes require migration
consideration.
