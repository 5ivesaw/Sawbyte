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
            cb_container_close: 18,
            cb_container_set_content: 19,
            cb_container_set_data: 20,
            cb_container_set_slot: 21,
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
            sb_player_action: 40,
            sb_rename_item: 48,
            sb_select_trade: 51,
            sb_set_carried_item: 52,
            sb_sign_update: 59,
            sb_swing: 60,
            sb_use_item_on: 63,
            sb_use_item: 64,
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
        serde_json::Value::Object(map) => {
            let mut out = String::new();
            if let Some(t) = map.get("text").and_then(|v| v.as_str()) {
                out.push_str(t);
            } else if let Some(tr) = map.get("translate").and_then(|v| v.as_str()) {
                out.push_str(tr);
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
        }
    }
}

impl PlayerStateTable {
    pub fn next_sequence(&mut self) -> i32 {
        let seq = self.action_sequence;
        self.action_sequence = self.action_sequence.wrapping_add(1).max(1);
        seq
    }

    pub fn apply_open_screen(&mut self, r: &mut Reader<'_>) -> Result<()> {
        let window_id = r.var()?;
        let menu_type = r.var()?;
        let title_nbt = r.nbt()?;
        let title = nbt_to_plain_text(&title_nbt);
        let screen = match menu_type {
            0..=5 => {
                let rows = (menu_type + 1) as u8;
                ScreenState::container(window_id, 1, rows, title.clone())
            }
            7 => ScreenState {
                window_id,
                state_id: 1,
                kind: ScreenKind::Anvil,
                title: title.clone(),
                slots: vec![None; 39],
                cursor_item: None,
                widgets: Vec::new(),
                fields: Vec::new(),
                trades: Vec::new(),
                properties: Vec::new(),
            },
            18 => ScreenState {
                window_id,
                state_id: 1,
                kind: ScreenKind::VillagerTrading,
                title: title.clone(),
                slots: vec![None; 39],
                cursor_item: None,
                widgets: Vec::new(),
                fields: Vec::new(),
                trades: Vec::new(),
                properties: Vec::new(),
            },
            _ => ScreenState {
                window_id,
                state_id: 1,
                kind: ScreenKind::GenericContainer { rows: 3 },
                title: title.clone(),
                slots: vec![None; 63],
                cursor_item: None,
                widgets: Vec::new(),
                fields: Vec::new(),
                trades: Vec::new(),
                properties: Vec::new(),
            },
        };
        self.open_screen = Some(screen);
        self.received_events.push(format!("screen_open:{title}"));
        Ok(())
    }

    pub fn apply_container_close(&mut self, r: &mut Reader<'_>) -> Result<()> {
        let _window_id = r.var()?;
        self.open_screen = None;
        self.received_events.push("screen_closed".into());
        Ok(())
    }

    pub fn apply_container_set_data(&mut self, r: &mut Reader<'_>) -> Result<()> {
        let window_id = r.var()?;
        let prop = i16::from_be_bytes(r.take(2)?.try_into()?);
        let val = i16::from_be_bytes(r.take(2)?.try_into()?);
        if let Some(screen) = &mut self.open_screen {
            if screen.window_id == window_id {
                screen.properties.push((prop, val));
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
        let count = r.var()?;
        let item = if count <= 0 {
            None
        } else {
            let item_id = r.var()? as u32;
            let name = packets
                .item_names
                .get(&item_id)
                .cloned()
                .unwrap_or_else(|| format!("minecraft:item_{item_id}"));
            Some(ItemStack::new(name, count as u32))
        };
        if window_id == 0 {
            if (0..self.inventory_slots.len() as i16).contains(&slot) {
                self.inventory_slots[slot as usize] = item;
            }
        } else if let Some(screen) = &mut self.open_screen {
            if screen.window_id == window_id {
                screen.state_id = state_id;
                if slot >= 0 {
                    screen.set_slot(slot as usize, item);
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
        if let Some(screen) = &mut self.open_screen {
            if screen.window_id == window_id {
                screen.state_id = state_id;
                if screen.slots.len() < slot_count {
                    screen.slots.resize(slot_count, None);
                }
            }
        }
        for idx in 0..slot_count {
            let Ok(count) = r.var() else {
                break;
            };
            if count <= 0 {
                if window_id == 0 && idx < self.inventory_slots.len() {
                    self.inventory_slots[idx] = None;
                } else if let Some(screen) = &mut self.open_screen {
                    if screen.window_id == window_id {
                        screen.set_slot(idx, None);
                    }
                }
            } else {
                let Ok(item_id) = r.var() else {
                    break;
                };
                let added_components = r.var().unwrap_or(0);
                let removed_components = r.var().unwrap_or(0);
                let name = packets
                    .item_names
                    .get(&(item_id as u32))
                    .cloned()
                    .unwrap_or_else(|| format!("minecraft:item_{item_id}"));
                let stack = Some(ItemStack::new(name, count as u32));
                if window_id == 0 && idx < self.inventory_slots.len() {
                    self.inventory_slots[idx] = stack;
                } else if let Some(screen) = &mut self.open_screen {
                    if screen.window_id == window_id {
                        screen.set_slot(idx, stack);
                    }
                }
                // If an item carries arbitrary variable-length component patches, stop slot loop cleanly after recording it.
                if added_components > 0 || removed_components > 0 {
                    break;
                }
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
            if text.to_lowercase().contains("purchased") || text.to_lowercase().contains("bought") {
                self.received_events.push("purchase_confirmed".into());
            }
            self.received_events.push(format!("chat:{text}"));
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
            out.extend(0.0f32.to_be_bytes());
            out.extend(0.0f32.to_be_bytes());
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
        assert_eq!(pkt_id, 40);
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
}
