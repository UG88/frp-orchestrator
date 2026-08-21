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

# Check for Rust / Cargo
if ! command -v cargo >/dev/null 2>&1; then
    echo "[+] Rust/Cargo not found. Installing Rust toolchain..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    source "$HOME/.cargo/env" || export PATH="$HOME/.cargo/bin:$PATH"
fi

echo "[+] Compiling workspace in release mode..."
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
