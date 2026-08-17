#!/usr/bin/env bash
# ==============================================================================
# FRP Orchestrator Controller Automated Installer
# ==============================================================================

set -euo pipefail

INSTALL_DIR="/var/lib/frp-orchestrator"
CONFIG_DIR="/etc/frp-orchestrator"
USER="frp"
GROUP="frp"

echo "=========================================================="
echo " Starting FRP Controller Installation"
echo "=========================================================="

if [[ $EUID -ne 0 ]]; then
   echo "[-] Error: This script must be run as root." >&2
   exit 1
fi

if ! id -u "$USER" >/dev/null 2>&1; then
    echo "[+] Creating dedicated system user '$USER'..."
    useradd -r -s /bin/false -d "$INSTALL_DIR" "$USER"
fi

mkdir -p "$INSTALL_DIR" "$CONFIG_DIR"
chown -R "$USER:$GROUP" "$INSTALL_DIR"

# Install systemd service
cat << 'EOF' > /etc/systemd/system/frp-controller.service
[Unit]
Description=FRP Gateway Orchestrator Controller
After=network.target network-online.target
Wants=network-online.target

[Service]
Type=simple
User=frp
Group=frp
WorkingDirectory=/var/lib/frp-orchestrator
ExecStart=/usr/local/bin/frp-controller --config /etc/frp-orchestrator/controller.toml
Restart=always
RestartSec=5s
LimitNOFILE=65536
ProtectSystem=strict
ProtectHome=yes
ReadWritePaths=/var/lib/frp-orchestrator
PrivateTmp=yes
NoNewPrivileges=yes

[Install]
WantedBy=multi-user.target
EOF

systemctl daemon-reload

echo "=========================================================="
echo " [✓] FRP Controller setup complete!"
echo " Next steps:"
echo " 1. Put controller binary at /usr/local/bin/frp-controller"
echo " 2. Configure /etc/frp-orchestrator/controller.toml"
echo " 3. Enable & start service: systemctl enable --now frp-controller"
echo "=========================================================="
