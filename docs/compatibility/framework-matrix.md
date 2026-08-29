# Framework and Native Addon Compatibility Matrix

CorexPM provides compatibility with standard Node.js module resolution, native C++ addons (`node-gyp`), popular web frameworks, build tools, and package managers.

## Web and Backend Frameworks

| Framework / Tool | Support Level | Module Resolution | Isolation Compatibility | Notes |
| --- | --- | --- | --- | --- |
| **React** | Tier 1 Supported | Standard `node_modules` | Full Isolation | Supports standard peer dependencies and dual CJS/ESM exports. |
| **Next.js** | Tier 1 Supported | Isolated / Symlinked | Full Isolation | Tested with Next.js App Router and standalone output builds. |
| **Vue.js / Nuxt** | Tier 1 Supported | Isolated / Symlinked | Full Isolation | Fully resolves `@vue/*` scoped dependencies and auto-imports. |
| **Svelte / SvelteKit**| Tier 1 Supported | ESM Export Maps | Full Isolation | Supports Svelte preprocessors and `svelte.config.js` entry points. |
| **Express.js** | Tier 1 Supported | CommonJS / CJS | Full Isolation | Legacy standard require resolution compatible. |
| **NestJS** | Tier 1 Supported | TypeScript / CJS | Full Isolation | Decorator metadata reflection and dynamic module loading tested. |
| **Vite** | Tier 1 Supported | Native ESM / Vite Rollup | Full Isolation | Vite pre-bundling and dev server HMR supported. |
| **Remix** | Tier 1 Supported | ESM / Node Adapters | Full Isolation | Supports Remix server build targets and asset bundles. |
| **Astro** | Tier 1 Supported | ESM / Vite Plugins | Full Isolation | Astro island architecture and SSR adapters supported. |

## Native C++ Addons & Build Tools

| Tool / Engine | Support Level | Execution Policy | Notes |
| --- | --- | --- | --- |
| **`node-gyp`** | Supported with Guard | Policy Controlled (`corex trust`) | Native builds execute inside writable package overlays. |
| **`prebuild-install`** | Supported with Guard | Policy Controlled | Precompiled binaries extracted cleanly into package target dir. |
| **`esbuild`** | Supported | Binary Execution | Executables linked portably under `node_modules/.bin`. |
| **`swc`** | Supported | Native Binary | Platform-specific native binaries resolved via `optionalDependencies`. |

## Package Manager Lockfile Migration Matrix

| Source Manager | Format | Migration Command | Foreign Lockfile Action |
| --- | --- | --- | --- |
| **npm** | `package-lock.json` (v1/v2/v3) | `corexpm migrate` | **Preserved untouched** |
| **pnpm** | `pnpm-lock.yaml` (v6/v9) | `corexpm migrate` | **Preserved untouched** |
| **Yarn** | `yarn.lock` (v1/berry) | `corexpm migrate` | **Preserved untouched** |
| **Bun** | `bun.lock` (v1 text/json) | `corexpm migrate` | **Preserved untouched** |
