#!/usr/bin/env bash
# ==============================================================================
# FRP Node Agent Automated Installer (frpc + frp-agent daemon)
# ==============================================================================

set -euo pipefail

FRP_VERSION="${FRP_VERSION:-0.60.0}"
INSTALL_DIR="/opt/frp"
CONF_D="$INSTALL_DIR/conf.d"
CONFIG_DIR="/etc/frp-agent"

echo "=========================================================="
echo " Starting FRP Node Agent Installation (v${FRP_VERSION})"
echo "=========================================================="

if [[ $EUID -ne 0 ]]; then
   echo "[-] Error: This script must be run as root." >&2
   exit 1
fi

OS=$(uname -s | tr '[:upper:]' '[:lower:]')
ARCH=$(uname -m)

case "$ARCH" in
    x86_64)  FRP_ARCH="amd64" ;;
    aarch64) FRP_ARCH="arm64" ;;
    armv7l)  FRP_ARCH="arm" ;;
    *) echo "[-] Error: Unsupported architecture: $ARCH" >&2; exit 1 ;;
esac

mkdir -p "$INSTALL_DIR" "$CONF_D" "$CONFIG_DIR"

TMP_DIR=$(mktemp -d)
trap 'rm -rf "$TMP_DIR"' EXIT

ARCHIVE_NAME="frp_${FRP_VERSION}_${OS}_${FRP_ARCH}.tar.gz"
DOWNLOAD_URL="https://github.com/fatedier/frp/releases/download/v${FRP_VERSION}/${ARCHIVE_NAME}"

echo "[+] Downloading FRP Client from $DOWNLOAD_URL..."
curl -fsSL "$DOWNLOAD_URL" -o "$TMP_DIR/$ARCHIVE_NAME"

tar -xzf "$TMP_DIR/$ARCHIVE_NAME" -C "$TMP_DIR"
EXTRACTED_DIR="$TMP_DIR/frp_${FRP_VERSION}_${OS}_${FRP_ARCH}"

install -m 0755 "$EXTRACTED_DIR/frpc" "$INSTALL_DIR/frpc"
echo "[+] Installed frpc binary to $INSTALL_DIR/frpc"

# Install systemd unit for frpc
cat << 'EOF' > /etc/systemd/system/frpc.service
[Unit]
Description=FRP Node Client (frpc)
After=network.target network-online.target
Wants=network-online.target

[Service]
Type=simple
User=root
WorkingDirectory=/opt/frp
ExecStart=/opt/frp/frpc -c /opt/frp/frpc.toml
ExecReload=/opt/frp/frpc reload -c /opt/frp/frpc.toml
Restart=always
RestartSec=5s
LimitNOFILE=65536

[Install]
WantedBy=multi-user.target
EOF

systemctl daemon-reload

echo "=========================================================="
echo " [✓] FRP Node Agent installed successfully!"
echo " Next steps:"
echo " 1. Configure /etc/frp-agent/agent.toml"
echo " 2. Enable & start service: systemctl enable --now frp-agent"
echo "=========================================================="
