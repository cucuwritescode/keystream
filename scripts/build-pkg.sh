#!/bin/sh
# build a macOS .pkg installer for keystream
# requires: rust toolchain (for building), pkgbuild, productbuild (ship with macOS)
set -eu

VERSION="${1:-$(grep '^version' Cargo.toml | head -1 | sed 's/.*"\(.*\)"/\1/')}"
PKG_ID="com.gauchodsp.keystream"
INSTALL_DIR="/usr/local/bin"
PLIST_DIR="/Library/LaunchAgents"

echo ""
echo "  building keystream ${VERSION} installer"
echo "  ----------------------------------------"
echo ""

# ── build universal binary ──────────────────────────────────────────
echo "  building x86_64..."
cargo build --release --target x86_64-apple-darwin 2>/dev/null

echo "  building aarch64..."
cargo build --release --target aarch64-apple-darwin 2>/dev/null

echo "  creating universal binary..."
mkdir -p build/pkg-root${INSTALL_DIR}
mkdir -p build/pkg-root${PLIST_DIR}
mkdir -p build/scripts
mkdir -p build/resources
mkdir -p dist

lipo -create \
    target/x86_64-apple-darwin/release/keystream \
    target/aarch64-apple-darwin/release/keystream \
    -output build/pkg-root${INSTALL_DIR}/keystream

chmod +x build/pkg-root${INSTALL_DIR}/keystream
file build/pkg-root${INSTALL_DIR}/keystream

# ── install launchd plist ───────────────────────────────────────────
sed "s|__INSTALL_DIR__|${INSTALL_DIR}|g" com.gauchodsp.keystream.plist \
    > build/pkg-root${PLIST_DIR}/${PKG_ID}.plist

# ── post-install script ────────────────────────────────────────────
cat > build/scripts/postinstall << 'POSTINSTALL'
#!/bin/sh
# load the launchd plist for the installing user
PLIST="/Library/LaunchAgents/com.gauchodsp.keystream.plist"
CURRENT_USER=$(stat -f "%Su" /dev/console)

if [ -n "$CURRENT_USER" ] && [ "$CURRENT_USER" != "root" ]; then
    su - "$CURRENT_USER" -c "launchctl load '$PLIST'" 2>/dev/null || true
    # open accessibility settings so the user can grant permission immediately
    su - "$CURRENT_USER" -c "open 'x-apple.systempreferences:com.apple.preference.security?Privacy_Accessibility'" 2>/dev/null || true
fi

exit 0
POSTINSTALL
chmod +x build/scripts/postinstall

# ── pre-uninstall (for upgrades) ───────────────────────────────────
cat > build/scripts/preinstall << 'PREINSTALL'
#!/bin/sh
# stop existing daemon before upgrade
PLIST="/Library/LaunchAgents/com.gauchodsp.keystream.plist"
CURRENT_USER=$(stat -f "%Su" /dev/console)

if [ -f "$PLIST" ] && [ -n "$CURRENT_USER" ] && [ "$CURRENT_USER" != "root" ]; then
    su - "$CURRENT_USER" -c "launchctl unload '$PLIST'" 2>/dev/null || true
fi
exit 0
PREINSTALL
chmod +x build/scripts/preinstall

# ── welcome text ────────────────────────────────────────────────────
cat > build/resources/welcome.html << 'WELCOME'
<html>
<head>
<meta charset="utf-8"/>
</head>
<body style="font-family: Menlo, Monaco, Courier, monospace; font-size: 12px; line-height: 1.8;">

<font color="#cc3300" size="5"><b>KEYSTREAM</b></font>
<br/>
<font color="#661100">--------------------------------</font>
<br/><br/>

<font color="#993322">real-time keyboard audio synthesis</font>
<br/><br/>

<font color="#884422">voices &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;</font><font color="#cc4433">32 concurrent</font><br/>
<font color="#884422">oscillator &nbsp;</font><font color="#cc4433">recursive sine</font><br/>
<font color="#884422">latency &nbsp;&nbsp;&nbsp;&nbsp;</font><font color="#cc4433">&lt; 1ms</font><br/>
<font color="#884422">binary &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;</font><font color="#cc4433">/usr/local/bin/keystream</font>
<br/><br/>

<font color="#cc3300"><b>ACCESSIBILITY PERMISSION REQUIRED</b></font>
<br/><br/>
<font color="#aa4433">
after installation, system settings will open
automatically. add your terminal to the
accessibility list or keystream cannot function.
</font>
<br/><br/>

<font color="#552211">READY</font>

</body>
</html>
WELCOME

# ── build component package ─────────────────────────────────────────
echo "  building component package..."
pkgbuild \
    --root build/pkg-root \
    --scripts build/scripts \
    --identifier "$PKG_ID" \
    --version "$VERSION" \
    --install-location / \
    build/keystream-component.pkg

# ── build product archive ───────────────────────────────────────────
cat > build/distribution.xml << DIST
<?xml version="1.0" encoding="utf-8"?>
<installer-gui-script minSpecVersion="2">
    <title>keystream ${VERSION}</title>
    <welcome file="welcome.html"/>
    <license file="LICENSE"/>
    <options customize="never" require-scripts="false"/>
    <choices-outline>
        <line choice="default"/>
    </choices-outline>
    <choice id="default" title="keystream">
        <pkg-ref id="${PKG_ID}"/>
    </choice>
    <pkg-ref id="${PKG_ID}" version="${VERSION}">keystream-component.pkg</pkg-ref>
</installer-gui-script>
DIST

cp LICENSE build/resources/LICENSE 2>/dev/null || echo "  (no LICENSE file found, skipping)"

echo "  building product archive..."
productbuild \
    --distribution build/distribution.xml \
    --resources build/resources \
    --package-path build \
    "dist/keystream-${VERSION}.pkg"

# ── checksums ───────────────────────────────────────────────────────
cd dist
shasum -a 256 "keystream-${VERSION}.pkg" > "keystream-${VERSION}.pkg.sha256"
cd ..

# ── clean up ────────────────────────────────────────────────────────
rm -rf build

echo ""
echo "  ✓ dist/keystream-${VERSION}.pkg"
echo "  ✓ dist/keystream-${VERSION}.pkg.sha256"
echo ""
echo "  to install: open dist/keystream-${VERSION}.pkg"
echo ""
