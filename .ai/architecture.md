# Architecture & System Invariants

## Invariants for AI Coding Agents

1. **Controller as Single Source of Truth**:
   - Never allow agents or nodes to fabricate mappings or bypass the controller.
   - Desired state flows strictly: `Pterodactyl Panel -> Controller DB -> Agent / Gateway -> Actual FRP State`.

2. **Zero-Downtime Hot Reload Requirement**:
   - NEVER write code that stops or restarts the `frpc` systemd service upon creating a single proxy mapping.
   - All mapping adjustments MUST be made by writing or removing files in `/opt/frp/conf.d/<mapping_name>.toml` and issuing an HTTP POST to `http://127.0.0.1:7400/api/reload`.

3. **Persistent Port Allocation Rules**:
   - TCP and UDP ports are managed in separate bitsets/pools.
   - When `Protocol::Both` is requested, the port allocated MUST be available in both pools simultaneously.
   - Reserved ports MUST never be handed out to customer servers.

4. **Reboot Resilience**:
   - The system must self-heal upon node or gateway reboot without requiring manual CLI commands.
   - Agents pull desired state upon startup and render the entire `/opt/frp/conf.d/` directory.
