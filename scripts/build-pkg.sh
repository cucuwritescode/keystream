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
fi

echo ""
echo "  ──────────────────────────────────────"
echo "  KEYSTREAM INSTALLATION COMPLETE"
echo "  ──────────────────────────────────────"
echo ""
echo "  binary    : /usr/local/bin/keystream"
echo "  service   : loaded"
echo ""
echo "  REQUIRED  : grant accessibility permission"
echo "              system settings > privacy & security > accessibility"
echo ""
echo "  status    : READY"
echo ""
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
<body style="background: #000; color: #c0c0c0; font-family: 'SF Mono', 'Menlo', 'Monaco', 'Courier New', monospace; font-size: 12px; line-height: 1.8; padding: 30px;">

<pre style="color: #e0e0e0; font-size: 14px; letter-spacing: 2px;">
KEYSTREAM 0.1
──────────────
</pre>

<p style="color: #888; margin-top: 20px;">
GAUCHO DSP LABORATORIES<br/>
AUDIO SYNTHESIS SUBSYSTEM
</p>

<p style="color: #c0c0c0; margin-top: 20px;">
this program converts keyboard input into pitched<br/>
sine tones in real time. 32 concurrent voices.<br/>
recursive oscillators. sub-millisecond latency.<br/>
no external dependencies.
</p>

<pre style="color: #707070; margin-top: 20px;">
INSTALL MANIFEST
────────────────
binary      /usr/local/bin/keystream
service     /Library/LaunchAgents/com.gauchodsp.keystream.plist
voices      32 concurrent
oscillator  recursive sine (no trig in audio thread)
latency     &lt; 1ms
</pre>

<p style="color: #e0e0e0; margin-top: 20px; border: 1px solid #444; padding: 12px; background: #111;">
<strong style="color: #ff6b35;">ATTENTION</strong><br/><br/>
after installation, grant accessibility permission:<br/><br/>
&nbsp;&nbsp;system settings &gt; privacy &amp; security &gt; accessibility<br/><br/>
without this permission, keyboard input cannot be captured.<br/>
the system will not function.
</p>

<p style="color: #555; margin-top: 30px; font-size: 10px;">
READY FOR INSTALLATION
</p>

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
