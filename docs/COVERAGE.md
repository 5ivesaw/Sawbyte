# Sawbyte 1:1 Vanilla Client Coverage Matrix

Extracted directly from the decompiled vanilla Minecraft 26.3 client source (`.cache/vanilla/26.3/client.jar`).
Every relevant client-side subsystem is audited against Sawbyte native implementation (`native/src`) and core engine (`crates/sawbyte-core`).

## Summary Metrics

- **Total Client Subsystems Audited:** 59
- **Ported & Parity-Tested:** 28 (47.5%)
- **Ported (Implemented):** 23 (39.0%)
- **Combined Implementation Coverage:** 51 / 59 (86.4%)
- **Partial / In Progress:** 8
- **Missing:** 0
- **Blocked:** 0

## Movement & Physics Modifiers

| Vanilla Client Class / System | Client-Side Vanilla Behavior | Engine Implementation Path | Status |
|---|---|---|---|
| `net.minecraft.world.level.block.Blocks#ICE` | Ice / Packed Ice / Blue Ice slipperiness (0.98 / 0.989) | `native/src/world.rs:slipperiness` | PORTED+PARITY-TESTED |
| `net.minecraft.world.level.block.Blocks#SLIME_BLOCK` | Slime block friction and bounce suppression | `native/src/world.rs` | PORTED+PARITY-TESTED |
| `net.minecraft.world.level.block.Blocks#SOUL_SAND` | Soul sand speed reduction (0.4x factor) | `native/src/world.rs` | PORTED+PARITY-TESTED |
| `net.minecraft.world.level.block.Blocks#HONEY_BLOCK` | Honey block friction, sliding and jump suppression | `native/src/world.rs` | PORTED+PARITY-TESTED |
| `net.minecraft.world.level.block.Blocks#COBWEB` | Cobweb movement factor (0.25x horiz, 0.05x vert) | `native/src/player.rs:337` | PORTED+PARITY-TESTED |
| `net.minecraft.world.level.block.LadderBlock` | Ladder climbing (0.2 vertical velocity upon jump/collision) | `native/src/player.rs:417` | PORTED+PARITY-TESTED |
| `net.minecraft.world.level.block.VineBlock` | Vine climbing collision and deceleration | `native/src/player.rs` | PORTED+PARITY-TESTED |
| `net.minecraft.world.level.block.ScaffoldingBlock` | Scaffolding ascent, descent, and bottom collision | `native/src/world.rs` | PORTED+PARITY-TESTED |
| `net.minecraft.world.level.material.WaterFluid` | Water drag (0.8 friction), buoyancy (+0.04), dolphin grace | `native/src/player.rs:252` | PORTED+PARITY-TESTED |
| `net.minecraft.world.level.material.LavaFluid` | Lava drag (0.5 friction) and vertical impulse | `native/src/player.rs:252` | PORTED+PARITY-TESTED |
| `net.minecraft.world.level.block.PowderSnowBlock` | Powder snow sinking and leather boots float prevention | `native/src/player.rs` | PARTIAL |
| `net.minecraft.world.level.block.BubbleColumnBlock` | Bubble column upward/downward velocity injection | `native/src/player.rs` | PARTIAL |
| `net.minecraft.client.player.LocalPlayer#aiStep` | Sneaking edge ledge clamping (cannot walk off cliffs) | `native/src/player.rs:344` | PORTED+PARITY-TESTED |
| `net.minecraft.client.player.LocalPlayer#aiStep` | Step-up height handling (0.6m auto-climb on solid obstacle) | `native/src/player.rs:388` | PORTED+PARITY-TESTED |
| `net.minecraft.world.entity.player.Player#knockback` | Explosion and damage knockback velocity impulse vector | `native/src/lib.rs:710` | PORTED+PARITY-TESTED |

## Vehicles & Mounts

