---
name: wirenet
description: Comprehensive operational skill for WireNet Minecraft & Pterodactyl kernel-level ingress, WireGuard tunneling, Anti-DDoS management, real-time TUI telemetry, and auto-diagnostics.
---

# WireNet Operational Skill

Use this skill whenever working on, debugging, configuring, deploying, or testing WireNet components across Gateway and Node VPS environments.

---

## 🛠️ CLI Quick Reference

| Task | Command |
|---|---|
| **Interactive TUI Manager** | `wirenet` |
| **Live Zero-Flicker Telemetry** | `wirenet tui` |
| **6-Point System Doctor & Self-Healing** | `wirenet doctor` |
| **Active Tunnel Status & Latency** | `wirenet status` |
| **Anti-DDoS Shield (Standard/Strict/Off)**| `wirenet shield [standard\|strict\|off]` |
| **Rust Daemon Background Management** | `wirenet daemon [install\|status\|start\|stop]` |
| **1-Click Sync from GitHub** | `wirenet update` |

---

## 🏗️ Core Architecture Invariants

1. **WireGuard Private Subnet**:
   - Gateway Virtual IP: `10.200.0.1/24`
   - Node Virtual IP: `10.200.0.2/24`
   - AllowedIPs in `wg0.conf` on Node **MUST ALWAYS** be `10.200.0.0/24` to prevent hijacking default internet/DNS routes.

2. **Port Forwarding Invariants**:
   - Game port range: `25565-25700` (TCP/UDP) and `30000-40000` (TCP/UDP).
   - Only ONE forwarder service may bind to `0.0.0.0:25565` at any time (`rinetd`, `haproxy`, or `wirenet-daemon`).
   - Stop competing forwarders before starting a new one (`systemctl stop wirenet-gateway 2>/dev/null`).

3. **Firewall Invariants**:
   - Unconditional ACCEPT for `lo`, `127.0.0.0/8`, `wg0`, and `10.200.0.0/24` in `INPUT` chains before adding any `DROP` rules on `eth0`.
   - Public interface (`eth0`) on backend Node has strict `DROP` rules to prevent backend IP leakage.

---

## 🔍 Instant Troubleshooting Workflows

### Gateway Diagnostic & Repair:
```bash
curl -fsSL https://raw.githubusercontent.com/UG88/wirenet/main/scripts/troubleshoot-gateway.sh | sudo bash
```

### Node Diagnostic & Repair:
```bash
curl -fsSL https://raw.githubusercontent.com/UG88/wirenet/main/scripts/troubleshoot-node.sh | sudo bash
```

### Enable PROXY Protocol v2 (Real Player IPs):
```bash
curl -fsSL https://raw.githubusercontent.com/UG88/wirenet/main/scripts/setup-proxy-protocol.sh | sudo bash
```
*(On Paper/Purpur: add `HAProxyDetectorPaper.jar` to `plugins/`)*.
