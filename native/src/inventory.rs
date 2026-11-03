//! Native 26.3 inventory, container screen state, entity table, player vitals,
//! clientbound state decoders, and serverbound interaction/GUI packet encoders.
#![allow(dead_code)]

use crate::wire::{Reader, Result, string, var};
use sawbyte_core::events::{BlockFace, ClickMode, EntityInteraction, Hand, SubTickEvent};
use sawbyte_core::screen::{ItemStack, ScreenKind, ScreenState};
use std::collections::{BTreeMap, HashMap};

/// Configurable 26.3 (`protocol 777`) play packet ID table populated from `generated/reports/packets.json`
/// via `block-profile.json`, with built-in 26.3 defaults when omitted.
#[derive(Clone, Debug)]
pub struct PacketTable {
    // Clientbound play packet IDs
    pub cb_add_entity: i32,
    pub cb_block_update: i32,
    pub cb_container_close: i32,
    pub cb_container_set_content: i32,
    pub cb_container_set_data: i32,
    pub cb_container_set_slot: i32,
    pub cb_open_screen: i32,
    pub cb_remove_entities: i32,
    pub cb_section_blocks_update: i32,
    pub cb_set_entity_motion: i32,
    pub cb_set_health: i32,
    pub cb_system_chat: i32,
    pub cb_teleport_entity: i32,
    // Serverbound play packet IDs
    pub sb_chat_command: i32,
    pub sb_chat: i32,
    pub sb_container_button_click: i32,
    pub sb_container_click: i32,
    pub sb_container_close: i32,
    pub sb_interact: i32,
    pub sb_place_recipe: i32,
    pub sb_player_action: i32,
    pub sb_rename_item: i32,
    pub sb_select_trade: i32,
    pub sb_set_carried_item: i32,
    pub sb_sign_update: i32,
    pub sb_swing: i32,
    pub sb_use_item_on: i32,
    pub sb_use_item: i32,
    // Registry ID -> Identifier tables
    pub item_names: HashMap<u32, String>,
}

impl Default for PacketTable {
    fn default() -> Self {
        Self {
            cb_add_entity: 1,
            cb_block_update: 8,
            cb_container_close: 17,
            cb_container_set_content: 18,
            cb_container_set_data: 19,
            cb_container_set_slot: 20,
            cb_open_screen: 55,
            cb_remove_entities: 76,
            cb_section_blocks_update: 86,
            cb_set_entity_motion: 95,
            cb_set_health: 98,
            cb_system_chat: 115,
            cb_teleport_entity: 119,
            sb_chat_command: 6,
            sb_chat: 8,
            sb_container_button_click: 16,
            sb_container_click: 17,
            sb_container_close: 18,
            sb_interact: 24,
            sb_place_recipe: 38,
            sb_player_action: 41,
            sb_rename_item: 48,
            sb_select_trade: 51,
            sb_set_carried_item: 52,
            sb_sign_update: 59,
            sb_swing: 60,
            sb_use_item_on: 66,
            sb_use_item: 67,
            item_names: HashMap::new(),
        }
    }
}

