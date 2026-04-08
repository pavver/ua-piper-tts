#!/usr/bin/env bash
# =============================================================================
# UA-Piper-TTS — Deployment Script
# Supports: install, start, stop, restart, status, uninstall
# Systemd service management
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

# Configuration
SERVICE_NAME="ua-piper-tts"
INSTALL_DIR="/opt/ua-piper-tts"
SERVICE_FILE="/etc/systemd/system/${SERVICE_NAME}.service"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"

# Usage
usage() {
    cat <<EOF
Usage: $0 {install|start|stop|restart|status|uninstall|logs}

Commands:
  install     Install and enable the service
  start       Start the service
  stop        Stop the service
  restart     Restart the service
  status      Show service status
  logs        Show service logs
  uninstall   Remove the service

Examples:
  $0 install      # Install as systemd service
  $0 start        # Start the service
  $0 logs -f      # Follow logs
  $0 status       # Check status
EOF
    exit 0
}

# Check if running as root for install/uninstall
check_root() {
    if [ "$EUID" -ne 0 ]; then
        error "This command requires root privileges"
        echo "Run with: sudo $0 $ACTION"
        exit 1
    fi
}

# Detect binary architecture
detect_binary_arch() {
    if [ ! -f "$PROJECT_DIR/target/release/ua-piper-tts" ]; then
        error "Binary not found. Run ./scripts/build.sh first"
        exit 1
    fi

    local binary="$PROJECT_DIR/target/release/ua-piper-tts"
    local arch_info
    arch_info=$(file "$binary" | grep -oP '(x86-64|aarch64|ARM)')

    case "$arch_info" in
        *x86-64*)
            success "Binary architecture: x86_64"
            ;;
        *aarch64*)
            success "Binary architecture: aarch64 (ARM64)"
            ;;
        *ARM*)
            success "Binary architecture: ARM"
            ;;
        *)
            warn "Could not detect binary architecture"
            ;;
    esac
}

# Install service
install_service() {
    check_root

    info "Installing UA-Piper-TTS service..."

    # Create install directory
    mkdir -p "$INSTALL_DIR"
    mkdir -p "$INSTALL_DIR/output"
    mkdir -p "$INSTALL_DIR/models"

    # Copy binary and files
    info "Copying files to $INSTALL_DIR..."
    cp "$PROJECT_DIR/target/release/ua-piper-tts" "$INSTALL_DIR/"
    cp "$PROJECT_DIR/config.json" "$INSTALL_DIR/" 2>/dev/null || {
        warn "config.json not found, creating default..."
        cat > "$INSTALL_DIR/config.json" <<'CFGEOF'
{
    "output_dir": "./output",
    "port": 9000,
    "host": "0.0.0.0",
    "speaker_id": 2,
    "model_dir": "./models/piper-uk_UK-dmytro-medium"
}
CFGEOF
    }

    if [ -d "$PROJECT_DIR/models/piper-uk_UK-dmytro-medium" ]; then
        info "Copying model files..."
        cp -r "$PROJECT_DIR/models/piper-uk_UK-dmytro-medium" "$INSTALL_DIR/models/"
    else
        warn "Model not found. Run ./download_models.sh first"
    fi

    # Detect binary architecture
    detect_binary_arch

    # Create systemd service
    info "Creating systemd service..."
    cat > "$SERVICE_FILE" <<SVCEOF
[Unit]
Description=UA-Piper-TTS Server - Ukrainian Text-to-Speech
After=network.target sound.target
Wants=network.target

[Service]
Type=simple
User=tts
Group=tts
WorkingDirectory=$INSTALL_DIR
ExecStart=$INSTALL_DIR/ua-piper-tts
Restart=on-failure
RestartSec=5
StandardOutput=journal
StandardError=journal
SyslogIdentifier=ua-piper-tts

# Security hardening
NoNewPrivileges=true
ProtectSystem=strict
ProtectHome=true
ReadWritePaths=$INSTALL_DIR/output $INSTALL_DIR/tts_errors.log
PrivateTmp=true

[Install]
WantedBy=multi-user.target
SVCEOF

    # Create tts user
    if ! id -u tts &>/dev/null; then
        info "Creating 'tts' system user..."
        useradd --system --no-create-home --shell /usr/sbin/nologin tts
    fi

    # Set permissions
    chown -R tts:tts "$INSTALL_DIR"
    chmod 755 "$INSTALL_DIR/ua-piper-tts"

    # Reload systemd and enable service
    systemctl daemon-reload
    systemctl enable "$SERVICE_NAME"

    success "Service installed successfully!"
    echo
    echo "Service file: $SERVICE_FILE"
    echo "Install directory: $INSTALL_DIR"
    echo
    echo "To start: sudo $0 start"
    echo "To check status: $0 status"
    echo "To view logs: $0 logs"
}

# Start service
start_service() {
    info "Starting UA-Piper-TTS service..."
    systemctl start "$SERVICE_NAME"
    sleep 2
    status_service
}

# Stop service
stop_service() {
    info "Stopping Sherpa-UA TTS service..."
    systemctl stop "$SERVICE_NAME"
    success "Service stopped"
}

# Restart service
restart_service() {
    info "Restarting UA-Piper-TTS service..."
    systemctl restart "$SERVICE_NAME"
    sleep 2
    status_service
}

# Show status
status_service() {
    if systemctl is-active --quiet "$SERVICE_NAME"; then
        success "Service is running"
        echo
        local pid
        pid=$(systemctl show --property=MainPID --value "$SERVICE_NAME")
        echo "  PID: $pid"

        # Try to get listening port from config
        if [ -f "$INSTALL_DIR/config.json" ]; then
            local port
            port=$(grep -o '"port": *[0-9]*' "$INSTALL_DIR/config.json" | grep -o '[0-9]*')
            echo "  Port: $port"
            echo "  URL: http://localhost:$port"
        fi
    else
        warn "Service is not running"
    fi
    echo
    systemctl status "$SERVICE_NAME" --no-pager -l
}

# Show logs
show_logs() {
    journalctl -u "$SERVICE_NAME" "$@"
}

# Uninstall service
uninstall_service() {
    check_root

    warn "This will remove the UA-Piper-TTS service"
    read -rp "Continue? [y/N]: " confirm
    if [[ ! "$confirm" =~ ^[Yy]$ ]]; then
        info "Aborted"
        exit 0
    fi

    # Stop and disable service
    systemctl stop "$SERVICE_NAME" 2>/dev/null || true
    systemctl disable "$SERVICE_NAME" 2>/dev/null || true

    # Remove service file
    rm -f "$SERVICE_FILE"
    systemctl daemon-reload

    # Remove install directory
    if [ -d "$INSTALL_DIR" ]; then
        info "Removing $INSTALL_DIR..."
        rm -rf "$INSTALL_DIR"
    fi

    success "Service uninstalled"
}

# Main
ACTION="${1:-}"

case "$ACTION" in
    install)
        install_service
        ;;
    start)
        start_service
        ;;
    stop)
        stop_service
        ;;
    restart)
        restart_service
        ;;
    status)
        status_service
        ;;
    logs)
        shift
        show_logs "$@"
        ;;
    uninstall)
        uninstall_service
        ;;
    ""|-h|--help)
        usage
        ;;
    *)
        error "Unknown command: $ACTION"
        usage
        ;;
esac
