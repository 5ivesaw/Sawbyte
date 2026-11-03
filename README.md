<p align="center">
  <img src="assets/sawbyte.png" width="180" alt="Sawbyte icon" />
</p>
<h1 align="center">Sawbyte</h1>
<p align="center"><strong>A smaller client. A fully programmable player.</strong><br />Minecraft automation by 5ivesaw.</p>
<p align="center">
  <a href="https://github.com/5ivesaw/Sawbyte/actions/workflows/ci.yml"><img src="https://github.com/5ivesaw/Sawbyte/actions/workflows/ci.yml/badge.svg" alt="Build status" /></a>
  <a href="LICENSE"><img src="https://img.shields.io/badge/license-MIT-00c977" alt="MIT license" /></a>
  <img src="https://img.shields.io/badge/status-early_development-111820" alt="Early development" />
</p>
<p align="center"><a href="#download-and-try">Downloads</a> · <a href="PLAN.md">Roadmap</a> · <a href="docs/COVERAGE.md">Capability coverage</a></p>

---

Sawbyte is being built as a **standalone native client and automation framework**. The optional normal-client bridge exists for integration and reference testing; it is not the main product.

**Foundation preview — not a playable Minecraft client yet.** The goal is comprehensive player automation with a stripped native runtime and a normal-client bridge. See [the plan](PLAN.md) and [honest capability coverage](docs/COVERAGE.md).

## What works in this preview

- A dependency-free Rust input scheduler: bounded queues, atomic batch admission, held buttons, relative camera input, explicit cancellation, and backend fault handling.
- Configurable input policy: bounded camera motion by default, direct mode, and a Rust trait for custom policies. This initial policy is only the first humanisation building block.
- A real TCP Minecraft Java server-status handshake and ping, with bounded packets, UTF-8 validation, and a total request deadline.
- Linux x86-64, Windows x86-64 and macOS ARM64 build/test jobs, with downloadable binaries and SHA-256 checksums.

An independent native player and an optional normal-client input bridge are under development on `feat/real-player`. The bridge builds; native gameplay must still pass the real-server comparison before it is considered usable. No version has passed gameplay parity testing. The trace demo is not a game simulation.

## Standalone player milestone

The separate `native/` Rust workspace implements the first 26.3 connection, authentication, chunk storage and ordinary-ground movement prototype. It uses low-level networking and cryptography libraries; it does not depend on Azalea or another gameplay engine. `bridge/` is an optional Fabric integration that plays the same input recording in the normal client for comparison.

The **Real player milestone** workflow builds the native player for Linux x86-64, Windows x86-64 and macOS ARM64, builds the bridge, then runs a controlled Linux server/client comparison. A successful build alone does not establish gameplay compatibility. See [the test scope and usage](docs/PLAYER.md).

Launcher UI, instance management, general scripting/API access, inventory and screen automation, complete physics and multi-version support remain unfinished. No memory reduction or superiority claim has been measured.

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

All compilation, tests, formatting and packaging run in GitHub Actions. The local workflow is source editing and review. Rust is pinned in `rust-toolchain.toml`; the three foundation crates have **zero external dependencies**. The separate native player uses standard protocol, HTTP and cryptography libraries. There is no JSON serialization in the scheduler.

| Crate | Responsibility |
|---|---|
| `sawbyte-core` | Typed actions, input policy and backend scheduling |
| `sawbyte-protocol` | Bounded wire primitives and status connection |
| `sawbyte` | CLI and trace demonstration |

The protocol implementation is independently written, with no Minecraft jars, assets or decompiled code included. See [design decisions](docs/DECISIONS.md) for limits and next steps.

Not an official Minecraft product. Not approved by or associated with Mojang or Microsoft.
