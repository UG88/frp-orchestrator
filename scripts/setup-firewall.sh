#!/usr/bin/env bash
# ==============================================================================
# FRP Gateway Firewall Hardening Script (UFW / nftables)
# Restricts access strictly to control port and configured Minecraft port ranges
# ==============================================================================

set -euo pipefail

CONTROL_PORT="${1:-7000}"
TCP_RANGE_START="${2:-30000}"
TCP_RANGE_END="${3:-40000}"
UDP_RANGE_START="${4:-30000}"
UDP_RANGE_END="${5:-40000}"

echo "=========================================================="
echo " Configuring FRP Gateway Firewall Rules"
echo " Control Port: ${CONTROL_PORT}/tcp"
echo " TCP Range:    ${TCP_RANGE_START}-${TCP_RANGE_END}/tcp"
echo " UDP Range:    ${UDP_RANGE_START}-${UDP_RANGE_END}/udp"
echo "=========================================================="

if [[ $EUID -ne 0 ]]; then
   echo "[-] Error: This script must be run as root." >&2
   exit 1
fi

if command -v ufw >/dev/null 2>&1; then
    echo "[+] Configuring UFW..."
    ufw allow "${CONTROL_PORT}/tcp" comment "FRP Control Traffic"
    ufw allow "${TCP_RANGE_START}:${TCP_RANGE_END}/tcp" comment "Minecraft FRP TCP Range"
    ufw allow "${UDP_RANGE_START}:${UDP_RANGE_END}/udp" comment "Minecraft FRP UDP Range"
    echo "[✓] UFW rules configured successfully."
elif command -v nft >/dev/null 2>&1; then
    echo "[+] Configuring nftables..."
    cat << EOF > /etc/nftables.d/frp-gateway.nft
table inet frp_filter {
    chain input {
        type filter hook input priority 0; policy accept;
        tcp dport ${CONTROL_PORT} accept comment "FRP Control Port"
        tcp dport ${TCP_RANGE_START}-${TCP_RANGE_END} accept comment "Minecraft TCP"
        udp dport ${UDP_RANGE_START}-${UDP_RANGE_END} accept comment "Minecraft UDP"
    }
}
EOF
    echo "[✓] nftables rules written to /etc/nftables.d/frp-gateway.nft"
else
    echo "[!] Neither UFW nor nftables detected. Please configure iptables manually."
fi
