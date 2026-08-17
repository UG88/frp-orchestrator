# Troubleshooting Guide — FRP Gateway Orchestrator

This guide details common operational issues and step-by-step diagnostic and remediation procedures.

---

## 1. Quick Diagnostic: `frpctl doctor`

Run `frpctl doctor` first whenever encountering issues. It performs end-to-end connectivity, token, service, and port pool validation:

```bash
frpctl doctor
```

---

## 2. Common Issues & Fixes

### Issue 1: `FRP admin API reload returned non-200 status`
**Cause**: The FRP client (`frpc`) is either not running or the admin API credentials in `agent.toml` do not match `frpc.toml`.
**Fix**:
1. Check frpc service status:
   ```bash
   sudo systemctl status frpc
   ```
2. Verify `webServer` block in `/opt/frp/frpc.toml`:
   ```toml
   webServer.addr = "127.0.0.1"
   webServer.port = 7400
   webServer.user = "admin"
   webServer.password = "your-password"
   ```
3. Test admin endpoint directly:
   ```bash
   curl -u admin:your-password http://127.0.0.1:7400/api/status
   ```

---

### Issue 2: `Port already in use on gateway` / `Port conflict`
**Cause**: Another process on the Gateway server is bound to the port or the port is in use outside the orchestrator.
**Fix**:
1. Inspect the port usage on the Gateway host:
   ```bash
   sudo ss -tulpn | grep 31025
   ```
2. Add the conflicting port to `reserved_ports` in `controller.toml`:
   ```toml
   gateways.reserved_ports = [30000, 31025]
   ```
3. Trigger reconciliation:
   ```bash
   frpctl reconcile
   ```

---

### Issue 3: `Pterodactyl API error (status 403 Forbidden)`
**Cause**: The provided `PTERODACTYL_API_KEY` lacks read permissions for Nodes/Allocations/Servers or is expired.
**Fix**:
1. Go to Pterodactyl Admin Panel ──► Application API ──► Create API Key.
2. Ensure permissions: `Nodes (Read)`, `Servers (Read)`, `Allocations (Read)`.
3. Update `controller.env` and restart `frp-controller`.

---

### Issue 4: `Cloudflare DNS record update failed (status 400/403)`
**Cause**: Cloudflare API token lacks `Zone.DNS (Edit)` permission or `zone_id` is mismatched.
**Fix**:
1. Verify token permissions in Cloudflare Dashboard: **Permissions**: `Zone - DNS - Edit`.
2. Confirm the `zone_id` in `controller.toml` matches the target domain.

---

### Issue 5: `Players cannot connect to Geyser server over Bedrock (UDP)`
**Cause**: UDP firewall port is blocked on the Gateway server or protocol was detected as pure TCP.
**Fix**:
1. Verify the firewall rule on the Gateway:
   ```bash
   sudo ufw status | grep udp
   ```
2. Check mapping protocol in CLI:
   ```bash
   frpctl mapping list
   ```
   Ensure `protocol` is listed as `"both"` or `"udp"`.
3. If mapped as `"tcp"`, add egg override in `controller.toml` or recreate mapping with `--protocol both`.
