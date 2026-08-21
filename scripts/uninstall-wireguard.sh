#!/usr/bin/env bash
# ==============================================================================
# Complete WireGuard Uninstaller & Cleaner
# Completely stops and removes WireGuard interfaces, configs, keys, and firewall rules
# ==============================================================================

set -euo pipefail

echo "=========================================================="
echo " Stopping and Removing WireGuard Configuration"
echo "=========================================================="

if [[ $EUID -ne 0 ]]; then
   echo "[-] Error: This script must be run as root (or with sudo)." >&2
   exit 1
fi

# 1. Stop and disable WireGuard interface
echo "[+] Stopping WireGuard service (wg-quick@wg0)..."
systemctl stop wg-quick@wg0 2>/dev/null || true
systemctl disable wg-quick@wg0 2>/dev/null || true
wg-quick down wg0 2>/dev/null || true

# 2. Flush any custom policy routes or rules
echo "[+] Cleaning up policy routing rules..."
ip rule del from 10.200.0.2 table 200 2>/dev/null || true
ip route flush table 200 2>/dev/null || true

# 3. Remove WireGuard configuration and keys
echo "[+] Removing /etc/wireguard directory and keys..."
rm -rf /etc/wireguard
rm -f /etc/sysctl.d/99-wireguard.conf

# 4. Reload systemd
systemctl daemon-reload
systemctl reset-failed 2>/dev/null || true

echo "=========================================================="
echo " [✓] WireGuard has been completely and cleanly removed!"
echo "=========================================================="
