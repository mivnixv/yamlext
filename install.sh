#!/usr/bin/env sh
set -e

REPO="mivnixv/yamlext"
BIN="yamlext"
INSTALL_DIR="${INSTALL_DIR:-/usr/local/bin}"

# Resolve latest version unless overridden
if [ -z "$VERSION" ]; then
  VERSION=$(curl -fsSL "https://api.github.com/repos/${REPO}/releases/latest" \
    | grep '"tag_name"' | sed 's/.*"tag_name": *"\(.*\)".*/\1/')
fi

if [ -z "$VERSION" ]; then
  echo "error: could not determine latest version" >&2
  exit 1
fi

# Detect OS
OS=$(uname -s)
case "$OS" in
  Linux)  OS="linux" ;;
  Darwin) OS="macos" ;;
  *)
    echo "error: unsupported OS: $OS" >&2
    exit 1
    ;;
esac

# Detect architecture
ARCH=$(uname -m)
case "$ARCH" in
  x86_64)          ARCH="x86_64" ;;
  arm64 | aarch64) ARCH="aarch64" ;;
  *)
    echo "error: unsupported architecture: $ARCH" >&2
    exit 1
    ;;
esac

ARTIFACT="${BIN}-${OS}-${ARCH}"
URL="https://github.com/${REPO}/releases/download/${VERSION}/${ARTIFACT}"

echo "Installing ${BIN} ${VERSION} (${OS}/${ARCH}) to ${INSTALL_DIR} ..."

curl -fsSL "$URL" -o "/tmp/${BIN}"
chmod +x "/tmp/${BIN}"

if [ -w "$INSTALL_DIR" ]; then
  mv "/tmp/${BIN}" "${INSTALL_DIR}/${BIN}"
else
  sudo mv "/tmp/${BIN}" "${INSTALL_DIR}/${BIN}"
fi

echo "Done. Run: ${BIN} --version"
