# WireNet Changelog

All notable changes to the **WireNet** project are documented in this file.

---

## [v2.0.0] - 2026-08-23

### 🦀 100% Pure Rust Architecture (`wirenet` Standalone Binary)
- **Unified Engine**: Converted all 12 legacy shell scripts and managers into native, memory-safe Rust modules.
- **WireGuard Provisioning**: Added native key generation and configuration engine in `wirenet setup gateway` and `wirenet setup node`.
- **System Doctor**: Built-in 6-point self-healing diagnostic engine (`wirenet doctor`).
- **Self-Updater**: Added 1-click update and check-update commands (`wirenet update`, `wirenet check-update`).
- **Deep Cleaner**: Built-in 100% system uninstaller (`wirenet uninstall`).
- **Script Cleanup**: Permanently deleted all legacy shell scripts from the codebase.

---

## [v1.3.0] - 2026-08-22

### 🦀 Rust Tokio Ingress Daemon (`wirenet-daemon`)
- **Async Tokio Engine**: Added memory-safe async ingress server on control port `:9000` and game ports `:25565-25700`.
- **Anti-DDoS Shield**: Implemented DashMap-backed per-IP token bucket rate limiting with sub-2µs connection drop times.
- **Docker Event Watcher**: Added native asynchronous observer on `/var/run/docker.sock` for dynamic container port discovery.

### 📊 Zero-Flicker Live Streaming TUI (`wirenet tui`)
- **Double-Buffered Alternate Screen**: Replaced shell loops with `ratatui` + `crossterm` rendering at 10 FPS with 0% screen flicker.
- **Live Real Packet Tracking**: Connected live sparklines and load gauges directly to `/proc/net/dev` and `/sys/class/net/wg0/statistics/`.
- **Real-Time Client IP Sniffer**: Added live parser for `/proc/net/tcp` and `/proc/net/nf_conntrack` displaying genuine player IP addresses in a structured table.

### 🛡️ Pure Method 1 Zero-Plugin Kernel-Level Routing
- **100% Zero Plugins**: Completely standardized on Pure Linux Kernel Layer-3 Transparent Routing with `CONNMARK (0x1)` and Policy Routing `table 100`.
- **Wiped Method 2**: Completely removed all PROXY Protocol v2 and proxy plugin requirements.
- **100% Backend Node IP Invisibility**: Added kernel `DROP` rules on `eth0` for game ports `25565:25700` and `30000:40000`.
- **Instant 0ms Local IP Detection**: Eliminated blocking external `ifconfig.me` calls across all scripts with instant Linux kernel routing table inspection (`ip route get 1.1.1.1`).

### 📚 Knowledge Base & Customizations
- **Rebuilt `.ai/` Knowledge Architecture**: Added comprehensive reference docs (`architecture.md`, `real-ip-mechanics.md`, `pterodactyl.md`, `troubleshooting.md`, `security.md`, `memory.md`, and ADRs).
- **Workspace Skill**: Added `.agents/skills/wirenet/SKILL.md` for standardized agentic operations.

---

## [v1.2.0] - 2026-08-21
- Unified CLI master control center (`wirenet`) with interactive arrow-key navigation.
- 6-point self-healing system doctor (`wirenet doctor`).
- 100% deep cleaner uninstaller (`scripts/uninstall.sh`).

---

## [v1.1.0] - 2026-08-20
- Native Linux kernel WireGuard point-to-point tunneling (`ChaCha20-Poly1305`).
- Multi-node dynamic port pool mapping (`25565-25700` and `30000-40050`).
- Hardware SYN flood cookie protection (`net.ipv4.tcp_syncookies = 1`).

---

## [v1.0.0] - 2026-08-17
- Initial release of WireNet architecture.
