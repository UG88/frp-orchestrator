# WireNet Changelog

All notable changes to the **WireNet** project are documented in this file.

---

## [v1.3.0] - 2026-08-22

### 🦀 Rust Tokio Ingress Daemon (`wirenet-daemon`)
- **Async Tokio Engine**: Added memory-safe async ingress server on control port `:9000` and game ports `:25565-25700`.
- **Anti-DDoS Shield**: Implemented DashMap-backed per-IP token bucket rate limiting with sub-2µs connection drop times.
- **PROXY Protocol v2**: Implemented binary encoder for injecting genuine client IPv4 headers.
- **Docker Event Watcher**: Added native asynchronous observer on `/var/run/docker.sock` for dynamic container port discovery.

### 📊 Zero-Flicker Live Streaming TUI (`wirenet tui`)
- **Double-Buffered Alternate Screen**: Replaced shell loops with `ratatui` + `crossterm` rendering at 10 FPS with 0% screen flicker.
- **Live Real Packet Tracking**: Connected live sparklines and load gauges directly to `/proc/net/dev` and `/sys/class/net/wg0/statistics/`.
- **Real-Time Client IP Sniffer**: Added live parser for `/proc/net/tcp` and `/proc/net/nf_conntrack` displaying genuine player IP addresses in a structured table.

### 🛡️ Networking & Tunneling Enhancements
- **100% Backend Node IP Invisibility**: Added kernel `DROP` rules on `eth0` for game ports `25565:25700` and `30000:40000`.
- **Symmetrical WireGuard Return Routing**: Applied `MASQUERADE` across `wg0` on Gateway to guarantee zero packet loss across cloud providers (AWS, DigitalOcean, Hetzner).
- **Instant 0ms Local IP Detection**: Eliminated blocking external `ifconfig.me` calls across all scripts with instant Linux kernel routing table inspection (`ip route get 1.1.1.1`).
- **Port Collision Protection**: Eliminated `EADDRINUSE` conflicts by orchestrating mutual exclusivity between `rinetd`, `haproxy`, and `wirenet-daemon`.

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
