# Production Deployment Standards

## Standards for Deployment Automation

1. **Systemd Services**:
   - `frp-controller.service`: Controller daemon.
   - `frps.service`: FRP Server daemon.
   - `frp-gateway.service`: Gateway health reporter.
   - `frpc.service`: FRP Client daemon.
   - `frp-agent.service`: Node agent daemon.

2. **Idempotency & Safety**:
   - Install scripts must be safe to re-run multiple times without damaging existing configurations.
   - Never overwrite user configurations without confirmation. Create `.bak` backups before modifying files.

3. **Firewall Safety**:
   - Never run `ufw default allow incoming` or blindly open `1-65535`.
   - Restrict port openings strictly to control port (`7000/tcp`) and configured allocation ranges.
