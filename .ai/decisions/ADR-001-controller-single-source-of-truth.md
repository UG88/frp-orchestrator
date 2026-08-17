# ADR-001: Central Controller as Single Source of Truth

## Context
In a multi-node, multi-gateway Pterodactyl deployment, managing port allocations and FRP tunnels locally on each node leads to race conditions, split-brain routing, port collisions across gateways, and brittle shell scripts.

## Decision
We established the central `frp-controller` as the authoritative single source of truth for desired state.
- Desired state is computed from Pterodactyl allocations and saved in a transactional SQLite store.
- Agents and Gateways pull desired state from the Controller and report back actual runtime state.
- A 3-way reconciliation engine continuously converges actual state to desired state.

## Status
Accepted.

## Consequences
- Clean horizontal scaling across dozens of nodes and multiple gateways.
- Deterministic port allocations and orphan port cleanup.
- Transparent failover and recovery after node/gateway reboots.
