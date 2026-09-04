# Contributing to WireNet

Read the [current architecture](WireNet/CURRENT_ARCHITECTURE.md), [gap analysis](WireNet/WIRENET_GAP_ANALYSIS.md), and [migration plan](WireNet/WIRENET_MIGRATION.md) before changing networking code.

Changes must preserve the kernel-only normal data path, use validated declarative desired state, avoid unowned firewall mutations, and include tests appropriate to their risk. CI now checks formatting, Clippy, and the current unit suite from the nested crate; the network/integration test suite remains an early required milestone rather than evidence of production readiness.