impl PacketTable {
    pub fn from_profile(profile: &serde_json::Value) -> Self {
        let mut table = Self::default();
        let cb = &profile["packets"]["clientbound"];
        let sb = &profile["packets"]["serverbound"];
        let get_id = |group: &serde_json::Value, key: &str, default: i32| -> i32 {
            group[key]["protocol_id"]
                .as_i64()
                .map(|v| v as i32)
                .unwrap_or(default)
        };
        table.cb_add_entity = get_id(cb, "minecraft:add_entity", table.cb_add_entity);
        table.cb_block_update = get_id(cb, "minecraft:block_update", table.cb_block_update);
        table.cb_container_close =
            get_id(cb, "minecraft:container_close", table.cb_container_close);
        table.cb_container_set_content = get_id(
            cb,
            "minecraft:container_set_content",
            table.cb_container_set_content,
        );
        table.cb_container_set_data = get_id(
            cb,
            "minecraft:container_set_data",
            table.cb_container_set_data,
        );
        table.cb_container_set_slot = get_id(
            cb,
            "minecraft:container_set_slot",
            table.cb_container_set_slot,
        );
        table.cb_open_screen = get_id(cb, "minecraft:open_screen", table.cb_open_screen);
        table.cb_remove_entities =
            get_id(cb, "minecraft:remove_entities", table.cb_remove_entities);
        table.cb_section_blocks_update = get_id(
            cb,
            "minecraft:section_blocks_update",
            table.cb_section_blocks_update,
        );
        table.cb_set_entity_motion = get_id(
            cb,
            "minecraft:set_entity_motion",
            table.cb_set_entity_motion,
        );
        table.cb_set_health = get_id(cb, "minecraft:set_health", table.cb_set_health);
        table.cb_system_chat = get_id(cb, "minecraft:system_chat", table.cb_system_chat);
        table.cb_teleport_entity =
            get_id(cb, "minecraft:teleport_entity", table.cb_teleport_entity);

        table.sb_chat_command = get_id(sb, "minecraft:chat_command", table.sb_chat_command);
        table.sb_chat = get_id(sb, "minecraft:chat", table.sb_chat);
        table.sb_container_button_click = get_id(
            sb,
            "minecraft:container_button_click",
            table.sb_container_button_click,
        );
        table.sb_container_click =
            get_id(sb, "minecraft:container_click", table.sb_container_click);
        table.sb_container_close =
            get_id(sb, "minecraft:container_close", table.sb_container_close);
        table.sb_interact = get_id(sb, "minecraft:interact", table.sb_interact);
        table.sb_place_recipe = get_id(sb, "minecraft:place_recipe", table.sb_place_recipe);
        table.sb_player_action = get_id(sb, "minecraft:player_action", table.sb_player_action);
        table.sb_rename_item = get_id(sb, "minecraft:rename_item", table.sb_rename_item);
        table.sb_select_trade = get_id(sb, "minecraft:select_trade", table.sb_select_trade);
        table.sb_set_carried_item =
            get_id(sb, "minecraft:set_carried_item", table.sb_set_carried_item);
        table.sb_sign_update = get_id(sb, "minecraft:sign_update", table.sb_sign_update);
        table.sb_swing = get_id(sb, "minecraft:swing", table.sb_swing);
        table.sb_use_item_on = get_id(sb, "minecraft:use_item_on", table.sb_use_item_on);
        table.sb_use_item = get_id(sb, "minecraft:use_item", table.sb_use_item);

        if let Some(items) = profile["items"].as_object() {
            for (name, id_val) in items {
                if let Some(id) = id_val.as_u64() {
                    table.item_names.insert(id as u32, name.clone());
                }
            }
        }
        table
    }
}

pub fn nbt_to_plain_text(val: &serde_json::Value) -> String {
    match val {
        serde_json::Value::String(s) => s.clone(),
        serde_json::Value::Array(arr) => {
            let mut out = String::new();
            for item in arr {
                out.push_str(&nbt_to_plain_text(item));
            }
            out
        }
        serde_json::Value::Object(map) => {
            let mut out = String::new();
            if let Some(t) = map.get("text").and_then(|v| v.as_str()) {
                out.push_str(t);
            } else if let Some(tr) = map.get("translate").and_then(|v| v.as_str()) {
                out.push_str(tr);
                if let Some(with_args) = map.get("with").and_then(|v| v.as_array()) {
                    let rendered: Vec<String> =
                        with_args.iter().map(nbt_to_plain_text).collect();
                    if !rendered.is_empty() {
                        out.push('(');
                        out.push_str(&rendered.join(", "));
                        out.push(')');
                    }
                }
            }
            if let Some(extra) = map.get("extra").and_then(|v| v.as_array()) {
                for item in extra {
                    out.push_str(&nbt_to_plain_text(item));
                }
            }
            out
        }
        _ => String::new(),
    }
}

