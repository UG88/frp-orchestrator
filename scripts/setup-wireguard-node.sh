#!/usr/bin/env bash
# ==============================================================================
# WireGuard Pterodactyl Node Installer (Transparent Tunnel Spoke)
# Connects Backend Node to Gateway VPS over WireGuard with Policy Routing
# ==============================================================================

set -euo pipefail

echo "=========================================================="
echo " Starting WireGuard Node Installation (Pterodactyl Node)"
echo "=========================================================="

if [[ $EUID -ne 0 ]]; then
   echo "[-] Error: This script must be run as root (or with sudo)." >&2
   exit 1
fi

# 1. Install WireGuard and IPRoute2
echo "[+] Installing WireGuard and networking tools..."
if command -v apt-get >/dev/null 2>&1; then
    export DEBIAN_FRONTEND=noninteractive
    apt-get update -qq || true
    apt-get install -y -qq wireguard wireguard-tools iproute2 curl || apt-get install -y wireguard wireguard-tools iproute2 curl
elif command -v dnf >/dev/null 2>&1; then
    dnf install -y wireguard-tools iproute curl >/dev/null 2>&1 || true
elif command -v yum >/dev/null 2>&1; then
    yum install -y epel-release >/dev/null 2>&1 || true
    yum install -y wireguard-tools iproute curl >/dev/null 2>&1 || true
fi

# 2. Interactive prompt for Gateway credentials if not provided as env vars
GW_ENDPOINT="${GW_ENDPOINT:-}"
GW_PUBLIC_KEY="${GW_PUBLIC_KEY:-}"

if [[ -z "$GW_ENDPOINT" ]]; then
    read -r -p "Enter Gateway Public IP (e.g. 3.108.50.20): " GW_ENDPOINT
fi

if [[ -z "$GW_PUBLIC_KEY" ]]; then
    read -r -p "Enter Gateway Public Key (from Gateway setup): " GW_PUBLIC_KEY
fi

# 3. Generate Node cryptographic keys
mkdir -p /etc/wireguard
chmod 700 /etc/wireguard

if [[ ! -f /etc/wireguard/node_private.key ]]; then
    echo "[+] Generating Node cryptographic keypair..."
    wg genkey | tee /etc/wireguard/node_private.key | wg pubkey > /etc/wireguard/node_public.key
    chmod 600 /etc/wireguard/node_private.key
fi

NODE_PRIVATE_KEY=$(cat /etc/wireguard/node_private.key)
NODE_PUBLIC_KEY=$(cat /etc/wireguard/node_public.key)

# 4. Create Node WireGuard configuration with Policy Routing
cat << EOF > /etc/wireguard/wg0.conf
[Interface]
Address = 10.200.0.2/24
PrivateKey = $NODE_PRIVATE_KEY

# Policy routing: Game response packets for traffic originating from the tunnel route back through Gateway
PostUp = ip rule add from 10.200.0.2 table 200 || true; ip route add default via 10.200.0.1 dev %i table 200 || true
PostDown = ip rule del from 10.200.0.2 table 200 || true; ip route del default via 10.200.0.1 dev %i table 200 || true

[Peer]
PublicKey = $GW_PUBLIC_KEY
Endpoint = $GW_ENDPOINT:51820
AllowedIPs = 10.200.0.0/24, 0.0.0.0/0
PersistentKeepalive = 25
EOF

# 5. Start WireGuard on Node
systemctl enable --now wg-quick@wg0
systemctl restart wg-quick@wg0

echo "=========================================================="
echo " [✓] Node WireGuard tunnel is ACTIVE!"
echo "=========================================================="
echo ""
echo " FINAL STEP: Run this SINGLE command on your Gateway VPS"
echo " to authorize this Node:"
echo ""
echo " sudo wg set wg0 peer $NODE_PUBLIC_KEY allowed-ips 10.200.0.2/32"
echo ""
echo "=========================================================="
