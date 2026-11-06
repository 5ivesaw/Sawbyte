# Sawbyte Desktop Launcher

Official lightweight, cross-platform desktop launcher, on-demand GitHub Releases edition installer, pure-Rust 3D/Radar visualizer, and 100% free AI agent / MCP hub for **Sawbyte** (Minecraft Java 26.3, Protocol 777).

## Key Highlights

- **100% Pure Sawbyte Architecture**: Built specifically for Sawbyte with zero third-party launcher bloat, zero subscriptions, and zero paywalls.
- **Sleek Creamy Luxe & Obsidian UI**: Proportional, high-contrast editorial design with one-click switching between **Cream Luxe** (`#F6F0E4` / `#2B6E5B` / `#C47A2C`) and **Obsidian Dark**, plus adaptive eco frame-rate throttling for low-end devices.
- **On-Demand GitHub Releases Edition Installer**: The launcher stays featherlight and fetches editions directly from `https://github.com/5ivesaw/Sawbyte/releases/latest` into `~/.sawbyte/bin`:
  - **Headless Native Engine (`sawbyte-play`)**: ~8.2 MB RAM, locked 20.0 TPS sub-voxel physics (`3.41e-7` block parity), TCP control port `25580`.
  - **Potato Software 3D & Radar Edition (`visualizer`)**: ~10.4 MB RAM pure-Rust raymarcher & 360° radar with zero GPU/JVM dependencies.
  - **Sawbyte CLI & MCP Daemon (`sawbyte`)**: Persistent instance supervisor, `.sbif` input tape generator, and 10-tool MCP stdio server.
  - **Graphical Fabric 26.3 Bridge (`sawbyte-bridge-0.3.3.jar`)**: Reference bridge mod for syncing/spectating headless bots inside the regular Java client.
  - **Minecraft 26.3 Block Profile (`block-profile.json`)**: Authoritative protocol 777 collision and hardness profile.
- **Built-In Free Sawbyte AI Agent & MCP Hub**: Dispatch autonomous survival workflows, test all 10 MCP tools live over JSON-RPC 2.0, or copy the ready-to-use `mcpServers` config for Claude, Gemini, Cursor, Antigravity, or local Ollama.

## Quick Start

```bash
cd launcher
pnpm install
pnpm run build:renderer
pnpm run dev
```
