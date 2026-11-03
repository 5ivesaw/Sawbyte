//! Ordered sub-tick discrete events, interaction primitives, and out-of-game client menu/settings actions.
//! Continuous button states live in [`crate::InputFrame`]; discrete click edges, inventory transactions,
//! world interactions, text edits, and client lifecycle actions live here so they are never coalesced away.

use crate::InputFrame;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MouseButton {
    Left,
    Right,
    Middle,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Hand {
    MainHand,
    OffHand,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum BlockFace {
    Down = 0,
    Up = 1,
    North = 2,
    South = 3,
    West = 4,
    East = 5,
}

impl BlockFace {
    pub fn normal(self) -> [i32; 3] {
        match self {
            Self::Down => [0, -1, 0],
            Self::Up => [0, 1, 0],
            Self::North => [0, 0, -1],
            Self::South => [0, 0, 1],
            Self::West => [-1, 0, 0],
            Self::East => [1, 0, 0],
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum EntityInteraction {
    Attack,
    Interact { hand: Hand },
    InteractAt { hand: Hand, target: [f32; 3] },
}

/// All 7 Minecraft `ClickType` modes used in `ServerboundContainerClickPacket`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum ClickMode {
    Pickup = 0,
    QuickMove = 1,
    Swap = 2,
    Clone = 3,
    Throw = 4,
    QuickCraft = 5,
    PickupAll = 6,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DialogResponse {
    Accept,
    Decline,
    CustomPayload { channel: String, data: Vec<u8> },
}

/// Out-of-game and pause-menu navigation targets controllable by scripts and MCP agents.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ClientMenuTarget {
    MainMenu,
    SingleplayerMenu,
    CreateWorldMenu { world_name: String, seed: String },
    MultiplayerMenu,
    DirectConnect { address: String },
    ConnectServerEntry { index: usize },
    OptionsMenu,
    VideoSettings,
    AudioSettings,
    ControlsSettings,
    KeybindsMenu,
    ResourcePacksMenu,
    LanguageMenu,
    PauseMenu,
    ResumeGame,
    RespawnPlayer,
    DisconnectToTitle,
    QuitClient,
}

/// Discrete sub-tick event executed in strict submission order within a single client tick.
#[derive(Clone, Debug, PartialEq)]
pub enum SubTickEvent {
    MouseClick {
        button: MouseButton,
        down: bool,
    },
    SelectHotbar {
        slot: u8,
    },
    SwapOffhand,
    DropItem {
        entire_stack: bool,
    },
    StartDestroyBlock {
        pos: [i32; 3],
        face: BlockFace,
        sequence: i32,
    },
    StopDestroyBlock {
        pos: [i32; 3],
        face: BlockFace,
        sequence: i32,
    },
    AbortDestroyBlock {
        pos: [i32; 3],
        face: BlockFace,
    },
    UseItemOn {
        hand: Hand,
        pos: [i32; 3],
        face: BlockFace,
        hit: [f32; 3],
        inside_block: bool,
        sequence: i32,
    },
    UseItem {
        hand: Hand,
        sequence: i32,
    },
    ReleaseUseItem,
    InteractEntity {
        entity_id: i32,
        interaction: EntityInteraction,
        sneaking: bool,
    },
    SwipeArm {
        hand: Hand,
    },
    ContainerClick {
        window_id: i32,
        state_id: i32,
        slot: i16,
        button: i8,
        mode: ClickMode,
    },
    ContainerButtonClick {
        window_id: i32,
        button_id: i32,
    },
    CloseContainer {
        window_id: i32,
    },
    PlaceRecipe {
        window_id: i32,
        recipe: String,
        make_all: bool,
    },
    RenameItem {
        name: String,
    },
    SelectTrade {
        slot: u16,
    },
    SetBeacon {
        primary_effect: Option<i32>,
        secondary_effect: Option<i32>,
    },
    UpdateSign {
        pos: [i32; 3],
        front: bool,
        lines: [String; 4],
    },
    EditBook {
        slot: u8,
        pages: Vec<String>,
        title: Option<String>,
    },
    TakeLecternBook {
        window_id: i32,
    },
    SendChat {
        message: String,
    },
    SendCommand {
        command: String,
    },
    RequestTabComplete {
        transaction_id: i32,
        input: String,
    },
    ServerDialog {
        dialog_id: String,
        response: DialogResponse,
    },
    SteerVehicle {
        left_right: f32,
        forward_backward: f32,
        jumping: bool,
        unmount: bool,
    },
    StartFallFlying,
    ToggleCreativeFlight {
        flying: bool,
    },
    NavigateClientMenu {
        target: ClientMenuTarget,
    },
    UpdateClientSetting {
        key: String,
        value: String,
    },
    ManualHandoff {
        reason: String,
    },
}

impl SubTickEvent {
    pub fn valid(&self) -> bool {
        match self {
            Self::SelectHotbar { slot } => *slot < 9,
            Self::UseItemOn { hit, .. } => {
                hit.iter().all(|v| v.is_finite() && (0.0..=1.0).contains(v))
            }
            Self::InteractEntity {
                interaction: EntityInteraction::InteractAt { target, .. },
                ..
            } => target.iter().all(|v| v.is_finite() && v.abs() <= 8.0),
            Self::ContainerClick {
                slot,
                button,
                mode,
                window_id,
                state_id,
            } => {
                *window_id >= 0
                    && *state_id >= 0
                    && (*slot >= 0 || *slot == -999 || *slot == -1)
                    && (*button >= 0 && *button <= 40)
                    && matches!(
                        mode,
                        ClickMode::Pickup
                            | ClickMode::QuickMove
                            | ClickMode::Swap
                            | ClickMode::Clone
                            | ClickMode::Throw
                            | ClickMode::QuickCraft
                            | ClickMode::PickupAll
                    )
            }
            Self::RenameItem { name } => {
                name.chars().count() <= 50 && !name.chars().any(char::is_control)
            }
            Self::UpdateSign { lines, .. } => lines
                .iter()
                .all(|l| l.chars().count() <= 384 && !l.chars().any(char::is_control)),
            Self::EditBook { slot, pages, title } => {
                *slot < 45
                    && pages.len() <= 100
                    && pages.iter().all(|p| p.chars().count() <= 1024)
                    && title.as_ref().is_none_or(|t| t.chars().count() <= 32)
            }
            Self::SendChat { message } => {
                !message.is_empty()
                    && message.chars().count() <= 256
                    && !message.chars().any(char::is_control)
            }
            Self::SendCommand { command } => {
                !command.is_empty()
                    && command.chars().count() <= 256
                    && !command.chars().any(char::is_control)
            }
            Self::SteerVehicle {
                left_right,
                forward_backward,
                ..
            } => left_right.is_finite() && forward_backward.is_finite(),
            Self::UpdateClientSetting { key, value } => {
                !key.is_empty()
                    && key.len() <= 64
                    && value.len() <= 256
                    && !key.chars().any(char::is_control)
                    && !value.chars().any(char::is_control)
            }
            _ => true,
        }
    }
}

/// Complete tick payload combining continuous movement/camera state with ordered discrete events.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct OrderedTickFrame {
    pub continuous: InputFrame,
    pub discrete: Vec<SubTickEvent>,
}

/// Bounded queue for ordered sub-tick discrete events alongside continuous tick scheduling.
pub struct DiscreteEventQueue {
    events: std::collections::VecDeque<(u64, SubTickEvent)>,
    capacity: usize,
    last_tick: Option<u64>,
    sequence_counter: i32,
}

impl DiscreteEventQueue {
    pub fn new(capacity: usize) -> Self {
        Self {
            events: std::collections::VecDeque::new(),
            capacity,
            last_tick: None,
            sequence_counter: 1,
        }
    }

    pub fn next_sequence(&mut self) -> i32 {
        let current = self.sequence_counter;
        self.sequence_counter = self.sequence_counter.wrapping_add(1).max(1);
        current
    }

    pub fn pending(&self) -> usize {
        self.events.len()
    }

    pub fn submit(&mut self, batch: &[(u64, SubTickEvent)]) -> Result<(), &'static str> {
        if batch.len() > self.capacity.saturating_sub(self.events.len()) {
            return Err("discrete event queue full");
        }
        let mut previous = self.events.back().map(|(t, _)| *t);
        for (tick, event) in batch {
            if self.last_tick.is_some_and(|last| *tick <= last)
                || previous.is_some_and(|prev| *tick < prev)
            {
                return Err("discrete event tick out of order");
            }
            if !event.valid() {
                return Err("invalid discrete event parameters");
            }
            previous = Some(*tick);
        }
        self.events.extend(batch.iter().cloned());
        Ok(())
    }

    pub fn drain_tick(&mut self, tick: u64) -> Result<Vec<SubTickEvent>, &'static str> {
        if self.last_tick.is_some_and(|last| tick <= last) {
            return Err("tick regression in discrete event queue");
        }
        self.last_tick = Some(tick);
        let mut out = Vec::new();
        while self.events.front().is_some_and(|(at, _)| *at <= tick) {
            if let Some((_, ev)) = self.events.pop_front() {
                out.push(ev);
            }
        }
        Ok(out)
    }

    pub fn clear(&mut self) {
        self.events.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn preserves_subtick_click_edges_and_rejects_invalid_events() {
        let mut q = DiscreteEventQueue::new(16);
        let seq = q.next_sequence();
        q.submit(&[
            (0, SubTickEvent::SelectHotbar { slot: 2 }),
            (
                0,
                SubTickEvent::MouseClick {
                    button: MouseButton::Left,
                    down: true,
                },
            ),
            (
                0,
                SubTickEvent::MouseClick {
                    button: MouseButton::Left,
                    down: false,
                },
            ),
            (
                1,
                SubTickEvent::UseItemOn {
                    hand: Hand::MainHand,
                    pos: [10, -60, 4],
                    face: BlockFace::Up,
                    hit: [0.5, 1.0, 0.5],
                    inside_block: false,
                    sequence: seq,
                },
            ),
        ])
        .unwrap();

        let tick0 = q.drain_tick(0).unwrap();
        assert_eq!(tick0.len(), 3);
        assert_eq!(tick0[0], SubTickEvent::SelectHotbar { slot: 2 });
        assert_eq!(
            tick0[1],
            SubTickEvent::MouseClick {
                button: MouseButton::Left,
                down: true
            }
        );
        assert_eq!(
            tick0[2],
            SubTickEvent::MouseClick {
                button: MouseButton::Left,
                down: false
            }
        );

        let tick1 = q.drain_tick(1).unwrap();
        assert_eq!(tick1.len(), 1);

        // Out-of-range hotbar slot or control chars in chat are rejected atomically.
        assert!(q
            .submit(&[(2, SubTickEvent::SelectHotbar { slot: 9 })])
            .is_err());
        assert!(q
            .submit(&[(
                2,
                SubTickEvent::SendChat {
                    message: "bad\nchat".into()
                }
            )])
            .is_err());
    }
}
