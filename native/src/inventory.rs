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
    pub cb_entity_position_sync: i32,
    pub cb_move_entity_pos: i32,
    pub cb_open_screen: i32,
    pub cb_player_combat_kill: i32,
    pub cb_remove_entities: i32,
    pub cb_respawn: i32,
    pub cb_section_blocks_update: i32,
    pub cb_set_cursor_item: i32,
    pub cb_set_entity_motion: i32,
    pub cb_set_health: i32,
    pub cb_set_player_inventory: i32,
    pub cb_system_chat: i32,
    pub cb_teleport_entity: i32,
    pub cb_update_mob_effect: i32,
    pub cb_remove_mob_effect: i32,
    pub cb_update_attributes: i32,
    pub cb_game_event: i32,
    pub cb_player_abilities: i32,
    pub cb_set_equipment: i32,
    pub cb_set_experience: i32,
    pub cb_block_changed_ack: i32,
    // Serverbound play packet IDs
    pub sb_attack: i32,
    pub sb_chat_command: i32,
    pub sb_chat: i32,
    pub sb_client_command: i32,
    pub sb_container_button_click: i32,
    pub sb_container_click: i32,
    pub sb_container_close: i32,
    pub sb_interact: i32,
    pub sb_place_recipe: i32,
    pub sb_player_action: i32,
    pub sb_player_abilities: i32,
    pub sb_player_command: i32,
    pub sb_rename_item: i32,
    pub sb_select_trade: i32,
    pub sb_set_carried_item: i32,
    pub sb_sign_update: i32,
    pub sb_steer_vehicle: i32,
    pub sb_swing: i32,
    pub sb_use_item_on: i32,
    pub sb_use_item: i32,
    // Registry ID -> Identifier tables
    pub item_names: HashMap<u32, String>,
}

