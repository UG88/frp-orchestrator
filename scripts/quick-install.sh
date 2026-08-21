#!/usr/bin/env bash
# ==============================================================================
# FRP Orchestrator — 1-Line Quick Installer for all Linux VPS
# ==============================================================================

set -euo pipefail

echo "=========================================================="
echo " Starting 1-Line FRP Orchestrator Tools Installation"
echo "=========================================================="

if [[ $EUID -ne 0 ]]; then
   echo "[-] Error: This script must be run as root (or with sudo)." >&2
   exit 1
fi

TMP_BUILD_DIR=$(mktemp -d)
trap 'rm -rf "$TMP_BUILD_DIR"' EXIT

echo "[+] Cloning repository from https://github.com/UG88/frp-orchestrator.git..."
if command -v git >/dev/null 2>&1; then
    git clone --depth 1 https://github.com/UG88/frp-orchestrator.git "$TMP_BUILD_DIR"
else
    # If git is not installed, install git first
    if command -v apt-get >/dev/null 2>&1; then
        export DEBIAN_FRONTEND=noninteractive
        apt-get update -qq && apt-get install -y -qq git curl >/dev/null 2>&1
    fi
    git clone --depth 1 https://github.com/UG88/frp-orchestrator.git "$TMP_BUILD_DIR"
fi

cd "$TMP_BUILD_DIR"
bash ./scripts/build-and-install.sh

echo "=========================================================="
echo " [✓] Quick installation finished! frpctl is now available."
echo " You can now run:"
echo "   sudo frpctl init controller"
echo "   sudo frpctl init gateway"
echo "   sudo frpctl init agent"
echo "   frpctl doctor"
echo "=========================================================="