#[derive(Clone, Debug)]
pub struct TrackedEntity {
    pub entity_id: i32,
    pub entity_type: i32,
    pub pos: [f64; 3],
    pub velocity: [f64; 3],
    pub yaw: f32,
    pub pitch: f32,
    pub width: f64,
    pub height: f64,
}

impl TrackedEntity {
    pub fn aabb(&self) -> [f64; 6] {
        let r = self.width * 0.5;
        [
            self.pos[0] - r,
            self.pos[1],
            self.pos[2] - r,
            self.pos[0] + r,
            self.pos[1] + self.height,
            self.pos[2] + r,
        ]
    }
}

pub struct PlayerStateTable {
    pub player_entity_id: Option<i32>,
    pub inventory_slots: Vec<Option<ItemStack>>,
    pub selected_hotbar: u8,
    pub open_screen: Option<ScreenState>,
    pub entities: BTreeMap<i32, TrackedEntity>,
    pub health: f32,
    pub food: i32,
    pub saturation: f32,
    pub air_supply: i32,
    pub xp_level: i32,
    pub xp_progress: f32,
    pub game_mode: u8,
    pub action_sequence: i32,
    pub chat_log: Vec<String>,
    pub received_events: Vec<String>,
    pub decode_errors: u64,
    pub last_decode_error: Option<String>,
}

impl Default for PlayerStateTable {
    fn default() -> Self {
        Self {
            player_entity_id: None,
            inventory_slots: vec![None; 46],
            selected_hotbar: 0,
            open_screen: None,
            entities: BTreeMap::new(),
            health: 20.0,
            food: 20,
            saturation: 5.0,
            air_supply: 300,
            xp_level: 0,
            xp_progress: 0.0,
            game_mode: 0,
            action_sequence: 1,
            chat_log: Vec::new(),
            received_events: Vec::new(),
            decode_errors: 0,
            last_decode_error: None,
        }
    }
}

fn decode_slot_item(
    r: &mut Reader<'_>,
    packets: &PacketTable,
) -> Result<(Option<ItemStack>, bool)> {
    let count = r.var()?;
    if count <= 0 {
        return Ok((None, true));
    }
    let item_id = r.var()?;
    if item_id < 0 {
        return Err(crate::wire::bad("negative item id in slot"));
    }
    let added_components = r.var()?;
    let removed_components = r.var()?;
    if added_components < 0 || removed_components < 0 {
        return Err(crate::wire::bad("negative component patch count"));
    }
    let name = packets
        .item_names
        .get(&(item_id as u32))
        .cloned()
        .unwrap_or_else(|| format!("minecraft:item_{item_id}"));
    let mut stack = ItemStack::new(name, count as u32);
    let mut exact_stream = true;
    for _ in 0..added_components {
        let Ok(comp_id) = r.var() else {
            exact_stream = false;
            break;
        };
        match comp_id {
            // 3 = max_damage (VarInt), 4 = damage (VarInt) in 26.3 data_component_type
            3 => {
                if let Ok(max_dmg) = r.var() {
                    let cur = stack.damage.map_or(0, |(d, _)| d);
                    stack.damage = Some((cur, max_dmg));
                } else {
                    exact_stream = false;
                    break;
                }
            }
            4 => {
                if let Ok(dmg) = r.var() {
                    let max_d = stack.damage.map_or(0, |(_, m)| m);
                    stack.damage = Some((dmg, max_d));
                } else {
                    exact_stream = false;
                    break;
                }
            }
            // 5 = custom_name (NBT), 6 = item_name (NBT)
            5 | 6 => {
                if let Ok(tag) = r.nbt() {
                    let text = nbt_to_plain_text(&tag);
                    if !text.is_empty() {
                        stack.display_name = Some(text);
                    }
                } else {
                    exact_stream = false;
                    break;
                }
            }
            // 7 = lore (list of NBT text components)
            7 => {
                if let Ok(line_count) = r.count(256) {
                    let mut lines = Vec::with_capacity(line_count);
                    for _ in 0..line_count {
                        if let Ok(tag) = r.nbt() {
                            lines.push(nbt_to_plain_text(&tag));
                        } else {
                            exact_stream = false;
                            break;
                        }
                    }
                    stack.lore = lines;
                } else {
                    exact_stream = false;
                    break;
                }
            }
            other => {
                stack
                    .components
                    .push((format!("component_{other}"), "present".into()));
                exact_stream = false;
                break;
            }
        }
    }
    if exact_stream {
        for _ in 0..removed_components {
            if r.var().is_err() {
                exact_stream = false;
                break;
            }
        }
    }
    Ok((Some(stack), exact_stream))
}

