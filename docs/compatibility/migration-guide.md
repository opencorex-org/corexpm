# CorexPM Migration Guide

This guide details how to migrate existing JavaScript and TypeScript projects from **npm**, **pnpm**, **Yarn**, or **Bun** to **CorexPM** deterministically and without data loss.

## Non-Negotiable Migration Invariant

> [!IMPORTANT]
> **No automatic deletion of foreign lockfiles**: CorexPM **never** deletes or mutates source lockfiles (`package-lock.json`, `pnpm-lock.yaml`, `yarn.lock`, `bun.lock`). The original lockfiles remain untouched on disk so you can rollback or compare resolution behavior at any time.

---

## Step-by-Step Migration Process

### Step 1: Run `corexpm migrate`

Navigate to your project root and execute:

```sh
corexpm migrate
```

CorexPM will:
1. Detect your existing foreign lockfile (`package-lock.json`, `pnpm-lock.yaml`, `yarn.lock`, or `bun.lock`).
2. Parse dependency requirements, resolved tarballs, and integrity hashes.
3. Write a canonical, versioned `corex.lock.json` file.
4. Verify and report that your original foreign lockfile was left untouched.

For structured automation, use `--json`:
```sh
corexpm migrate --json
```

### Step 2: Verify `corex.lock.json`

Validate that the generated `corex.lock.json` matches your `package.json` requirements:

```sh
corexpm install --frozen
```

### Step 3: Test Local Build & Workspaces

If your project is a monorepo or workspace:
```sh
corexpm workspace list
corexpm run build --all
```

### Step 4: Optional Clean Up

When you are completely satisfied with CorexPM, you may manually delete or archive your old foreign lockfiles:
```sh
rm package-lock.json # Or pnpm-lock.yaml / yarn.lock / bun.lock
```
CorexPM leaves this decision entirely up to you.

---

## Migration Troubleshooting

If `corexpm migrate` returns an error:
- **`CXLOCK0013`**: No foreign lockfile was found in the current directory. Ensure you run the command in the project root containing `package-lock.json`, `pnpm-lock.yaml`, `yarn.lock`, or `bun.lock`.
- **`CXLOCK0008`**: If `package.json` requirements differ from the lockfile, run `corexpm install` to reconcile state.
