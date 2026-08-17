# Pterodactyl Integration Specs

## Invariants for AI Coding Agents

1. **Application API Authentication**:
   - Uses Bearer token authentication via `PTERODACTYL_API_KEY` (format: `ptla_...`).
   - Standard endpoints:
     - `GET /api/application/nodes/{id}/allocations`
     - `GET /api/application/servers`
     - `GET /api/application/nodes`

2. **Allocation Filtering**:
   - Only process allocations with `assigned == true` (linked to an active server).
   - Allocations with `assigned == false` or unassigned from deleted servers must trigger teardown in the Controller's reconciliation cycle.

3. **Egg & Docker Image Inspection**:
   - Pterodactyl server objects contain `egg` ID and `docker_image` string.
   - Inspect these attributes in `ProtocolDetector` to automatically detect GeyserMC / Floodgate / Bedrock servers.
