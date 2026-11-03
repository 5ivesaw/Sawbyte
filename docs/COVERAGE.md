# Capability ledger

The product requirement is full player programmability. This ledger describes implemented coverage, not the final scope. Every missing capability is remaining work. Future version-specific inventories must be derived from actual client interfaces and validated against the real client.

| Area | Current status | Evidence / next gate |
|---|---|---|
| Typed scheduling | Implemented, trace backend only | Atomic queues, cancellation, monotonic tick tests |
| Camera policy | Bounded motion and direct mode | Total motion and custom-policy validation tests |
| Network status / ping | Implemented | Loopback TCP fixture with handshake and pong verification |
| Packet bounds / deadline | Implemented for status | Invalid frame, VarInt, stalled-peer tests |
| Native login / accounts | Experimental code, not validated live | 26.3 login, encryption and Microsoft device flow; registered application and account required for online validation |
| Native player movement | Experimental, comparison pending | Bounded chunk palettes and ordinary full-cube motion; no general gameplay support |
| Full-client bridge | Builds; replay validation pending | Optional Fabric 26.3 bridge, shared tick input recording, cancellation on screen/death/disconnect |
| Native platform builds | CI matrix configured | Linux x86-64, Windows x86-64, macOS ARM64; real-client comparison currently Linux only |
| Inventory and screens | Missing | Screen model, revision-aware selectors and clicks |
| Text input / books / signs / dialogs | Missing | Version-specific input adapters |
| Crafting / trading / building / combat | Missing | Complete low-level inputs before helpers |
| Settings / keybindings / lifecycle | Missing | Full-client and launcher adapters |
| Mods / custom screens | Missing | Capability discovery and extension contracts |
| Humanisation beyond camera rate | Missing | Cursor, typing, pacing policies with reproducible tests |
| External API and SDKs | Design pending | Transport and serialization measured before selection |
| Multi-agent worlds / training | Missing | Working single player and controlled server harness |
| Launcher instances / updates | Missing | Version manifests, isolated profiles, rollback |
| Memory / performance claims | Unmeasured | Real gameplay workload and competitor baselines |

No supported gameplay version is claimed by this preview. The scheduler's button states are tick snapshots: the final write to a button in one tick wins. Sub-tick click edges, text and GUI operations need a separate ordered-event contract; they must not be collapsed into these snapshots.

The native prototype is intentionally restricted to a controlled flat-world movement fixture. See [PLAYER.md](PLAYER.md) for its unsupported mechanics and evidence requirements. It is not ready for arbitrary multiplayer servers.
