#!/usr/bin/env bash
set -euo pipefail

# Build Debian (.deb) package for Ubuntu / Debian / Linux Mint
VERSION="${LIGHTRAW_VERSION:-1.2.0}"
ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
DIST_DIR="${ROOT_DIR}/dist"
PKG_ROOT="${DIST_DIR}/deb_staging"

echo "==> Building Debian Package for LightRaw ${VERSION}..."
rm -rf "${PKG_ROOT}"
mkdir -p "${PKG_ROOT}/DEBIAN"
mkdir -p "${PKG_ROOT}/usr/bin"
mkdir -p "${PKG_ROOT}/usr/share/applications"
mkdir -p "${PKG_ROOT}/usr/share/icons/hicolor/256x256/apps"
mkdir -p "${PKG_ROOT}/usr/share/doc/lightraw"

# Copy binary & resources (stamp package version dynamically)
sed "s/^Version:.*/Version: ${VERSION}/" "${ROOT_DIR}/packaging/linux/debian/control" > "${PKG_ROOT}/DEBIAN/control"
cp "${ROOT_DIR}/target/release/lightraw" "${PKG_ROOT}/usr/bin/lightraw"
chmod 755 "${PKG_ROOT}/usr/bin/lightraw"

cp "${ROOT_DIR}/packaging/linux/lightraw.desktop" "${PKG_ROOT}/usr/share/applications/"
cp "${ROOT_DIR}/assets/icon.png" "${PKG_ROOT}/usr/share/icons/hicolor/256x256/apps/lightraw.png"
if [ -f "${ROOT_DIR}/README.md" ]; then
    cp "${ROOT_DIR}/README.md" "${PKG_ROOT}/usr/share/doc/lightraw/"
fi

# Build .deb with dpkg-deb
mkdir -p "${DIST_DIR}"
dpkg-deb --build --root-owner-group "${PKG_ROOT}" "${DIST_DIR}/lightraw_${VERSION}_amd64.deb"
rm -rf "${PKG_ROOT}"

echo "==> Successfully created: ${DIST_DIR}/lightraw_${VERSION}_amd64.deb"
