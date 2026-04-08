#!/usr/bin/env bash
# =============================================================================
# Sherpa-UA TTS — Dependency Installation Script
# Supports: x86_64, aarch64 (ARM64)
# Distributions: Debian/Ubuntu, Fedora/RHEL, Arch Linux
# =============================================================================

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Logging functions
info()    { echo -e "${BLUE}[INFO]${NC} $*"; }
success() { echo -e "${GREEN}[OK]${NC} $*"; }
warn()    { echo -e "${YELLOW}[WARN]${NC} $*"; }
error()   { echo -e "${RED}[ERROR]${NC} $*"; }

# Detect architecture
detect_arch() {
    local arch
    arch=$(uname -m)
    case "$arch" in
        x86_64|amd64)
            ARCH="x86_64"
            success "Detected architecture: x86_64"
            ;;
        aarch64|arm64)
            ARCH="aarch64"
            success "Detected architecture: aarch64 (ARM64)"
            ;;
        armv7l|armhf)
            ARCH="armv7"
            warn "ARMv7 detected — some dependencies may not be available"
            ;;
        *)
            error "Unsupported architecture: $arch"
            echo "Supported: x86_64, aarch64, armv7l"
            exit 1
            ;;
    esac
}

# Detect package manager
detect_pkg_manager() {
    if command -v apt-get &>/dev/null; then
        PKG_MANAGER="apt"
        success "Package manager: apt (Debian/Ubuntu)"
    elif command -v dnf &>/dev/null; then
        PKG_MANAGER="dnf"
        success "Package manager: dnf (Fedora/RHEL)"
    elif command -v yum &>/dev/null; then
        PKG_MANAGER="yum"
        success "Package manager: yum (RHEL/CentOS)"
    elif command -v pacman &>/dev/null; then
        PKG_MANAGER="pacman"
        success "Package manager: pacman (Arch Linux)"
    else
        error "No supported package manager found"
        exit 1
    fi
}

# Install system dependencies
install_system_deps() {
    info "Installing system dependencies..."

    case "$PKG_MANAGER" in
        apt)
            sudo apt-get update -qq
            sudo apt-get install -y \
                build-essential \
                pkg-config \
                libssl-dev \
                python3 \
                python3-pip \
                python3-venv \
                curl \
                wget \
                lame
            ;;
        dnf|yum)
            if [ "$PKG_MANAGER" = "dnf" ]; then
                sudo dnf install -y \
                    gcc gcc-c++ make \
                    openssl-devel \
                    python3 \
                    python3-pip \
                    curl \
                    wget \
                    lame
            else
                sudo yum install -y \
                    gcc gcc-c++ make \
                    openssl-devel \
                    python3 \
                    python3-pip \
                    curl \
                    wget \
                    lame
            fi
            ;;
        pacman)
            sudo pacman -Syu --noconfirm \
                base-devel \
                openssl \
                python \
                python-pip \
                curl \
                wget \
                lame
            ;;
    esac

    success "System dependencies installed"
}

# Install Rust
install_rust() {
    if command -v rustc &>/dev/null && command -v cargo &>/dev/null; then
        local rust_version
        rust_version=$(rustc --version)
        info "Rust already installed: $rust_version"

        # Check minimum version (1.70)
        local major
        major=$(rustc --version | grep -oP '\d+\.\d+' | head -1 | cut -d. -f1)
        if [ "$major" -lt 1 ]; then
            error "Rust version too old, need >= 1.70"
            exit 1
        fi
        return
    fi

    info "Installing Rust..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    source "$HOME/.cargo/env"
    success "Rust installed: $(rustc --version)"
}

# Install Piper TTS
install_piper() {
    if command -v piper &>/dev/null; then
        info "Piper TTS already installed: $(piper --version 2>&1 | head -1)"
        return
    fi

    info "Installing Piper TTS via pip..."

    # Create virtual environment if not exists
    if [ ! -d "$HOME/.local/piper-venv" ]; then
        python3 -m venv "$HOME/.local/piper-venv"
    fi

    source "$HOME/.local/piper-venv/bin/activate"
    pip install --upgrade pip
    pip install piper-tts
    deactivate

    # Add to PATH
    local piper_bin="$HOME/.local/piper-venv/bin"
    if ! echo "$PATH" | grep -q "$piper_bin"; then
        echo "export PATH=\"\$HOME/.local/piper-venv/bin:\$PATH\"" >> "$HOME/.bashrc"
        export PATH="$piper_bin:$PATH"
    fi

    success "Piper TTS installed"
}

# Download Piper model
download_model() {
    local model_dir="${1:-./models/piper-uk_UK-dmytro-medium}"

    if [ -f "$model_dir/model.onnx" ] && [ -f "$model_dir/model.onnx.json" ]; then
        info "Model already exists at $model_dir"
        return
    fi

    info "Downloading Piper Ukrainian model..."
    mkdir -p "$model_dir"

    # Run existing download script if available
    if [ -f "./download_models.sh" ]; then
        chmod +x ./download_models.sh
        ./download_models.sh
    else
        error "download_models.sh not found"
        echo "Please download model manually from:"
        echo "https://huggingface.co/rhasspy/piper-voices"
        exit 1
    fi

    success "Model downloaded to $model_dir"
}

# Main
main() {
    echo "============================================"
    echo "  Sherpa-UA TTS — Dependency Installer"
    echo "============================================"
    echo

    detect_arch
    detect_pkg_manager

    echo
    info "Starting installation..."
    echo

    install_system_deps
    install_rust
    install_piper

    # Optional: download model
    echo
    read -rp "Download Piper Ukrainian model now? [Y/n]: " download_model_choice
    if [[ "$download_model_choice" =~ ^[Nn]$ ]]; then
        warn "Model not downloaded. Run ./download_models.sh manually."
    else
        download_model
    fi

    echo
    echo "============================================"
    success "Installation complete!"
    echo "============================================"
    echo
    echo "Next steps:"
    echo "  1. Build:    ./scripts/build.sh"
    echo "  2. Deploy:   ./scripts/deploy.sh"
    echo "  3. Run:      ./scripts/deploy.sh start"
    echo
}

main "$@"
