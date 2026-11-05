# Sawbyte Desktop Launcher

Official cross-platform desktop launcher, instance supervisor, and AI pair-programming interface for the **Sawbyte** Minecraft Java Edition automation platform and native client.

## Core Capabilities

- **Zero-JVM & Native Integration**: Directly configure, manage, and launch the pure-Rust standalone client (`sawbyte-play`) with sub-12 MB RSS and locked 20.00 TPS physics.
- **Fabric Graphical Bridge**: Launch the official Fabric 26.3 graphical reference client equipped with `sawbyte-bridge-0.3.2.jar` for synchronized in-game HUD and telemetry.
- **Potato Minimal Visualizer**: Direct toggle for the pure-Rust software 3D raymarching engine and 360-degree top-down mob radar with zero GPU dependencies.
- **Integrated Mod Store**: Full native search and one-click installation from Modrinth and CurseForge with automated dependency resolution (Fabric API, Sodium, etc.).
- **Account Management**: Seamless support for Microsoft OAuth login and deterministic offline/cracked UUID profiles.
- **Autonomous AI Hub**: Integrated Model Context Protocol (MCP) server daemon supervisor (`skills/sawbyte/SKILL.md`) for pairing Claude, Gemini, and Antigravity agents.
- **Deep Dark Obsidian Theme**: Authentic dark styling (`#07090e`, `#0f141c`) with cyan diamond (`#38bdf8`) and emerald (`#22c55e`) accents, acrylic glassmorphism, and backgroundless Sawbyte brand icons.

## Workspace Layout

```
launcher/
├── xmcl-electron-app/    # Electron main process, window management, and native packaging
├── xmcl-keystone-ui/      # Vue 3 + Vite renderer UI, views, components, and deep dark theme
├── xmcl-runtime/          # Background Node.js services for instances, launch, and mods
├── xmcl-runtime-api/      # Service interface contracts and type definitions
└── packages/              # Modular utilities (@xmcl/installer, @xmcl/client, @xmcl/model, etc.)
```

## Quick Start

```bash
# Using bun or pnpm
cd launcher
pnpm install

# Run the renderer in development mode
pnpm run --prefix=xmcl-keystone-ui dev

# Run the desktop Electron app
pnpm run --prefix=xmcl-electron-app dev
```
