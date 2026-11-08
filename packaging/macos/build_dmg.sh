#!/usr/bin/env bash
set -euo pipefail

# Build macOS DMG Installer for LightRaw 1.0.0
ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
DIST_DIR="${ROOT_DIR}/dist"
STAGING_DIR="${DIST_DIR}/dmg_staging"
APP_BUNDLE="${ROOT_DIR}/LightRaw.app"
DMG_OUTPUT="${DIST_DIR}/LightRaw-1.0.0-macOS.dmg"

echo "==> Building macOS .dmg installer for LightRaw 1.0.0..."
mkdir -p "${DIST_DIR}"
rm -rf "${STAGING_DIR}" "${DMG_OUTPUT}"
mkdir -p "${STAGING_DIR}"

if [ ! -d "${APP_BUNDLE}" ]; then
    echo "ERROR: ${APP_BUNDLE} not found! Run 'cargo build --release' first."
    exit 1
fi

echo "--> Staging LightRaw.app and Applications symlink..."
cp -R "${APP_BUNDLE}" "${STAGING_DIR}/"
ln -s /Applications "${STAGING_DIR}/Applications"

echo "--> Creating compressed UDZO disk image..."
hdiutil create \
    -volname "LightRaw 1.0.0" \
    -srcfolder "${STAGING_DIR}" \
    -ov \
    -format UDZO \
    -imagekey zlib-level=9 \
    "${DMG_OUTPUT}"

rm -rf "${STAGING_DIR}"
echo "==> Successfully created: ${DMG_OUTPUT} ($(du -h "${DMG_OUTPUT}" | cut -f1))"
