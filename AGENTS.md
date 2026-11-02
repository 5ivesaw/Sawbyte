# Development workflow

- Product priorities: aggressive runtime stripping and optimisation; comprehensive programmable player and GUI operations; configurable humanisation enabled by default; and a shared automation framework for both the native headless runtime and normal graphical Minecraft.
- Treat user examples as illustrations, not an exhaustive feature list. Proactively research related tools and design missing capabilities. Use PLAN.md for the architecture and revised priorities.
- Complete programmability is a governing requirement: every player operation in a supported version/backend must have an automation route, with composable low-level primitives beneath helpers. Maintain comprehensive coverage and track missing support as unfinished work. Do not narrow scope to user examples or claim universal support before it is verified.
- User-suggested technologies (including JSON) are examples, not mandates. Select languages, transports and formats using engineering evidence across efficiency, memory, latency, usability, compatibility and maintenance. Document assumptions and unmeasured claims.
- Measure performance and behavioural parity; do not represent aspirational memory reductions, universal compatibility, or superiority over competitors as established facts.

- Keep this project structured as a complete, GitHub-ready repository, including source, configuration, documentation, and GitHub Actions workflows.
- Build, compile, package, and produce launchers, clients, binaries, installers, and release artifacts through GitHub Actions. Do not run these jobs on the user's machine unless the user explicitly requests an exception.
- Put automated test and validation jobs in GitHub Actions too. Local work should focus on editing and reviewing repository files; do not install build toolchains or run resource-heavy jobs locally.
- Make workflow outputs available as downloadable Actions artifacts, or GitHub Release assets when a release is requested.
- Apply this approach to future components and related projects as well; it is the user's general development preference.
