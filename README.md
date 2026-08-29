<div align="center">
  <img src="assets/logo.png" alt="CorexPM Logo" width="120" />
  <h1>CorexPM</h1>
  <p><strong>Download once. Store once. Use everywhere.</strong></p>

  <p>
    <a href="https://github.com/opencorex-org/corexpm/actions/workflows/ci.yml"><img src="https://img.shields.io/github/actions/workflow/status/opencorex-org/corexpm/ci.yml?branch=main&label=CI&logo=github" alt="CI Status" /></a>
    <a href="LICENSE"><img src="https://img.shields.io/badge/license-MIT-blue.svg" alt="License" /></a>
    <img src="https://img.shields.io/badge/version-0.1.0--dev-orange" alt="Version" />
    <img src="https://img.shields.io/badge/rust-2021-blue?logo=rust" alt="Rust Edition" />
    <img src="https://img.shields.io/badge/ecosystem-npm%20%7C%20pnpm%20%7C%20yarn%20%7C%20bun-cb3837?logo=npm" alt="Ecosystem" />
  </p>
</div>

CorexPM is a secure, disk-efficient native package manager for the JavaScript and TypeScript ecosystem. It combines **npm ecosystem compatibility**, an **immutable global content-addressed store (CAS)**, **strict dependency isolation**, **deterministic lockfile resolution**, and **explicit lifecycle-script trust**.

---

## Installation Options

### 1. On-Demand via `npx` (No Install Needed)

Run CorexPM instantly without global installation:

```sh
npx corexpm doctor
npx corexpm migrate
```

### 2. Global Install via `npm`

Install CorexPM globally across your machine:

```sh
npm install -g corexpm
corexpm --help
```

### 3. Universal Shell Installers

**macOS & Linux (POSIX Shell)**:
```sh
curl -fsSL https://corex.dev/install.sh | sh
```

**Windows (PowerShell)**:
```powershell
iwr -useb https://corex.dev/install.ps1 | iex
```

### 4. Build from Source (Rust)

Prerequisites: Rust toolchain (`>=1.80`).

```sh
cargo build --workspace --release
./target/release/corexpm --help
```

---

## Complete CLI Usage Guide

### 1. Installing Packages & Managing Dependencies

#### Install All Project Dependencies
Resolves `package.json` requirements, downloads missing packages to the global CAS store, and materializes isolated `node_modules`:

```sh
corexpm install
```

#### Install Options
- **Frozen Mode (CI/CD)**: Fails if `package.json` and `corex.lock.json` disagree without mutating files.
  ```sh
  corexpm install --frozen
  ```
- **Offline Mode**: Operates strictly using cached store tarballs without network requests.
  ```sh
  corexpm install --offline
  ```
- **Linker Strategy**: Select between isolated symlinks (default), hoisted `node_modules`, or hardlinks.
  ```sh
  corexpm install --linker=isolated
  ```

#### Add New Dependency
Adds package requirements to `package.json` and updates `node_modules`:

```sh
# Add runtime dependency
corexpm add express

# Add development dependency
corexpm add typescript --dev

# Add optional dependency
corexpm add @swc/core-darwin-arm64 --optional
```

#### Remove Dependency
Removes package entries from `package.json` and cleans up `node_modules`:

```sh
corexpm remove express
```

#### CI Deterministic Pipeline Mode
Equivalent to `corexpm install --frozen` for production deployment scripts:

```sh
corexpm ci
```

---

### 2. Lockfile Migration (`npm`, `pnpm`, `Yarn`, `Bun`)

Migrate existing projects to CorexPM without losing existing resolution state:

```sh
corexpm migrate
```

> [!IMPORTANT]
> **Non-Destructive Guarantee**: `corexpm migrate` auto-detects `package-lock.json`, `pnpm-lock.yaml`, `yarn.lock`, or `bun.lock`, converts dependencies into `corex.lock.json`, and **preserves your original foreign lockfile untouched**.

For machine-readable JSON output:
```sh
corexpm migrate --json
```

---

### 3. Security Auditing & Vulnerability Scanning

Scan project dependency graphs against advisory databases:

```sh
# Audit all security advisories
corexpm audit

# Filter by minimum severity level
corexpm audit --severity high

# Ignore specific advisory IDs
corexpm audit --severity critical --ignore CX-ADV-2026-001
```

---

### 4. Lifecycle Script Policy (Corex Guard)

Lifecycle scripts (`preinstall`, `postinstall`, `build`) are **denied by default** for security:

```sh
# List effective script permissions
corexpm trust list

# Approve lifecycle script execution for a package
corexpm trust approve esbuild

# Deny lifecycle script execution for a package
corexpm trust deny suspicious-package
```

---

### 5. Workspace Monorepo Management

Schedule and execute commands across monorepo package graphs:

```sh
# List all workspace member packages
corexpm workspace list

# Run build script across all workspaces
corexpm run build --all

# Run test script only in changed workspace packages
corexpm changed
corexpm run test -w @app/web --concurrency 4
```

---

### 6. Content-Addressed Store & Cache Maintenance

Inspect physical disk space savings and manage the global store:

```sh
# Display CAS store statistics and physical vs logical disk savings
corexpm store status

# Reclaim unreferenced package objects
corexpm store prune --grace-period 86400

# View or clean HTTP metadata cache
corexpm cache status
corexpm cache clean
```

---

## Node.js & TypeScript SDK Usage

CorexPM provides a programmatic JavaScript/TypeScript SDK for build tool integrations:

```javascript
const { install, migrate, audit, doctor } = require("corexpm");

// 1. Programmatically run deterministic install
const result = install({ frozen: true });
console.log("Install status:", result.code);

// 2. Import foreign lockfiles
const migration = migrate();
console.log("Migrated packages count:", migration.data.packages_migrated);

// 3. Audit vulnerabilities
const auditReport = audit({ severity: "high" });
console.log("Security report:", auditReport.data);
```

---

## Architecture Overview

The CorexPM data path is designed for determinism, immutability, and security:

```text
package.json -> resolver -> dependency graph -> corex.lock.json
                                      |
                         registry -> fetch -> verify SHA-512
                                      |
                           Global Corex CAS Store
                                      |
                            isolated node_modules
                                      |
                               approved scripts
```

Read the complete [Architecture Overview](docs/architecture/overview.md), [Lockfile Policy](docs/compatibility/lockfile-policy.md), and [Migration Guide](docs/compatibility/migration-guide.md).

---

## Repository Layout

```text
assets/       Project brand assets and logo
crates/       Native Rust crates (cli, core, store, lockfile, policy, workspace)
packages/     NPM distribution package and Node.js/TypeScript SDK
scripts/      Universal install.sh, install.ps1, and local test automation
docs/         Specifications, ADRs, RFCs, and roadmap documentation
examples/     Supported JavaScript and TypeScript project examples
```

## License

CorexPM is open-source software available under the [MIT License](LICENSE).
