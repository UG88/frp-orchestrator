#!/usr/bin/env bash
# ==============================================================================
# FRP Orchestrator — Build & Install All Binaries from Source
# ==============================================================================

set -euo pipefail

echo "=========================================================="
echo " Building & Installing FRP Orchestrator Binaries"
echo "=========================================================="

if [[ $EUID -ne 0 ]]; then
   echo "[-] Error: This script must be run as root (or with sudo)." >&2
   exit 1
fi

# 1. Install prerequisites (C compiler, git, curl)
if command -v apt-get >/dev/null 2>&1; then
    echo "[+] Updating apt and installing build tools (build-essential, git, curl, pkg-config, libssl-dev)..."
    export DEBIAN_FRONTEND=noninteractive
    apt-get update -qq
    apt-get install -y -qq build-essential git curl pkg-config libssl-dev >/dev/null 2>&1 || true
elif command -v dnf >/dev/null 2>&1; then
    dnf groupinstall -y "Development Tools" >/dev/null 2>&1 || true
    dnf install -y git curl pkg-config openssl-devel >/dev/null 2>&1 || true
elif command -v yum >/dev/null 2>&1; then
    yum groupinstall -y "Development Tools" >/dev/null 2>&1 || true
    yum install -y git curl pkgconfig openssl-devel >/dev/null 2>&1 || true
elif command -v apk >/dev/null 2>&1; then
    apk add --no-cache build-base git curl openssl-dev
fi

# 2. Check for Rust / Cargo
if ! command -v cargo >/dev/null 2>&1; then
    if [[ -f "$HOME/.cargo/env" ]]; then
        # shellcheck disable=SC1091
        source "$HOME/.cargo/env"
    elif [[ -f "/root/.cargo/env" ]]; then
        # shellcheck disable=SC1091
        source "/root/.cargo/env"
    fi
fi

if ! command -v cargo >/dev/null 2>&1; then
    echo "[+] Rust/Cargo not found. Installing official Rust toolchain via rustup..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --profile minimal
    if [[ -f "$HOME/.cargo/env" ]]; then
        # shellcheck disable=SC1091
        source "$HOME/.cargo/env"
    elif [[ -f "/root/.cargo/env" ]]; then
        # shellcheck disable=SC1091
        source "/root/.cargo/env"
    fi
    export PATH="$HOME/.cargo/bin:/root/.cargo/bin:$PATH"
fi

export PATH="$HOME/.cargo/bin:/root/.cargo/bin:$PATH"
echo "[+] Using Rust: $(cargo --version)"

echo "[+] Compiling FRP Orchestrator workspace in release mode (this may take 1-2 minutes)..."
cargo build --release --workspace

echo "[+] Installing binaries to /usr/local/bin/..."
install -m 0755 target/release/frp-controller /usr/local/bin/frp-controller
install -m 0755 target/release/frp-agent /usr/local/bin/frp-agent
install -m 0755 target/release/frp-gateway /usr/local/bin/frp-gateway
install -m 0755 target/release/frpctl /usr/local/bin/frpctl

echo "=========================================================="
echo " [✓] All FRP Orchestrator binaries installed successfully!"
echo " Available commands:"
echo "   - frpctl (Management CLI & Setup Wizard)"
echo "   - frp-controller (Central Orchestrator)"
echo "   - frp-agent (Pterodactyl Node Daemon)"
echo "   - frp-gateway (Gateway Management Daemon)"
echo "=========================================================="
