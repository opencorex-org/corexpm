# CorexPM Standard Release Process

CorexPM releases are reproducible native artifacts, signed checksums, and npm ecosystem distribution packages.

## Standard Release Execution

CorexPM provides an automated, standard release script [`scripts/release.sh`](../../scripts/release.sh).

### Step 1: Pre-Release Dry Run

Run pre-release validation checks (`cargo fmt`, `clippy`, `cargo test`, `cargo build --release`, `npm pack`, `SHA256SUMS`):

```sh
./scripts/release.sh --dry-run v1.0.0
```

### Step 2: Formal Release & Git Tagging

When validation passes and the working tree is clean:

```sh
./scripts/release.sh v1.0.0
```

This will:
1. Validate code formatting, clippy lints, and unit tests.
2. Compile native release binaries (`target/release/corexpm`).
3. Package the npm distribution archive (`packages/corexpm/corexpm-1.0.0.tgz`).
4. Generate release checksums (`SHA256SUMS`).
5. Create an annotated git release tag `v1.0.0`.

### Step 3: Push Tags & Publish NPM

Push the release commit and tag to GitHub:

```sh
git push origin main --tags
```

Publish the npm package to the npm registry:

```sh
cd packages/corexpm
npm publish --access public
```

---

## Supported Target Triples

- `aarch64-apple-darwin` (macOS Apple Silicon)
- `x86_64-apple-darwin` (macOS Intel)
- `x86_64-unknown-linux-gnu` (Linux x64)
- `aarch64-unknown-linux-gnu` (Linux ARM64)
- `x86_64-pc-windows-msvc` (Windows x64)
