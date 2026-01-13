#!/bin/bash
#
# Alfred Installer
# https://github.com/wesscope/alfred
#
# Usage:
#   curl -fsSL https://raw.githubusercontent.com/wesscope/alfred/main/install.sh | bash
#
# Options (via environment variables):
#   ALFRED_INSTALL_DIR  - Installation directory (default: /usr/local/bin)
#   ALFRED_VERSION      - Specific version to install (default: latest)
#   ALFRED_NO_MODIFY_PATH - Set to 1 to skip PATH modification
#

set -e

# Configuration
REPO="wesscope/alfred"
INSTALL_DIR="${ALFRED_INSTALL_DIR:-/usr/local/bin}"
VERSION="${ALFRED_VERSION:-latest}"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[0;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color
BOLD='\033[1m'

# Helper functions
info() {
    echo -e "${BLUE}i${NC} $1"
}

success() {
    echo -e "${GREEN}✓${NC} $1"
}

warn() {
    echo -e "${YELLOW}!${NC} $1"
}

error() {
    echo -e "${RED}✗${NC} $1"
    exit 1
}

# Detect OS
detect_os() {
    case "$(uname -s)" in
        Linux*)     OS="linux" ;;
        Darwin*)    OS="macos" ;;
        CYGWIN*|MINGW*|MSYS*) OS="windows" ;;
        *)          error "Unsupported operating system: $(uname -s)" ;;
    esac
}

# Detect architecture
detect_arch() {
    case "$(uname -m)" in
        x86_64|amd64)   ARCH="x86_64" ;;
        arm64|aarch64)  ARCH="aarch64" ;;
        *)              error "Unsupported architecture: $(uname -m)" ;;
    esac
}

# Get latest version from GitHub
get_latest_version() {
    if [ "$VERSION" = "latest" ]; then
        VERSION=$(curl -fsSL "https://api.github.com/repos/${REPO}/releases/latest" | grep '"tag_name"' | sed -E 's/.*"([^"]+)".*/\1/')
        if [ -z "$VERSION" ]; then
            error "Failed to fetch latest version. Please specify a version with ALFRED_VERSION=vX.Y.Z"
        fi
    fi

    # Ensure version has 'v' prefix for tag (used in download URL path)
    if [[ "$VERSION" != v* ]]; then
        VERSION="v$VERSION"
    fi
}

# Check if command exists
command_exists() {
    command -v "$1" >/dev/null 2>&1
}

# Check dependencies
check_dependencies() {
    if ! command_exists curl; then
        error "curl is required but not installed. Please install curl first."
    fi

    if ! command_exists tar; then
        error "tar is required but not installed. Please install tar first."
    fi
}

