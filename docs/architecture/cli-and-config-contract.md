# CorexPM 1.0 Stable CLI and Configuration Contract

This document defines the stable, versioned Command-Line Interface (CLI) and Configuration precedence contract for CorexPM `1.0.0`.

## Configuration Precedence Order

CorexPM resolves configuration settings using strict priority ordering (highest to lowest):

1. **CLI Flag Overrides**: Explicit flags passed to `corexpm` commands (e.g. `--offline`, `--linker=symlink`, `--frozen`).
2. **Environment Variables**: Variables prefixed with `COREX_` (e.g. `COREX_HOME`, `COREX_OFFLINE`, `COREX_LINKER`).
3. **Local Project Configuration**: `.corexrc.toml` or `corex.config.toml` located in the project root.
4. **Global Configuration**: `~/.corex/config.toml` located in the user home directory.
5. **Default Settings**: Built-in default values (e.g. `linker = "isolated"`, `registry = "https://registry.npmjs.org"`).

## Stable CLI Command Matrix

| Command | Subcommands / Flags | Stability | Description |
| --- | --- | --- | --- |
| `corexpm install` | `[--frozen] [--offline] [--linker=<mode>]` | **Stable (1.0)** | Resolves dependencies, updates lockfile, and materializes `node_modules`. |
| `corexpm add` | `<pkg> [--dev] [--optional]` | **Stable (1.0)** | Adds a dependency to `package.json` and updates install state. |
| `corexpm remove` | `<pkg>` | **Stable (1.0)** | Removes a dependency from `package.json` and node_modules. |
| `corexpm ci` | | **Stable (1.0)** | Runs frozen deterministic install without mutating lockfile. |
| `corexpm migrate` | `[--json]` | **Stable (1.0)** | Imports foreign lockfiles (npm, pnpm, Yarn, Bun) preserving source files. |
| `corexpm store` | `path`, `stats`, `status`, `prune [--grace-period]` | **Stable (1.0)** | Manages the global immutable Content-Addressed Store (CAS). |
| `corexpm cache` | `path`, `stats`, `clean` | **Stable (1.0)** | Manages HTTP and metadata tarball cache directories. |
| `corexpm trust` | `approve`, `deny`, `list` | **Stable (1.0)** | Evaluates and manages lifecycle script execution policy. |
| `corexpm audit` | `[--severity=<level>] [--ignore=<id>]` | **Stable (1.0)** | Audits graph against advisory databases for security vulnerabilities. |
| `corexpm workspace` | `list`, `run`, `changed` | **Stable (1.0)** | Discovers monorepo workspaces and schedules graph tasks. |
| `corexpm doctor` | `[--json]` | **Stable (1.0)** | Diagnostics report of system environment, platform tier, and storage. |

## Stability Guarantees

- **No Breaking Syntax Changes**: Command names, flag options, and default behaviors are frozen for the 1.x series.
- **Machine-Readable Output**: All commands support `--json` output guaranteed to serialize standard `CliOutput::Success` or `Diagnostic` schemas.
