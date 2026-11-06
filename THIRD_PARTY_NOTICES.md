# Third-Party Notices & Open-Source Credits (Sawbyte v0.5.0)

Sawbyte v0.5.0 is licensed under the MIT License. It incorporates and refactors architectural components, simulation primitives, agent control vocabularies, and assimilated crates from the following open-source projects:

## 1. Horizon (`horizon-rust/horizon` & `0CYBER/horizon-free`)
- **Repository:** https://github.com/horizon-rust/horizon (and offline fork https://github.com/0CYBER/horizon-free)
- **Authors:** Ludovic Marie (`@ludovic111`) and the 238+ Horizon contributors
- **License:** MIT License
- **Usage in Sawbyte v0.5.0:**
  - Assimilated and refactored into native `sawbyte-*` crates (`sawbyte-client`, `sawbyte-ctl`, `sawbyte-prepare`, `sawbyte-sim-core`, `sawbyte-world`, `sawbyte-items`, `sawbyte-command`, `sawbyte-assets`, `sawbyte-server`, etc.) via `tools/assimilate_horizon_into_sawbyte.py`.
  - 65-command `sawbyte-ctl` (formerly `horizon-ctl`) agent control & MCP vocabulary and differential Java-Oracle vs Rust-Probe parity harness architecture (`tools/port_ledger.py`).

## 2. wgpu / naga (`gfx-rs`)
- **Authors:** The `gfx-rs` developers
- **License:** MIT OR Apache-2.0

## 3. Tauri (`tauri-apps/tauri`)
- **Authors:** Tauri Programme within The Commons Conservancy
- **License:** MIT OR Apache-2.0
