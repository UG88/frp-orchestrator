# WireNet

WireNet is being reconstructed as a transparent, kernel-forwarded IPv4 ingress system for private Pterodactyl/Docker game-server backends. WireGuard provides the encrypted gateway-to-node transport; nftables, conntrack, and policy routing carry customer traffic. Customers must not need a plugin, proxy protocol, or special server configuration.

The historical implementation is not approved for deployment: it includes a competing TCP userspace proxy, mutable global firewall changes, fixed addressing, and a shared default control token. Do not follow archived one-command setup or troubleshooting instructions.

The implementation repository and authoritative documentation are in [`WireNet/`](WireNet/):

* [Current implementation inventory](WireNet/CURRENT_ARCHITECTURE.md)
* [Gap analysis and component decisions](WireNet/WIRENET_GAP_ANALYSIS.md)
* [Target architecture](WireNet/WIRENET_ARCHITECTURE.md)
* [Safe migration plan](WireNet/WIRENET_MIGRATION.md)
* [Networking, security, operations, and testing references](WireNet/WIRENET_SPEC.ai)

No production installation command is published until the replacement has passed the integration acceptance matrix.
