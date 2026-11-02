# Sawbyte

A lightweight, programmable Minecraft player framework by 5ivesaw.

**Foundation preview — not a playable Minecraft client yet.** The goal is comprehensive player automation with a stripped native runtime and a normal-client bridge. See [the plan](PLAN.md) and [honest capability coverage](docs/COVERAGE.md).

## What works in this preview

- A dependency-free Rust input scheduler: bounded queues, atomic batch admission, held buttons, relative camera input, explicit cancellation, and backend fault handling.
- Configurable input policy: bounded camera motion by default, direct mode, and a Rust trait for custom policies. This initial policy is only the first humanisation building block.
- A real TCP Minecraft Java server-status handshake and ping, with bounded packets, UTF-8 validation, and a total request deadline.
- Linux x86-64, Windows x86-64 and macOS ARM64 build/test jobs, with downloadable binaries and SHA-256 checksums.

No Minecraft authentication, gameplay simulation, GUI automation, launcher instances, Python SDK, or graphical-client bridge is implemented yet. No version has passed gameplay parity testing. The trace demo is not a game simulation.

## Download and try

Open this repository's **Actions → Build and verify → a successful run → Artifacts**. Download the artifact for your operating system and extract it. On Linux/macOS, run `chmod +x sawbyte` if the artifact archive did not preserve executable permissions. `SHA256SUMS` covers the included files.

```text
sawbyte --help
sawbyte capabilities
sawbyte demo
sawbyte demo --direct
sawbyte status 127.0.0.1:25565
sawbyte status 127.0.0.1:25565 example.test -1
```

The status probe takes an IP address and port; the optional virtual host supports server routing. DNS/SRV lookup is not implemented. Server text is displayed escaped, and no status text is executed. A successful status query proves status connectivity only, not gameplay compatibility.

## Development

All compilation, tests, formatting and packaging run in GitHub Actions. The local workflow is source editing and review. Rust is pinned in `rust-toolchain.toml`; all three crates currently have **zero external dependencies**. This is a measured starting point, not a permanent ban on useful libraries. There is no JSON serialization in the scheduler.

| Crate | Responsibility |
|---|---|
| `sawbyte-core` | Typed actions, input policy and backend scheduling |
| `sawbyte-protocol` | Bounded wire primitives and status connection |
| `sawbyte` | CLI and trace demonstration |

The protocol implementation is independently written, with no Minecraft jars, assets or decompiled code included. See [design decisions](docs/DECISIONS.md) for limits and next steps.

Not an official Minecraft product. Not approved by or associated with Mojang or Microsoft.