# Download and install
install_alfred() {
    local tmp_dir
    tmp_dir=$(mktemp -d)
    trap "rm -rf $tmp_dir" EXIT

    # Check if pre-built binary is available for this platform
    # Linux ARM64 requires building from source due to llama-cpp C++ dependencies
    if [[ "$OS" == "linux" && "$ARCH" == "aarch64" ]]; then
        warn "Pre-built binaries are not available for Linux ARM64"
        echo ""
        echo -e "${BOLD}Linux ARM64 requires building from source:${NC}"
        echo ""
        echo "  # Install Rust if needed"
        echo "  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
        echo ""
        echo "  # Build Alfred"
        echo "  git clone https://github.com/${REPO}.git"
        echo "  cd alfred"
        echo "  cargo build --release"
        echo "  sudo cp target/release/alfred target/release/alferd ${INSTALL_DIR}/"
        echo ""
        exit 1
    fi

    # Construct download URL
    local archive_name="alfred-${VERSION}-${OS}-${ARCH}.tar.gz"
    local download_url="https://github.com/${REPO}/releases/download/${VERSION}/${archive_name}"

    info "Downloading Alfred ${VERSION} for ${OS}-${ARCH}..."

    # Download archive
    if ! curl -fsSL "$download_url" -o "$tmp_dir/alfred.tar.gz" 2>/dev/null; then
        # If pre-built binary doesn't exist, provide build instructions
        warn "Pre-built binary not found for ${OS}-${ARCH}"
        echo ""
        echo -e "${BOLD}To install from source:${NC}"
        echo ""
        echo "  git clone https://github.com/${REPO}.git"
        echo "  cd alfred"
        echo "  cargo build --release"
        echo "  sudo cp target/release/alfred target/release/alferd ${INSTALL_DIR}/"
        echo ""
        exit 1
    fi

    # Extract archive
    info "Extracting..."
    tar -xzf "$tmp_dir/alfred.tar.gz" -C "$tmp_dir"

    # Create install directory if needed
    if [ ! -d "$INSTALL_DIR" ]; then
        info "Creating directory: $INSTALL_DIR"
        sudo mkdir -p "$INSTALL_DIR"
    fi

    # Install binaries
    info "Installing to $INSTALL_DIR..."

    # Check if we need sudo
    if [ -w "$INSTALL_DIR" ]; then
        cp "$tmp_dir/alfred" "$INSTALL_DIR/"
        cp "$tmp_dir/alferd" "$INSTALL_DIR/" 2>/dev/null || true
        chmod +x "$INSTALL_DIR/alfred"
        chmod +x "$INSTALL_DIR/alferd" 2>/dev/null || true
    else
        sudo cp "$tmp_dir/alfred" "$INSTALL_DIR/"
        sudo cp "$tmp_dir/alferd" "$INSTALL_DIR/" 2>/dev/null || true
        sudo chmod +x "$INSTALL_DIR/alfred"
        sudo chmod +x "$INSTALL_DIR/alferd" 2>/dev/null || true
    fi

    success "Alfred installed successfully!"
}

# Check if install dir is in PATH
check_path() {
    if [[ ":$PATH:" != *":$INSTALL_DIR:"* ]]; then
        warn "$INSTALL_DIR is not in your PATH"
        echo ""
        echo "Add it to your shell profile:"
        echo ""

        local shell_name
        shell_name=$(basename "$SHELL")

        case "$shell_name" in
            bash)
                echo "  echo 'export PATH=\"$INSTALL_DIR:\$PATH\"' >> ~/.bashrc"
                echo "  source ~/.bashrc"
                ;;
            zsh)
                echo "  echo 'export PATH=\"$INSTALL_DIR:\$PATH\"' >> ~/.zshrc"
                echo "  source ~/.zshrc"
                ;;
            fish)
                echo "  set -U fish_user_paths $INSTALL_DIR \$fish_user_paths"
                ;;
            *)
                echo "  export PATH=\"$INSTALL_DIR:\$PATH\""
                ;;
        esac
        echo ""
    fi
}

# Print next steps
print_next_steps() {
    echo ""
    echo -e "${BOLD}${CYAN}Alfred installed!${NC}"
    echo ""
    echo "Next steps:"
    echo ""
    echo "  1. Download the AI model:"
    echo "     ${CYAN}alfred setup${NC}"
    echo ""
    echo "  2. Start using Alfred:"
    echo "     ${CYAN}alfred commit${NC}      # AI-generated commit messages"
    echo "     ${CYAN}alfred branch new${NC}  # AI-suggested branch names"
    echo "     ${CYAN}alfred resolve${NC}     # AI-assisted conflict resolution"
    echo ""
    echo "  3. (Optional) Start the daemon for faster responses:"
    echo "     ${CYAN}alfred daemon start${NC}"
    echo ""
    echo "Documentation: https://github.com/${REPO}#readme"
    echo ""
}

# Main installation flow
main() {
    echo ""
    echo -e "${BOLD}Alfred Installer${NC}"
    echo ""

    check_dependencies
    detect_os
    detect_arch

    info "Detected: ${OS}-${ARCH}"

    get_latest_version

    install_alfred
    check_path
    print_next_steps
}

# Handle Windows separately
if [[ "$(uname -s)" == CYGWIN* ]] || [[ "$(uname -s)" == MINGW* ]] || [[ "$(uname -s)" == MSYS* ]]; then
    error "This installer doesn't support Windows. Please use:

  1. Download from: https://github.com/${REPO}/releases
  2. Or build from source with: cargo install --path .
"
fi

# Run main
main "$@"
