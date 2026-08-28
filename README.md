<div align="center">
  <img src="assets/logo.png" alt="CorexPM Logo" width="120" />
  <h1>CorexPM</h1>
  <p><strong>Download once. Store once. Use everywhere.</strong></p>

  <p>
    <a href="https://github.com/opencorex-org/corexpm/actions/workflows/ci.yml"><img src="https://img.shields.io/github/actions/workflow/status/opencorex-org/corexpm/ci.yml?branch=main&label=CI&logo=github" alt="CI Status" /></a>
    <a href="LICENSE"><img src="https://img.shields.io/badge/license-MIT-blue.svg" alt="License" /></a>
    <img src="https://img.shields.io/badge/version-0.1.0--dev-orange" alt="Version" />
    <img src="https://img.shields.io/badge/rust-2021-blue?logo=rust" alt="Rust Edition" />
    <img src="https://img.shields.io/badge/ecosystem-npm%20%7C%20pnpm-cb3837?logo=npm" alt="Ecosystem" />
  </p>
</div>

CorexPM is an experimental, secure, disk-efficient package manager for the
JavaScript and TypeScript ecosystem. It is designed around an immutable global
content-addressed store, strict dependency isolation, deterministic installs,
and explicit lifecycle-script trust while retaining npm ecosystem
compatibility.

CorexPM is in the specification and bootstrap stage. It is **not ready for
production use** and does not install packages yet.

## Project status

The current `0.1.0-dev` scaffold provides:

- a buildable Rust workspace and dependency-free CLI bootstrap;
- stable crate boundaries for the first implementation milestones;
- architecture, compatibility, security, and lockfile specifications;
- roadmap, contribution, governance, and release processes; and
- CI, issue, pull request, and RFC templates.

The first usable milestone is an npm-compatible isolated installer backed by
Corex CAS. See the [roadmap](docs/roadmap/ROADMAP.md).

## Try the bootstrap CLI

Prerequisites: a recent stable Rust toolchain.

```sh
cargo run -p corex-cli -- --help
cargo run -p corex-cli -- --version
cargo run -p corex-cli -- doctor
```

The commands listed by `--help` are the planned public interface. Commands not
implemented yet exit with a clear development-stage error.

## Architecture

The v1 data path is:

```text
package.json -> resolver -> dependency graph -> corex.lock
                                      |
                         registry -> fetch -> verify
                                      |
                                  Corex CAS
                                      |
                           isolated node_modules
                                      |
                              approved scripts
```

CorexPM's four long-term pillars are:

- **Corex CAS** — immutable package content shared across projects;
- **Corex Graph** — dependency and workspace graph intelligence;
- **Corex Guard** — lifecycle-script trust and package policy controls; and
- **Corex Virtual** — an optional future `node_modules`-less mode.

Start with the [documentation index](docs/README.md) and the
[architecture overview](docs/architecture/overview.md). The complete visual
design is in the [architecture diagram pack](docs/architecture/diagrams/README.md),
with the detailed [`node_modules` size-reduction plan](docs/architecture/node-modules-size-reduction.md).
See the evidence-based [package-manager comparison](docs/comparisons/package-managers.md)
for npm, pnpm, Yarn Modern, Bun, and CorexPM.

## Repository layout

```text
assets/       Project brand assets and logo
crates/       Rust implementation
packages/     Future TypeScript SDKs and Node integration
apps/         Future documentation and benchmark applications
integrations/ Ecosystem integration contracts and fixtures
tests/        Cross-crate and compatibility tests
benchmarks/   Reproducible performance scenarios
fuzz/         Parser, resolver, and archive fuzz targets
docs/         Specifications, ADRs, RFCs, and project plans
examples/     Example JavaScript projects
scripts/      Repository automation
```

## Contributing

Read [CONTRIBUTING.md](CONTRIBUTING.md), then choose a roadmap issue with a
clearly defined acceptance test. Architecture changes begin as an RFC or ADR;
compatibility claims require fixtures and reproducible evidence.

## License

CorexPM is available under the [MIT License](LICENSE).
