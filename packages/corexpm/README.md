# CorexPM NPM Distribution Package

Native package manager for JavaScript and TypeScript projects — **Download once. Store once. Use everywhere.**

## Installation & CLI Usage

### Run on-demand with `npx`:

```sh
npx corexpm install
```

### Install globally via `npm`:

```sh
npm install -g corexpm
```

Then run `corexpm` from any terminal:

```sh
corexpm doctor
corexpm migrate
corexpm audit
```

## Node.js / TypeScript SDK Usage

Install as a dependency in your Node.js application or toolchain:

```sh
npm install corexpm
```

### Programmatic API Example (JavaScript / TypeScript):

```javascript
const { install, migrate, audit, doctor } = require("corexpm");

// 1. Run deterministic frozen install
const installResult = install({ frozen: true });
console.log("Install output:", installResult.data);

// 2. Import foreign lockfiles (package-lock.json / pnpm-lock.yaml / yarn.lock / bun.lock)
const migrateResult = migrate();
console.log("Migrated packages:", migrateResult.data.packages_migrated);

// 3. Security audit graph
const auditResult = audit({ severity: "high" });
console.log("Audit matches:", auditResult.data.matches);
```

## Supported Platforms

- macOS (`darwin-x64`, `darwin-arm64`)
- Linux (`linux-x64`, `linux-arm64`)
- Windows (`win32-x64`)
