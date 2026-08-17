# Configuration Reference — FRP Gateway Orchestrator

Complete configuration reference for all components.

---

## 1. Controller Configuration (`controller.toml`)

```toml
# IP address and port the Controller binds to
listen_addr = "0.0.0.0:8080"

# Master API token for authenticating CLI and Agent requests
api_key = "ENV:CONTROLLER_API_KEY"

# SQLite database storage location
database_path = "/var/lib/frp-orchestrator/controller.db"

# Interval (in seconds) between automatic state reconciliation cycles
reconciliation_interval_secs = 30

[pterodactyl]
# Base URL of your Pterodactyl Panel
url = "https://panel.example.com"

# Pterodactyl Application API Key (starts with ptla_)
api_key = "ENV:PTERODACTYL_API_KEY"

# Interval for synchronizing allocations from Pterodactyl API
sync_interval_secs = 30

# Automatically expose newly created server allocations through FRP
auto_expose_default = true

[pterodactyl.egg_protocol_overrides]
# Map specific egg names to explicit protocols (tcp | udp | both)
"geyser-standalone" = "both"
"bedrock-dedicated" = "udp"

[dns]
# Base domain used for generating Minecraft player connection addresses
default_domain = "mc.example.com"

[dns.cloudflare]
# Automated DNS A-record provisioning via Cloudflare (disabled by default)
enabled = false
api_token = "ENV:CLOUDFLARE_API_TOKEN"
zone_id = "023e105f4ecef8ad9ca31a8372d0c353"
proxied = false
ttl = 1

[[gateways]]
# Unique identifier for the gateway
id = "gw-sg-01"

# Geographical region
region = "singapore"

# Public IP address of the gateway (players connect here)
public_ip = "3.108.50.20"

# Control traffic port for FRP
control_port = 7000

# Shared authentication secret between gateway and agent
token = "ENV:FRP_GATEWAY_TOKEN"

# Force TLS encryption on tunnel traffic
tls_enable = true

# FRP Server dashboard metrics port
dashboard_port = 7500

[gateways.tcp_port_range]
start = 30000
end = 40000

[gateways.udp_port_range]
start = 30000
end = 40000

# Reserved ports excluded from dynamic customer allocation
reserved_ports = [30000, 30001]
```

---

## 2. Agent Configuration (`agent.toml`)

```toml
# Unique identifier of the node agent
agent_id = "agent-sg-node-01"

# URL of the FRP Orchestrator Controller
controller_url = "https://controller.internal.example.com"

# Secret authentication token for this agent
agent_token = "ENV:FRP_AGENT_TOKEN"

# Corresponding Node ID in Pterodactyl Panel
pterodactyl_node_id = 1

# Path to the frpc executable
frpc_binary_path = "/opt/frp/frpc"

# Directory where individual proxy configurations are managed
frpc_config_dir = "/opt/frp/conf.d"

# Main frpc.toml base configuration file
frpc_main_config = "/opt/frp/frpc.toml"

# Admin API bind address for zero-downtime hot reloads
frpc_admin_addr = "127.0.0.1:7400"
frpc_admin_user = "admin"
frpc_admin_password = "ENV:FRPC_ADMIN_PASSWORD"

# Heartbeat interval with controller in seconds
heartbeat_interval_secs = 15
```

---

## 3. Gateway Configuration (`gateway.toml`)

```toml
id = "gw-sg-01"
region = "singapore"
public_ip = "3.108.50.20"
control_port = 7000
token = "ENV:FRP_GATEWAY_TOKEN"
tls_enable = true
dashboard_port = 7500

[tcp_port_range]
start = 30000
end = 40000

[udp_port_range]
start = 30000
end = 40000

reserved_ports = [30000, 30001]
```
