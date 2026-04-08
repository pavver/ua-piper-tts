#!/usr/bin/env bash
# =============================================================================
# UA-Piper-TTS — Update Script
# Pulls latest changes from GitHub, rebuilds, and restarts the service
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
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"

# Usage
usage() {
    cat <<EOF
Usage: $0 [OPTIONS]

Options:
  -b, --branch BRANCH   Target branch (default: main)
  -f, --force           Force rebuild even if no new commits
  -n, --dry-run         Show what would be done without making changes
  -h, --help            Show this help

Examples:
  $0                    # Update from main branch
  $0 -b develop         # Update from develop branch
  $0 -f                 # Force rebuild
  $0 -n                 # Dry run (check only)
EOF
    exit 0
}

# Parse arguments
BRANCH="main"
FORCE=false
DRY_RUN=false

while [[ $# -gt 0 ]]; do
    case "$1" in
        -b|--branch)
            BRANCH="$2"
            shift 2
            ;;
        -f|--force)
            FORCE=true
            shift
            ;;
        -n|--dry-run)
            DRY_RUN=true
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

# Check if running from project root
check_project_root() {
    if [ ! -f "$PROJECT_DIR/Cargo.toml" ] || [ ! -d "$PROJECT_DIR/src" ]; then
        error "Not running from project root!"
        echo "Current directory: $(pwd)"
        echo "Expected: Directory containing Cargo.toml and src/"
        echo
        echo "Usage: cd /path/to/ua-piper-tts && ./scripts/update.sh"
        exit 1
    fi
    success "Project root verified: $PROJECT_DIR"
}

# Check if service is deployed
check_service_deployed() {
    if ! systemctl list-unit-files | grep -q "$SERVICE_NAME"; then
        error "Service '$SERVICE_NAME' is not installed!"
        echo
        echo "Deploy first with:"
        echo "  sudo ./scripts/deploy.sh install"
        exit 1
    fi
    success "Service is deployed: $SERVICE_NAME"
}

# Check if service is running (warn if not)
check_service_running() {
    if systemctl is-active --quiet "$SERVICE_NAME"; then
        success "Service is running"
    else
        warn "Service is installed but not running"
        info "Will start it after update"
    fi
}

# Check if we have uncommitted changes
check_uncommitted_changes() {
    if [ -n "$(git status --porcelain)" ]; then
        warn "You have uncommitted changes"
        echo
        echo "Options:"
        echo "  1. Stash changes:    git stash"
        echo "  2. Commit changes:   git add . && git commit -m '...'"
        echo "  3. Discard changes:  git reset --hard HEAD"
        echo
        read -rp "Stash changes and continue? [Y/n]: " stash_choice
        if [[ "$stash_choice" =~ ^[Nn]$ ]]; then
            info "Aborted"
            exit 0
        fi
        git stash
        STASHED=true
    fi
}

# Pull latest changes
pull_changes() {
    info "Fetching latest changes from '$BRANCH' branch..."

    local before_commit
    before_commit=$(git rev-parse HEAD)

    git fetch origin "$BRANCH"

    local origin_commit
    origin_commit=$(git rev-parse origin/"$BRANCH")

    if [ "$before_commit" = "$origin_commit" ]; then
        if [ "$FORCE" = true ]; then
            warn "Already up to date, but force rebuild requested"
            return
        fi
        success "Already up to date (commit: ${before_commit:0:8})"
        echo
        info "No new commits found. Nothing to update."
        exit 0
    fi

    info "New commits found!"
    echo
    echo "Changes to pull:"
    git log --oneline HEAD..origin/"$BRANCH"
    echo

    if [ "$DRY_RUN" = true ]; then
        info "Dry run — not pulling changes"
        return
    fi

    git pull origin "$BRANCH"

    local after_commit
    after_commit=$(git rev-parse HEAD)
    success "Pulled updates (${before_commit:0:8} → ${after_commit:0:8})"
}

# Rebuild project
rebuild() {
    info "Building project..."

    if [ "$DRY_RUN" = true ]; then
        info "Dry run — would execute: ./scripts/build.sh"
        return
    fi

    if "$SCRIPT_DIR/build.sh"; then
        success "Build completed successfully"
    else
        error "Build failed!"
        echo
        echo "To rollback to previous version:"
        echo "  git log --oneline  # find last good commit"
        echo "  git reset --hard <commit>"
        echo "  ./scripts/build.sh"
        echo "  sudo ./scripts/deploy.sh restart"
        exit 1
    fi
}

# Deploy and restart service
restart_service() {
    if [ "$DRY_RUN" = true ]; then
        info "Dry run — would execute: sudo ./scripts/deploy.sh install && sudo ./scripts/deploy.sh restart"
        return
    fi

    info "Updating service..."

    if sudo "$SCRIPT_DIR/deploy.sh" install; then
        success "Service updated"
    else
        error "Service update failed!"
        exit 1
    fi

    info "Restarting service..."

    if sudo "$SCRIPT_DIR/deploy.sh" restart; then
        success "Service restarted successfully"
    else
        error "Service restart failed!"
        exit 1
    fi
}

# Show status after update
show_status() {
    echo
    echo "============================================"
    success "Update completed successfully!"
    echo "============================================"
    echo

    echo "Current commit: $(git log --oneline -1)"
    echo
    sudo "$SCRIPT_DIR/deploy.sh" status
    echo
    echo "To view recent logs:"
    echo "  sudo ./scripts/deploy.sh logs -f"
    echo
}

# Main
main() {
    echo "============================================"
    echo "  UA-Piper-TTS — Update"
    echo "============================================"
    echo "Branch: $BRANCH"
    echo "Force: $FORCE"
    echo "Dry run: $DRY_RUN"
    echo

    STASHED=false

    # Pre-flight checks
    check_project_root
    check_service_deployed
    check_service_running

    echo
    info "Starting update process..."
    echo

    # Check for uncommitted changes
    check_uncommitted_changes

    # Pull latest changes
    pull_changes

    # Rebuild
    echo
    rebuild

    # Restart service
    echo
    restart_service

    # Restore stashed changes if any
    if [ "$STASHED" = true ]; then
        info "Restoring stashed changes..."
        git stash pop || warn "Stash pop failed — resolve conflicts manually"
    fi

    # Show final status
    show_status
}

main "$@"
