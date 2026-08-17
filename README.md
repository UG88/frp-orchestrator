# FRP Orchestrator — Automated Gateway System for Pterodactyl & Minecraft

[![CI](https://github.com/example/frp-orchestrator/actions/workflows/ci.yml/badge.svg)](https://github.com/example/frp-orchestrator/actions)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

**FRP Orchestrator** is a production-ready, zero-downtime gateway orchestration system designed specifically for Minecraft hosting providers operating on [Pterodactyl](https://pterodactyl.io).

It establishes an automated, encrypted networking layer that connects external players to backend Pterodactyl nodes via an elastic fleet of Fast Reverse Proxy (FRP) gateways. Backend node IP addresses are kept completely private and are never exposed to customers.

---

## Architecture Overview

```mermaid
flowchart TD
    Player[Minecraft Java / Bedrock Player] -->|play.example.com:31025| GW[FRP Gateway - frps\nPublic IP: 3.108.x.x]
    
    subgraph Gateway Fleet
        GW
        GW2[FRP Gateway Europe\nPublic IP: 198.51.100.2]
    end

    GW -->|Encrypted/Authenticated FRP Tunnel| Agent[FRP Node Agent - frpc\nNode SG-01]
    
    subgraph Pterodactyl Node
        Agent
        Wings[Pterodactyl Wings]
        MC[Minecraft Server\n10.10.0.20:25567]
        Agent -->|Local Forwarding| MC
    end

    subgraph Central Control Plane
        Ctrl[FRP Controller\nREST API + Reconciler]
        DB[(SQLite / Persistent DB)]
        Ptero[Pterodactyl Application API]
        DNS[DNS Provider\nManual / Cloudflare / Route53]
        
        Ctrl <--> DB
        Ctrl <--> Ptero
        Ctrl --> DNS
        Ctrl <-->|Heartbeat & Dynamic Config| Agent
        Ctrl <-->|Health Monitoring| GW
    end
```

---

## Key Features

- **Zero-Downtime Hot Reloading**: Uses FRP's multi-file configuration (`conf.d/*.toml`) and admin reload API. Adding, modifying, or removing a server never interrupts existing active Minecraft sessions.
- **Hidden Node Topology**: Players connect to the FRP Gateway IP or FQDN. The backend node's public IP remains completely unexposed.
- **Protocol Intelligence**:
  - `tcp`: Standard Minecraft Java Edition.
  - `udp`: Bedrock standalone / PocketMine.
  - `both`: GeyserMC & Floodgate crossplay servers (TCP and UDP mapped simultaneously to the same public gateway port).
  - `auto`: Automatic egg / docker image detection with manual administrator overrides.
- **State Reconciliation**: Continuous 3-way reconciliation between Pterodactyl Panel, Controller DB, and FRP Agents. Automatically creates missing tunnels, purges obsolete allocations, and reclaims ports.
- **Persistent Port Allocation Engine**: Separate TCP and UDP port bitmaps, reserved range exclusions, conflict avoidance, and instant port reclamation upon server deletion.
- **Multi-Gateway & Regional Routing**: Nodes map to specific regional gateways (e.g. Singapore, India, Germany) with health monitoring and failover.
- **Built-in DNS Management**: Automated FQDN provisioning (`srv-<id>.domain` or `<alias>.domain`). Supports Cloudflare (disabled by default; easily enabled via config), Route53, and manual modes.
- **CLI & Diagnostic Suite (`frpctl`)**: Unified CLI tool for installation, cluster monitoring, manual mappings, and self-healing diagnostic (`frpctl doctor`).

---

## Component Layout

| Component | Binary | Description |
|---|---|---|
| **Controller** | `frp-controller` | Central orchestrator, REST API, SQLite state store, port allocator, and reconciler |
| **Agent** | `frp-agent` | Daemon installed on Pterodactyl nodes. Manages `frpc` multi-file configuration and hot reloads |
| **Gateway** | `frp-gateway` | Daemon installed on FRP Gateway servers. Manages `frps`, telemetry, and firewall rules |
| **CLI** | `frpctl` | Unified administrator CLI and diagnostic utility |

---

## Quick Start

### 1. Start the Controller
```bash
# Generate configuration template
frp-controller --config controller.toml

# Run the controller
frp-controller --config controller.toml
```

### 2. Install Gateway on Public Server
```bash
# Automated non-interactive install
curl -fsSL https://raw.githubusercontent.com/example/frp-orchestrator/main/scripts/install-gateway.sh | sudo bash

# Or via CLI
sudo frpctl install gateway
```

### 3. Install Agent on Pterodactyl Node
```bash
# Automated installer
curl -fsSL https://raw.githubusercontent.com/example/frp-orchestrator/main/scripts/install-agent.sh | sudo bash

# Or via CLI
sudo frpctl install agent
```

### 4. Verify System Health with `frpctl doctor`
```bash
frpctl doctor
```

Output:
```
FRP Orchestrator Doctor Diagnostic
==================================
[✓] Operating System & Architecture
    Details: OS: linux, Arch: amd64

[✓] FRP Binary Check
    Details: Found FRP executable at /opt/frp/frpc

[✓] Controller API Connectivity
    Details: Connected to Controller v0.1.0 (Uptime: 142s, DB: OK)

[✓] Gateway Cluster Status
    Details: Gateways online: 1/1 healthy

[✓] Pterodactyl Nodes Status
    Details: Nodes connected: 1/1 healthy (Active mappings: 2)

Result: All checks PASSED! System is ready.
```

---

## CLI Reference

```bash
# Gateway inspection
frpctl gateway list
frpctl gateway status gw-sg-01

# Node & Agent inspection
frpctl agent list
frpctl agent status agent-sg-node-01

# Allocations & Mappings
frpctl allocation list
frpctl mapping list
frpctl mapping create --node-id node-sg-01 --allocation-id alloc-101 --local-ip 10.10.0.20 --local-port 25565 --protocol both
frpctl mapping delete <mapping-id>

# Port pool metrics
frpctl port list

# Trigger immediate reconciliation
frpctl reconcile

# Cluster health report
frpctl health
```

---

## Documentation

- [**INSTALLATION.md**](file:///INSTALLATION.md) — Complete setup guide for Gateways, Agents, Controllers, and Firewalls.
- [**ARCHITECTURE.md**](file:///ARCHITECTURE.md) — Deep architectural dive, reconciliation flow, and zero-downtime mechanics.
- [**CONFIGURATION.md**](file:///CONFIGURATION.md) — Complete configuration specification for all components.
- [**SECURITY.md**](file:///SECURITY.md) — Security model, token hashing, firewall rules, and systemd hardening.
- [**TROUBLESHOOTING.md**](file:///TROUBLESHOOTING.md) — Diagnostic guides and common error resolutions.
- [**CONTRIBUTING.md**](file:///CONTRIBUTING.md) — Guidelines for contributing and running local test environments.

---

## License

This project is licensed under the MIT License — see the [LICENSE](file:///LICENSE) file for details.
