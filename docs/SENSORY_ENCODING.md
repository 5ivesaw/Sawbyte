# Sawbyte Sensory Vision v2 & `SVPE-1` Encoding Specification

Sawbyte v0.5.1 bridges the gap between human graphical vision and headless AI perception through **"Seeing the Game with Words v2"** (`inspect.sensory` / `see`) and the **Sawbyte Voxel Perception Encoding (`SVPE-1`)** standard.

---

## 1. Architecture Overview (10-Section Sensory Report)

When an AI agent, MCP client, or TCP controller invokes `inspect.sensory` (or `see`, `observe.words`, `inspect.vision`), Sawbyte outputs a 10-section perception report:

1. **`[1. SELF, POSE & VITALS]`**: Sub-voxel coordinates `(x, y, z)`, block/chunk coordinates, velocity (`m/s`), cardinal compass heading, ground/collision state, HP, hunger, saturation, XP, and game mode.
2. **`[2. CROSSHAIR & 360-DEGREE SPATIAL ENVIRONMENT]`**: Exact block or entity under the crosshair plus 6-direction horizontal raycasts (`Ahead 0°`, `±45°`, `±90°`, `180°`) detecting walls, 1-block jumpable steps, and void/drop ledges up to 12m out.
3. **`[3. INVENTORY & HOTBAR (46 SLOTS)]`**: Active hotbar slot, held item, and every populated slot (`CraftOut`, `Craft[0..3]`, `Armor[0..3]`, `MainInv[0..26]`, `Hotbar[0..8]`, `Offhand`) with item ID, count, custom `display_name`, and `lore` lines.
4. **`[4. OPEN GUI CONTAINER SCREEN]`**: Window ID, state ID, title, and `[Row, Col]` coordinates with custom names and lore for every populated container slot.
5. **`[5. NEARBY ENTITIES, PLAYERS & HOLOGRAM NPCs]`**: Distance-sorted list of entities with stacked ArmorStand/TextDisplay hologram lines (`entity.find` / `npc.interact`) and relative bearing angles.
6. **`[6. FIRST-PERSON RETINAL VIEWPORT (24x11 FOV 72°x44°) & DEPTH BUFFER]`**: A first-person 24×11 ASCII camera frustum screen showing sky (` `), ground (`_`), walls by distance tier (`#` `<3m`, `=` `3..8m`, `-` `8..14m`, `:` `>14m`), water (`~`), lava (`!`), climbables (`+`), entities (`@`/`M`), and crosshair center (`+` at col 12, row 5), paired with `SVPE1:DEPTH24x11`.
7. **`[7. COMPACT SVPE-1 SPATIAL ENCODINGS]`**: Lossless, token-efficient `SVPE1:RLE3D` (3D voxel volume) and `SVPE1:HD25D` (2.5D surface elevation delta & hazard grid) streams.
8. **`[8. COMPENSATORY PERCEPTION: THROUGH-BLOCK X-RAY, ESP & LONG-RANGE HORIZON]`**:
   - **Through-Block Penetrating Raycasts (`inspect.xray`)**: Multi-layer rays up to 16m through solid walls and floors revealing wall thickness, hidden rooms/cavities behind walls, underground lava hazards under the player's feet, and occluded entities.
   - **Through-Wall Entity Awareness (`ESP`) with `HumanisedPerceptionPolicy` Gate**: Every entity is tagged with its `VisibilityTier` (`DIRECT_FOV_LOS`, `PERIPHERAL_ACOUSTIC_LOS`, `MEMORY_RECALLED`, or `PRIVILEGED_THROUGH_WALL_HINT`), `WallBlocks` count, `Confidence`, and fair-play action guidance.
   - **Long-Range 8-Sector Horizon Scanner (`64m`, `inspect.long_range`)**: Scans out to 64m across all 8 compass sectors to detect void gaps, distant BedWars/SkyWars islands, bridges, and elevation peaks.
9. **`[9. SMALL-MAP FULL-ARENA AWARENESS & PERSISTENT CROSS-RUN MEMORY]`**: Automatically detects small maps/arenas ($\le 49$ chunks) and outputs a complete `SVPE1:HD25D` arena map (`inspect.small_map`), plus cross-run persistent memory status (`SVMEM-1`).
10. **`[10. HUD TITLES, ACTIONBAR & RECENT CHAT]`**: Active screen title, subtitle, actionbar text, and recent chat stream.

---

## 2. `SVPE-1` (Sawbyte Voxel Perception Encoding v1) Specification

To avoid multi-megabyte JSON voxel dumps while keeping outputs 100% interpretable by LLMs and scripts, `SVPE-1` uses a strict rule:
- **Digits `0..9` are reserved exclusively for Run-Length Encoding (`<count><symbol>`) repeat counts** (when `<count>` is omitted, the count is `1`).
- **All voxel, height, and depth values use non-digit ASCII symbols**, so RLE parsing is unambiguous and requires ~10 lines of code in any language.

### 2.1 `SVPE1:RLE3D` (3D Volume Run-Length Palette Stream)

Format:
```text
SVPE1:RLE3D|origin=(ox,oy,oz)|dims=(dx,dy,dz)|order=YZX|legend={...}|<layer_0>/<layer_1>/.../<layer_dy-1>
```
- **Traversal Order**: `Y` (bottom-to-top) $\to$ `Z` (north-to-south) $\to$ `X` (west-to-east). Each `/`-separated segment encodes one horizontal `dx * dz` Y-slice.
- **Layer Deduplication**: If Y-slice `y` is identical to Y-slice `y - 1`, it is emitted as `@dup`.
- **Symbol Alphabet**:
  - `A`: Air / open space
  - `S`: Solid full block
  - `W`: Water
  - `L`: Lava / fire hazard
  - `C`: Climbable (ladder, vine, scaffolding)
  - `h`: Bottom slab (`0.5` block step)
  - `t`: Top slab
  - `s`: Stairs
  - `f`: Fence / wall (`1.5` block collision height)
  - `c`: Carpet (`0.0625` height)
  - `I`: Ice (`slipperiness = 0.98`)
  - `O`: Special / POI block state
  - `U`: Unloaded chunk

