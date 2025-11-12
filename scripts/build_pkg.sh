#!/usr/bin/env bash

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"
APP_NAME="PepTrack"
APP_CERT="${PEPTRACK_APP_CERT:-PepTrack Dev}"
INSTALLER_CERT="${PEPTRACK_INSTALLER_CERT:-PepTrack Dev Installer}"
BUILD_DIR="${REPO_ROOT}/src-tauri/target/release/bundle/macos"
APP_PATH="${BUILD_DIR}/${APP_NAME}.app"
DIST_DIR="${REPO_ROOT}/dist"
PKG_PATH="${DIST_DIR}/${APP_NAME}-selfsigned.pkg"

echo "==> Building Tauri app bundle"
(cd "${REPO_ROOT}" && cargo tauri build --bundles app)

if [[ ! -d "${APP_PATH}" ]]; then
  echo "Error: app bundle not found at ${APP_PATH}" >&2
  exit 1
fi

echo "==> Signing app bundle with certificate: ${APP_CERT}"
codesign --force --deep --options runtime --timestamp --sign "${APP_CERT}" "${APP_PATH}"

mkdir -p "${DIST_DIR}"

echo "==> Building installer PKG signed with: ${INSTALLER_CERT}"
productbuild \
  --component "${APP_PATH}" /Applications \
  --sign "${INSTALLER_CERT}" \
  --timestamp \
  "${PKG_PATH}"

echo "✅ PKG created at ${PKG_PATH}"
