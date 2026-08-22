---
name: wirenet
description: Comprehensive operational skill for WireNet Minecraft & Pterodactyl kernel-level ingress, WireGuard tunneling, Anti-DDoS management, real-time TUI telemetry, and auto-diagnostics.
---

# WireNet Operational Skill (Method 1: Pure Zero-Plugin Kernel Routing)

Use this skill whenever working on, debugging, configuring, deploying, or testing WireNet components across Gateway and Node VPS environments.

---

## 🛠️ CLI Quick Reference

| Task | Command |
|---|---|
| **Setup Gateway VPS (Hub)** | `wirenet setup gateway` |
| **Setup Node VPS (Spoke)** | `wirenet setup node --gateway <IP> --gateway-key <KEY>` |
| **Live Zero-Flicker Telemetry TUI** | `wirenet tui` |
| **6-Point System Doctor & Self-Healing** | `wirenet doctor` |
| **Active Tunnel Status & Latency** | `wirenet status` |
| **Anti-DDoS Shield (Standard/Strict/Off)**| `wirenet shield [standard\|strict\|off]` |
| **Check for Updates** | `wirenet check-update` |
| **1-Click Self-Updater** | `wirenet update` |
| **100% Deep Uninstaller** | `wirenet uninstall` |

---

## 🏗️ Method 1 Kernel Routing Invariants

1. **Gateway Ingress**:
   - Pure Layer-3 DNAT from public port `25565:25700` to `10.200.0.2`.
   - **NO MASQUERADE on `wg0`**: Preserves player's true public IPv4 address.

2. **Node Return Path**:
   - `wg0.conf`: `Table = off` with `AllowedIPs = 0.0.0.0/0`.
   - Incoming `wg0` packets marked with `CONNMARK (0x1)`.
   - `ip rule add fwmark 0x1 table 100` with default route `via 10.200.0.1 dev wg0`.
   - `sysctl -w net.ipv4.conf.all.rp_filter=2` (loose reverse path filter).

3. **Backend Node IP Invisibility**:
   - Direct public access on `eth0` for game ports is **100% BLOCKED (DROPPED)**.
   - Unconditional `ACCEPT` for `lo`, `127.0.0.0/8`, and `wg0` before `DROP` rules.

---

## 🔍 1-Click System Doctor & Self-Healing:
```bash
wirenet doctor
```