impl PlayerStateTable {
    pub fn next_sequence(&mut self) -> i32 {
        self.next_sequence_n(1)
    }

    pub fn next_sequence_n(&mut self, count: i32) -> i32 {
        let seq = self.action_sequence;
        let step = count.max(1);
        self.action_sequence = self.action_sequence.wrapping_add(step).max(1);
        seq
    }

    pub fn push_event(&mut self, event: impl Into<String>) {
        if self.received_events.len() >= 256 {
            self.received_events.remove(0);
        }
        self.received_events.push(event.into());
    }

    pub fn record_decode_error(&mut self, context: &str, err: &dyn std::error::Error) {
        self.decode_errors = self.decode_errors.saturating_add(1);
        let msg = format!("{context}: {err}");
        self.last_decode_error = Some(msg.clone());
        self.push_event(format!("decode_error:{msg}"));
    }

    pub fn apply_open_screen(&mut self, r: &mut Reader<'_>) -> Result<()> {
        let window_id = r.var()?;
        let menu_type = r.var()?;
        let title_nbt = r.nbt()?;
        let title = nbt_to_plain_text(&title_nbt);
        let (kind, slot_count) = match menu_type {
            0..=5 => {
                let rows = (menu_type + 1) as u8;
                (ScreenKind::GenericContainer { rows }, (rows as usize * 9) + 36)
            }
            6 => (ScreenKind::DropperOrDispenser, 9 + 36),
            7 => (ScreenKind::Crafter3x3, 10 + 36),
            8 => (ScreenKind::Anvil, 3 + 36),
            9 => (ScreenKind::Beacon, 1 + 36),
            10 => (ScreenKind::BlastFurnace, 3 + 36),
            11 => (ScreenKind::BrewingStand, 5 + 36),
            12 => (ScreenKind::CraftingTable, 10 + 36),
            13 => (ScreenKind::EnchantingTable, 2 + 36),
            14 => (ScreenKind::Furnace, 3 + 36),
            15 => (ScreenKind::Grindstone, 3 + 36),
            16 => (ScreenKind::Hopper, 5 + 36),
            17 => (ScreenKind::Lectern, 1),
            18 => (ScreenKind::Loom, 4 + 36),
            19 => (ScreenKind::VillagerTrading, 3 + 36),
            20 => (ScreenKind::ShulkerBox, 27 + 36),
            21 => (ScreenKind::SmithingTable, 4 + 36),
            22 => (ScreenKind::Smoker, 3 + 36),
            23 => (ScreenKind::CartographyTable, 3 + 36),
            24 => (ScreenKind::Stonecutter, 2 + 36),
            other => (
                ScreenKind::CustomModScreen {
                    screen_id: format!("menu_type_{other}"),
                },
                0,
            ),
        };
        let screen = ScreenState {
            window_id,
            state_id: 1,
            kind,
            title: title.clone(),
            slots: vec![None; slot_count],
            cursor_item: None,
            widgets: Vec::new(),
            fields: Vec::new(),
            trades: Vec::new(),
            properties: Vec::new(),
        };
        self.open_screen = Some(screen);
        self.push_event(format!("screen_open:{title}"));
        Ok(())
    }

