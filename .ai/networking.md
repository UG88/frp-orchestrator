# Networking & Topology Guidelines

## Invariants for AI Coding Agents

1. **IP Isolation**:
   - The Pterodactyl node's public IP address must never be published to users.
   - The connection address provided to customers is always `<gateway_public_ip>:<gateway_port>` or `<fqdn>:<gateway_port>`.

2. **Protocol Handling**:
   - `tcp`: Minecraft Java Edition (default).
   - `udp`: Minecraft Bedrock dedicated / PocketMine.
   - `both`: GeyserMC / Floodgate crossplay.
   - `auto`: Uses server metadata, egg names, and docker image inspects to choose protocol.

3. **Cloudflare vs Direct Proxying**:
   - Minecraft raw game traffic (TCP & UDP) cannot pass through standard Cloudflare HTTP/HTTPS CDN proxies.
   - When Cloudflare DNS is configured, records must be created with `proxied = false` (DNS-only) unless Cloudflare Spectrum is explicitly in use.
