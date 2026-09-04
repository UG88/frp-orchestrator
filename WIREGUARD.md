# WireNet and WireGuard

WireNet uses the Linux WireGuard implementation as encrypted transport; it does not implement custom VPN cryptography. WireGuard is only one part of the required packet path: transparent source-IP delivery also depends on gateway DNAT without tunnel SNAT, node conntrack marks, policy routing for replies, and Docker-compatible node forwarding.

The complete design and validation requirements are in [WireNet/WIRENET_NETWORKING.md](WireNet/WIRENET_NETWORKING.md) and [WireNet/WIRENET_ARCHITECTURE.md](WireNet/WIRENET_ARCHITECTURE.md). A successful handshake alone is not proof of end-to-end ingress or real-IP preservation.