    pub fn apply_container_close(&mut self, r: &mut Reader<'_>) -> Result<()> {
        let _window_id = r.var()?;
        self.open_screen = None;
        self.push_event("screen_closed");
        Ok(())
    }

    pub fn apply_container_set_data(&mut self, r: &mut Reader<'_>) -> Result<()> {
        let window_id = r.var()?;
        let prop = i16::from_be_bytes(r.take(2)?.try_into()?);
        let val = i16::from_be_bytes(r.take(2)?.try_into()?);
        if let Some(screen) = &mut self.open_screen {
            if screen.window_id == window_id {
                if let Some(existing) = screen.properties.iter_mut().find(|(p, _)| *p == prop) {
                    existing.1 = val;
                } else {
                    if screen.properties.len() >= 64 {
                        screen.properties.remove(0);
                    }
                    screen.properties.push((prop, val));
                }
            }
        }
        Ok(())
    }

    pub fn apply_container_set_slot(
        &mut self,
        r: &mut Reader<'_>,
        packets: &PacketTable,
    ) -> Result<()> {
        let window_id = r.var()?;
        let state_id = r.var()?;
        let slot = i16::from_be_bytes(r.take(2)?.try_into()?);
        let (item, _) = decode_slot_item(r, packets)?;
        if window_id == -1 && slot == -1 {
            if let Some(screen) = &mut self.open_screen {
                screen.state_id = state_id;
                screen.cursor_item = item;
            }
            return Ok(());
        }
        if window_id == 0 {
            if (0..self.inventory_slots.len() as i16).contains(&slot) {
                self.inventory_slots[slot as usize] = item.clone();
            }
        }
        if let Some(screen) = &mut self.open_screen {
            if screen.window_id == window_id {
                screen.state_id = state_id;
                if slot == -1 {
                    screen.cursor_item = item;
                } else if slot >= 0 {
                    screen.set_slot_authoritative(slot as usize, item);
                }
            }
        }
        Ok(())
    }

    pub fn apply_container_set_content(
        &mut self,
        r: &mut Reader<'_>,
        packets: &PacketTable,
    ) -> Result<()> {
        let window_id = r.var()?;
        let state_id = r.var()?;
        let slot_count = r.count(256)?;
        let mut staged_slots = vec![None; slot_count];
        let mut stream_intact = true;
        for slot_ref in staged_slots.iter_mut() {
            let (stack, exact) = decode_slot_item(r, packets)?;
            *slot_ref = stack;
            if !exact {
                stream_intact = false;
                break;
            }
        }
        let cursor_item = if stream_intact && !r.0.is_empty() {
            decode_slot_item(r, packets).ok().and_then(|(it, _)| it)
        } else {
            None
        };
        if window_id == 0 {
            for (idx, item) in staged_slots.iter().enumerate() {
                if idx < self.inventory_slots.len() {
                    self.inventory_slots[idx] = item.clone();
                }
            }
        }
        if let Some(screen) = &mut self.open_screen {
            if screen.window_id == window_id {
                screen.apply_server_content(state_id, staged_slots, cursor_item);
            }
        }
        Ok(())
    }

    pub fn apply_set_health(&mut self, r: &mut Reader<'_>) -> Result<()> {
        self.health = r.f32()?;
        self.food = r.var()?;
        self.saturation = r.f32()?;
        Ok(())
    }

