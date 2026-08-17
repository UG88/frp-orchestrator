# Security Guide — FRP Gateway Orchestrator

## 1. Threat Model & Security Boundaries

```
[ Untrusted Internet ]
         │ (Public Game Traffic on Gateway IP:Port)
         ▼
[ FRP Gateway (frps) ]  <─── Strict Firewall: Only 7000/tcp & 30000-40000 open
         │
         │ (FRP Token Auth + TLS Encrypted Multiplex)
         ▼
[ FRP Agent (frpc) ]    <─── Loopback Admin API only (127.0.0.1:7400)
         │
         ▼
[ Pterodactyl Node ]    <─── Node Public IP never published
```

---

## 2. Authentication & Credential Isolation

- **Controller REST API**: Protected by constant-time Bearer token verification. Timing attacks are mitigated using bitwise XOR comparison (`constant_time_eq`).
- **Gateway <-> Agent Tunnel**: Authenticated via SHA-256 tokens and forced TLS encryption (`transport.tls.enable = true`).
- **Secrets Management**: Secrets are resolved from environment variables (`ENV:VAR_NAME`) and are never written in plaintext to Git.
- **Log Scrubbing**: Sensitive keys are masked in structured logs showing only the trailing 4 characters.

---

## 3. Systemd Hardening

Production systemd unit files implement strict Linux security primitives:
- `DynamicUser=no` / dedicated unprivileged user `frp`
- `ProtectSystem=strict`: Read-only system filesystem
- `ProtectHome=yes`: Denies access to `/root` and `/home`
- `PrivateTmp=yes`: Isolated `/tmp` namespace
- `NoNewPrivileges=yes`: Prevents privilege escalation
- `CapabilityBoundingSet=CAP_NET_BIND_SERVICE`: Minimal network capability to bind ports below 1024 if needed

---

## 4. Firewall Hardening

- **Never Open 1-65535**: FRP Gateways must only expose their control port (`7000/tcp`) and explicitly configured port ranges (`30000-40000/tcp`, `30000-40000/udp`).
- **Controller API Isolation**: The Controller API port (`8080`) should be bound to a private management VPC / WireGuard network or protected behind an authenticated reverse proxy with rate limiting.

---

## 5. Vulnerability Reporting

Please report security vulnerabilities confidentially to the maintainers at `security@example.com`.
