# ADR-002: Zero-Downtime Hot Reloading via FRP Multi-File `includes` and Admin API

## Context
Restarting `frpc` whenever a single Minecraft server allocation is created, modified, or removed interrupts active TCP and UDP streams for all other servers running on that node. In a hosting environment, this disrupts paying players on unrelated servers.

## Decision
We implemented dynamic multi-file configuration (`includes = ["/opt/frp/conf.d/*.toml"]`) combined with the embedded `frpc` HTTP administrative reload API (`http://127.0.0.1:7400/api/reload`).
- Each server mapping is stored in an independent file `/opt/frp/conf.d/mc_<mapping_id>.toml`.
- When an allocation is added or removed, `frp-agent` writes or deletes only that specific file.
- `frp-agent` triggers an authenticated reload via `/api/reload`.
- `frpc` re-parses configuration files without dropping existing active proxy connections.

## Status
Accepted and verified through automated integration tests.

## Consequences
- Guaranteed zero-downtime provisioning and teardown for Minecraft servers.
- Safe dynamic scaling.
