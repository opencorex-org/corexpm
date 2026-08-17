# Security Policy

## Supported versions

CorexPM is pre-release software. No version is currently supported for
production use. Security fixes will target the latest development branch until
a supported release channel is announced.

## Reporting a vulnerability

Do not open a public issue for a suspected vulnerability. Use GitHub's private
security advisory feature for `opencorex-org/corexpm`. If that channel is not
available, contact the maintainers through an organization-private channel and
include:

- affected version or commit;
- reproduction steps or proof of concept;
- expected impact;
- platform and filesystem details; and
- any suggested mitigation.

Maintainers should acknowledge reports within three business days and provide
a status update within seven business days. These are project targets, not a
service-level guarantee.

## Security invariants

Changes must preserve these non-negotiable properties:

- downloaded content is verified before it is committed or executed;
- global CAS entries are immutable;
- dependency scripts are denied unless policy permits them;
- archive extraction cannot escape its destination;
- credentials and registry tokens never appear in normal logs;
- project and global-store mutations are crash-safe and concurrency-safe; and
- lockfile and manifest mismatches fail frozen installs.

See [the threat model](docs/security/threat-model.md) and
[Corex Guard design](docs/architecture/security.md).