| Vanilla Client Class / System | Client-Side Vanilla Behavior | Engine Implementation Path | Status |
|---|---|---|---|
| `net.minecraft.world.entity.vehicle.Boat` | Boat steering, paddle packets (minecraft:paddle_boat), forward velocity | `native/src/inventory.rs:1275` | PORTED |
| `net.minecraft.world.entity.vehicle.Minecart` | Minecart riding, rail following and unmount packets | `crates/sawbyte-core/src/events.rs` | PARTIAL |
| `net.minecraft.world.entity.animal.horse.Horse` | Horse riding, steering, jump strength charge, horse armor/inventory | `native/src/inventory.rs` | PARTIAL |
| `net.minecraft.world.entity.monster.Strider` | Strider lava steering, speed boost with warped fungus on a stick | `native/src/lib.rs` | PARTIAL |
| `net.minecraft.world.entity.animal.Pig` | Pig saddle riding and carrot on a stick boost steering | `native/src/lib.rs` | PARTIAL |
| `net.minecraft.client.player.LocalPlayer#startRiding` | Client vehicle dismount and steer packets (sb_steer_vehicle/player_input) | `native/src/inventory.rs` | PORTED |

## Menus & Container Handlers

| Vanilla Client Class / System | Client-Side Vanilla Behavior | Engine Implementation Path | Status |
|---|---|---|---|
| `net.minecraft.world.inventory.ChestMenu` | Single & Double Chest 27/54 slots, custom names, quick transfer | `crates/sawbyte-core/src/screen.rs:288` | PORTED+PARITY-TESTED |
| `net.minecraft.world.inventory.CraftingMenu` | 3x3 Crafting grid, recipe placement, result slot retrieval | `crates/sawbyte-core/src/screen.rs:308` | PORTED+PARITY-TESTED |
| `net.minecraft.world.inventory.AnvilMenu` | Anvil repair, rename text packet (sb_rename_item), xp cost check | `native/src/inventory.rs:1140` | PORTED |
| `net.minecraft.world.inventory.EnchantmentMenu` | Enchanting table 3-tier seed preview, button click packet | `native/src/inventory.rs:1147` | PORTED |
| `net.minecraft.world.inventory.BeaconMenu` | Beacon primary/secondary power selection (sb_set_beacon) | `native/src/inventory.rs:1185` | PORTED |
| `net.minecraft.world.inventory.MerchantMenu` | Villager trading recipe index selection (sb_select_trade) | `native/src/inventory.rs:1193` | PORTED |
| `net.minecraft.world.inventory.FurnaceMenu` | Furnace / Smoker / Blast Furnace cook time and fuel inspection | `crates/sawbyte-core/src/screen.rs:328` | PORTED |
| `net.minecraft.world.inventory.BrewingStandMenu` | Brewing stand 3 potion slots, ingredient, fuel gauge | `crates/sawbyte-core/src/screen.rs:338` | PORTED |
| `net.minecraft.world.inventory.GrindstoneMenu` | Grindstone disenchanting and repair outcome preview | `crates/sawbyte-core/src/screen.rs` | PORTED |
| `net.minecraft.world.inventory.SmithingMenu` | Smithing table template, base item, addition ingredient combiner | `crates/sawbyte-core/src/screen.rs` | PORTED |
| `net.minecraft.world.inventory.StonecutterMenu` | Stonecutter recipe button selection and single-input recipe output | `crates/sawbyte-core/src/screen.rs` | PORTED |
| `net.minecraft.world.inventory.LoomMenu` | Loom banner, dye, and pattern selection | `crates/sawbyte-core/src/screen.rs` | PORTED |
| `net.minecraft.world.inventory.CartographyTableMenu` | Cartography table map zooming, locking, and duplication | `crates/sawbyte-core/src/screen.rs` | PORTED |
| `net.minecraft.world.inventory.LecternMenu` | Lectern book page turning and take book packet | `native/src/inventory.rs` | PORTED |
| `net.minecraft.world.inventory.ShulkerBoxMenu` | Shulker box 27-slot container and nested bundle inspection | `crates/sawbyte-core/src/screen.rs` | PORTED |
| `net.minecraft.world.inventory.HorseInventoryMenu` | Horse/Donkey/Llama saddle, armor, and chest slots | `crates/sawbyte-core/src/screen.rs` | PORTED |
| `net.minecraft.world.inventory.InventoryMenu` | Player survival 2x2 crafting grid, 4 armor slots, offhand slot | `crates/sawbyte-core/src/screen.rs:374` | PORTED+PARITY-TESTED |

## Entities, Combat & Attributes

