# Sawbyte continuation and resource handoff

Updated 2026-11-04. Read [VISION.md](VISION.md), [PLAN.md](../PLAN.md), [AGENTS.md](../AGENTS.md), and this file before building. The user's goal is the complete standalone automation client and launcher, not a demo or a predefined server workflow. Keep working through the plan without asking to start each phase.

## Current work and evidence

The baseline before the latest pending changes is commit `58d01b1`. Its [cross-platform Build and verify run](https://github.com/5ivesaw/Sawbyte/actions/runs/41798534414) passed. Its [real-player run](https://github.com/5ivesaw/Sawbyte/actions/runs/41798534961) was still running when this was written. Check its actual result and use the current `git status`, commit and workflow runs as authoritative.

Recent work adds mining and placement observation, click revision tracking, incomplete-container rejection, system chat/event inspection and complete bounded NBT arrays. These are partial implementations, not proof of general interaction coverage. A server revision or block update proves only the state observed, and no change must be labelled successful solely because it was sent. Validate all success, cancellation, timeout, stale state, disconnect and concurrent-input paths against a real server and the reference client.

The changes accompanying this handoff include atomic account lock acquisition, child cleanup on PID/state persistence failure, a missing-profile error, screen transition handling, corrected development status and workflow dispatch publishing restrictions. Confirm these changes and CI results rather than assuming they pass. Both release workflows still publish independently on tags, so unify release gating before making a release. Avoid force-updating existing release tags.

The built-in 26.3 profile still contains only two states. `block-profile.json` in the worktree is untracked user data generated from heuristic classifications. Do not overwrite, commit, or treat it as authoritative without provenance and validation. Obtain complete version data and validate collision shapes, hardness, protocol and item components against the reference implementation. The Minecraft server's normal block report alone does not prove those properties. Broad player mechanics, launcher GUI and supervision, SDKs, multi-version support, real-server multi-bot tests, and complete performance evidence remain open.

## Usage and handoff

Codex has a `get_usage_limits` app tool when available. Check both the five-hour and weekly windows at useful work checkpoints, and before consuming a large remaining allowance. The tool reports percent **used**, so remaining is `100 - usedPercent`. Limits are shared across the account and can change. As of this handoff the five-hour window had 48% remaining and the weekly window 13% remaining. Re-read instead of assuming these numbers are current.

When the smaller remaining window approaches 10%, update this file with the exact current commit, dirty files, running workflow URLs and next concrete tasks. Leave enough time to capture a clean handoff. If limits prevent continuation, do not call the objective complete. Gemini cannot automatically see this Codex chat unless the user gives it the handoff or an integration explicitly provides it. Another Codex thread's handoff tool does not transfer this thread to Gemini.

Build, test, format, run servers/clients, and benchmark only in GitHub Actions. Preserve `assets/sawbyte.png` and the original untracked `sawbyte icon.png`. Never commit credentials, Minecraft account tokens or private server addresses. User examples are illustrations of general programmability.

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
