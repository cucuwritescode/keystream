#!/bin/sh
# keystream uninstaller
# removes binary, daemon plist, and pid/mode files
set -eu

BINARY_NAME="keystream"
INSTALL_DIR="${HOME}/.local/bin"
PLIST_NAME="com.gauchodsp.keystream.plist"
PLIST_DIR="${HOME}/Library/LaunchAgents"

main() {
    echo ""
    echo "  keystream uninstaller"
    echo "  ---------------------"
    echo ""

    # stop daemon if running
    if launchctl list "$PLIST_NAME" >/dev/null 2>&1; then
        echo "  stopping daemon..."
        launchctl unload "${PLIST_DIR}/${PLIST_NAME}" 2>/dev/null || true
    fi

    # remove plist
    if [ -f "${PLIST_DIR}/${PLIST_NAME}" ]; then
        rm -f "${PLIST_DIR}/${PLIST_NAME}"
        echo "  removed ${PLIST_DIR}/${PLIST_NAME}"
    fi

    # kill running process via pid file
    if [ -f /tmp/keystream.pid ]; then
        pid="$(cat /tmp/keystream.pid)"
        if kill -0 "$pid" 2>/dev/null; then
            echo "  stopping keystream (pid ${pid})..."
            kill "$pid" 2>/dev/null || true
        fi
        rm -f /tmp/keystream.pid
        rm -f /tmp/keystream.mode
    fi

    # remove binary
    if [ -f "${INSTALL_DIR}/${BINARY_NAME}" ]; then
        rm -f "${INSTALL_DIR}/${BINARY_NAME}"
        echo "  removed ${INSTALL_DIR}/${BINARY_NAME}"
    else
        echo "  binary not found at ${INSTALL_DIR}/${BINARY_NAME}"
    fi

    echo ""
    echo "  keystream has been removed"
    echo ""
}

main
