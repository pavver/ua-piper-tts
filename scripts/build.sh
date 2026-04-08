#!/usr/bin/env bash
# =============================================================================
# UA-Piper-TTS — Build Script
# Supports: x86_64, aarch64 (ARM64), armv7
# Builds: dev, release, cross-compile
# =============================================================================

set -euo pipefail

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

info()    { echo -e "${BLUE}[INFO]${NC} $*"; }
success() { echo -e "${GREEN}[OK]${NC} $*"; }
warn()    { echo -e "${YELLOW}[WARN]${NC} $*"; }
error()   { echo -e "${RED}[ERROR]${NC} $*"; }

# Defaults
BUILD_TYPE="release"
TARGET=""
OUTPUT_DIR="./target/dist"

# Usage
usage() {
    cat <<EOF
Usage: $0 [OPTIONS]

Options:
  -t, --type TYPE       Build type: dev, release (default: release)
  -a, --arch ARCH       Target architecture: x86_64, aarch64, armv7
  -o, --output DIR      Output directory (default: ./target/dist)
  -c, --clean           Clean before build
  -h, --help            Show this help

Examples:
  $0                          # Release build for current arch
  $0 -t dev                   # Dev build
  $0 -a aarch64               # Cross-compile for ARM64
  $0 -a armv7 -t release      # Cross-compile for ARMv7
  $0 -c                       # Clean and build
EOF
    exit 0
}

# Parse arguments
while [[ $# -gt 0 ]]; do
    case "$1" in
        -t|--type)
            BUILD_TYPE="$2"
            shift 2
            ;;
        -a|--arch)
            TARGET="$2"
            shift 2
            ;;
        -o|--output)
            OUTPUT_DIR="$2"
            shift 2
            ;;
        -c|--clean)
            CLEAN=true
            shift
            ;;
        -h|--help)
            usage
            ;;
        *)
            error "Unknown option: $1"
            usage
            ;;
    esac
done

# Validate build type
case "$BUILD_TYPE" in
    dev|debug)
        BUILD_TYPE="dev"
        CARGO_FLAGS=""
        ;;
    release)
        BUILD_TYPE="release"
        CARGO_FLAGS="--release"
        ;;
    *)
        error "Invalid build type: $BUILD_TYPE"
        echo "Use 'dev' or 'release'"
        exit 1
        ;;
esac

# Detect architecture if not specified
detect_arch() {
    if [ -z "$TARGET" ]; then
        TARGET=$(uname -m)
        case "$TARGET" in
            x86_64|amd64) TARGET="x86_64" ;;
            aarch64|arm64) TARGET="aarch64" ;;
            armv7l|armhf) TARGET="armv7" ;;
        esac
    fi
    success "Target architecture: $TARGET"
}

# Install cross-compilation tools if needed
install_cross_deps() {
    local current_arch
    current_arch=$(uname -m)

    # Only need cross tools if target != current
    if [ "$TARGET" = "$current_arch" ] || \
       { [ "$current_arch" = "x86_64" ] && [ "$TARGET" = "x86_64" ]; } || \
       { [ "$current_arch" = "aarch64" ] && [ "$TARGET" = "aarch64" ]; }; then
        return
    fi

    info "Cross-compilation detected: $current_arch → $TARGET"

    # Install cross toolchain
    if command -v apt-get &>/dev/null; then
        case "$TARGET" in
            aarch64)
                info "Installing ARM64 cross-compilation tools..."
                sudo apt-get install -y \
                    gcc-aarch64-linux-gnu \
                    g++-aarch64-linux-gnu \
                    libc6-dev-arm64-cross
                export CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER=aarch64-linux-gnu-gcc
                TARGET_TRIPLE="aarch64-unknown-linux-gnu"
                ;;
            armv7)
                info "Installing ARMv7 cross-compilation tools..."
                sudo apt-get install -y \
                    gcc-arm-linux-gnueabihf \
                    g++-arm-linux-gnueabihf \
                    libc6-dev-armhf-cross
                export CARGO_TARGET_ARMV7_UNKNOWN_LINUX_GNUEABIHF_LINKER=arm-linux-gnueabihf-gcc
                TARGET_TRIPLE="armv7-unknown-linux-gnueabihf"
                ;;
        esac
    else
        warn "Auto-install not supported for your package manager"
        echo "Please install cross-compilation tools manually for $TARGET"
    fi
}

# Clean if requested
if [ "${CLEAN:-false}" = true ]; then
    info "Cleaning build artifacts..."
    cargo clean
fi

# Detect architecture
detect_arch

# Install cross-compilation dependencies if needed
install_cross_deps

# Build
echo
info "Building UA-Piper-TTS..."
echo "  Type: $BUILD_TYPE"
echo "  Target: $TARGET"
echo "  Output: $OUTPUT_DIR"
echo

mkdir -p "$OUTPUT_DIR"

if [ -n "${TARGET_TRIPLE:-}" ]; then
    # Cross-compilation
    info "Cross-compiling for $TARGET_TRIPLE..."
    cargo build --target "$TARGET_TRIPLE" $CARGO_FLAGS
    cp "target/$TARGET_TRIPLE/$BUILD_TYPE/ua-piper-tts" "$OUTPUT_DIR/"
else
    # Native build
    cargo build $CARGO_FLAGS
    if [ "$BUILD_TYPE" = "release" ]; then
        cp "target/release/ua-piper-tts" "$OUTPUT_DIR/"
    else
        cp "target/debug/ua-piper-tts" "$OUTPUT_DIR/"
    fi
fi

# Copy supporting files
info "Copying supporting files..."
cp config.json "$OUTPUT_DIR/" 2>/dev/null || warn "config.json not found"
cp download_models.sh "$OUTPUT_DIR/" 2>/dev/null || warn "download_models.sh not found"
chmod +x "$OUTPUT_DIR/download_models.sh" 2>/dev/null || true

# Create run script
cat > "$OUTPUT_DIR/run.sh" <<'RUNEOF'
#!/usr/bin/env bash
set -euo pipefail
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

if [ ! -f "config.json" ]; then
    echo "Error: config.json not found"
    exit 1
fi

exec ./ua-piper-tts "$@"
RUNEOF
chmod +x "$OUTPUT_DIR/run.sh"

success "Build complete!"
echo
echo "Binary location: $OUTPUT_DIR/ua-piper-tts"
echo
echo "To run:"
echo "  cd $OUTPUT_DIR && ./run.sh"
echo
echo "To deploy:"
echo "  ./scripts/deploy.sh"
echo