impl Default for PacketTable {
    fn default() -> Self {
        Self {
            cb_add_entity: 1,              // minecraft:add_entity
            cb_block_update: 8,            // minecraft:block_update
            cb_container_close: 17,        // minecraft:container_close
            cb_container_set_content: 18,  // minecraft:container_set_content
            cb_container_set_data: 19,     // minecraft:container_set_data
            cb_container_set_slot: 20,     // minecraft:container_set_slot
            cb_entity_position_sync: 35,   // minecraft:entity_position_sync
            cb_move_entity_pos: 54,        // minecraft:move_entity_pos
            cb_open_screen: 60,            // minecraft:open_screen
            cb_player_combat_kill: 69,     // minecraft:player_combat_kill
            cb_remove_entities: 78,        // minecraft:remove_entities
            cb_respawn: 84,                // minecraft:respawn
            cb_section_blocks_update: 86,  // minecraft:section_blocks_update
            cb_set_cursor_item: 98,        // minecraft:set_cursor_item
            cb_set_entity_motion: 103,     // minecraft:set_entity_motion
            cb_set_health: 106,            // minecraft:set_health
            cb_set_player_inventory: 110,  // minecraft:set_player_inventory
            cb_system_chat: 124,           // minecraft:system_chat
            cb_teleport_entity: 128,       // minecraft:teleport_entity
            cb_update_mob_effect: 134,     // minecraft:update_mob_effect
            cb_remove_mob_effect: 81,      // minecraft:remove_mob_effect
            cb_update_attributes: 133,     // minecraft:update_attributes
            cb_game_event: 37,             // minecraft:game_event
            cb_player_abilities: 51,       // minecraft:player_abilities
            cb_set_equipment: 105,         // minecraft:set_equipment
            cb_set_experience: 107,        // minecraft:set_experience
            cb_block_changed_ack: 4,       // minecraft:block_changed_ack
            sb_attack: 1,                  // minecraft:attack
            sb_chat_command: 7,            // minecraft:chat_command
            sb_chat: 9,                    // minecraft:chat
            sb_client_command: 12,         // minecraft:client_command
            sb_container_button_click: 17, // minecraft:container_button_click
            sb_container_click: 18,        // minecraft:container_click
            sb_container_close: 19,        // minecraft:container_close
            sb_interact: 26,               // minecraft:interact
            sb_place_recipe: 39,           // minecraft:place_recipe
            sb_player_action: 41,          // minecraft:player_action
            sb_player_abilities: 40,       // minecraft:player_abilities
            sb_player_command: 42,         // minecraft:player_command
            sb_rename_item: 49,            // minecraft:rename_item
            sb_select_trade: 52,           // minecraft:select_trade
            sb_set_carried_item: 54,       // minecraft:set_carried_item
            sb_sign_update: 62,            // minecraft:sign_update
            sb_steer_vehicle: 43,          // minecraft:player_input / steer
            sb_swing: 46,                  // minecraft:punch (swing)
            sb_use_item_on: 66,            // minecraft:use_item_on
            sb_use_item: 67,               // minecraft:use_item
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
        table.cb_entity_position_sync =
            get_id(cb, "minecraft:entity_position_sync", table.cb_entity_position_sync);
        table.cb_move_entity_pos =
            get_id(cb, "minecraft:move_entity_pos", table.cb_move_entity_pos);
        table.cb_open_screen = get_id(cb, "minecraft:open_screen", table.cb_open_screen);
        table.cb_player_combat_kill =
            get_id(cb, "minecraft:player_combat_kill", table.cb_player_combat_kill);
        table.cb_remove_entities =
            get_id(cb, "minecraft:remove_entities", table.cb_remove_entities);
        table.cb_respawn = get_id(cb, "minecraft:respawn", table.cb_respawn);
        table.cb_section_blocks_update = get_id(
            cb,
            "minecraft:section_blocks_update",
            table.cb_section_blocks_update,
        );
        table.cb_set_cursor_item =
            get_id(cb, "minecraft:set_cursor_item", table.cb_set_cursor_item);
        table.cb_set_entity_motion = get_id(
            cb,
            "minecraft:set_entity_motion",
            table.cb_set_entity_motion,
        );
        table.cb_set_health = get_id(cb, "minecraft:set_health", table.cb_set_health);
        table.cb_set_player_inventory =
            get_id(cb, "minecraft:set_player_inventory", table.cb_set_player_inventory);
        table.cb_system_chat = get_id(cb, "minecraft:system_chat", table.cb_system_chat);
        table.cb_teleport_entity =
            get_id(cb, "minecraft:teleport_entity", table.cb_teleport_entity);
        table.cb_update_mob_effect =
            get_id(cb, "minecraft:update_mob_effect", table.cb_update_mob_effect);
        table.cb_remove_mob_effect =
            get_id(cb, "minecraft:remove_mob_effect", table.cb_remove_mob_effect);
        table.cb_update_attributes =
            get_id(cb, "minecraft:update_attributes", table.cb_update_attributes);
        table.cb_game_event = get_id(cb, "minecraft:game_event", table.cb_game_event);
        table.cb_player_abilities =
            get_id(cb, "minecraft:player_abilities", table.cb_player_abilities);
        table.cb_set_equipment =
            get_id(cb, "minecraft:set_equipment", table.cb_set_equipment);
        table.cb_set_experience =
            get_id(cb, "minecraft:set_experience", table.cb_set_experience);
        table.cb_block_changed_ack =
            get_id(cb, "minecraft:block_changed_ack", table.cb_block_changed_ack);

        table.sb_attack = get_id(sb, "minecraft:attack", table.sb_attack);
        table.sb_chat_command = get_id(sb, "minecraft:chat_command", table.sb_chat_command);
        table.sb_chat = get_id(sb, "minecraft:chat", table.sb_chat);
        table.sb_client_command = get_id(sb, "minecraft:client_command", table.sb_client_command);
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
        table.sb_player_abilities =
            get_id(sb, "minecraft:player_abilities", table.sb_player_abilities);
        table.sb_player_command =
            get_id(sb, "minecraft:player_command", table.sb_player_command);
        table.sb_rename_item = get_id(sb, "minecraft:rename_item", table.sb_rename_item);
        table.sb_select_trade = get_id(sb, "minecraft:select_trade", table.sb_select_trade);
        table.sb_set_carried_item =
            get_id(sb, "minecraft:set_carried_item", table.sb_set_carried_item);
        table.sb_sign_update = get_id(sb, "minecraft:sign_update", table.sb_sign_update);
        table.sb_steer_vehicle = get_id(
            sb,
            "minecraft:player_input",
            get_id(sb, "minecraft:steer_vehicle", table.sb_steer_vehicle),
        );
        table.sb_swing = get_id(
            sb,
            "minecraft:punch",
            get_id(sb, "minecraft:swing", table.sb_swing),
        );
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

pub fn encode_client_command_respawn(packets: &PacketTable) -> Vec<u8> {
    let mut p = Vec::new();
    var(packets.sb_client_command, &mut p);
    var(0, &mut p);
    p
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
    pub equipment: HashMap<u8, ItemStack>,
}

pub fn entity_dimensions(entity_type: i32) -> (f64, f64) {
    match entity_type {
        // Chicken
        18 => (0.4, 0.7),
        // Cow, Pig, Sheep
        19 | 82 | 90 => (0.9, 1.4),
        // Spider
        96 => (1.4, 0.9),
        // Iron Golem
        58 => (1.4, 2.7),
        // Creeper
        21 => (0.6, 1.7),
        // Enderman
        29 => (0.6, 2.9),
        // Item entity
        59 => (0.25, 0.25),
        // Boat
        10 => (1.375, 0.5625),
        // Minecart
        62 => (0.98, 0.7),
        // Player / Humanoids (Zombie, Skeleton, etc.) default
        _ => (0.6, 1.8),
    }
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
    pub cursor_item: Option<ItemStack>,
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
            cursor_item: None,
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
            // 0 = custom_data (NBT)
            0 => {
                if let Ok(tag) = r.nbt() {
                    stack.components.push(("custom_data".into(), tag.to_string()));
                } else {
                    exact_stream = false;
                    break;
                }
            }
            // 1 = max_stack_size (VarInt)
            1 => {
                if let Ok(mss) = r.var() {
                    stack.components.push(("max_stack_size".into(), mss.to_string()));
                } else {
                    exact_stream = false;
                    break;
                }
            }
            // 2 = max_damage (VarInt)
            2 => {
                if let Ok(max_dmg) = r.var() {
                    let cur = stack.damage.map_or(0, |(d, _)| d);
                    stack.damage = Some((cur, max_dmg));
                } else {
                    exact_stream = false;
                    break;
                }
            }
            // 3 = max_damage (VarInt, legacy/alt), 4 = damage (VarInt) in 26.3 data_component_type
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
            // 8 = rarity (VarInt)
            8 => {
                if let Ok(rar) = r.var() {
                    stack.components.push(("rarity".into(), rar.to_string()));
                } else {
                    exact_stream = false;
                    break;
                }
            }
            // 9 = enchantments (VarInt count of pairs)
            9 => {
                if let Ok(count) = r.count(256) {
                    for _ in 0..count {
                        if let (Ok(ench_id), Ok(lvl)) = (r.var(), r.var()) {
                            stack.enchantments.push((format!("enchantment_{ench_id}"), lvl));
                        } else {
                            exact_stream = false;
                            break;
                        }
                    }
                } else {
                    exact_stream = false;
                    break;
                }
            }
            // 13 = custom_model_data (VarInt)
            13 => {
                if let Ok(cmd) = r.var() {
                    stack.custom_model_data = Some(cmd);
                } else {
                    exact_stream = false;
                    break;
                }
            }
            // 14 = hide_additional_tooltip, 15 = hide_tooltip (Unit)
            14 => {
                stack.components.push(("hide_additional_tooltip".into(), "true".into()));
            }
            15 => {
                stack.components.push(("hide_tooltip".into(), "true".into()));
            }
            // 16 = repair_cost (VarInt)
            16 => {
                if let Ok(rc) = r.var() {
                    stack.components.push(("repair_cost".into(), rc.to_string()));
                } else {
                    exact_stream = false;
                    break;
                }
            }
            // 17 = creative_slot_lock (Unit)
            17 => {
                stack.components.push(("creative_slot_lock".into(), "true".into()));
            }
            // 18 = glint_override, 27 = enchantment_glint_override (boolean)
            18 | 27 => {
                if let Ok(b) = r.take(1) {
                    stack.components.push(("glint_override".into(), (b[0] != 0).to_string()));
                } else {
                    exact_stream = false;
                    break;
                }
            }
            // 10 = food (nutrition: VarInt, saturation: f32, can_always_eat: bool)
            10 => {
                if let (Ok(nutrition), Ok(sat), Ok(always_eat)) = (r.var(), r.f32(), r.byte()) {
                    stack.components.push(("food_nutrition".into(), nutrition.to_string()));
                    stack.components.push(("food_saturation".into(), sat.to_string()));
                    stack.components.push(("food_always_eat".into(), (always_eat != 0).to_string()));
                } else {
                    exact_stream = false;
                    break;
                }
            }
            // 11 = potion_contents
            11 => {
                if let Ok(has_potion) = r.byte() {
                    if has_potion != 0 {
                        let _ = r.var(); // potion_id
                    }
                    if let Ok(has_color) = r.byte() {
                        if has_color != 0 {
                            let _ = r.i32(); // custom_color
                        }
                    }
                    if let Ok(effect_count) = r.count(64) {
                        for _ in 0..effect_count {
                            let _ = r.var(); // effect_id
                            let _ = r.byte(); // amp
                            let _ = r.var(); // duration
                            let _ = r.byte(); // flags
                        }
                    }
                    stack.components.push(("potion_contents".into(), "present".into()));
                } else {
                    exact_stream = false;
                    break;
                }
            }
            // 19 = intangible_projectile (Unit)
            19 => {
                stack.components.push(("intangible_projectile".into(), "true".into()));
            }
            // 20 = fire_resistant (Unit)
            20 => {
                stack.components.push(("fire_resistant".into(), "true".into()));
            }
            // 23 = tool (rules)
            23 => {
                if let (Ok(default_speed), Ok(dmg_per_block), Ok(rule_count)) = (r.f32(), r.var(), r.count(64)) {
                    for _ in 0..rule_count {
                        let _ = r.var(); // block_set
                        if let Ok(has_speed) = r.byte() {
                            if has_speed != 0 {
                                let _ = r.f32();
                            }
                        }
                        if let Ok(has_correct) = r.byte() {
                            if has_correct != 0 {
                                let _ = r.byte();
                            }
                        }
                    }
                    stack.components.push(("tool_speed".into(), default_speed.to_string()));
                    stack.components.push(("tool_damage".into(), dmg_per_block.to_string()));
                } else {
                    exact_stream = false;
                    break;
                }
            }
            // 24 | 44 = unbreakable (show_in_tooltip: bool)
            24 | 44 => {
                let show = r.byte().unwrap_or(0) != 0;
                stack.components.push(("unbreakable".into(), show.to_string()));
            }
            // 32 = trim (material: VarInt, pattern: VarInt)
            32 => {
                if let (Ok(mat), Ok(pat)) = (r.var(), r.var()) {
                    stack.components.push(("trim_material".into(), mat.to_string()));
                    stack.components.push(("trim_pattern".into(), pat.to_string()));
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
    pub fn main_hand_item(&self) -> Option<&ItemStack> {
        let slot = 36 + (self.selected_hotbar as usize).min(8);
        self.inventory_slots.get(slot).and_then(|opt| opt.as_ref())
    }

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
        let (mut item, exact) = decode_slot_item(r, packets)?;
        if !exact {
            return Err(crate::wire::bad("unrecognized component in container slot"));
        }
        if let Some(ref mut it) = item {
            if !r.0.is_empty() && it.components_raw.is_empty() {
                it.components_raw = r.0.to_vec();
            }
        }
        r.0 = &[];
        if window_id == -1 && slot == -1 {
            self.cursor_item = item.clone();
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
                    self.cursor_item = item.clone();
                    screen.cursor_item = item;
                } else if slot >= 0 {
                    screen.set_slot_authoritative(slot as usize, item);
                }
            }
        }
        Ok(())
    }

    pub fn apply_set_player_inventory(
        &mut self,
        r: &mut Reader<'_>,
        packets: &PacketTable,
    ) -> Result<()> {
        let slot = r.var()?;
        let (mut item, _) = decode_slot_item(r, packets)?;
        if let Some(ref mut it) = item {
            if !r.0.is_empty() && it.components_raw.is_empty() {
                it.components_raw = r.0.to_vec();
            }
        }
        r.0 = &[];
        if slot >= 0 && (slot as usize) < self.inventory_slots.len() {
            self.inventory_slots[slot as usize] = item;
        }
        Ok(())
    }

    pub fn apply_set_cursor_item(
        &mut self,
        r: &mut Reader<'_>,
        packets: &PacketTable,
    ) -> Result<()> {
        let (mut item, _) = decode_slot_item(r, packets)?;
        if let Some(ref mut it) = item {
            if !r.0.is_empty() && it.components_raw.is_empty() {
                it.components_raw = r.0.to_vec();
            }
        }
        r.0 = &[];
        self.cursor_item = item.clone();
        if let Some(screen) = &mut self.open_screen {
            screen.cursor_item = item;
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
        for slot_ref in staged_slots.iter_mut() {
            let (stack, exact) = decode_slot_item(r, packets)?;
            if !exact {
                return Err(crate::wire::bad("unrecognized component in container content"));
            }
            *slot_ref = stack;
        }
        let cursor_item = if !r.0.is_empty() {
            let (item, exact) = decode_slot_item(r, packets)?;
            if !exact {
                return Err(crate::wire::bad("unrecognized component in carried item"));
            }
            item
        } else {
            None
        };
        self.cursor_item = cursor_item.clone();
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
        let (width, height) = entity_dimensions(entity_type);
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
                width,
                height,
                equipment: HashMap::new(),
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

    pub fn apply_move_entity_pos(&mut self, r: &mut Reader<'_>) -> Result<()> {
        let entity_id = r.var()?;
        let dx = i16::from_be_bytes(r.take(2)?.try_into()?) as f64 / 4096.0;
        let dy = i16::from_be_bytes(r.take(2)?.try_into()?) as f64 / 4096.0;
        let dz = i16::from_be_bytes(r.take(2)?.try_into()?) as f64 / 4096.0;
        let _on_ground = r.byte()? != 0;
        if let Some(ent) = self.entities.get_mut(&entity_id) {
            ent.pos[0] += dx;
            ent.pos[1] += dy;
            ent.pos[2] += dz;
        }
        Ok(())
    }

    pub fn apply_set_entity_motion(&mut self, r: &mut Reader<'_>) -> Result<(i32, [f64; 3])> {
        let entity_id = r.var()?;
        let vx = i16::from_be_bytes(r.take(2)?.try_into()?) as f64 / 8000.0;
        let vy = i16::from_be_bytes(r.take(2)?.try_into()?) as f64 / 8000.0;
        let vz = i16::from_be_bytes(r.take(2)?.try_into()?) as f64 / 8000.0;
        if let Some(ent) = self.entities.get_mut(&entity_id) {
            ent.velocity = [vx, vy, vz];
        }
        Ok((entity_id, [vx, vy, vz]))
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

    pub fn apply_update_mob_effect(&mut self, r: &mut Reader<'_>) -> Result<(i32, i32, u8)> {
        let entity_id = r.var()?;
        let effect_id = r.var()?;
        let amplifier = r.byte()?;
        let _duration = r.var()?;
        let _flags = r.byte()?;
        self.push_event(format!("mob_effect_updated:entity={entity_id},effect={effect_id},amp={amplifier}"));
        Ok((entity_id, effect_id, amplifier))
    }

    pub fn apply_remove_mob_effect(&mut self, r: &mut Reader<'_>) -> Result<(i32, i32)> {
        let entity_id = r.var()?;
        let effect_id = r.var()?;
        self.push_event(format!("mob_effect_removed:entity={entity_id},effect={effect_id}"));
        Ok((entity_id, effect_id))
    }

    pub fn apply_update_attributes(&mut self, r: &mut Reader<'_>) -> Result<(i32, Vec<(String, f64)>)> {
        let entity_id = r.var()?;
        let count = r.count(256)?;
        let mut attrs = Vec::with_capacity(count);
        for _ in 0..count {
            let key = r.string()?;
            let base = r.f64()?;
            let mod_count = r.count(256)?;
            for _ in 0..mod_count {
                let _uuid = r.take(16)?;
                let _amount = r.f64()?;
                let _operation = r.byte()?;
            }
            attrs.push((key, base));
        }
        Ok((entity_id, attrs))
    }

    pub fn apply_game_event(&mut self, r: &mut Reader<'_>) -> Result<(u8, f32)> {
        let event = r.byte()?;
        let value = r.f32()?;
        if event == 3 {
            self.game_mode = value as u8;
            self.push_event(format!("game_mode_changed:{}", self.game_mode));
        }
        Ok((event, value))
    }

    pub fn apply_player_abilities(&mut self, r: &mut Reader<'_>) -> Result<(bool, f32)> {
        let flags = r.byte()?;
        let flying = (flags & 2) != 0;
        let fly_speed = r.f32()?;
        let _walk_speed = r.f32()?;
        self.push_event(format!("player_abilities:flying={flying},fly_speed={fly_speed}"));
        Ok((flying, fly_speed))
    }

    pub fn apply_set_experience(&mut self, r: &mut Reader<'_>) -> Result<()> {
        self.xp_progress = r.f32()?;
        self.xp_level = r.var()?;
        let _total = r.var()?;
        self.push_event(format!("experience_updated:level={},progress={}", self.xp_level, self.xp_progress));
        Ok(())
    }

    pub fn apply_set_equipment(&mut self, r: &mut Reader<'_>, packets: &PacketTable) -> Result<()> {
        let entity_id = r.var()?;
        loop {
            let raw_slot = r.byte()?;
            let slot = raw_slot & 0x7f;
            let (item, _) = decode_slot_item(r, packets)?;
            if let Some(ent) = self.entities.get_mut(&entity_id) {
                if let Some(it) = item {
                    ent.equipment.insert(slot, it);
                } else {
                    ent.equipment.remove(&slot);
                }
            }
            if (raw_slot & 0x80) == 0 {
                break;
            }
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
        SubTickEvent::PerformRespawn => {
            var(0, &mut out);
            Some((packets.sb_client_command, out))
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
        SubTickEvent::SteerVehicle {
            left_right,
            forward_backward,
            jumping,
            unmount,
        } => {
            out.extend(left_right.to_be_bytes());
            out.extend(forward_backward.to_be_bytes());
            let mut flags = 0u8;
            if *jumping {
                flags |= 1;
            }
            if *unmount {
                flags |= 2;
            }
            out.push(flags);
            Some((packets.sb_steer_vehicle, out))
        }
        SubTickEvent::StartFallFlying => {
            var(0, &mut out); // entity_id = 0 for player
            var(8, &mut out); // action = 8: start fall flying
            var(0, &mut out); // jump_boost_percentage
            Some((packets.sb_player_command, out))
        }
        SubTickEvent::ToggleCreativeFlight { flying } => {
            out.push(if *flying { 2 } else { 0 }); // bit 1 = is flying
            Some((packets.sb_player_abilities, out))
        }
        SubTickEvent::RequestTabComplete { transaction_id, input } => {
            var(*transaction_id, &mut out);
            string(input, &mut out);
            Some((packets.sb_chat_command, out))
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
        assert_eq!(click_id, 18);
    }

    #[test]
    fn container_set_content_preserves_authoritative_server_state_id_and_cursor() {
        let mut table = PlayerStateTable {
            open_screen: Some(ScreenState::container(2, 1, 1, "Chest")),
            ..Default::default()
        };
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

    #[test]
    fn set_player_inventory_and_cursor_item_packets_update_state() {
        let mut table = PlayerStateTable::default();
        let mut packets = PacketTable::default();
        packets.item_names.insert(1, "minecraft:stone".to_string());
        packets.item_names.insert(250, "minecraft:iron_sword".to_string());

        // Test SetPlayerInventory (slot 36, stone x64)
        let mut raw_inv = Vec::new();
        var(36, &mut raw_inv); // slot 36
        var(64, &mut raw_inv); // count 64
        var(1, &mut raw_inv);  // item_id 1 (stone)
        var(0, &mut raw_inv);  // 0 added components
        var(0, &mut raw_inv);  // 0 removed components
        let mut r1 = Reader(&raw_inv);
        table.apply_set_player_inventory(&mut r1, &packets).unwrap();

        let slot_36 = table.inventory_slots[36].as_ref().unwrap();
        assert_eq!(slot_36.id, "minecraft:stone");
        assert_eq!(slot_36.count, 64);

        // Test SetCursorItem with components (iron_sword x1, damage component 4 with dmg=15)
        let mut raw_cursor = Vec::new();
        var(1, &mut raw_cursor);   // count 1
        var(250, &mut raw_cursor); // item_id 250 (iron sword)
        var(1, &mut raw_cursor);   // 1 added component
        var(0, &mut raw_cursor);   // 0 removed components
        var(4, &mut raw_cursor);   // component id 4 = damage
        var(15, &mut raw_cursor);  // damage value 15
        let mut r2 = Reader(&raw_cursor);
        table.apply_set_cursor_item(&mut r2, &packets).unwrap();

        let cursor = table.cursor_item.as_ref().unwrap();
        assert_eq!(cursor.id, "minecraft:iron_sword");
        assert_eq!(cursor.count, 1);
        assert_eq!(cursor.damage, Some((15, 0)));
    }

    #[test]
    fn container_set_slot_with_unknown_component_is_rejected() {
        let mut table = PlayerStateTable::default();
        let mut packets = PacketTable::default();
        packets.item_names.insert(100, "minecraft:diamond_sword".to_string());

        // ContainerSetSlot: window 0, state_id 5, slot 0
        let mut raw = Vec::new();
        var(0, &mut raw);          // window 0
        var(5, &mut raw);          // state_id 5
        raw.extend(0i16.to_be_bytes()); // slot 0
        var(1, &mut raw);          // count 1
        var(100, &mut raw);        // item 100
        var(1, &mut raw);          // 1 added component
        var(0, &mut raw);          // 0 removed
        var(999, &mut raw);        // unknown component id 999
        raw.extend(&[0xde, 0xad, 0xbe, 0xef]); // arbitrary trailing bytes

        let mut reader = Reader(&raw);
        assert!(table.apply_container_set_slot(&mut reader, &packets).is_err());
        assert!(table.inventory_slots[0].is_none());
    }

    #[test]
    fn container_set_content_with_unknown_component_preserves_subsequent_slots() {
        let mut table = PlayerStateTable {
            open_screen: Some(ScreenState::container(1, 1, 1, "Chest")),
            ..Default::default()
        };
        // Pre-populate slot 1 with valuable item that must not be truncated to None
        table.open_screen.as_mut().unwrap().slots[1] = Some(ItemStack::new("minecraft:diamond", 64));
        let packets = PacketTable::default();

        // Build ContainerSetContent packet: window 1, state 10, 2 slots
        let mut raw = Vec::new();
        var(1, &mut raw);  // window 1
        var(10, &mut raw); // state 10
        var(2, &mut raw);  // 2 slots
        // Slot 0 has an unknown component 999
        var(1, &mut raw);   // count 1
        var(50, &mut raw);  // item 50
        var(1, &mut raw);   // 1 added component
        var(0, &mut raw);   // 0 removed
        var(999, &mut raw); // unknown component id 999

        let mut reader = Reader(&raw);
        assert!(table.apply_container_set_content(&mut reader, &packets).is_err());

        let screen = table.open_screen.as_ref().unwrap();
        assert_eq!(screen.state_id, 1);
        // Slot 1 must still be diamond x64, preserved!
        assert_eq!(screen.slots[1].as_ref().unwrap().id, "minecraft:diamond");
        assert_eq!(screen.slots[1].as_ref().unwrap().count, 64);
    }
}
