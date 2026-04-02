#!/bin/sh
# keystream installer
# usage: curl -sSL https://raw.githubusercontent.com/cucuwritescode/keystream/main/scripts/install.sh | sh
set -eu

REPO="cucuwritescode/keystream"
INSTALL_DIR="${HOME}/.local/bin"
BINARY_NAME="keystream"

main() {
    echo ""
    echo "  keystream installer"
    echo "  -------------------"
    echo ""

    # detect platform
    os="$(uname -s)"
    if [ "$os" != "Darwin" ]; then
        error "keystream only supports macos. detected: ${os}"
    fi

    # resolve latest version
    version="$(resolve_version)"
    echo "  version   : ${version}"

    # download binary
    url="https://github.com/${REPO}/releases/download/${version}/${BINARY_NAME}"
    checksum_url="https://github.com/${REPO}/releases/download/${version}/${BINARY_NAME}.sha256"

    tmpdir="$(mktemp -d)"
    trap 'rm -rf "$tmpdir"' EXIT

    echo "  downloading..."
    download "$url" "${tmpdir}/${BINARY_NAME}"
    download "$checksum_url" "${tmpdir}/${BINARY_NAME}.sha256"

    # verify checksum
    echo "  verifying checksum..."
    cd "$tmpdir"
    if ! shasum -a 256 -c "${BINARY_NAME}.sha256" >/dev/null 2>&1; then
        error "checksum verification failed. the download may be corrupted."
    fi
    cd - >/dev/null

    # install
    mkdir -p "$INSTALL_DIR"
    cp "${tmpdir}/${BINARY_NAME}" "${INSTALL_DIR}/${BINARY_NAME}"
    chmod +x "${INSTALL_DIR}/${BINARY_NAME}"

    echo ""
    echo "  installed to ${INSTALL_DIR}/${BINARY_NAME}"
    echo ""

    # check PATH
    case ":${PATH}:" in
        *":${INSTALL_DIR}:"*) ;;
        *)
            echo "  add to PATH:"
            echo "    export PATH=\"${INSTALL_DIR}:\$PATH\""
            echo ""
            echo "  add the above line to your shell profile (~/.zshrc or ~/.bashrc)"
            echo ""
            ;;
    esac

    echo "  run 'keystream start' to begin"
    echo ""
}

resolve_version() {
    if command -v curl >/dev/null 2>&1; then
        curl -sI "https://github.com/${REPO}/releases/latest" \
            | grep -i '^location:' \
            | sed 's|.*/||' \
            | tr -d '\r\n'
    elif command -v wget >/dev/null 2>&1; then
        wget -qO- --server-response "https://github.com/${REPO}/releases/latest" 2>&1 \
            | grep -i '^  location:' \
            | sed 's|.*/||' \
            | tr -d '\r\n'
    else
        error "curl or wget is required"
    fi
}

download() {
    if command -v curl >/dev/null 2>&1; then
        curl -fsSL -o "$2" "$1"
    elif command -v wget >/dev/null 2>&1; then
        wget -qO "$2" "$1"
    else
        error "curl or wget is required"
    fi
}

error() {
    echo "  error: $1" >&2
    exit 1
}

main
