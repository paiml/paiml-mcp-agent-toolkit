#!/bin/bash
# PMAT Claude Code Agent Deployment Script
# Automates the deployment of PMAT agent on Linux systems

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Configuration
PMAT_USER="pmat"
PMAT_GROUP="pmat"
INSTALL_DIR="/opt/pmat-agent"
CONFIG_DIR="/etc/pmat"
STATE_DIR="/var/lib/pmat-agent"
LOG_DIR="/var/log/pmat-agent"
BINARY_PATH="/usr/local/bin/pmat"

# Functions
print_status() {
    echo -e "${GREEN}[✓]${NC} $1"
}

print_error() {
    echo -e "${RED}[✗]${NC} $1"
    exit 1
}

print_warning() {
    echo -e "${YELLOW}[!]${NC} $1"
}

check_root() {
    if [[ $EUID -ne 0 ]]; then
        print_error "This script must be run as root"
    fi
}

create_user() {
    if ! id "$PMAT_USER" &>/dev/null; then
        print_status "Creating user $PMAT_USER"
        useradd -r -s /bin/false -d "$STATE_DIR" -c "PMAT Agent" "$PMAT_USER"
    else
        print_status "User $PMAT_USER already exists"
    fi
}

create_directories() {
    print_status "Creating directories"
    
    mkdir -p "$INSTALL_DIR"
    mkdir -p "$CONFIG_DIR"
    mkdir -p "$STATE_DIR"
    mkdir -p "$LOG_DIR"
    mkdir -p "$STATE_DIR/state"
    mkdir -p "$STATE_DIR/backups"
    mkdir -p "$STATE_DIR/reports"
    
    chown -R "$PMAT_USER:$PMAT_GROUP" "$STATE_DIR"
    chown -R "$PMAT_USER:$PMAT_GROUP" "$LOG_DIR"
    chmod 755 "$INSTALL_DIR"
    chmod 755 "$CONFIG_DIR"
    chmod 700 "$STATE_DIR"
}

install_binary() {
    if [[ ! -f "./target/release/pmat" ]]; then
        print_warning "Binary not found. Building from source..."
        cargo build --release --package pmat
    fi
    
    print_status "Installing binary to $BINARY_PATH"
    cp ./target/release/pmat "$BINARY_PATH"
    chmod 755 "$BINARY_PATH"
    
    # Verify installation
    if command -v pmat &> /dev/null; then
        print_status "Binary installed successfully"
        pmat --version
    else
        print_error "Failed to install binary"
    fi
}

install_configs() {
    print_status "Installing configuration files"
    
    # Copy configuration templates
    cp configs/agent-production.toml "$CONFIG_DIR/agent-production.toml"
    cp configs/agent-development.toml "$CONFIG_DIR/agent-development.toml"
    cp configs/agent-ci.toml "$CONFIG_DIR/agent-ci.toml"
    
    # Set permissions
    chmod 644 "$CONFIG_DIR"/*.toml
    
    # Create default symlink
    ln -sf "$CONFIG_DIR/agent-production.toml" "$CONFIG_DIR/config.toml"
}

install_systemd_service() {
    print_status "Installing systemd service"
    
    cp deployment/pmat-agent.service /etc/systemd/system/
    chmod 644 /etc/systemd/system/pmat-agent.service
    
    systemctl daemon-reload
    systemctl enable pmat-agent.service
    
    print_status "Systemd service installed and enabled"
}

configure_logrotate() {
    print_status "Configuring log rotation"
    
    cat > /etc/logrotate.d/pmat-agent <<EOF
$LOG_DIR/*.log {
    daily
    rotate 7
    compress
    delaycompress
    missingok
    notifempty
    create 0644 $PMAT_USER $PMAT_GROUP
    postrotate
        systemctl reload pmat-agent 2>/dev/null || true
    endscript
}
EOF
    
    chmod 644 /etc/logrotate.d/pmat-agent
}

setup_claude_code() {
    print_status "Setting up Claude Code integration"
    
    # Create MCP wrapper script
    cat > "$BINARY_PATH-mcp" <<'EOF'
#!/bin/bash
exec /usr/local/bin/pmat agent mcp-server --config /etc/pmat/config.toml "$@"
EOF
    
    chmod 755 "$BINARY_PATH-mcp"
    
    print_status "Claude Code MCP wrapper created at $BINARY_PATH-mcp"
    echo ""
    echo "Add to Claude Code settings:"
    echo '  "mcpServers": {'
    echo '    "pmat": {'
    echo '      "command": "pmat-mcp",'
    echo '      "args": [],'
    echo '      "env": {}'
    echo '    }'
    echo '  }'
}

start_service() {
    print_status "Starting PMAT agent service"
    
    systemctl start pmat-agent.service
    sleep 2
    
    if systemctl is-active --quiet pmat-agent.service; then
        print_status "Service started successfully"
        systemctl status pmat-agent.service --no-pager
    else
        print_error "Failed to start service"
        journalctl -u pmat-agent.service --no-pager -n 20
    fi
}

print_summary() {
    echo ""
    echo "========================================"
    echo "PMAT Claude Code Agent Deployment Complete!"
    echo "========================================"
    echo ""
    echo "Installation Details:"
    echo "  Binary: $BINARY_PATH"
    echo "  Config: $CONFIG_DIR/config.toml"
    echo "  State:  $STATE_DIR"
    echo "  Logs:   $LOG_DIR"
    echo ""
    echo "Service Management:"
    echo "  Start:   systemctl start pmat-agent"
    echo "  Stop:    systemctl stop pmat-agent"
    echo "  Status:  systemctl status pmat-agent"
    echo "  Logs:    journalctl -u pmat-agent -f"
    echo ""
    echo "Claude Code Integration:"
    echo "  Use 'pmat-mcp' command in Claude Code MCP settings"
    echo ""
    echo "Next Steps:"
    echo "  1. Review configuration at $CONFIG_DIR/config.toml"
    echo "  2. Configure Claude Code MCP settings"
    echo "  3. Monitor logs: tail -f $LOG_DIR/agent.log"
    echo ""
}

# Main deployment flow
main() {
    echo "PMAT Claude Code Agent Deployment Script"
    echo "========================================="
    echo ""
    
    check_root
    create_user
    create_directories
    install_binary
    install_configs
    install_systemd_service
    configure_logrotate
    setup_claude_code
    
    read -p "Start the service now? (y/n) " -n 1 -r
    echo
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        start_service
    fi
    
    print_summary
}

# Run main function
main "$@"