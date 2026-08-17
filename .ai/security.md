# Security Policies & Coding Guidelines

## Invariants for AI Coding Agents

1. **Authentication Verification**:
   - All REST API endpoints (except `/health`) require constant-time Bearer token verification (`constant_time_eq`).
   - Plain `==` string comparisons for secrets or token hashes are strictly forbidden.

2. **Secret Scrubbing**:
   - API tokens, passwords, and private keys must never appear in structured logs or exception messages.
   - Use `frp_shared::crypto::mask_secret` when logging diagnostic key references.

3. **No Plaintext Secrets in Code or Git**:
   - Configurations support `ENV:VAR_NAME` syntax for dynamic secret injection.
   - Never commit API keys or gateway tokens to test files or example configurations.

4. **Principle of Least Privilege**:
   - The Gateway service (`frps`) and Controller run under the unprivileged `frp` system user.
   - Systemd units must include `ProtectSystem=strict`, `ProtectHome=yes`, and `PrivateTmp=yes`.
