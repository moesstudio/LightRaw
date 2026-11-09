#!/usr/bin/env bash
set -euo pipefail

# Build Universal Linux AppImage (runs on Ubuntu, Fedora, Arch, Debian, openSUSE)
ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
DIST_DIR="${ROOT_DIR}/dist"
APP_DIR="${DIST_DIR}/AppDir"

echo "==> Preparing AppDir for LightRaw 1.0.0..."
rm -rf "${APP_DIR}"
mkdir -p "${APP_DIR}/usr/bin"
mkdir -p "${APP_DIR}/usr/share/applications"
mkdir -p "${APP_DIR}/usr/share/icons/hicolor/256x256/apps"

cp "${ROOT_DIR}/target/release/lightraw" "${APP_DIR}/usr/bin/lightraw"
chmod 755 "${APP_DIR}/usr/bin/lightraw"

cp "${ROOT_DIR}/packaging/linux/lightraw.desktop" "${APP_DIR}/"
cp "${ROOT_DIR}/packaging/linux/lightraw.desktop" "${APP_DIR}/usr/share/applications/"
cp "${ROOT_DIR}/assets/icon.png" "${APP_DIR}/lightraw.png"
cp "${ROOT_DIR}/assets/icon.png" "${APP_DIR}/usr/share/icons/hicolor/256x256/apps/lightraw.png"

# Create standard AppRun launcher
cat << 'EOF' > "${APP_DIR}/AppRun"
#!/usr/bin/env bash
HERE="$(dirname "$(readlink -f "${0}")")"
export PATH="${HERE}/usr/bin:${PATH}"
export LD_LIBRARY_PATH="${HERE}/usr/lib:${LD_LIBRARY_PATH:-}"
exec "${HERE}/usr/bin/lightraw" "$@"
EOF
chmod 755 "${APP_DIR}/AppRun"

mkdir -p "${DIST_DIR}"
# Always produce clean standalone Linux tarball
tar -czf "${DIST_DIR}/lightraw-1.0.0-linux-x86_64.tar.gz" -C "${APP_DIR}" .
echo "==> Successfully built: ${DIST_DIR}/lightraw-1.0.0-linux-x86_64.tar.gz"

if command -v appimagetool >/dev/null 2>&1; then
    export APPIMAGE_EXTRACT_AND_RUN=1
    ARCH=x86_64 appimagetool --appimage-extract-and-run "${APP_DIR}" "${DIST_DIR}/LightRaw-1.0.0-x86_64.AppImage" || true
    if [ -f "${DIST_DIR}/LightRaw-1.0.0-x86_64.AppImage" ]; then
        echo "==> Successfully built: ${DIST_DIR}/LightRaw-1.0.0-x86_64.AppImage"
    fi
fi
