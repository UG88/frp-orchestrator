# Installation Guide — FRP Gateway Orchestrator

This document provides step-by-step instructions for deploying the FRP Orchestrator stack across your infrastructure.

---

## Architecture Topology

```
                  INTERNET
                      │
                      ▼
              ┌───────────────┐
              │  FRP Gateway  │  (frps: 3.108.x.x)
              │  Port 31025   │
              └───────┬───────┘
                      │
            FRP Encrypted Tunnel (Control Port 7000)
                      │
                      ▼
              ┌───────────────┐
              │   FRP Agent   │  (frpc on Node SG-01)
              └───────┬───────┘
                      │
              Local Network (10.10.0.20:25567)
                      │
                      ▼
              ┌───────────────┐
              │  Pterodactyl  │
              │  Wings Server │
              └───────────────┘
```

---

## 1. Controller Deployment

The Controller is the central orchestrator and single source of truth. It manages gateway port pools, monitors Pterodactyl allocations, synchronizes DNS records, and serves desired state to FRP agents.

### Requirements
- Linux host (Debian 11+, Ubuntu 20.04+, RHEL 8+, Alpine 3.18+)
- SQLite3 runtime support (bundled automatically)
- Outbound HTTPS access to Pterodactyl API and DNS providers

### Setup Steps
1. Create dedicated user and directory:
   ```bash
   sudo useradd -r -s /bin/false -d /var/lib/frp-orchestrator frp
   sudo mkdir -p /var/lib/frp-orchestrator /etc/frp-orchestrator
   sudo chown -R frp:frp /var/lib/frp-orchestrator
   ```

2. Copy the controller binary:
   ```bash
   sudo cp target/release/frp-controller /usr/local/bin/frp-controller
   sudo chmod 755 /usr/local/bin/frp-controller
   ```

3. Create configuration file `/etc/frp-orchestrator/controller.toml`:
   ```toml
   listen_addr = "0.0.0.0:8080"
   api_key = "ENV:CONTROLLER_API_KEY"
   database_path = "/var/lib/frp-orchestrator/controller.db"
   reconciliation_interval_secs = 30

   [pterodactyl]
   url = "https://panel.example.com"
   api_key = "ENV:PTERODACTYL_API_KEY"
   sync_interval_secs = 30
   auto_expose_default = true

   [dns]
   default_domain = "mc.example.com"

   [dns.cloudflare]
   enabled = false # Set to true to enable automated Cloudflare A-record provisioning
   api_token = "ENV:CLOUDFLARE_API_TOKEN"
   zone_id = "your_cloudflare_zone_id"
   proxied = false
   ttl = 1

   [[gateways]]
   id = "gw-sg-01"
   region = "singapore"
   public_ip = "3.108.50.20"
   control_port = 7000
   token = "ENV:FRP_GATEWAY_TOKEN"
   tls_enable = true
   dashboard_port = 7500

   [gateways.tcp_port_range]
   start = 30000
   end = 40000

   [gateways.udp_port_range]
   start = 30000
   end = 40000

   gateways.reserved_ports = [30000, 30001]
   ```

4. Create environment file `/etc/frp-orchestrator/controller.env`:
   ```bash
   CONTROLLER_API_KEY="your-secure-controller-api-key-here"
   PTERODACTYL_API_KEY="ptla_your_pterodactyl_application_api_key"
   FRP_GATEWAY_TOKEN="your-super-secret-gateway-token"
   ```
   ```bash
   sudo chmod 600 /etc/frp-orchestrator/controller.env
   sudo chown frp:frp /etc/frp-orchestrator/controller.env
   ```

5. Install and start systemd service:
   ```bash
   sudo cp systemd/frp-controller.service /etc/systemd/system/
   sudo systemctl daemon-reload
   sudo systemctl enable --now frp-controller
   ```

6. Verify controller:
   ```bash
   curl http://127.0.0.1:8080/health
   ```

---

## 2. Gateway Deployment (`frps`)

The Gateway is installed on dedicated servers located in each target region (e.g. Singapore, Germany, US).

### Requirements
- Public IPv4 / IPv6 address
- Open control port (default `7000/tcp`)
- Open allocation port range (e.g., `30000-40000/tcp` and `30000-40000/udp`)

### Automated Installation
```bash
curl -fsSL https://raw.githubusercontent.com/example/frp-orchestrator/main/scripts/install-gateway.sh | sudo bash
```

### Manual Installation
1. Download and unpack FRP Server:
   ```bash
   sudo mkdir -p /opt/frp /etc/frp-gateway
   sudo useradd -r -s /bin/false -d /opt/frp frp
   # Download frps binary to /opt/frp/frps
   ```

2. Configure `/opt/frp/frps.toml`:
   ```toml
   bindAddr = "0.0.0.0"
   bindPort = 7000
   auth.method = "token"
   auth.token = "your-super-secret-gateway-token"

   allowPorts = [
     { start = 30000, end = 40000 }
   ]

   webServer.addr = "127.0.0.1"
   webServer.port = 7500
   webServer.user = "admin"
   webServer.password = "your-super-secret-gateway-token"
   ```

3. Configure Firewall (UFW example):
   ```bash
   sudo ufw allow 7000/tcp comment 'FRP Control Port'
   sudo ufw allow 30000:40000/tcp comment 'Minecraft FRP TCP Ports'
   sudo ufw allow 30000:40000/udp comment 'Minecraft FRP UDP Ports'
   ```

4. Start `frps` service:
   ```bash
   sudo cp systemd/frps.service /etc/systemd/system/
   sudo systemctl daemon-reload
   sudo systemctl enable --now frps
   ```

---

## 3. Node Agent Deployment (`frpc` + `frp-agent`)

The Agent is installed on every Pterodactyl node running Wings.

### Automated Installation
```bash
curl -fsSL https://raw.githubusercontent.com/example/frp-orchestrator/main/scripts/install-agent.sh | sudo bash
```

### Configuration `/etc/frp-agent/agent.toml`:
```toml
agent_id = "agent-sg-node-01"
controller_url = "https://controller.internal.example.com"
agent_token = "ENV:FRP_AGENT_TOKEN"
pterodactyl_node_id = 1

frpc_binary_path = "/opt/frp/frpc"
frpc_config_dir = "/opt/frp/conf.d"
frpc_main_config = "/opt/frp/frpc.toml"

frpc_admin_addr = "127.0.0.1:7400"
frpc_admin_user = "admin"
frpc_admin_password = "ENV:FRPC_ADMIN_PASSWORD"
heartbeat_interval_secs = 15
```

### Start Services:
```bash
sudo cp systemd/frpc.service systemd/frp-agent.service /etc/systemd/system/
sudo systemctl daemon-reload
sudo systemctl enable --now frpc
sudo systemctl enable --now frp-agent
```

---

## 4. Verification with `frpctl doctor`

Run the automated diagnostic suite from any machine:
```bash
export CONTROLLER_URL="https://controller.internal.example.com"
export CONTROLLER_API_KEY="your-secure-controller-api-key-here"

frpctl doctor
```
