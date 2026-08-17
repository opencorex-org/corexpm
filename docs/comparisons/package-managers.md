# JavaScript package-manager comparison

Last verified: **2026-08-17**

This document compares npm, pnpm, Yarn Modern, Bun's package manager, and the
planned CorexPM architecture. It is a product and architecture guide—not a
claim that CorexPM already competes with production package managers.

CorexPM currently provides only a bootstrap native CLI and domain scaffolding.
Resolution, registry access, CAS storage, installation, lockfiles, workspaces,
and Corex Guard are **not implemented**.

## Executive summary

- **npm** is the compatibility baseline and uses a conventional hoisted
  `node_modules` installation. Its global content-addressed cache avoids some
  downloads, but npm documents the cache as a cache rather than a persistent
  shared installed-package store.
- **pnpm** is the closest baseline for CorexPM's v1 disk architecture. It uses a
  content-addressable store, hard-linked package files, and a symlinked
  dependency layout, with compatibility hoisting available.
- **Yarn Modern** uses Plug'n'Play by default for new projects. PnP generates a
  loader instead of `node_modules` and references package artifacts directly
  from Yarn's cache, giving strong dependency-boundary enforcement with some
  tool and IDE integration costs.
- **Bun** combines a fast native runtime and package manager. It supports
  hoisted and isolated layouts, platform-efficient cache materialization, and
  an optional global virtual store that links projects to shared package data.
- **CorexPM** should enter this field as a focused native package manager—not as
  “another faster npm.” Its credible v1 target is npm compatibility plus an
  immutable package-level CAS, a compatible isolated linker, deterministic
  observability, and explicit lifecycle capability policy.

## High-level matrix

| Capability | npm | pnpm | Yarn Modern | Bun | CorexPM |
| --- | --- | --- | --- | --- | --- |
| Product state | mature, production | mature, production | mature, production | production runtime and package manager | `0.1.0-dev` scaffold |
| npm registry ecosystem | native baseline | supported | supported | supported | required v1 goal |
| Default dependency layout | hoisted `node_modules` | symlinked isolated-style `node_modules` with compatibility hoisting | Plug'n'Play loader | depends on project: hoisted or isolated | isolated `node_modules` planned |
| Shared package data | content-addressed download cache | content-addressable file store | shared package cache directly referenced by PnP | global cache; optional global virtual store | immutable package-level CAS planned |
| Strict undeclared-dependency protection | limited by hoisting | strong layout, with some default compatibility hoisting | strong under PnP | strong in isolated mode | strict by default planned |
| `node_modules`-less mode | no | no | PnP default for modern new projects | not the normal package install model | Corex Virtual after v1 |
| Lockfile | `package-lock.json` | `pnpm-lock.yaml` | `yarn.lock` | `bun.lock` | `corex.lock` planned |
| Frozen/immutable CI install | `npm ci` | `--frozen-lockfile` | `--immutable` | `--frozen-lockfile` | `install --frozen` / `ci` planned |
| Workspaces | supported | supported | advanced workspace tools | supported | graph-native workspace support planned |
| Dependency script controls | approval metadata currently documented as advisory | dependency postinstalls disabled by default in current major releases | postinstalls disabled by default in current releases | trusted dependency model | deny-by-default capability policy planned |
| Native executable independent of Node | no | distribution-dependent | no | yes | yes |
| Built-in measured cross-project disk savings | cache inspection, not a shared install savings model | store-oriented tooling | cache-oriented tooling | cache/global-store tooling | explicit logical vs physical savings planned |

The table deliberately uses qualified language. Defaults and security behavior
change between major versions and configuration profiles.

## Storage and `node_modules`

### npm

