#!/usr/bin/env bash

set -euo pipefail

INSTALL_DIR="/usr/local/bin"

os="$(uname | tr '[:upper:]' '[:lower:]')"
arch="$(uname -m)"

case "${arch}" in
  x86_64) arch="amd64" ;;
  aarch64|arm64) arch="arm64" ;;
  *)
    echo "Unsupported architecture: ${arch}" >&2
    exit 1
    ;;
esac

BIN_NAME="bazelisk-${os}-${arch}"

sudo -v

tmpdir="$(mktemp -d)"
trap 'rm -rf "${tmpdir}"' EXIT

curl -fL \
  "https://github.com/bazelbuild/bazelisk/releases/latest/download/${BIN_NAME}" \
  -o "${tmpdir}/${BIN_NAME}"

chmod +x "${tmpdir}/${BIN_NAME}"
sudo install "${tmpdir}/${BIN_NAME}" "${INSTALL_DIR}/bazelisk"

if [ ! -e "${INSTALL_DIR}/bazel" ]; then
  sudo ln -s "${INSTALL_DIR}/bazelisk" "${INSTALL_DIR}/bazel"
fi

BAZELISK_PRINT_VERSION=1 "${INSTALL_DIR}/bazelisk" version
