# Troubleshooting & Diagnostic Rules

## Invariants for AI Coding Agents

1. **Diagnostic First Principle**:
   - When debugging cluster issues, always run `frpctl doctor` first.
   - Look for:
     - FRP binary existence and version.
     - Controller API HTTP 200 response.
     - Gateway health and available port counts.
     - Node heartbeat timestamps.

2. **Useful Remediation Hints**:
   - When reporting failure states in `DoctorCheckItem`, always provide an actionable `fix_hint` rather than generic errors.