npm uses a conventional physical `node_modules` tree with configurable install
strategies and hoisting. Its `_cacache` directory is content-addressable and
integrity-verified, but npm explicitly describes it as a disposable cache, not
as a persistent installed-package store. Projects still materialize their own
dependency trees. See the official [npm install documentation](https://docs.npmjs.com/cli/v11/commands/npm-install/)
and [npm cache design](https://docs.npmjs.com/cli/v11/commands/npm-cache/).

Implication for CorexPM: npm is the behavior and compatibility reference, not
the target disk architecture.

### pnpm

pnpm stores package files in a content-addressable store and hard-links them
into a project's virtual store, then uses symlinks to express the dependency
graph. Only direct dependencies are linked at the project root, although pnpm
also uses compatibility hoisting by default and can create a conventional
hoisted layout when necessary. See pnpm's [motivation](https://pnpm.io/motivation)
and [symlinked `node_modules` structure](https://pnpm.io/symlinked-node-modules-structure).

Implication for CorexPM: package-level CAS is simpler than pnpm's file-level
deduplication but may use more physical space across nearly identical package
versions. CorexPM must prove that this simplicity improves transactional
safety, observability, or speed enough to justify the trade-off.

### Yarn Modern

Plug'n'Play is Yarn Modern's default installation strategy for new projects.
It generates `.pnp.cjs` instead of a traditional `node_modules` tree and can
reference shared cache artifacts directly. Yarn documents PnP's minimal install
footprint and ghost-dependency protection, but also calls out IDE integration
work and the need for regular `node_modules` installs in environments such as
React Native/Expo. See [Yarn Plug'n'Play](https://yarnpkg.com/features/pnp) and
[Yarn cache strategies](https://yarnpkg.com/features/caching).

Implication for CorexPM: isolated `node_modules` should prioritize broad tool
compatibility for v1. Corex Virtual must wait until a loader and integration
story can compete with Yarn's mature PnP ecosystem.

### Bun

Bun maintains a global cache and materializes packages using hardlinks on Linux
and Windows, copy-on-write clones on macOS, and file copies as fallback. Bun
also supports a pnpm-like isolated linker. Its optional global virtual store
materializes package data once and lets project virtual stores link to it, but
the feature is documented as off by default. See Bun's [global cache](https://bun.sh/docs/pm/global-cache),
[isolated installs](https://bun.sh/docs/pm/isolated-installs), and
[global virtual store](https://bun.sh/docs/pm/global-store).

Implication for CorexPM: “native and fast” plus “global store” is no longer
enough differentiation. CorexPM needs stronger transaction, explanation,
measurement, compatibility, and policy behavior.

### CorexPM target

CorexPM plans an immutable package-level CAS under `~/.corex/store`, a strict
isolated project tree, and platform-aware linking through symlinks, junctions,
reflinks, protected hardlinks, or a copy fallback. It will measure CAS, cache,
build-artifact, link/state, reclaimable, logical, and physical bytes separately.

The mechanism and acceptance criteria are defined in the
[`node_modules` size-reduction plan](../architecture/node-modules-size-reduction.md).

## Dependency isolation

### npm

Hoisting maximizes compatibility but can expose transitive packages at paths
where application code can import them without declaring them. A lockfile can
reproduce the resolved tree, but it does not turn hoisted visibility into a
strict dependency boundary.

### pnpm

pnpm's symlink graph normally exposes only declared relationships, while
compatibility hoisting may make additional packages visible through a hidden
modules location. Users can disable hoisting for stronger strictness or choose
a hoisted linker for maximum legacy compatibility.

### Yarn Modern

PnP has the strongest mature enforcement model in this comparison. The loader
owns the dependency map and can produce semantic errors explaining an
undeclared access or an unsatisfied peer dependency.

### Bun

Bun's isolated linker prevents phantom dependencies through a virtual-store
layout. Hoisted mode remains available for projects and tools that depend on a
flat tree.

### CorexPM target

CorexPM's isolated linker should expose only graph edges. Compatibility fixes
must be explicit package extensions or named compatibility profiles, never
silent accidental hoisting. Errors should show the requesting package,
undeclared package, dependency path, and safe remediation.

## Lockfiles and reproducibility

| Manager | Reproducible CI behavior |
| --- | --- |
| npm | `npm ci` requires an existing lockfile, rejects manifest mismatch, and replaces the installed tree. |
| pnpm | `pnpm install --frozen-lockfile` rejects missing or out-of-sync lock data; frozen behavior is the CI default when a lockfile exists. |
| Yarn | `yarn install --immutable` rejects lockfile changes and defaults to immutable behavior on CI; cache mutation and checksum validation have separate flags. |
| Bun | `bun install --frozen-lockfile` uses locked versions and rejects manifest disagreement without updating the lockfile. |
| CorexPM | Planned `corexpm install --frozen` and `corexpm ci` reject any relevant manifest/workspace mismatch before project mutation. |

Official references: [npm ci](https://docs.npmjs.com/cli/v11/commands/npm-ci/),
[pnpm install](https://pnpm.io/cli/install),
[Yarn install](https://yarnpkg.com/cli/install), and
[Bun install](https://bun.sh/docs/pm/cli/install).

CorexPM's differentiator should not be merely “has a lockfile.” The planned
value is deterministic canonical output, stable diagnostic codes, graph
explanations, transactional activation, and explicit recording of decisions
that change artifacts or dependency identity.

## Lifecycle and supply-chain security

This area changes quickly and must be re-verified before releases or marketing
claims.

### npm

The documented npm 11.19 approval commands maintain an `allowScripts` policy in
`package.json`, including version-pinned approvals and explicit denials. npm's
own documentation says the field is currently advisory: install scripts still
run by default while npm reports unreviewed packages, with blocking planned for
a future release. See [npm approve-scripts](https://docs.npmjs.com/cli/v11/commands/npm-approve-scripts/)
and [npm deny-scripts](https://docs.npmjs.com/cli/v11/commands/npm-deny-scripts/).

### pnpm

Current pnpm documentation says dependency postinstall scripts are disabled by
default from pnpm v10 and recommends explicitly listing trusted dependencies
through `allowBuilds`. It also documents controls for exotic transitive sources,
minimum release age, and publisher trust downgrade policy. See
[pnpm supply-chain guidance](https://pnpm.io/supply-chain-security).

### Yarn Modern

Current Yarn documentation says postinstall execution is disabled by default
from Yarn 4.14. Yarn also documents a default package age gate and hardened mode
that validates resolution and lockfile metadata in public pull-request contexts.
See [Yarn security](https://yarnpkg.com/features/security).

### Bun

Bun exposes trusted and untrusted dependency lists and can run/record scripts
for explicitly trusted packages. Its project configuration supports trusted
dependencies, including a documented default-trusted set. See
[`bun pm untrusted` and `bun pm trust`](https://bun.sh/docs/pm/cli/pm).

### CorexPM target

CorexPM should not claim that deny-by-default scripts alone are unique. Mature
competitors now implement similar baselines. Corex Guard's intended distinction
is a versioned capability model that can explain and, where the platform
supports it, enforce process, network, filesystem, native-execution, and
environment access independently.

The UI must label a capability as **enforced**, **detected**, or **advisory**.
CAS content is verified and immutable before any approved script runs, and
build output belongs in a separately keyed overlay rather than shared source.

## Workspaces and monorepos

All four existing managers support workspaces, so workspace discovery alone is
baseline compatibility:

- npm installs and links workspace packages and can run commands in selected
  workspace contexts.
- pnpm treats workspace installation and filtering as core workflows.
- Yarn provides workspace protocols, constraints, focused installs, and
  foreach/list commands. See [Yarn workspaces](https://yarnpkg.com/features/workspaces).
- Bun supports workspaces, catalogs, filters, and isolated workspace installs.

CorexPM's planned value is a reusable typed workspace graph shared by install,
`why`, topological execution, changed-package detection, affected-package
expansion, and machine-readable scheduling. Remote task caching remains outside
v1 and belongs to a separate future CorexBuild product.

## Performance comparison policy

No manager receives a permanent “fastest” label. Results depend on package
graph, cache state, network, filesystem, platform, script workload, runtime,
and version.

CorexPM comparisons must measure:

- cold install;
- second project with an overlapping graph;
- exact warm reinstall;
- offline install into a new project;
- existing-tree reconciliation;
- native package builds;
- small, medium, large, and monorepo fixtures;
- wall time, CPU, peak RAM, network bytes, physical writes, allocated disk
  blocks, file count, link count, and cache/store/build bytes.

Reports must publish exact commands, versions, hardware, OS, filesystem, cache
preparation, repetitions, dispersion, and raw samples. Marketing may summarize
published results but may not replace them.

## Where CorexPM must match competitors

Before 1.0, CorexPM must demonstrate:

1. npm manifest, registry, semver, peer, optional, native, binary, script, and
   workspace compatibility across a published fixture corpus;
2. deterministic frozen installs and useful lockfile conflict diagnostics;
3. safe offline and prefer-offline operation;
4. cross-platform link fallbacks and transactional recovery;
5. explicit dependency script trust at least as safe as current competitors;
6. practical framework, IDE, container, and CI workflows; and
7. migration that never deletes another manager's lockfile automatically.

## Where CorexPM can differentiate

These are hypotheses requiring implementation and evidence:

- **Corex CAS observability:** report measured logical, physical, reusable, and
  reclaimable bytes rather than only exposing a cache path.
- **Corex Guard capabilities:** move from package-level script permission toward
  explainable per-capability policy with honest platform enforcement labels.
- **Transactional guarantees:** specify and test atomic project activation,
  last-good rollback, concurrent object commit, and index reconstruction.
- **Unified explanations:** use the same dependency graph for resolver errors,
  `why`, workspace effects, security policy, and installation diagnostics.
- **Compatibility-first isolation:** provide strict dependency boundaries
  through ordinary Node-visible paths before introducing virtual mode.
- **Native independence:** deliver a focused package-manager binary without
  coupling the product to a new JavaScript runtime.

## What CorexPM should not claim

- “The first package manager with a global store.”
- “The only package manager that blocks install scripts.”
- “Strict dependencies are unique to CorexPM.”
- “Always faster than npm/pnpm/Yarn/Bun.”
- “A fixed percentage smaller on disk.”
- “More secure” without a named threat, control, enforcement level, and test.
- “npm compatible” without compatibility fixtures and documented deviations.

The honest initial positioning remains:

> CorexPM is designing a native, npm-compatible package manager around an
> immutable package-level CAS, strict but compatible dependency isolation,
> deterministic observability, and capability-oriented lifecycle security.

## Primary sources

- npm: [install](https://docs.npmjs.com/cli/v11/commands/npm-install/),
  [cache](https://docs.npmjs.com/cli/v11/commands/npm-cache/),
  [`npm ci`](https://docs.npmjs.com/cli/v11/commands/npm-ci/), and
  [script approval](https://docs.npmjs.com/cli/v11/commands/npm-approve-scripts/)
- pnpm: [motivation](https://pnpm.io/motivation),
  [symlinked layout](https://pnpm.io/symlinked-node-modules-structure),
  [install](https://pnpm.io/cli/install), and
  [supply-chain security](https://pnpm.io/supply-chain-security)
- Yarn: [Plug'n'Play](https://yarnpkg.com/features/pnp),
  [cache strategies](https://yarnpkg.com/features/caching),
  [install](https://yarnpkg.com/cli/install),
  [workspaces](https://yarnpkg.com/features/workspaces), and
  [security](https://yarnpkg.com/features/security)
- Bun: [global cache](https://bun.sh/docs/pm/global-cache),
  [global virtual store](https://bun.sh/docs/pm/global-store),
  [isolated installs](https://bun.sh/docs/pm/isolated-installs),
  [install](https://bun.sh/docs/pm/cli/install), and
  [package-manager utilities](https://bun.sh/docs/pm/cli/pm)

