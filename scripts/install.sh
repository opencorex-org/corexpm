#!/usr/bin/env sh
set -e

# CorexPM Universal POSIX Shell Installer
# Usage: curl -fsSL https://corex.dev/install.sh | sh

COREX_VERSION="${COREX_VERSION:-v1.0.0}"
COREX_INSTALL_DIR="${COREX_INSTALL_DIR:-$HOME/.corex/bin}"

OS="$(uname -s | tr '[:upper:]' '[:lower:]')"
ARCH="$(uname -m)"

case "$OS" in
  darwin) OS_NAME="darwin" ;;
  linux)  OS_NAME="linux" ;;
  *)      echo "Unsupported OS: $OS"; exit 1 ;;
esac

case "$ARCH" in
  x86_64|amd64) ARCH_NAME="x64" ;;
  aarch64|arm64) ARCH_NAME="arm64" ;;
  *)            echo "Unsupported Architecture: $ARCH"; exit 1 ;;
esac

TARGET_NAME="corexpm-${OS_NAME}-${ARCH_NAME}"
DOWNLOAD_URL="https://github.com/opencorex-org/corexpm/releases/download/${COREX_VERSION}/${TARGET_NAME}"

echo "Downloading CorexPM ${COREX_VERSION} for ${OS_NAME}-${ARCH_NAME}..."
mkdir -p "$COREX_INSTALL_DIR"

TMP_BIN="$(mktemp)"
curl -fsSL "$DOWNLOAD_URL" -o "$TMP_BIN" || {
  echo "Failed to download binary from $DOWNLOAD_URL"
  rm -f "$TMP_BIN"
  exit 1
}

chmod +x "$TMP_BIN"
mv "$TMP_BIN" "$COREX_INSTALL_DIR/corexpm"

echo ""
echo "CorexPM successfully installed to $COREX_INSTALL_DIR/corexpm"
echo ""
echo "To add CorexPM to your PATH, add this line to your shell configuration (.bashrc, .zshrc):"
echo "  export PATH=\"\$HOME/.corex/bin:\$PATH\""
echo ""
