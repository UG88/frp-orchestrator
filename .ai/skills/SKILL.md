---
name: frp-orchestrator-ops
description: Operational workflows and troubleshooting procedures for the FRP Gateway Orchestrator stack.
---

# FRP Gateway Orchestrator Operations Skill

## 1. Running System Diagnostics
Whenever investigating a system problem, start by running:
```bash
frpctl doctor
```

## 2. Triggering Manual State Reconciliation
To force an immediate sync between Pterodactyl Panel allocations, database desired state, and live FRP proxies:
```bash
frpctl reconcile
```

## 3. Provisioning a Test Mapping
```bash
frpctl mapping create \
  --node-id node-sg-01 \
  --allocation-id alloc-test-01 \
  --local-ip 10.10.0.20 \
  --local-port 25565 \
  --protocol both \
  --alias play
```

## 4. Verifying Port Pool Usage
```bash
frpctl port list
```
