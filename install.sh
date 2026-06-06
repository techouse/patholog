#!/usr/bin/env sh

# patholog installer - https://github.com/techouse/patholog
# Usage: curl -fsSL https://raw.githubusercontent.com/techouse/patholog/refs/heads/main/install.sh | sh

set -e

REPO="techouse/patholog"
BINARY_NAME="patholog"
INSTALL_DIR="${PATHOLOG_INSTALL_DIR:-$HOME/.local/bin}"
TEMP_DIR=""

if [ -t 1 ]; then
    RED='\033[0;31m'
    GREEN='\033[0;32m'
    YELLOW='\033[1;33m'
    NC='\033[0m'
else
    RED=''
    GREEN=''
    YELLOW=''
    NC=''
fi

info() {
    printf "%b[INFO]%b %s\n" "$GREEN" "$NC" "$1"
}

warn() {
    printf "%b[WARN]%b %s\n" "$YELLOW" "$NC" "$1" >&2
}

error() {
    printf "%b[ERROR]%b %s\n" "$RED" "$NC" "$1" >&2
    exit 1
}

cleanup() {
    if [ -n "$TEMP_DIR" ] && [ -d "$TEMP_DIR" ]; then
        rm -rf "$TEMP_DIR"
    fi
}

need_cmd() {
    if ! command -v "$1" >/dev/null 2>&1; then
        error "Required command not found: $1"
    fi
}

detect_os() {
    case "$(uname -s)" in
        Linux*) OS="linux" ;;
        Darwin*) OS="darwin" ;;
        *) error "Unsupported operating system: $(uname -s)" ;;
    esac
}

detect_arch() {
    case "$(uname -m)" in
        x86_64 | amd64) ARCH="x86_64" ;;
        arm64 | aarch64) ARCH="aarch64" ;;
        *) error "Unsupported architecture: $(uname -m)" ;;
    esac
}

detect_linux_libc() {
    LINUX_LIBC="${PATHOLOG_LINUX_LIBC:-}"

    if [ -n "$LINUX_LIBC" ]; then
        case "$LINUX_LIBC" in
            gnu | musl) return ;;
            *) error "PATHOLOG_LINUX_LIBC must be 'gnu' or 'musl'" ;;
        esac
    fi

    LINUX_LIBC="gnu"
    if command -v ldd >/dev/null 2>&1; then
        if LDD_OUTPUT=$(ldd --version 2>&1); then
            :
        else
            LDD_OUTPUT=$(ldd 2>&1 || true)
        fi

        if printf "%s" "$LDD_OUTPUT" | grep -qi "musl"; then
            LINUX_LIBC="musl"
        fi
    fi
}

get_version() {
    if [ -n "${PATHOLOG_VERSION:-}" ]; then
        case "$PATHOLOG_VERSION" in
            v*)
                VERSION_TAG="$PATHOLOG_VERSION"
                ASSET_VERSION="${PATHOLOG_VERSION#v}"
                ;;
            *)
                VERSION_TAG="v$PATHOLOG_VERSION"
                ASSET_VERSION="$PATHOLOG_VERSION"
                ;;
        esac
    else
        VERSION_TAG=$(
            curl -fsSL "https://api.github.com/repos/${REPO}/releases/latest" |
                sed -n 's/.*"tag_name"[[:space:]]*:[[:space:]]*"\([^"]*\)".*/\1/p' |
                head -n 1
        )
        ASSET_VERSION="${VERSION_TAG#v}"
    fi

    if [ -z "$VERSION_TAG" ] || [ -z "$ASSET_VERSION" ]; then
        error "Failed to determine latest release version"
    fi
}

get_artifact() {
    case "$OS" in
        linux)
            detect_linux_libc
            TARGET="${ARCH}-unknown-linux-${LINUX_LIBC}"
            ARCHIVE_NAME="${BINARY_NAME}-${ASSET_VERSION}-${TARGET}.tar.gz"
            PACKAGE_NAME="${ARCHIVE_NAME%.tar.gz}"
            ;;
        darwin)
            TARGET="universal-apple-darwin"
            ARCHIVE_NAME="${BINARY_NAME}-${ASSET_VERSION}-${TARGET}.zip"
            PACKAGE_NAME="${ARCHIVE_NAME%.zip}"
            ;;
    esac

    DOWNLOAD_URL="https://github.com/${REPO}/releases/download/${VERSION_TAG}/${ARCHIVE_NAME}"
    ARCHIVE_PATH="${TEMP_DIR}/${ARCHIVE_NAME}"
    CHECKSUM_NAME="${ARCHIVE_NAME}.sha256"
    CHECKSUM_PATH="${TEMP_DIR}/${CHECKSUM_NAME}"
    BINARY_PATH="${TEMP_DIR}/${PACKAGE_NAME}/${BINARY_NAME}"
    INSTALL_PATH="${INSTALL_DIR}/${BINARY_NAME}"
}

download_artifact() {
    info "Downloading from: $DOWNLOAD_URL"
    if ! curl -fsSL "$DOWNLOAD_URL" -o "$ARCHIVE_PATH"; then
        error "Failed to download release artifact"
    fi

    info "Downloading checksum"
    if ! curl -fsSL "${DOWNLOAD_URL}.sha256" -o "$CHECKSUM_PATH"; then
        error "Failed to download checksum"
    fi
}

