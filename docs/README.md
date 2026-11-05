# Sawbyte — Minecraft 26.3 Portal (WinRAR Archive Edition)

## What Was Upgraded
1. **Zero Local Media / 100% Public GitHub Fetching**:
   - All custom SVG lookalikes have been completely removed.
   - Official Minecraft 26.3 / 1.21+ item, block, and HUD PNGs are dynamically fetched from public GitHub (`InventivetalentDev/minecraft-assets` + `cdn.jsdelivr.net/gh` mirrors) with automatic branch discovery and multi-CDN failover.
   - Official Minecraft `.ogg` sound effects (`records/pigstep.ogg`, `mob/pig/say1..3.ogg`, `item/totem/use_totem.ogg`, `random/orb.ogg`, `random/click.ogg`, `random/levelup.ogg`, `note/bell.ogg`, `random/eat1..3.ogg`, `fireworks/launch1.ogg`) are fetched directly from public GitHub + dynamically resolved via the Minecraft Asset Index JSON on GitHub.
2. **Actual Minecraft Font (Remote GitHub `@font-face` + `FontFace` API)**:
   - Loads `Minecraft.otf` and `Minecraft-Bold.otf` directly from `IdreesInc/Minecraft-Font` and `Minecraft.ttf` from `Magisk-Modules-Alt-Repo/MinecraftFont` on GitHub.
3. **Background Low-Volume Pigstep Jukebox**:
   - Automatically starts with **Lena Raine — Pigstep** at low background volume (`18%`), with an interactive Jukebox dock to pause/play, adjust volume (`0%–100%`), or switch discs (`Pigstep`, `otherside`, `Relic`, `cat`, `stal`).
4. **Technoblade Memorial Soundboard & Relics**:
   - Interactive Technoblade soundboard featuring authentic Minecraft Pig sounds (`mob/pig/say1..3.ogg`), Totem of Undying (`use_totem.ogg`), Skyblock Potato War counter (`eat.ogg` + `orb.ogg`), Channel Member Bell (`bell.ogg`), Skyblock Firework Salute (`launch1.ogg`), and Tribute Poppy (`[o7]`).
5. **100% Auto-Fetched GitHub Releases & Links**:
   - Queries `https://api.github.com/repos/5ivesaw/Sawbyte/releases/latest`, `/releases`, and `/repos/5ivesaw/Sawbyte` at runtime to populate every platform binary link, file size, download count, release tag, and live asset manifest table—zero hardcoded release URLs.

## Files Included
- `index.html` — Main portal markup
- `style.css` — Authentic Minecraft 26.3 UI stylesheet, `@font-face` declarations, enchanted glint, advancement toasts, and tooltips
- `app.js` — Dynamic GitHub asset/audio resolver, Pigstep jukebox, Technoblade soundboard, 3D textured voxel renderer, and GitHub Releases API client
- `README.md` — Documentation
