# Capability ledger

The product requirement is full player programmability. This ledger describes implemented coverage, not the final scope. Every missing capability is remaining work. Future version-specific inventories must be derived from actual client interfaces and validated against the real client.

| Area | Current status | Evidence / next gate |
|---|---|---|
| Typed scheduling | Implemented, trace backend only | Atomic queues, cancellation, monotonic tick tests |
| Camera policy | Bounded motion and direct mode | Total motion and custom-policy validation tests |
| Network status / ping | Implemented | Loopback TCP fixture with handshake and pong verification |
| Packet bounds / deadline | Implemented for status | Invalid frame, VarInt, stalled-peer tests |
| Native login / accounts | Missing | Auth, encryption, configuration state |
| Native player movement | Missing | World cache, version rules, reference-client comparison |
| Full-client bridge | Missing | Loader/version choice and observable input contract |
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
