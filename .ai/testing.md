# Testing Strategy & Automated Checks

## Invariants for AI Coding Agents

1. **Non-Interruption Test Requirement**:
   - Every modification to proxy provisioning logic must verify that adding or removing a new server does not affect or drop existing mappings on other ports.
   - Run `tests/integration_test.rs` to validate multi-allocation behavior.

2. **Deterministic Mocks**:
   - Unit and integration tests must run without external internet dependencies.
   - Use `MockDnsProvider` and in-memory SQLite database (`Database::open_in_memory()`) for fast, deterministic unit test execution.

3. **Running the Test Suite**:
   ```bash
   cargo test --workspace
   cargo test --test integration_test
   ```