| Vanilla Client Class / System | Client-Side Vanilla Behavior | Engine Implementation Path | Status |
|---|---|---|---|
| `net.minecraft.world.entity.LivingEntity#attributes` | Attribute modifiers: movement speed, attack speed, reach, break speed | `native/src/lib.rs:728` | PORTED+PARITY-TESTED |
| `net.minecraft.world.entity.LivingEntity#effects` | Status effects: Speed, Slowness, Haste, Mining Fatigue, Jump Boost | `native/src/player.rs:306` | PORTED+PARITY-TESTED |
| `net.minecraft.world.entity.LivingEntity#equipment` | Equipment slots (mainhand, offhand, feet, legs, chest, head) tracking | `native/src/inventory.rs:698` | PORTED+PARITY-TESTED |
| `net.minecraft.world.entity.Entity#dimensions` | Dynamic entity bounding boxes by entity type ID | `native/src/inventory.rs:295` | PORTED+PARITY-TESTED |
| `net.minecraft.world.entity.player.Player#attack` | Melee attack raycasting, reach validation, line-of-sight occlusion | `native/src/lib.rs:1997` | PORTED+PARITY-TESTED |
| `net.minecraft.world.item.BowItem` | Bow charging duration, pullback animation and projectile release | `native/src/inventory.rs:1134` | PORTED |
| `net.minecraft.world.item.ShieldItem` | Shield raise, active blocking angle, attack deflection | `native/src/inventory.rs` | PARTIAL |
| `net.minecraft.world.item.TridentItem` | Trident throw, riptide in rain/water acceleration impulse | `native/src/inventory.rs` | PARTIAL |
| `net.minecraft.world.food.FoodData` | Food level (0..20), saturation, exhaustion and auto-health regen | `native/src/inventory.rs:342` | PORTED+PARITY-TESTED |
| `net.minecraft.world.entity.player.Player#respawn` | Client respawn packet (sb_client_command / perform_respawn) | `native/src/inventory.rs:1205` | PORTED+PARITY-TESTED |

## Observable Events & Protocol

| Vanilla Client Class / System | Client-Side Vanilla Behavior | Engine Implementation Path | Status |
|---|---|---|---|
| `net.minecraft.network.protocol.game.ClientboundSoundPacket` | Positional sound event decoding (category, sound id, x, y, z, volume, pitch) | `native/src/inventory.rs:745` | PORTED |
| `net.minecraft.network.protocol.game.ClientboundLevelParticlesPacket` | Particle emission decoding (particle type, count, offset, speed) | `native/src/inventory.rs:755` | PORTED |
| `net.minecraft.network.protocol.game.ClientboundSystemChatPacket` | System chat JSON / NBT component parsing into plain text and raw NBT | `native/src/inventory.rs:250` | PORTED+PARITY-TESTED |
| `net.minecraft.network.protocol.game.ClientboundPlayerChatPacket` | Player chat message verification, sender UUID, plain text body | `native/src/inventory.rs:260` | PORTED+PARITY-TESTED |
| `net.minecraft.network.protocol.game.ClientboundSetTitleTextPacket` | Screen title, subtitle, actionbar and fade times | `native/src/inventory.rs:780` | PORTED |
| `net.minecraft.network.protocol.game.ClientboundBossEventPacket` | Boss bar UUID, title, health percent, color, overlay style | `native/src/inventory.rs:790` | PORTED |
| `net.minecraft.network.protocol.game.ClientboundSetObjectivePacket` | Scoreboard objectives, display slots, and scores per entity name | `native/src/inventory.rs:810` | PORTED |
| `net.minecraft.network.protocol.game.ClientboundUpdateAdvancementsPacket` | Advancement tree progress, earned criteria, reward triggers | `native/src/inventory.rs:825` | PORTED |
| `net.minecraft.network.protocol.game.ClientboundBlockChangedAckPacket` | Block change sequence acknowledgement matching client prediction | `native/src/lib.rs:776` | PORTED+PARITY-TESTED |
| `net.minecraft.network.protocol.game.ClientboundBlockUpdatePacket` | Single block coordinate state update into loaded chunk section palette | `native/src/world.rs:298` | PORTED+PARITY-TESTED |
| `net.minecraft.network.protocol.game.ClientboundSectionBlocksUpdatePacket` | Multi-block coordinate section chunk updates | `native/src/world.rs:313` | PORTED+PARITY-TESTED |

