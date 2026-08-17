#!/usr/bin/env bash
# ==============================================================================
# FRP Gateway Automated Installer (frps)
# Safe, idempotent, non-destructive installer for production Linux hosts
# ==============================================================================

set -euo pipefail

FRP_VERSION="${FRP_VERSION:-0.60.0}"
INSTALL_DIR="/opt/frp"
CONFIG_DIR="/etc/frp-gateway"
USER="frp"
GROUP="frp"

echo "=========================================================="
echo " Starting FRP Gateway Installation (v${FRP_VERSION})"
echo "=========================================================="

# 1. Root check
if [[ $EUID -ne 0 ]]; then
   echo "[-] Error: This script must be run as root." >&2
   exit 1
fi

# 2. Detect OS & Architecture
OS=$(uname -s | tr '[:upper:]' '[:lower:]')
ARCH=$(uname -m)

case "$ARCH" in
    x86_64)  FRP_ARCH="amd64" ;;
    aarch64) FRP_ARCH="arm64" ;;
    armv7l)  FRP_ARCH="arm" ;;
    *) echo "[-] Error: Unsupported architecture: $ARCH" >&2; exit 1 ;;
esac

echo "[+] Detected OS: $OS, Arch: $FRP_ARCH"

# 3. Create dedicated system user
if ! id -u "$USER" >/dev/null 2>&1; then
    echo "[+] Creating dedicated system user '$USER'..."
    useradd -r -s /bin/false -d "$INSTALL_DIR" "$USER"
fi

# 4. Create directories
mkdir -p "$INSTALL_DIR" "$CONFIG_DIR"
chown -R "$USER:$GROUP" "$INSTALL_DIR"

# 5. Download and install FRP Server binary
TMP_DIR=$(mktemp -d)
trap 'rm -rf "$TMP_DIR"' EXIT

ARCHIVE_NAME="frp_${FRP_VERSION}_${OS}_${FRP_ARCH}.tar.gz"
DOWNLOAD_URL="https://github.com/fatedier/frp/releases/download/v${FRP_VERSION}/${ARCHIVE_NAME}"

echo "[+] Downloading FRP Server from $DOWNLOAD_URL..."
curl -fsSL "$DOWNLOAD_URL" -o "$TMP_DIR/$ARCHIVE_NAME"

echo "[+] Extracting binary..."
tar -xzf "$TMP_DIR/$ARCHIVE_NAME" -C "$TMP_DIR"
EXTRACTED_DIR="$TMP_DIR/frp_${FRP_VERSION}_${OS}_${FRP_ARCH}"

install -m 0755 "$EXTRACTED_DIR/frps" "$INSTALL_DIR/frps"
echo "[+] Installed frps binary to $INSTALL_DIR/frps"

# 6. Install systemd service
cat << 'EOF' > /etc/systemd/system/frps.service
[Unit]
Description=FRP Gateway Server (frps)
After=network.target network-online.target
Wants=network-online.target

[Service]
Type=simple
User=frp
Group=frp
WorkingDirectory=/opt/frp
ExecStart=/opt/frp/frps -c /opt/frp/frps.toml
Restart=always
RestartSec=5s
LimitNOFILE=65536
CapabilityBoundingSet=CAP_NET_BIND_SERVICE
AmbientCapabilities=CAP_NET_BIND_SERVICE
ProtectSystem=strict
ProtectHome=yes
ReadWritePaths=/opt/frp
PrivateTmp=yes
NoNewPrivileges=yes

[Install]
WantedBy=multi-user.target
EOF

systemctl daemon-reload

echo "=========================================================="
echo " [✓] FRP Gateway installed successfully!"
echo " Next steps:"
echo " 1. Configure /opt/frp/frps.toml or run frpctl gateway init"
echo " 2. Configure firewall: scripts/setup-firewall.sh"
echo " 3. Start service: systemctl enable --now frps"
echo "=========================================================="