    pub fn apply_add_entity(&mut self, r: &mut Reader<'_>) -> Result<()> {
        let entity_id = r.var()?;
        let _uuid = r.take(16)?;
        let entity_type = r.var()?;
        let x = r.f64()?;
        let y = r.f64()?;
        let z = r.f64()?;
        if self.entities.len() >= 1024 && !self.entities.contains_key(&entity_id) {
            if let Some(&oldest) = self.entities.keys().next() {
                self.entities.remove(&oldest);
            }
        }
        self.entities.insert(
            entity_id,
            TrackedEntity {
                entity_id,
                entity_type,
                pos: [x, y, z],
                velocity: [0.0; 3],
                yaw: 0.0,
                pitch: 0.0,
                width: 0.6,
                height: 1.8,
            },
        );
        Ok(())
    }

    pub fn apply_remove_entities(&mut self, r: &mut Reader<'_>) -> Result<()> {
        let count = r.count(4096)?;
        for _ in 0..count {
            let id = r.var()?;
            self.entities.remove(&id);
        }
        Ok(())
    }

    pub fn apply_teleport_entity(&mut self, r: &mut Reader<'_>) -> Result<()> {
        let entity_id = r.var()?;
        let x = r.f64()?;
        let y = r.f64()?;
        let z = r.f64()?;
        let vx = r.f64()?;
        let vy = r.f64()?;
        let vz = r.f64()?;
        let yaw = r.f32()?;
        let pitch = r.f32()?;
        if let Some(ent) = self.entities.get_mut(&entity_id) {
            ent.pos = [x, y, z];
            ent.velocity = [vx, vy, vz];
            ent.yaw = yaw;
            ent.pitch = pitch;
        }
        Ok(())
    }

    pub fn apply_system_chat(&mut self, r: &mut Reader<'_>) -> Result<()> {
        let content = r.nbt()?;
        let text = nbt_to_plain_text(&content);
        if !text.is_empty() {
            self.push_event(format!("chat:{text}"));
            if self.chat_log.len() >= 128 {
                self.chat_log.remove(0);
            }
            self.chat_log.push(text);
        }
        Ok(())
    }
}

pub fn pack_block_pos(pos: [i32; 3]) -> u64 {
    let x = (pos[0] as i64) & 0x3ff_ffff;
    let z = (pos[2] as i64) & 0x3ff_ffff;
    let y = (pos[1] as i64) & 0xfff;
    ((x << 38) | (z << 12) | y) as u64
}

/// Encodes a discrete `SubTickEvent` into its corresponding 26.3 serverbound `(packet_id, payload)` pair.
pub fn encode_serverbound_event(event: &SubTickEvent) -> Option<(i32, Vec<u8>)> {
    encode_serverbound_event_with_table(event, &PacketTable::default())
}

pub fn encode_serverbound_event_with_table(
    event: &SubTickEvent,
    packets: &PacketTable,
) -> Option<(i32, Vec<u8>)> {
    encode_serverbound_event_with_pose(event, packets, 0.0, 0.0)
}

