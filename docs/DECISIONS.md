# Foundation decisions

## Typed core before transport

The user's JSON example describes expressiveness, not a required format. The core uses Rust enums/structs and direct calls. An external transport will be chosen using workload measurements, ecosystem fit and debugging needs. JSON remains a possible control-plane format, not the execution engine. No workflow language is committed yet.

## Small independent status slice

Implement the status handshake as the first real network operation. This exercises framing, deadlines and CLI packaging without pretending authentication or physics are complete. Wire sequencing was checked against the established Java handshake/status design; test fixtures assert packet structure and fragmentation. This does not certify any gameplay version or substitute for a vanilla-server integration test.

## Caller-owned ticks and controls

The scheduler never sleeps or creates threads. Its owner supplies consecutive tick numbers. Missing a tick is an explicit error because silently coalescing inputs can erase press/release transitions. Button state persists until changed or cancelled. A new camera intent replaces unfinished camera motion. The default policy caps camera-vector magnitude to 12 degrees per tick; it is configurable and is not a scientific model of human movement.

Pending actions use a bounded FIFO. Submission is ordered and atomic. Failures clear pending actions, attempt control release, and halt the scheduler until explicit cancellation succeeds. A partially applied action is never automatically retried. Drop attempts release as a best effort; callers must explicitly cancel to observe errors. Killing a process cannot guarantee remote releases; connected backends will need a watchdog.

## Backends and breadth

The trait is the first input execution boundary, not the final screen/observation API. Experimental native and graphical adapters now consume the same bounded binary input recording. Keep discrete events (click edges, text edits, UI actions) separate from continuous button snapshots. Extend capability discovery rather than treating two current capability variants as the entire product.

## CI-only validation

Use GitHub-hosted machines to compile, test, format and package. Pin Rust and action revisions; no credentials in builds. No external crates means locked offline Cargo execution works once the toolchain is installed. Artifacts include checksums, license and current capability ledger. Toolchain setup itself requires network access.

## Stage 0 completion and next engineering slice

Stage 0's exit criterion—reproducing and comparing a real 26.3 join/move/turn/jump/stop trace between the independent native Rust runtime (`sawbyte-play`) and the reference Fabric 26.3 client (`sawbyte-bridge`) against a live vanilla 26.3 server in GitHub Actions—is now met (`max_position_error_blocks`: `3.41e-7`, `rms_position_error_blocks`: `2.29e-7`, `0` ground-state mismatches, `4,380 KiB` sampled native peak RSS).

Key protocol fact verified during Stage 0: in Minecraft 26.3 (protocol `777`), `ServerboundAcceptTeleportationPacket` (`packet 0`) carries `(id, x, y, z, yRot, xRot)` and sets `ServerGamePacketListenerImpl.receivedPositionThisTick = true`, which is only cleared by `ServerboundClientTickEndPacket` (`packet 13`). Sending an extra `ServerboundMovePlayerPosRot` (`packet 31`) in the same tick immediately triggers `multiplayer.disconnect.invalid_player_movement`.

Next engineering priorities (Stage 1):

1. Broaden native world-state and motion mechanics beyond ordinary full cubes (block updates in loaded chunks, sprinting, sneaking, stepping, stairs/slabs, fluids, attributes/effects, and target raycasting).
2. Validate live online authentication and session lifecycle recovery/reconnection.
3. Extend the ordered-event contract for discrete sub-tick actions, inventory transactions, and screens across both native and full-client backends, then benchmark candidate external API transports.
