#!/usr/bin/env bash
# ==============================================================================
# High-Performance Minecraft Anti-DDoS & Packet Filter System for Gateway VPS
# Filters TCP (Java) and UDP (Bedrock) traffic directly in the Linux Kernel
# ==============================================================================

set -euo pipefail

echo "=========================================================="
echo " Deploying Minecraft Anti-DDoS & Packet Filtering Shield"
echo "=========================================================="

if [[ $EUID -ne 0 ]]; then
   echo "[-] Error: This script must be run as root (or with sudo)." >&2
   exit 1
fi

DEFAULT_IFACE=$(ip route show default 2>/dev/null | awk '{print $5}' | head -n1)
DEFAULT_IFACE="${DEFAULT_IFACE:-eth0}"
echo "[+] Detected primary public interface: $DEFAULT_IFACE"

# 1. Optimize Linux Kernel Network Stack for Gaming & Flood Resistance
echo "[+] Applying kernel anti-flood & high-throughput tuning..."
cat << 'EOF' > /etc/sysctl.d/98-minecraft-security.conf
# SYN Flood Protection (Cryptographic SYN Cookies)
net.ipv4.tcp_syncookies = 1
net.ipv4.tcp_syn_retries = 2
net.ipv4.tcp_synack_retries = 2
net.ipv4.tcp_max_syn_backlog = 65536

# Anti-Spoofing & Reverse Path Filtering
net.ipv4.conf.all.rp_filter = 1
net.ipv4.conf.default.rp_filter = 1

# Connection Backlog & Memory Tuning
net.core.somaxconn = 65535
net.core.netdev_max_backlog = 65536
net.ipv4.tcp_max_tw_buckets = 1440000
net.ipv4.tcp_tw_reuse = 1
net.ipv4.tcp_fin_timeout = 15

# Ignore ICMP Broadcast & Bogus Error Responses
net.ipv4.icmp_echo_ignore_broadcasts = 1
net.ipv4.icmp_ignore_bogus_error_responses = 1
EOF

sysctl --system >/dev/null 2>&1 || sysctl -p /etc/sysctl.d/98-minecraft-security.conf >/dev/null 2>&1 || true

# 2. Configure IPTables Filter Rules
echo "[+] Installing Minecraft kernel packet filters..."

# A. Create dedicated chains for clean isolation
iptables -N MC_TCP_FILTER 2>/dev/null || iptables -F MC_TCP_FILTER
iptables -N MC_UDP_FILTER 2>/dev/null || iptables -F MC_UDP_FILTER

# B. Drop Invalid & Malformed TCP Packets (XMAS, NULL, Broken flags)
iptables -A MC_TCP_FILTER -m state --state INVALID -j DROP
iptables -A MC_TCP_FILTER -p tcp --tcp-flags ALL NONE -j DROP
iptables -A MC_TCP_FILTER -p tcp --tcp-flags ALL ALL -j DROP
iptables -A MC_TCP_FILTER -p tcp --tcp-flags ALL FIN,PSH,URG -j DROP
iptables -A MC_TCP_FILTER -p tcp --tcp-flags SYN,FIN SYN,FIN -j DROP
iptables -A MC_TCP_FILTER -p tcp --tcp-flags SYN,RST SYN,RST -j DROP
iptables -A MC_TCP_FILTER -p tcp --tcp-flags ALL SYN,RST,ACK,FIN,URG -j DROP

# C. Minecraft TCP Connection Rate Limiting (Protects against Bot Joining Floods)
# Allows 20 new TCP connections/second per IP with a burst of 40
iptables -A MC_TCP_FILTER -p tcp --syn -m hashlimit \
    --hashlimit-above 20/sec \
    --hashlimit-burst 40 \
    --hashlimit-mode srcip \
    --hashlimit-name mc_tcp_limit \
    -j DROP
iptables -A MC_TCP_FILTER -j ACCEPT

# D. Minecraft Bedrock UDP Packet Rate Limiting (Protects against UDP Reflection Floods)
# Allows up to 60 UDP packets/second per IP with a burst of 120 (plenty for Bedrock gameplay)
iptables -A MC_UDP_FILTER -p udp -m hashlimit \
    --hashlimit-above 60/sec \
    --hashlimit-burst 120 \
    --hashlimit-mode srcip \
    --hashlimit-name mc_udp_limit \
    -j DROP
iptables -A MC_UDP_FILTER -j ACCEPT

# E. Direct Game Ports (25565:25600 and 30000:40000) through the Filter Chains
iptables -D INPUT -i "$DEFAULT_IFACE" -p tcp -m multiport --dports 25565:25600,30000:40000 -j MC_TCP_FILTER 2>/dev/null || true
iptables -A INPUT -i "$DEFAULT_IFACE" -p tcp -m multiport --dports 25565:25600,30000:40000 -j MC_TCP_FILTER

iptables -D INPUT -i "$DEFAULT_IFACE" -p udp -m multiport --dports 25565:25600,30000:40000 -j MC_UDP_FILTER 2>/dev/null || true
iptables -A INPUT -i "$DEFAULT_IFACE" -p udp -m multiport --dports 25565:25600,30000:40000 -j MC_UDP_FILTER

# 3. Save IPTables Rules Permanently
if command -v netfilter-persistent >/dev/null 2>&1; then
    netfilter-persistent save >/dev/null 2>&1 || true
elif command -v iptables-save >/dev/null 2>&1; then
    iptables-save > /etc/iptables.rules 2>/dev/null || true
fi

echo "=========================================================="
echo " [✓] Minecraft Anti-DDoS & Packet Filtering Shield is ACTIVE!"
echo " Features Enabled:"
echo "   - Hardware SYN Cookie Protection against TCP SYN Floods"
echo "   - Invalid & Malformed Packet Dropper (XMAS/NULL/Scans)"
echo "   - Bot Join Spam Rate Limiter (20 conns/sec per IP)"
echo "   - Bedrock UDP Flood Shield (60 pkts/sec per IP)"
echo "=========================================================="
