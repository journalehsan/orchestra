#!/usr/bin/env bash
# ============================================================
#  Orchestra Framework — Build & Install Script
#  Usage: ./build.sh [--release] [--install] [--uninstall] [--help]
# ============================================================

set -euo pipefail

# ── Colors ──────────────────────────────────────────────────
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
BOLD='\033[1m'
RESET='\033[0m'

# ── Config ───────────────────────────────────────────────────
BINARY_NAME="orchestra"
INSTALL_DIR="${ORCHESTRA_INSTALL_DIR:-$HOME/.local/bin}"
PROFILE_FILES=("$HOME/.bashrc" "$HOME/.zshrc" "$HOME/.config/fish/config.fish")

# ── Flags ────────────────────────────────────────────────────
MODE="debug"
DO_INSTALL=false
DO_UNINSTALL=false
RUN_TESTS=false

# ── Helpers ──────────────────────────────────────────────────
info()    { echo -e "${CYAN}  →${RESET} $*"; }
success() { echo -e "${GREEN}  ✓${RESET} $*"; }
warn()    { echo -e "${YELLOW}  ⚠${RESET} $*"; }
error()   { echo -e "${RED}  ✗ ERROR:${RESET} $*" >&2; exit 1; }
section() { echo -e "\n${BOLD}$*${RESET}"; }

usage() {
    echo -e "${BOLD}Orchestra Build Script${RESET}"
    echo
    echo "  Usage: ./build.sh [options]"
    echo
    echo "  Options:"
    echo "    --release      Build in release mode (optimized)"
    echo "    --install      Install binary to ${INSTALL_DIR}"
    echo "    --uninstall    Remove installed binary"
    echo "    --test         Run tests before building"
    echo "    --help         Show this help message"
    echo
    echo "  Environment:"
    echo "    ORCHESTRA_INSTALL_DIR   Override install directory (default: ~/.local/bin)"
    echo
    echo "  Examples:"
    echo "    ./build.sh                        # Debug build"
    echo "    ./build.sh --release              # Release build"
    echo "    ./build.sh --release --install    # Build + install to PATH"
    echo "    ./build.sh --uninstall            # Remove installed binary"
    exit 0
}

# ── Parse args ───────────────────────────────────────────────
for arg in "$@"; do
    case "$arg" in
        --release)   MODE="release" ;;
        --install)   DO_INSTALL=true ;;
        --uninstall) DO_UNINSTALL=true ;;
        --test)      RUN_TESTS=true ;;
        --help|-h)   usage ;;
        *) error "Unknown option: $arg. Run ./build.sh --help" ;;
    esac
done

# ── Banner ───────────────────────────────────────────────────
echo -e "${BOLD}${CYAN}"
echo "  🎼  Orchestra Framework"
echo "  ─────────────────────────────────────────"
echo -e "${RESET}"

# ── Uninstall ────────────────────────────────────────────────
if $DO_UNINSTALL; then
    section "Uninstalling Orchestra..."
    TARGET="$INSTALL_DIR/$BINARY_NAME"
    if [[ -f "$TARGET" ]]; then
        rm -f "$TARGET"
        success "Removed $TARGET"
    else
        warn "Binary not found at $TARGET — nothing to remove."
    fi
    exit 0
fi

# ── Check: Rust / Cargo ──────────────────────────────────────
section "Checking requirements..."

if ! command -v cargo &>/dev/null; then
    error "Cargo not found. Install Rust from https://rustup.rs/"
fi

RUST_VERSION=$(rustc --version)
CARGO_VERSION=$(cargo --version)
success "Rust  : $RUST_VERSION"
success "Cargo : $CARGO_VERSION"

# Check we're in the workspace root
if [[ ! -f "Cargo.toml" ]]; then
    error "Cargo.toml not found. Run this script from the orchestra workspace root."
fi

if ! grep -q '\[workspace\]' Cargo.toml; then
    error "Not in the workspace root. Expected a [workspace] Cargo.toml."
fi

# ── Tests ────────────────────────────────────────────────────
if $RUN_TESTS; then
    section "Running tests..."
    cargo test --workspace 2>&1 | sed 's/^/  /'
    success "All tests passed."
fi

# ── Build ────────────────────────────────────────────────────
section "Building Orchestra CLI (${MODE} mode)..."

if [[ "$MODE" == "release" ]]; then
    cargo build --release --package orchestra 2>&1 | sed 's/^/  /'
    BINARY_PATH="target/release/$BINARY_NAME"
else
    cargo build --package orchestra 2>&1 | sed 's/^/  /'
    BINARY_PATH="target/debug/$BINARY_NAME"
fi

if [[ ! -f "$BINARY_PATH" ]]; then
    error "Build succeeded but binary not found at $BINARY_PATH"
fi

BINARY_SIZE=$(du -sh "$BINARY_PATH" | cut -f1)
success "Binary built: ${BOLD}$BINARY_PATH${RESET} (${BINARY_SIZE})"

# ── Install ──────────────────────────────────────────────────
if $DO_INSTALL; then
    section "Installing to $INSTALL_DIR..."

    mkdir -p "$INSTALL_DIR"
    cp "$BINARY_PATH" "$INSTALL_DIR/$BINARY_NAME"
    chmod +x "$INSTALL_DIR/$BINARY_NAME"
    success "Installed: $INSTALL_DIR/$BINARY_NAME"

    # Check PATH
    if echo "$PATH" | grep -q "$INSTALL_DIR"; then
        success "$INSTALL_DIR is already in PATH."
    else
        warn "$INSTALL_DIR is not in your PATH."
        echo
        echo -e "  Add it by appending one of the following to your shell config:"
        echo
        echo -e "  ${BOLD}bash / zsh:${RESET}"
        echo -e "    export PATH=\"\$HOME/.local/bin:\$PATH\""
        echo
        echo -e "  ${BOLD}fish:${RESET}"
        echo -e "    fish_add_path \$HOME/.local/bin"
        echo
    fi

    # Verify
    if command -v "$BINARY_NAME" &>/dev/null; then
        INSTALLED_VERSION=$("$BINARY_NAME" --version 2>&1 || true)
        success "Verified: \`orchestra\` is available in PATH  ($INSTALLED_VERSION)"
    else
        warn "orchestra not found in current PATH yet. Open a new terminal or source your shell config."
    fi
fi

# ── Done ─────────────────────────────────────────────────────
echo
echo -e "${GREEN}${BOLD}  Build complete!${RESET}"
echo
if ! $DO_INSTALL; then
    echo -e "  Run the binary directly:"
    echo -e "    ${BOLD}./$BINARY_PATH --help${RESET}"
    echo
    echo -e "  Or install it to PATH:"
    echo -e "    ${BOLD}./build.sh --release --install${RESET}"
fi
echo
