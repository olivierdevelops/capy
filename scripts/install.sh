#!/bin/sh
# Capy installer. Downloads the latest release for the current OS/arch
# from https://github.com/olivierdevelops/capy/releases and installs into a
# directory on $PATH.
#
# Usage:
#   curl -fsSL https://raw.githubusercontent.com/olivierdevelops/capy/main/scripts/install.sh | sh
#   curl -fsSL https://raw.githubusercontent.com/olivierdevelops/capy/main/scripts/install.sh | sh -s -- --version v0.1.0
#   curl -fsSL https://raw.githubusercontent.com/olivierdevelops/capy/main/scripts/install.sh | sh -s -- --dir ~/.local/bin

set -e

REPO="olivierdevelops/capy"
VERSION="latest"
INSTALL_DIR=""

while [ $# -gt 0 ]; do
  case "$1" in
    --version) VERSION="$2"; shift 2 ;;
    --dir)     INSTALL_DIR="$2"; shift 2 ;;
    *) printf "unknown flag: %s\n" "$1"; exit 1 ;;
  esac
done

# Detect OS
OS=$(uname -s | tr '[:upper:]' '[:lower:]')
case "$OS" in
  linux)  OS=linux ;;
  darwin) OS=darwin ;;
  *) printf "unsupported OS: %s\n" "$OS"; exit 1 ;;
esac

# Detect ARCH
ARCH=$(uname -m)
case "$ARCH" in
  x86_64|amd64) ARCH=amd64 ;;
  aarch64|arm64) ARCH=arm64 ;;
  *) printf "unsupported arch: %s\n" "$ARCH"; exit 1 ;;
esac

# Resolve latest version if needed
if [ "$VERSION" = "latest" ]; then
  VERSION=$(curl -fsSL "https://api.github.com/repos/${REPO}/releases/latest" \
    | grep '"tag_name":' \
    | head -1 \
    | sed -E 's/.*"([^"]+)".*/\1/')
  if [ -z "$VERSION" ]; then
    printf "could not resolve latest version\n" >&2
    exit 1
  fi
fi

# Release archives are named by Rust target triple since the switch from
# goreleaser to cargo: capy_v0.21.0_aarch64-apple-darwin.tar.gz, where older
# releases were capy_0.20.0_darwin_arm64.tar.gz.
case "${OS}_${ARCH}" in
  linux_amd64)  TARGET=x86_64-unknown-linux-gnu ;;
  linux_arm64)  TARGET=aarch64-unknown-linux-gnu ;;
  darwin_amd64) TARGET=x86_64-apple-darwin ;;
  darwin_arm64) TARGET=aarch64-apple-darwin ;;
  *) printf "unsupported platform: %s/%s\n" "$OS" "$ARCH" >&2; exit 1 ;;
esac

ARCHIVE="capy_${VERSION}_${TARGET}.tar.gz"
URL="https://github.com/${REPO}/releases/download/${VERSION}/${ARCHIVE}"

printf "downloading %s\n" "$URL"
TMP=$(mktemp -d)
trap "rm -rf $TMP" EXIT

curl -fsSL -o "$TMP/$ARCHIVE" "$URL"

# Verify checksum if possible. One .sha256 per archive now, rather than a single
# combined checksums.txt — that is what the release workflow uploads.
CHECKSUM_URL="https://github.com/${REPO}/releases/download/${VERSION}/${ARCHIVE}.sha256"
if curl -fsSL -o "$TMP/$ARCHIVE.sha256" "$CHECKSUM_URL" 2>/dev/null; then
  ACTUAL=$(sha256sum "$TMP/$ARCHIVE" 2>/dev/null | awk '{print $1}' \
    || shasum -a 256 "$TMP/$ARCHIVE" | awk '{print $1}')
  EXPECTED=$(awk '{print $1}' "$TMP/$ARCHIVE.sha256")
  if [ "$ACTUAL" != "$EXPECTED" ]; then
    printf "checksum mismatch for %s\n" "$ARCHIVE" >&2
    exit 1
  fi
  printf "checksum OK\n"
fi

# Extract
tar -xzf "$TMP/$ARCHIVE" -C "$TMP"

# Decide install dir
if [ -z "$INSTALL_DIR" ]; then
  if [ -w "/usr/local/bin" ]; then
    INSTALL_DIR="/usr/local/bin"
  elif [ -d "$HOME/.local/bin" ]; then
    INSTALL_DIR="$HOME/.local/bin"
  else
    INSTALL_DIR="$HOME/.local/bin"
    mkdir -p "$INSTALL_DIR"
  fi
fi

# The archive may place the binary at the root or inside a versioned directory;
# find it either way rather than assuming.
CAPY_BIN=$(find "$TMP" -type f -name capy -perm -u+x 2>/dev/null | head -1)
if [ -z "$CAPY_BIN" ]; then
  printf "capy binary not found in %s\n" "$ARCHIVE" >&2
  exit 1
fi
mv "$CAPY_BIN" "$INSTALL_DIR/capy"
chmod +x "$INSTALL_DIR/capy"

printf "installed %s/capy\n" "$INSTALL_DIR"
"$INSTALL_DIR/capy" version || true

case ":$PATH:" in
  *":$INSTALL_DIR:"*) ;;
  *)
    printf "\nNote: %s is not on PATH. Add this to your shell rc:\n" "$INSTALL_DIR"
    printf "  export PATH=\"%s:\$PATH\"\n" "$INSTALL_DIR"
    ;;
esac