pub fn encode_serverbound_event_with_pose(
    event: &SubTickEvent,
    packets: &PacketTable,
    player_yaw: f32,
    player_pitch: f32,
) -> Option<(i32, Vec<u8>)> {
    let mut out = Vec::new();
    match event {
        SubTickEvent::SelectHotbar { slot } => {
            out.extend(u16::from(*slot).to_be_bytes());
            Some((packets.sb_set_carried_item, out))
        }
        SubTickEvent::StartDestroyBlock {
            pos,
            face,
            sequence,
        } => {
            var(0, &mut out);
            out.extend(pack_block_pos(*pos).to_be_bytes());
            out.push(*face as u8);
            var(*sequence, &mut out);
            Some((packets.sb_player_action, out))
        }
        SubTickEvent::AbortDestroyBlock { pos, face } => {
            var(1, &mut out);
            out.extend(pack_block_pos(*pos).to_be_bytes());
            out.push(*face as u8);
            var(0, &mut out);
            Some((packets.sb_player_action, out))
        }
        SubTickEvent::StopDestroyBlock {
            pos,
            face,
            sequence,
        } => {
            var(2, &mut out);
            out.extend(pack_block_pos(*pos).to_be_bytes());
            out.push(*face as u8);
            var(*sequence, &mut out);
            Some((packets.sb_player_action, out))
        }
        SubTickEvent::DropItem { entire_stack } => {
            var(if *entire_stack { 3 } else { 4 }, &mut out);
            out.extend(0u64.to_be_bytes());
            out.push(BlockFace::Down as u8);
            var(0, &mut out);
            Some((packets.sb_player_action, out))
        }
        SubTickEvent::ReleaseUseItem => {
            var(5, &mut out);
            out.extend(0u64.to_be_bytes());
            out.push(BlockFace::Down as u8);
            var(0, &mut out);
            Some((packets.sb_player_action, out))
        }
        SubTickEvent::SwapOffhand => {
            var(6, &mut out);
            out.extend(0u64.to_be_bytes());
            out.push(BlockFace::Down as u8);
            var(0, &mut out);
            Some((packets.sb_player_action, out))
        }
        SubTickEvent::UseItemOn {
            hand,
            pos,
            face,
            hit,
            inside_block,
            sequence,
        } => {
            var(if *hand == Hand::MainHand { 0 } else { 1 }, &mut out);
            out.extend(pack_block_pos(*pos).to_be_bytes());
            var(*face as i32, &mut out);
            for c in hit {
                out.extend(c.to_be_bytes());
            }
            out.push(u8::from(*inside_block));
            out.push(0);
            var(*sequence, &mut out);
            Some((packets.sb_use_item_on, out))
        }
        SubTickEvent::UseItem { hand, sequence } => {
            var(if *hand == Hand::MainHand { 0 } else { 1 }, &mut out);
            var(*sequence, &mut out);
            out.extend(player_yaw.to_be_bytes());
            out.extend(player_pitch.to_be_bytes());
            Some((packets.sb_use_item, out))
        }
        SubTickEvent::SwipeArm { hand } => {
            var(if *hand == Hand::MainHand { 0 } else { 1 }, &mut out);
            Some((packets.sb_swing, out))
        }
        SubTickEvent::InteractEntity {
            entity_id,
            interaction,
            sneaking,
        } => {
            var(*entity_id, &mut out);
            match interaction {
                EntityInteraction::Interact { hand } => {
                    var(0, &mut out);
                    var(if *hand == Hand::MainHand { 0 } else { 1 }, &mut out);
                }
                EntityInteraction::Attack => {
                    var(1, &mut out);
                }
                EntityInteraction::InteractAt { hand, target } => {
                    var(2, &mut out);
                    for c in target {
                        out.extend(c.to_be_bytes());
                    }
                    var(if *hand == Hand::MainHand { 0 } else { 1 }, &mut out);
                }
            }
            out.push(u8::from(*sneaking));
            Some((packets.sb_interact, out))
        }
        SubTickEvent::ContainerClick {
            window_id,
            state_id,
            slot,
            button,
            mode,
        } => {
            var(*window_id, &mut out);
            var(*state_id, &mut out);
            out.extend(slot.to_be_bytes());
            out.push(*button as u8);
            var(*mode as i32, &mut out);
            var(0, &mut out);
            out.push(0);
            Some((packets.sb_container_click, out))
        }
        SubTickEvent::ContainerButtonClick {
            window_id,
            button_id,
        } => {
            var(*window_id, &mut out);
            var(*button_id, &mut out);
            Some((packets.sb_container_button_click, out))
        }
        SubTickEvent::CloseContainer { window_id } => {
            var(*window_id, &mut out);
            Some((packets.sb_container_close, out))
        }
        SubTickEvent::PlaceRecipe {
            window_id,
            recipe,
            make_all,
        } => {
            var(*window_id, &mut out);
            string(recipe, &mut out);
            out.push(u8::from(*make_all));
            Some((packets.sb_place_recipe, out))
        }
        SubTickEvent::RenameItem { name } => {
            string(name, &mut out);
            Some((packets.sb_rename_item, out))
        }
        SubTickEvent::SelectTrade { slot } => {
            var(i32::from(*slot), &mut out);
            Some((packets.sb_select_trade, out))
        }
        SubTickEvent::SendCommand { command } => {
            string(command, &mut out);
            Some((packets.sb_chat_command, out))
        }
        SubTickEvent::SendChat { message } => {
            string(message, &mut out);
            out.extend(0u64.to_be_bytes());
            out.extend(0u64.to_be_bytes());
            out.push(0);
            var(0, &mut out);
            out.extend([0u8; 3]);
            out.push(0);
            Some((packets.sb_chat, out))
        }
        SubTickEvent::UpdateSign { pos, front, lines } => {
            out.extend(pack_block_pos(*pos).to_be_bytes());
            out.push(u8::from(*front));
            for line in lines {
                string(line, &mut out);
            }
            Some((packets.sb_sign_update, out))
        }
        _ => None,
    }
}

