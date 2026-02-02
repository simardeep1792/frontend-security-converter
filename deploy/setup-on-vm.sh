#!/bin/bash
# Setup script to run ON the GCP VM
# This script installs dependencies, builds the frontend, and sets up systemd service

set -e

INSTALL_DIR="/opt/frontend-security-converter"
SERVICE_NAME="frontend-security-converter"
# Default to connecting to backend on localhost:8080
BACKEND_API_URL="${BACKEND_API_URL:-http://localhost:8080/graphql}"

echo "=== Setting up Frontend Security Converter on VM ==="

# Install Rust if not present
if ! command -v cargo &> /dev/null; then
    echo "Installing Rust..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    source "$HOME/.cargo/env"
fi

# Ensure cargo is in PATH
export PATH="$HOME/.cargo/bin:$PATH"
source "$HOME/.cargo/env" 2>/dev/null || true

# Install build dependencies
echo "Installing build dependencies..."
apt-get update
apt-get install -y build-essential pkg-config libssl-dev cmake

# Create install directory
echo "Creating install directory..."
mkdir -p "$INSTALL_DIR"

# Extract source code
echo "Extracting source code..."
cd "$INSTALL_DIR"
tar -xzf /tmp/frontend-source.tar.gz

# Create .env file if it doesn't exist
if [ ! -f "$INSTALL_DIR/.env" ]; then
    echo "Creating .env file..."
    cat > "$INSTALL_DIR/.env" << 'EOF'
COOKIE_SECRET_KEY=abcdefghijklmnopqrstuvwxyz0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZ01
ENVIRONMENT=production
EOF
fi

# Build the application
echo "Building application (this may take a while)..."
cd "$INSTALL_DIR"
cargo build --release

# Create systemd service file
echo "Creating systemd service..."
cat > /etc/systemd/system/${SERVICE_NAME}.service << EOF
[Unit]
Description=Frontend Security Converter Web Application
After=network.target

[Service]
Type=simple
User=root
WorkingDirectory=${INSTALL_DIR}
Environment="RUST_LOG=info"
Environment="API_URL=${BACKEND_API_URL}"
ExecStart=${INSTALL_DIR}/target/release/frontend
Restart=always
RestartSec=5

[Install]
WantedBy=multi-user.target
EOF

# Reload systemd and start service
echo "Starting service..."
systemctl daemon-reload
systemctl enable ${SERVICE_NAME}
systemctl restart ${SERVICE_NAME}

# Wait a moment and check status
sleep 3
systemctl status ${SERVICE_NAME} --no-pager || true

echo ""
echo "=== Setup Complete ==="
echo "Service status: systemctl status ${SERVICE_NAME}"
echo "View logs: journalctl -u ${SERVICE_NAME} -f"
echo "Frontend should be accessible on port 8088"
