# Foundation decisions

## Typed core before transport

The user's JSON example describes expressiveness, not a required format. The core uses Rust enums/structs and direct calls. An external transport will be chosen using workload measurements, ecosystem fit and debugging needs. JSON remains a possible control-plane format, not the execution engine. No workflow language is committed yet.

## Small independent status slice

Implement the status handshake as the first real network operation. This exercises framing, deadlines and CLI packaging without pretending authentication or physics are complete. Wire sequencing was checked against the established Java handshake/status design; test fixtures assert packet structure and fragmentation. This does not certify any gameplay version or substitute for a vanilla-server integration test.

## Caller-owned ticks and controls

The scheduler never sleeps or creates threads. Its owner supplies consecutive tick numbers. Missing a tick is an explicit error because silently coalescing inputs can erase press/release transitions. Button state persists until changed or cancelled. A new camera intent replaces unfinished camera motion. The default policy caps camera-vector magnitude to 12 degrees per tick; it is configurable and is not a scientific model of human movement.

Pending actions use a bounded FIFO. Submission is ordered and atomic. Failures clear pending actions, attempt control release, and halt the scheduler until explicit cancellation succeeds. A partially applied action is never automatically retried. Drop attempts release as a best effort; callers must explicitly cancel to observe errors. Killing a process cannot guarantee remote releases; connected backends will need a watchdog.

## Backends and breadth

The trait is the first input execution boundary, not the final screen/observation API. Native and graphical gameplay adapters are still to be built. Keep discrete events (click edges, text edits, UI actions) separate from continuous button snapshots. Extend capability discovery rather than treating two current capability variants as the entire product.

## CI-only validation

Use GitHub-hosted machines to compile, test, format and package. Pin Rust and action revisions; no credentials in builds. No external crates means locked offline Cargo execution works once the toolchain is installed. Artifacts include checksums, license and current capability ledger. Toolchain setup itself requires network access.

## Next engineering slice

1. Choose and audit the 26.3 login/world foundation, comparing selective Azalea reuse to custom implementation; do not adopt a dependency based on README claims alone.
2. Establish the full-client bridge and trace comparison harness on a verified loader/version combination.
3. Implement authenticated connection, configuration/play lifecycle and world observations, then drive a real player's movement through the scheduler.
4. Extend the ordered-event contract for all screens and player interactions, then select the external API using representative workloads.

Stage 0 is not complete until a real join/move/stop trace can be compared against the reference client.
