#!/usr/bin/env sh
set -e

export PATH="$HOME/.cargo/bin:$PATH"
ROOT_DIR="$(pwd)"

# Local NPM Distribution & Installation Test Script

echo "=== 1. Building CorexPM Release Binary ==="
cargo build --workspace --release

echo ""
echo "=== 2. Testing SDK and launcher inside packages/corexpm ==="
node packages/corexpm/bin/corexpm.js doctor

echo ""
echo "=== 3. Packaging npm tarball (`npm pack ./packages/corexpm`) ==="
TARBALL_NAME="$(npm pack ./packages/corexpm 2>/dev/null | tail -n 1)"
TARBALL_PATH="$(pwd)/${TARBALL_NAME}"

echo "Created package archive: ${TARBALL_PATH}"

echo ""
echo "=== 4. Testing isolated local installation via `npm install` and `npx` ==="
TMP_PROJECT="$(mktemp -d)"
echo "Created temporary test project directory: ${TMP_PROJECT}"

cd "$TMP_PROJECT"
npm init -y >/dev/null
echo "Installing ${TARBALL_PATH}..."
npm install "$TARBALL_PATH" >/dev/null

echo ""
echo "--- Testing npx corexpm doctor ---"
npx corexpm doctor

echo ""
echo "--- Testing Node.js SDK require('corexpm') ---"
node -e 'const corex = require("corexpm"); console.log("SDK Doctor:", corex.doctor({ json: true }).data);'

cd "$ROOT_DIR"
rm -rf "$TMP_PROJECT"

echo ""
echo "=== All local NPM packaging and installation tests PASSED successfully! ==="