### 2.2 `SVPE1:HD25D` (2.5D Surface Elevation Delta & Hazard Grid)

Format:
```text
SVPE1:HD25D|center=(px,py,pz)|radius=R|legend={=:level,A..Z:+1..+26_up,a..z:-1..-26_down,~:water,!:lava,_:void_drop,?:unloaded}|<row_0>/<row_1>/...
```
- Each `/`-separated row encodes `2*R + 1` columns along `X` (west-to-east) at `Z = pz - R .. pz + R`.
- **Symbol Alphabet**:
  - `=`: Surface block is at the exact same elevation (`Δy = 0`) as the block under the player's feet (`py`).
  - `A` .. `Z`: Surface is `+1` .. `+26` blocks higher than `py` (`A = +1` jumpable step, `B = +2` wall, etc.).
  - `a` .. `z`: Surface is `-1` .. `-26` blocks lower than `py` (`a = -1`, `b = -2`, `c = -3` safe drop, `d..z` fall damage risk).
  - `~`: Water surface.
  - `!`: Lava surface hazard.
  - `_`: Void / bottomless drop (`> 26` blocks down or open air to void).
  - `?`: Unloaded chunk.

### 2.3 `SVPE1:DEPTH24x11` (First-Person Retinal Depth Buffer)

Format:
```text
SVPE1:DEPTH24x11|legend={a..x:1..24m}|<rle_264_cells>
```
- Encodes the `24 * 11 = 264` cells of the first-person viewport in row-major order, where `a = 1m`, `b = 2m`, ..., `x = 24m+`.

### 2.4 `SVPE1:DELTA` (Temporal Differential Tick Stream)

Format:
```text
SVPE1:DELTA|tick=T|pos=(x,y,z)|dpos=(dx,dy,dz)|dyaw=dY|dpitch=dP|dhp=dH|entities=N|new_chat=[...]
```
- Invoked via `inspect.sensory.delta` (or `svpe.delta`), emitting only state deltas since the previous observation (~92% smaller than a full frame).

---

## 3. Decoding `SVPE-1` in Python & TypeScript

Both the Sawbyte Python SDK (`sdks/python/sawbyte`) and TypeScript SDK (`sdks/typescript`) include built-in decoders:

```python
from sawbyte import SawbyteLiveTcpClient, decode_svpe1_rle3d, decode_svpe1_hd25d

with SawbyteLiveTcpClient("127.0.0.1:25580") as client:
    rle3d_str = client.inspect_svpe(rx=8, ry=4, rz=8)
    vol = decode_svpe1_rle3d(rle3d_str)
    print("Origin:", vol["origin"], "Dims:", vol["dims"])

    hd_str = client.inspect_svpe_height(radius=12)
    surface = decode_svpe1_hd25d(hd_str)
    print("Center elevation delta:", surface["height_deltas"][12][12])
```

---

## 4. Humanisation & Fair-Play Perception Gate (`HumanisedPerceptionPolicy`)

Sawbyte's compensatory perception (through-wall ESP, through-block X-ray, and 64m horizon scans) is governed by `HumanisedPerceptionPolicy` (`humanisation.perception [on|off|status]`, **enabled by default**):

- **Strict Vanilla Reach**: Entity interaction reach is locked to `3.0` blocks (`VANILLA_ENTITY_INTERACT_REACH`) and block reach is locked to `4.5` blocks (`VANILLA_BLOCK_INTERACT_REACH`). Attacks or block placements through solid walls are rejected before any packet is sent.
- **4 Epistemic Visibility Tiers**:
  1. `DIRECT_FOV_LOS` (`confidence = 1.00`): Unobstructed line-of-sight inside the player's camera frustum (`75°x55°`). Immediate aiming and interaction allowed.
  2. `PERIPHERAL_ACOUSTIC_LOS` (`confidence = 0.78`): Unobstructed line-of-sight or within 16m footstep/audio radius, outside current camera frustum. Smooth camera rotation + 3-tick visual acquisition reaction delay applied.
  3. `MEMORY_RECALLED` (`confidence = 0.65`): Previously observed with direct line-of-sight earlier in the session (`direct_los_seen_entities`) and now behind cover. Guides the agent to navigate to the last known vantage point.
  4. `PRIVILEGED_THROUGH_WALL_HINT` (`confidence = 0.18`): Detected only via through-wall ESP with no prior line-of-sight. Treated strictly as a supplementary directional hint (`sector_hint`, e.g., `"North-East (~8m)"`)—agents must never pre-aim or track through walls.

---

## 5. Persistent Cross-Run World Memory (`SVMEM-1`)

Enabled by default in `~/.sawbyte/memory/<world_id>.svmem.json`:
- Automatically caches compact `SVPE1:HD25D` 16×16 surface tiles for up to 512 explored chunks and up to 256 deduplicated spatial landmarks (NPCs with hologram text, beds, chests, portals).
- Controlled via TCP/MCP commands:
  - `memory.status`: Inspect run count, total ticks observed, explored chunks, and landmark count.
  - `memory.save [path]` / `memory.load [path]`: Save or reload persistent world memory on disk.
  - `memory.landmarks` / `memory.mark <kind> <label>`: Query or record persistent spatial landmarks.
  - `memory.enable [on|off]` / `memory.clear`: Toggle or reset memory.
