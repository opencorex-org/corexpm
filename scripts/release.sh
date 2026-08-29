#!/usr/bin/env sh
set -e

# CorexPM Standard Release Automation Script
# Usage: ./scripts/release.sh [options] <vX.Y.Z>
# Options:
#   --dry-run      Run validation and artifact packaging without tagging git

export PATH="$HOME/.cargo/bin:$PATH"
ROOT_DIR="$(pwd)"

DRY_RUN=false
VERSION_ARG=""

for arg in "$@"; do
  case "$arg" in
    --dry-run) DRY_RUN=true ;;
    v*|1.*|0.*) VERSION_ARG="$arg" ;;
    *) echo "Unknown option or version argument: $arg"; exit 1 ;;
  esac
done

if [ -z "$VERSION_ARG" ]; then
  echo "Error: Release version argument required."
  echo "Usage: ./scripts/release.sh [--dry-run] v1.0.0"
  exit 1
fi

# Standardize version string with leading 'v'
case "$VERSION_ARG" in
  v*) VERSION="$VERSION_ARG" ;;
  *)  VERSION="v$VERSION_ARG" ;;
esac

RAW_VERSION="${VERSION#v}"

echo "=========================================================="
echo " Starting CorexPM Release Process for ${VERSION}"
echo "=========================================================="

echo ""
echo "=== Step 1: Validating Working Tree Cleanliness ==="
if [ "$DRY_RUN" = "false" ] && [ -n "$(git status --porcelain)" ]; then
  echo "Error: Git working tree is dirty. Please commit or stash changes before releasing."
  exit 1
fi
echo "Working tree is clean."

echo ""
echo "=== Step 2: Running Automated Code Formatting Check ==="
cargo fmt --all --check

echo ""
echo "=== Step 3: Running Cargo Clippy Lints ==="
cargo clippy --workspace --all-targets -- -D warnings

echo ""
echo "=== Step 4: Running Workspace Unit and Integration Tests ==="
cargo test --workspace

echo ""
echo "=== Step 5: Compiling Native Release Binaries ==="
cargo build --workspace --release

echo ""
echo "=== Step 6: Packaging NPM Distribution Artifact ==="
NPM_TARBALL="$(npm pack ./packages/corexpm 2>/dev/null | tail -n 1)"
echo "Created npm archive: ${NPM_TARBALL}"

echo ""
echo "=== Step 7: Generating SHA-256 Checksums (SHA256SUMS) ==="
BIN_NAME="corexpm"
if [ -f "target/release/corexpm.exe" ]; then
  BIN_NAME="corexpm.exe"
fi

rm -f SHA256SUMS
shasum -a 256 "target/release/${BIN_NAME}" "${NPM_TARBALL}" > SHA256SUMS
cat SHA256SUMS

if [ "$DRY_RUN" = "true" ]; then
  echo ""
  echo "=========================================================="
  echo " [DRY RUN COMPLETE] Pre-release validation succeeded!"
  echo " Target Version: ${VERSION} (${RAW_VERSION})"
  echo " SHA256SUMS generated cleanly."
  echo "=========================================================="
  exit 0
fi

echo ""
echo "=== Step 8: Creating Git Release Tag ${VERSION} ==="
git tag -a "${VERSION}" -m "CorexPM Release ${VERSION}"

echo ""
echo "=========================================================="
echo " CorexPM Release ${VERSION} Prepared Successfully!"
echo "=========================================================="
echo "Next steps to publish release:"
echo "  1. Push release commit and tag:"
echo "     git push origin main --tags"
echo "  2. Publish NPM package:"
echo "     cd packages/corexpm && npm publish --access public"
echo "=========================================================="
