# AI Changelog — 2026-08-17 Initial Architecture & Implementation

## Summary of Changes
- Designed and built the complete Rust workspace for `frp-orchestrator`:
  - `crates/frp-shared`: Core models, configuration parser, crypto token hashing, DNS provider abstraction (Cloudflare, Route53, Manual, Mock), and API types.
  - `crates/frp-controller`: Central Axum REST API, SQLite database with WAL and migrations, persistent `PortManager` with independent TCP/UDP pools, `ProtocolDetector`, `AllocationManager`, `PterodactylClient`, and `Reconciler`.
  - `crates/frp-agent`: Dynamic `FrpcManager` with multi-file `conf.d` rendering, admin reload API integration, and self-healing runner.
  - `crates/frp-gateway`: Automated `frps.toml` generation, UFW/nftables firewall rules generator, and telemetry reporter.
  - `crates/frpctl`: Comprehensive CLI tool with `frpctl doctor`, installer, and management commands.
- Created hardened systemd service unit files and automated Linux installation scripts.
- Created complete documentation suite: `README.md`, `INSTALLATION.md`, `ARCHITECTURE.md`, `SECURITY.md`, `CONFIGURATION.md`, `TROUBLESHOOTING.md`, `CONTRIBUTING.md`, `LICENSE`, `CHANGELOG.md`.
- Implemented and passed all unit and end-to-end integration tests (`tests/integration_test.rs`) verifying multi-server non-interruption during dynamic tunnel lifecycle.
