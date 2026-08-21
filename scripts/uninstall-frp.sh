#!/usr/bin/env bash
# ==============================================================================
# Complete FRP Uninstaller & Cleaner
# Completely removes all FRP daemons, agents, gateways, configs, and systemd units
# ==============================================================================

set -euo pipefail

echo "=========================================================="
echo " Stopping and Removing All FRP Services & Configurations"
echo "=========================================================="

if [[ $EUID -ne 0 ]]; then
   echo "[-] Error: This script must be run as root (or with sudo)." >&2
   exit 1
fi

# 1. Stop and disable all FRP services
echo "[+] Stopping systemd services..."
for svc in frps frpc frp-controller frp-agent frp-gateway; do
    if systemctl list-unit-files | grep -q "${svc}.service"; then
        echo "    - Stopping and disabling ${svc}.service..."
        systemctl stop "${svc}.service" 2>/dev/null || true
        systemctl disable "${svc}.service" 2>/dev/null || true
        rm -f "/etc/systemd/system/${svc}.service"
    fi
done

# Kill any remaining FRP processes
killall frps frpc frp-controller frp-agent frp-gateway 2>/dev/null || true

# 2. Reload systemd daemon
echo "[+] Reloading systemd..."
systemctl daemon-reload
systemctl reset-failed 2>/dev/null || true

# 3. Remove FRP directories and configuration files
echo "[+] Cleaning up directories and configuration files..."
rm -rf /opt/frp
rm -rf /etc/frp*
rm -rf /var/lib/frp*
rm -rf /etc/systemd/system/frp*

# 4. Remove binaries from /usr/local/bin
echo "[+] Removing binaries from /usr/local/bin..."
rm -f /usr/local/bin/frp-controller
rm -f /usr/local/bin/frp-agent
rm -f /usr/local/bin/frp-gateway
rm -f /usr/local/bin/frpctl
rm -f /usr/local/bin/frps
rm -f /usr/local/bin/frpc

# 5. Remove dedicated system user if exists
if id -u frp >/dev/null 2>&1; then
    echo "[+] Removing 'frp' system user..."
    userdel -r frp 2>/dev/null || userdel frp 2>/dev/null || true
fi

echo "=========================================================="
echo " [✓] All FRP services, binaries, and configurations have"
echo "     been completely and cleanly removed from this server!"
echo "=========================================================="