verify_checksum() {
    info "Verifying checksum"

    if command -v sha256sum >/dev/null 2>&1; then
        if ! (cd "$TEMP_DIR" && sha256sum -c "$CHECKSUM_NAME" >/dev/null); then
            error "Checksum verification failed"
        fi
    elif command -v shasum >/dev/null 2>&1; then
        if ! (cd "$TEMP_DIR" && shasum -a 256 -c "$CHECKSUM_NAME" >/dev/null); then
            error "Checksum verification failed"
        fi
    else
        warn "No SHA-256 verifier found; skipping checksum verification"
    fi
}

extract_artifact() {
    info "Extracting"

    case "$ARCHIVE_NAME" in
        *.tar.gz)
            need_cmd tar
            tar -xzf "$ARCHIVE_PATH" -C "$TEMP_DIR"
            ;;
        *.zip)
            need_cmd unzip
            unzip -q "$ARCHIVE_PATH" -d "$TEMP_DIR"
            ;;
        *)
            error "Unsupported archive format: $ARCHIVE_NAME"
            ;;
    esac

    if [ ! -f "$BINARY_PATH" ]; then
        error "Archive did not contain expected binary: ${PACKAGE_NAME}/${BINARY_NAME}"
    fi
}

install_binary() {
    mkdir -p "$INSTALL_DIR"
    cp "$BINARY_PATH" "$INSTALL_PATH"
    chmod 755 "$INSTALL_PATH"
    info "Successfully installed ${BINARY_NAME} to ${INSTALL_PATH}"
}

install_completion() {
    COMPLETION_SHELL="$1"
    COMPLETION_SOURCE_NAME="$2"
    COMPLETION_DIR="$3"
    COMPLETION_PATH="$4"
    COMPLETION_SOURCE_PATH="${TEMP_DIR}/${PACKAGE_NAME}/completions/${COMPLETION_SOURCE_NAME}"

    if ! mkdir -p "$COMPLETION_DIR"; then
        warn "Failed to create completion directory: ${COMPLETION_DIR}"
        return
    fi

    if [ -f "$COMPLETION_SOURCE_PATH" ]; then
        if cp "$COMPLETION_SOURCE_PATH" "$COMPLETION_PATH"; then
            info "Installed ${COMPLETION_SHELL} completions to ${COMPLETION_PATH}"
        else
            warn "Failed to install ${COMPLETION_SHELL} completions to ${COMPLETION_PATH}"
        fi
        return
    fi

    if "$INSTALL_PATH" completions "$COMPLETION_SHELL" >"$COMPLETION_PATH"; then
        info "Generated ${COMPLETION_SHELL} completions at ${COMPLETION_PATH}"
    else
        rm -f "$COMPLETION_PATH"
        warn "Failed to generate ${COMPLETION_SHELL} completions"
    fi
}

install_completions() {
    if [ "${PATHOLOG_INSTALL_COMPLETIONS:-1}" = "0" ]; then
        info "Skipping shell completions"
        return
    fi

    install_completion \
        bash \
        patholog.bash \
        "$HOME/.local/share/bash-completion/completions" \
        "$HOME/.local/share/bash-completion/completions/patholog"
    install_completion \
        zsh \
        _patholog \
        "$HOME/.zfunc" \
        "$HOME/.zfunc/_patholog"
    install_completion \
        fish \
        patholog.fish \
        "$HOME/.config/fish/completions" \
        "$HOME/.config/fish/completions/patholog.fish"

    info "For zsh completions, ensure ~/.zfunc is in fpath before running compinit."
}

verify_installation() {
    if ! INSTALLED_VERSION=$("$INSTALL_PATH" --version 2>/dev/null); then
        error "Installed binary failed to run: $INSTALL_PATH"
    fi

    info "Verification: $INSTALLED_VERSION"

    if command -v "$BINARY_NAME" >/dev/null 2>&1; then
        PATH_BINARY=$(command -v "$BINARY_NAME")
        if [ "$PATH_BINARY" != "$INSTALL_PATH" ]; then
            warn "${BINARY_NAME} on PATH resolves to ${PATH_BINARY}"
            warn "Add ${INSTALL_DIR} earlier in PATH to use this installation"
        fi
    else
        warn "Binary installed but not in PATH. Add this to your shell profile:"
        warn "  export PATH=\"${INSTALL_DIR}:\$PATH\""
    fi
}

main() {
    trap cleanup EXIT INT TERM

    need_cmd curl
    need_cmd grep
    need_cmd head
    need_cmd mktemp
    need_cmd sed
    detect_os
    detect_arch
    get_version

    TEMP_DIR=$(mktemp -d)
    get_artifact

    info "Installing $BINARY_NAME"
    info "Detected: $OS $ARCH"
    info "Target: $TARGET"
    info "Version: $VERSION_TAG"

    download_artifact
    verify_checksum
    extract_artifact
    install_binary
    install_completions
    verify_installation

    echo ""
    info "Installation complete. Run '$BINARY_NAME --help' to get started."
}

main
