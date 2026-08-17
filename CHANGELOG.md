# Changelog

All notable changes to the **FRP Gateway Orchestrator** project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

---

## [0.1.0] - 2026-08-17

### Added
- **Core Controller Plane (`frp-controller`)**:
  - Central Axum REST API with constant-time Bearer token authentication.
  - Persistent SQLite store with WAL mode and embedded foreign key constraints.
  - Persistent Port Allocation Engine (`PortManager`) supporting independent TCP and UDP bitmaps, reserved ports, and dual-protocol (`Both`) allocation.
  - Protocol detector supporting Minecraft Java (TCP), Bedrock (UDP), GeyserMC (TCP+UDP / Both), and egg overrides.
  - 3-Way State Reconciliation engine synchronizing Pterodactyl Panel, SQLite DB, and FRP Agent state.
  - Pluggable DNS Provider system (Cloudflare disabled by default, Route53, and default Manual provider).
- **Node Agent Daemon (`frp-agent`)**:
  - Multi-file configuration manager (`/opt/frp/conf.d/*.toml`).
  - Zero-downtime hot-reload engine via FRP Admin API (`/api/reload`).
  - Process monitor and automatic recovery after system reboot.
- **Gateway Manager (`frp-gateway`)**:
  - Automated `frps.toml` generator with port range enforcement.
  - Firewall manager for UFW and nftables.
  - Health and telemetry reporter.
- **Unified CLI (`frpctl`)**:
  - Comprehensive self-healing diagnostic tool (`frpctl doctor`).
  - Automated installer for Gateway, Agent, and Controller.
  - Management subcommands for Gateways, Agents, Allocations, Mappings, and Ports.
- **Systemd Service Definitions**: Hardened service files for `frp-controller`, `frp-agent`, `frp-gateway`, `frps`, and `frpc`.
- **Complete Test Suite**: Unit tests and end-to-end integration tests proving multi-server non-interruption during dynamic tunnel provisioning.
