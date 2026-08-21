# WireGuard Transparent Gateway System for Pterodactyl & Minecraft

This guide explains how to deploy a high-performance, kernel-level **WireGuard Transparent Gateway** that bridges public player traffic to private backend Pterodactyl nodes while **preserving 100% authentic, real player IP addresses** across all game servers (Vanilla, Forge, Fabric, Paper, Spigot, Bedrock, Geyser) with **zero plugins required**.

---

## 1. Network Architecture Overview

```
[ Public Internet / Players (Player IP: 1.2.3.4) ]
                     │
                     │ Connects to Gateway Public IP: 3.108.50.20:25565
                     ▼
┌─────────────────────────────────────────────────────────┐
│              Public Gateway VPS (Hub)                   │
│  - Public IP: 3.108.50.20                               │
│  - WireGuard Virtual IP: 10.200.0.1                     │
│  - Kernel DNAT (No SNAT -> Source IP 1.2.3.4 preserved!)│
└────────────────────────────┬────────────────────────────┘
                             │
                             │ Encrypted WireGuard Kernel Pipe (ChaCha20-Poly1305)
                             ▼
┌─────────────────────────────────────────────────────────┐
│            Private Pterodactyl Node VPS (Spoke)         │
│  - Real Public IP is 100% HIDDEN                        │
│  - WireGuard Virtual IP: 10.200.0.2                     │
│  - Policy Routing (table 200 return path via Gateway)   │
└────────────────────────────┬────────────────────────────┘
                             │
                             ▼
┌─────────────────────────────────────────────────────────┐
│            Customer's Minecraft Server (Wings)          │
│  - Receives packet with Source IP = 1.2.3.4             │
│  - Works for Vanilla, Forge, Fabric, Paper, Bedrock     │
│  - /ban-ip <player> safely bans ONLY that player!       │
└─────────────────────────────────────────────────────────┘
```

---

## 2. Fast 2-Step Installation

### Step A: On your Public Gateway VPS (Run First)

```bash
curl -fsSL https://raw.githubusercontent.com/UG88/frp-orchestrator/main/scripts/setup-wireguard-gateway.sh | sudo bash
```

This automated script will:
1. Install `wireguard`, `wireguard-tools`, and `iptables`.
2. Enable Linux kernel IP forwarding (`net.ipv4.ip_forward=1`).
3. Generate the Gateway cryptographic keypair.
4. Configure `/etc/wireguard/wg0.conf` with DNAT forwarding for ports `25565–25600` and `30000–40000`.
5. Enable and start the `wg-quick@wg0` systemd service.
6. **Print the Gateway Public IP and Public Key.** *(Save these for Step B!)*

---

### Step B: On your Pterodactyl Node VPS (Run Second)

```bash
curl -fsSL https://raw.githubusercontent.com/UG88/frp-orchestrator/main/scripts/setup-wireguard-node.sh | sudo bash
```

When prompted:
1. Enter your **Gateway Public IP** (e.g. `3.108.50.20`).
2. Enter your **Gateway Public Key** (from Step A).

The script configures the node with policy routing and outputs a single authorization command:
```bash
sudo wg set wg0 peer <NODE_PUBLIC_KEY> allowed-ips 10.200.0.2/32
```

---

### Step C: Authorize the Node (Run on Gateway VPS)

Paste the authorization command printed by Step B into your Gateway VPS terminal:
```bash
sudo wg set wg0 peer <NODE_PUBLIC_KEY> allowed-ips 10.200.0.2/32
```

---

## 3. Verification & Health Check

### 1. Test Tunnel Ping:
On your **Pterodactyl Node VPS**, ping the Gateway hub:
```bash
ping -c 3 10.200.0.1
```
*(Should return responses with `<1ms` ping!)*

### 2. View WireGuard Status:
On either server:
```bash
sudo wg show
```
You will see active handshakes and real-time transfer telemetry:
```
interface: wg0
  public key: ...
  private key: (hidden)
  listening port: 51820

peer: ...
  endpoint: ...:51820
  allowed ips: 10.200.0.2/32
  latest handshake: 12 seconds ago
  transfer: 1.42 KiB received, 2.10 KiB sent
```

---

## 4. How to Connect Multiple Pterodactyl Nodes

To connect multiple backend nodes to the same Gateway VPS:

```
Gateway Hub: 10.200.0.1
├── Node 1: 10.200.0.2 ──► Ports 30001–30500
├── Node 2: 10.200.0.3 ──► Ports 30501–31000
└── Node 3: 10.200.0.4 ──► Ports 31001–31500
```

On the Gateway VPS in `/etc/wireguard/wg0.conf`, add a port forwarding block for each node:
```ini
# Forward ports 30501:31000 to Node 2 (10.200.0.3)
PostUp = iptables -t nat -A PREROUTING -i eth0 -p tcp -m multiport --dports 30501:31000 -j DNAT --to-destination 10.200.0.3
PostUp = iptables -t nat -A PREROUTING -i eth0 -p udp -m multiport --dports 30501:31000 -j DNAT --to-destination 10.200.0.3
```

---

## 5. Security & Firewall Guarantees

1. **Backend IP Obfuscation**: The Node VPS only talks to the Gateway over encrypted WireGuard tunnel (`51820/udp`).
2. **Zero Public Access to Backend Game Ports**: Pterodactyl node ports are blocked from the open internet and only accessible via `10.200.0.1`.
3. **Real Source IP Preservation**: The Linux kernel passes raw player IP packets without NAT translation, allowing `/ban-ip` and anti-cheat to function normally.
4. **Kernel Speed**: ChaCha20-Poly1305 encryption runs in kernel space at multi-gigabit speeds.