#[allow(dead_code)]
const _: ClickMode = ClickMode::Pickup;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn block_pos_packing_and_event_encoding_round_trip() {
        let pos = [-15, -60, 42];
        let packed = pack_block_pos(pos) as i64;
        let x = (packed >> 38) as i32;
        let y = ((packed << 52) >> 52) as i32;
        let z = ((packed << 26) >> 38) as i32;
        assert_eq!([x, y, z], pos);

        let (pkt_id, payload) = encode_serverbound_event(&SubTickEvent::StartDestroyBlock {
            pos,
            face: BlockFace::Up,
            sequence: 7,
        })
        .unwrap();
        assert_eq!(pkt_id, 41);
        assert!(!payload.is_empty());

        let (click_id, _) = encode_serverbound_event(&SubTickEvent::ContainerClick {
            window_id: 1,
            state_id: 5,
            slot: 13,
            button: 0,
            mode: ClickMode::Pickup,
        })
        .unwrap();
        assert_eq!(click_id, 17);
    }

    #[test]
    fn container_set_content_preserves_authoritative_server_state_id_and_cursor() {
        let mut table = PlayerStateTable::default();
        table.open_screen = Some(ScreenState::container(2, 1, 1, "Chest"));
        let mut packets = PacketTable::default();
        packets.item_names.insert(1, "minecraft:stone".into());
        packets.item_names.insert(4, "minecraft:cobblestone".into());

        // Build a synthetic ContainerSetContent packet: window_id=2, state_id=42, 3 slots + cursor item
        let mut raw = Vec::new();
        var(2, &mut raw); // window_id
        var(42, &mut raw); // authoritative server state_id
        var(3, &mut raw); // slot_count = 3
        // slot 0: 64x stone, 0 added, 0 removed
        var(64, &mut raw);
        var(1, &mut raw);
        var(0, &mut raw);
        var(0, &mut raw);
        // slot 1: empty
        var(0, &mut raw);
        // slot 2: 16x cobblestone, 0 added, 0 removed
        var(16, &mut raw);
        var(4, &mut raw);
        var(0, &mut raw);
        var(0, &mut raw);
        // cursor item: 8x stone
        var(8, &mut raw);
        var(1, &mut raw);
        var(0, &mut raw);
        var(0, &mut raw);

        let mut reader = Reader(&raw);
        table
            .apply_container_set_content(&mut reader, &packets)
            .unwrap();
        let screen = table.open_screen.as_ref().unwrap();
        assert_eq!(screen.state_id, 42);
        assert_eq!(screen.slots[0].as_ref().unwrap().id, "minecraft:stone");
        assert_eq!(screen.slots[0].as_ref().unwrap().count, 64);
        assert!(screen.slots[1].is_none());
        assert_eq!(
            screen.slots[2].as_ref().unwrap().id,
            "minecraft:cobblestone"
        );
        assert_eq!(screen.cursor_item.as_ref().unwrap().count, 8);
    }
}
