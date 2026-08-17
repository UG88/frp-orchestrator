# Contributing to FRP Orchestrator

Thank you for your interest in contributing to the FRP Gateway Orchestrator!

---

## 1. Development Environment Setup

### Prerequisites
- [Rust](https://rustup.rs/) (1.75+)
- [Docker & Docker Compose](https://docs.docker.com/compose/)

### Clone and Build
```bash
git clone https://github.com/UG88/frp-orchestrator.git
cd frp-orchestrator

# Build all workspace crates
cargo build --workspace

# Run all unit and integration tests
cargo test --workspace
```

---

## 2. Running Local Development Stack

A full multi-service simulation environment (Mock Pterodactyl, Mock Minecraft Java/Bedrock, FRPS Gateway, FRPC Agent, and Controller) is provided in `examples/docker-compose.yml`:

```bash
docker-compose -f examples/docker-compose.yml up --build
```

---

## 3. Pull Request Guidelines

1. Ensure all tests pass:
   ```bash
   cargo test --workspace
   ```
2. Format code according to rustfmt standard:
   ```bash
   cargo fmt --all -- --check
   cargo clippy --workspace -- -D warnings
   ```
3. Provide clear explanations for changes, especially any modifications touching network routing, zero-downtime hot-reloads, or port allocation mechanics.
