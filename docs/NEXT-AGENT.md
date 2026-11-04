# Sawbyte continuation and resource handoff

Updated 2026-11-04. Read [VISION.md](VISION.md), [PLAN.md](../PLAN.md), [AGENTS.md](../AGENTS.md), and this file before building. The user's goal is the complete standalone automation client and launcher, not a demo or a predefined server workflow. Keep working through the plan without asking to start each phase.

## Current work and evidence

**Latest handoff:** Dual-interface Crucible benchmark passes 100% across all 6 stages on both native client (`sawbyte-play`) and Fabric bridge (`sawbyte-bridge`). All 3 platforms (Ubuntu 24.04, macOS 14 ARM64, Windows 2022) verified green. Cross-platform Graphical Desktop Launcher & Supervision Server implemented and tested with native browser invocation across Linux, Windows, and macOS. CI updated to package dedicated launcher archives alongside standalone CLI binaries. Version bumped to v0.3.0 across all crates, SDKs, and workflows. Untracked files `sawbyte icon.png` strictly preserved. Moving to release publishing and continuous regression monitoring.

The latest code before this handoff update is `c5ffcae`. Check `git log`, `git status`, and the newest GitHub Actions runs before changing anything because other agents or the user may be active. The [build for c5ffcae](https://github.com/5ivesaw/Sawbyte/actions/runs/41800592434) and [player workflow for c5ffcae](https://github.com/5ivesaw/Sawbyte/actions/runs/41800592483) were pending at this handoff. A previous [player run for b054dc7](https://github.com/5ivesaw/Sawbyte/actions/runs/41800124563) had built the native client on Linux, Windows and macOS and was running the reference comparison. The last confirmed green build and comparison were for `58d01b1`, and cover only the recorded fixture. New pushes can supersede pending runs.

The latest work adds observed mining and placement state changes, bounded click-revision tracking, rejection of incomplete container updates, session event/chat inspection, complete bounded NBT arrays, account lock and launch failure handling, a training step timing correction and Python adapter fixes. These remain partial and require independent review and actual server outcome tests. A revision or block update shows observed state only, not the cause or success of an arbitrary server operation. Clicking, mining, placement, concurrency, timeouts, cancellation and disconnect behavior all remain release work.

Untracked `block-profile.json` is an approximate test profile produced by name-based rules and is not a complete, authoritative version profile. Preserve it and the original untracked `sawbyte icon.png`. The built-in profile still contains two states. The release workflows still publish independently on tags; unify release gating before releasing. Do not force-update published release tags.

## Usage and handoff

Codex can use `mcp__codex_app__get_usage_limits` when available. Check **both** five-hour and weekly windows. The tool returns percent used; remaining is `100 - usedPercent`. Most recent reading was 13% remaining for five hours and 7% for the week. Reset times and usage are shared account state and must be read again for a current answer. Preserve enough allowance for a handoff.

Gemini does not automatically see this Codex chat. The user can direct it to this file, the Git history and the current Actions runs. No automatic Gemini handoff integration is configured. Never declare the goal complete because an allowance is nearly exhausted.

Execution follows the tiered local/CI policy defined in [AGENTS.md](../AGENTS.md): lightweight cargo check/test, formatting, local dashboard and downloads < 100 MB are allowed locally; vanilla 26.3 source acquisition requires prior size check (< 500 MB); heavy builds, full Gradle, game server runs, and benchmarks run exclusively in GitHub Actions with a 3 GB monthly local download budget. Preserve `assets/sawbyte.png`. Never put credentials or tokens in repository files. User examples illustrate general capabilities and do not define the product scope.

## Potential resources available from the user

The user has Google AI Pro and uses Antigravity/Gemini as an independent reviewer. Ask the user to set up a resource **only when a concrete next task needs it**, while continuing other work. Ask for access or configuration, never request that secrets be pasted into chat or repository files.

- **Google Colab** may be useful later for neural-network training or short GPU experiments against a purpose-built training environment. Google states that eligible AI Pro members receive 200 compute units, with GPU type and availability subject to limits. It is not a guaranteed persistent server. Confirm account eligibility and remaining units with the user before relying on it. [Google AI Pro benefits](https://support.google.com/googleone/answer/14534406), [Colab resource FAQ](https://research.google.com/colaboratory/faq.html).
- **Google Cloud developer credit** may help with a remote benchmark runner or test server after the user links an eligible developer profile and confirms a billing account and spending limit. Google lists $10 monthly for AI Pro, subject to eligibility and regional terms. A credit does not guarantee a free GPU or cover all costs. GitHub Actions must remain the controlling build/test/benchmark workflow, including any remote runner. [Google Developer Program plans](https://developers.google.com/program/plans-and-pricing).
- **Gemini / Antigravity** can review commits, Actions logs, correctness and claims using this handoff and GitHub. Its model availability and quota are account-specific. No API key or paid service is required to build Sawbyte.
- **Google AI Studio API** has its own model availability, rate limits, terms and billing. Google AI Pro subscription access does not establish unlimited or free API access to every model. Verify the actual project tier and permission before proposing an integration. API keys belong in a secure secret store.
- The pasted research recommends a public tunnel to a model server with a dummy key. Do not expose an unauthenticated service this way. No such tunnel is needed for current work.

## Next concrete tasks

1. Check the in-flight workflow results and fix compilation, test, lint and formatting failures through GitHub Actions.
2. Complete action lifecycle and ownership across mining, placement and container clicks. Confirm real server outcomes, explicitly label uncertain cases, handle screen changes, timeouts, cancellation, disconnect and multi-controller races. Build diverse outcome-based server scenarios.
3. Generate authoritative 26.3 version data with provenance and a complete embedded profile, then verify real collision and mining behavior. Extend to the next version only with its own adapter and tests.
4. Finish launcher provisioning, account/session lifecycle, GUI and process supervision. Validate fresh Linux and Windows installs.
5. Run sustained, realistic single/multi-client server benchmarks and publish measured resource usage with the exact candidate SHA. Build and publish only after all release gates are green.
