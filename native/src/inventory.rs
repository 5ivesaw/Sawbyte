//! Native 26.3 inventory, container screen state, entity table, player vitals,
//! and serverbound interaction/GUI packet encoders.
#![allow(dead_code)]

use crate::wire::{string, var};
use sawbyte_core::events::{BlockFace, ClickMode, EntityInteraction, Hand, SubTickEvent};
use sawbyte_core::screen::{ItemStack, ScreenState};
use std::collections::BTreeMap;

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
}

impl Default for PlayerStateTable {
    fn default() -> Self {
        Self {
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
        }
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
    let mut out = Vec::new();
    match event {
        SubTickEvent::SelectHotbar { slot } => {
            out.extend(u16::from(*slot).to_be_bytes());
            Some((52, out)) // ServerboundSetCarriedItemPacket
        }
        SubTickEvent::StartDestroyBlock {
            pos,
            face,
            sequence,
        } => {
            var(0, &mut out); // Action::START_DESTROY_BLOCK
            out.extend(pack_block_pos(*pos).to_be_bytes());
            out.push(*face as u8);
            var(*sequence, &mut out);
            Some((40, out)) // ServerboundPlayerActionPacket
        }
        SubTickEvent::AbortDestroyBlock { pos, face } => {
            var(1, &mut out); // Action::ABORT_DESTROY_BLOCK
            out.extend(pack_block_pos(*pos).to_be_bytes());
            out.push(*face as u8);
            var(0, &mut out);
            Some((40, out))
        }
        SubTickEvent::StopDestroyBlock {
            pos,
            face,
            sequence,
        } => {
            var(2, &mut out); // Action::STOP_DESTROY_BLOCK
            out.extend(pack_block_pos(*pos).to_be_bytes());
            out.push(*face as u8);
            var(*sequence, &mut out);
            Some((40, out))
        }
        SubTickEvent::DropItem { entire_stack } => {
            var(if *entire_stack { 3 } else { 4 }, &mut out);
            out.extend(0u64.to_be_bytes());
            out.push(BlockFace::Down as u8);
            var(0, &mut out);
            Some((40, out))
        }
        SubTickEvent::ReleaseUseItem => {
            var(5, &mut out); // Action::RELEASE_USE_ITEM
            out.extend(0u64.to_be_bytes());
            out.push(BlockFace::Down as u8);
            var(0, &mut out);
            Some((40, out))
        }
        SubTickEvent::SwapOffhand => {
            var(6, &mut out); // Action::SWAP_ITEM_WITH_OFFHAND
            out.extend(0u64.to_be_bytes());
            out.push(BlockFace::Down as u8);
            var(0, &mut out);
            Some((40, out))
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
            out.push(0); // world border hit = false
            var(*sequence, &mut out);
            Some((63, out)) // ServerboundUseItemOnPacket
        }
        SubTickEvent::UseItem { hand, sequence } => {
            var(if *hand == Hand::MainHand { 0 } else { 1 }, &mut out);
            var(*sequence, &mut out);
            out.extend(0.0f32.to_be_bytes());
            out.extend(0.0f32.to_be_bytes());
            Some((64, out)) // ServerboundUseItemPacket
        }
        SubTickEvent::SwipeArm { hand } => {
            var(if *hand == Hand::MainHand { 0 } else { 1 }, &mut out);
            Some((60, out)) // ServerboundSwingPacket
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
            Some((24, out)) // ServerboundInteractPacket
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
            var(0, &mut out); // 0 changed slots in simplified delta
            out.push(0); // empty carried item hashed stack
            Some((17, out)) // ServerboundContainerClickPacket
        }
        SubTickEvent::ContainerButtonClick {
            window_id,
            button_id,
        } => {
            var(*window_id, &mut out);
            var(*button_id, &mut out);
            Some((16, out)) // ServerboundContainerButtonClickPacket
        }
        SubTickEvent::CloseContainer { window_id } => {
            var(*window_id, &mut out);
            Some((18, out)) // ServerboundContainerClosePacket
        }
        SubTickEvent::RenameItem { name } => {
            string(name, &mut out);
            Some((48, out)) // ServerboundRenameItemPacket
        }
        SubTickEvent::SelectTrade { slot } => {
            var(i32::from(*slot), &mut out);
            Some((51, out)) // ServerboundSelectTradePacket
        }
        SubTickEvent::SendCommand { command } => {
            string(command, &mut out);
            Some((6, out)) // ServerboundChatCommandPacket
        }
        SubTickEvent::SendChat { message } => {
            string(message, &mut out);
            out.extend(0u64.to_be_bytes()); // timestamp
            out.extend(0u64.to_be_bytes()); // salt
            out.push(0); // no signature
            var(0, &mut out); // message count
            out.extend([0u8; 3]); // fixed bitset(20)
            out.push(0); // checksum
            Some((8, out)) // ServerboundChatPacket
        }
        SubTickEvent::UpdateSign { pos, front, lines } => {
            out.extend(pack_block_pos(*pos).to_be_bytes());
            out.push(u8::from(*front));
            for line in lines {
                string(line, &mut out);
            }
            Some((59, out)) // ServerboundSignUpdatePacket
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
