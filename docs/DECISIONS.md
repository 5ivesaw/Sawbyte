# Current engineering decisions

## Product boundary
[The user’s product vision](VISION.md) governs this document. Sawbyte is an independent Rust Minecraft Java client, a general automation engine and a real launcher. User examples describe breadth; they do not define special features. Native operation is primary; normal-client integration remains an optional goal.

## Language and interfaces
Rust remains the natural implementation language for the small native runtime and shared engine. Python and TypeScript are optional control adapters, not required runtime dependencies. No custom workflow language or JSON execution format is committed. First build a complete typed, observable control boundary; choose serialization and scripting formats from measured needs.

## Correctness before policy
All controllers must share one rule-validating execution path. Humanised behaviour is the default where applicable; direct mode disables optional shaping only. Version, state, reach, collision, target, timing and inventory constraints still apply. Report queued actions separately from verified outcomes. Client-side prediction and synchronized memory are normal parts of a native client; invented production state/results are not.

## Backends and version support
The standalone client is the primary backend. The Fabric bridge is optional graphical integration and a useful reference harness. A profile label does not establish a working adapter. Every claimed version/backend requires a capability list and independent tests.

## Performance
Measure representative CPU, memory, power, allocations, reliability and multi-client loads. The existing movement comparison and small RSS samples are starting evidence, not general limits or proof of superiority. Preserve required behaviour while optimizing.

## GitHub-only validation
All builds, tests, formatting, analysis and packaging run through GitHub Actions. Keep scenario inputs, generated reports and limits reviewable. A passing build is not a gameplay result; a passing workflow must assert the outcome it claims.

## Current verified fact
At commit `d70a744`, Linux CI matches the normal client on one 160-tick 26.3 flat-world movement recording. This does not establish broad physics, interactions, screens, online accounts, training, other versions or desktop installation. See [the whole-repository audit](AUDIT-2026-11-03.md).
