# Experimental native player

Sawbyte's main runtime is independently written Rust. The optional Fabric bridge provides reference/graphical integration. Neither the bridge nor a Java client is required by the native executable.

Current implementation targets Minecraft Java 26.3, protocol 777. It supports a controlled replay and an experimental live command loop. It is not ready as a general-purpose client for arbitrary servers. See [coverage](COVERAGE.md) and [known defects](AUDIT-2026-11-03.md).

## Existing evidence

The GitHub player workflow builds/tests native executables on Linux x86-64, Windows x86-64 and macOS ARM64. Its Linux integration job downloads the official server, generates a limited block/protocol profile, and compares a 160-tick input recording against a normal Fabric client under a virtual display.

[Run 41581827919](https://github.com/5ivesaw/Sawbyte/actions/runs/41581827919), commit d70a744, passed that basic walk/turn/jump/stop comparison with maximum error about 0.000000341 blocks and zero ground-state mismatches. Native sampled RSS was 5,080 KiB. Server/reference-client memory is separate.

The later live-command scenario demonstrates movement scheduling but only checks that the player moved more than 1.5 blocks. Its final position missed the requested destination. Its synthetic screen/result fallback is not evidence of real GUI interaction. The test must be strengthened.

## Commands that exist

All compilation/testing stays in GitHub Actions. These are experimental interfaces, not supported production setup instructions:

```text
sawbyte tape movement.sbif
sawbyte-play 127.0.0.1:25565 --offline SawbyteNative movement.sbif native.csv block-profile.json
sawbyte-play live 127.0.0.1:25565 --offline SawbyteNative block-profile.json --listen 127.0.0.1:25580
sawbyte-play login <registered-microsoft-client-id> <new-token-file>
sawbyte launcher list
```

The CLI and native binary are separate artifacts. The native runtime still requires a matching data profile such as the limited one produced in the CI evidence artifact. There is no completed installer/data provisioning path. The launcher expects the native executable beside the CLI or at a configured path.

Online login code requires an eligible account and permitted registered application. Live online validation, token refresh and OS credential-store integration are unfinished. The existing flow saves a short-lived access token; keep it out of repositories and shared artifacts.

## Limits relevant to experiments

- Only the named basic movement fixture has reference parity evidence. Extended mechanics are partial or approximate. Unknown collision states still stop execution.
- Loaded chunks are capped at 256. Dynamic block update code exists, but broad changing-world behaviour is not certified.
- Item/component and entity decoding is incomplete; some decoder errors are discarded. Do not rely on current screen state as complete.
- Some discrete events have no native encoder and are silently skipped. Coordinate-based interaction paths can bypass raycasting, and mining uses a fixed duration.
- Live navigation compiles fixed-duration movement segments without a completed arrival/replanning controller.
- The public workflow path can synthesize screens and completion events. It must be removed from production execution, with general state-driven control implemented instead.
- MCP is experimental and currently lacks required tool input schemas. Control listener authentication/bounds and SDK interoperability need work.
- Training step currently queues input; it does not wait for a completed server-synchronized step. Reset does not restore a world.
- Launcher version/mode labels do not dispatch different engines yet. A 26.2 profile name is not 26.2 compatibility.
- The optional bridge currently supports replay and a small command set, not full graphical input/screen equivalence.

Platform PNG/ICO/ICNS assets are packaged, and Windows executables embed the icon. Current workflows do not create a finished graphical launcher, macOS application bundle or system installer.

## Next implementation principle

Complete the reusable live observation/input/execution layer and verify its outcomes across varied situations. Do not build a privileged feature around a user's illustrative sequence. Every convenience task must use the same general primitives available to external programs.
