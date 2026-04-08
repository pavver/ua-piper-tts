#!/usr/bin/env bash
# =============================================================================
# Sherpa-UA TTS — Quick Start Script
# One-command deployment: install deps → build → deploy → start
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

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

echo "============================================"
echo "  Sherpa-UA TTS — Quick Start"
echo "============================================"
echo
echo "This will:"
echo "  1. Install dependencies"
echo "  2. Build the project"
echo "  3. Deploy as systemd service"
echo "  4. Start the service"
echo

read -rp "Continue? [Y/n]: " confirm
if [[ "$confirm" =~ ^[Nn]$ ]]; then
    info "Aborted"
    exit 0
fi

echo
info "Step 1/4: Installing dependencies..."
echo
"$SCRIPT_DIR/install_deps.sh"

echo
info "Step 2/4: Building project..."
echo
"$SCRIPT_DIR/build.sh"

echo
info "Step 3/4: Deploying service..."
echo
sudo "$SCRIPT_DIR/deploy.sh" install

echo
info "Step 4/4: Starting service..."
echo
sudo "$SCRIPT_DIR/deploy.sh" start

echo
echo "============================================"
success "Sherpa-UA TTS is now running!"
echo "============================================"
echo
echo "Check status:  sudo $SCRIPT_DIR/deploy.sh status"
echo "View logs:     sudo $SCRIPT_DIR/deploy.sh logs -f"
echo "Stop service:  sudo $SCRIPT_DIR/deploy.sh stop"
echo
echo "API endpoint: http://localhost:9000"
echo "Health check: curl http://localhost:9000/health"
echo
