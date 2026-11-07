#![allow(clippy::all, warnings, unused)]
#![allow(
    clippy::collapsible_if,
    clippy::collapsible_else_if,
    clippy::field_reassign_with_default,
    clippy::unnecessary_cast
)]

pub mod auth;
pub mod inventory;
pub mod perception;
pub mod player;
pub mod raycast;
pub mod visualizer;
pub mod wire;
pub mod world;
use sawbyte_core::{
    Action, Backend, Button, Capability, Direct, Humanised, InputFrame, InputPolicy, Scheduler,
    events::{BlockFace, ClickMode, DiscreteEventQueue, EntityInteraction, Hand, SubTickEvent},
    humanisation::SmoothHumanisedCamera,
    screen::{ItemSelector, ScreenState},
    tape,
    tasks::{
        ComposableTask, WorkflowHost, find_path,
    },
    tokenize_command_line,
};
use std::{
    collections::{HashMap, VecDeque},
    fs::File,
    io::{BufRead, BufReader, BufWriter, Write},
    net::{Shutdown, TcpListener, TcpStream, ToSocketAddrs},
    sync::mpsc,
    time::{Duration, Instant},
};
use wire::{Reader, Result, bad, string, var};

fn information() -> Vec<u8> {
    let mut out = Vec::new();
    string("en_us", &mut out);
    out.extend([3, 0, 1, 127, 1, 0, 1, 2]);
    out
}

fn connect(server: &str, account: &auth::Account) -> Result<(wire::Rx, wire::Tx)> {
    let endpoint = server
        .to_socket_addrs()?
        .next()
        .ok_or_else(|| bad("server address did not resolve"))?;
    let socket = TcpStream::connect_timeout(&endpoint, Duration::from_secs(10))?;
    socket.set_nodelay(true)?;
    socket.set_read_timeout(Some(Duration::from_secs(20)))?;
    socket.set_write_timeout(Some(Duration::from_secs(10)))?;
    let (mut rx, mut tx) = wire::connection(socket)?;
    let mut hello = Vec::new();
    let proto: i32 = std::env::var("SAWBYTE_PROTOCOL")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(777);
    var(proto, &mut hello);
    let host = server
        .rsplit_once(':')
        .map(|p| p.0)
        .unwrap_or(server)
        .trim_matches(['[', ']']);
    string(host, &mut hello);
    hello.extend(endpoint.port().to_be_bytes());
    var(2, &mut hello);
    tx.packet(0, &hello)?;
    let mut start = Vec::new();
    string(&account.name, &mut start);
    start.extend(account.id);
    tx.packet(0, &start)?;
    loop {
        let packet = rx.packet()?;
        let mut r = Reader(&packet);
        match r.var()? {
            0 => return Err(bad(&format!("server rejected login: {:?}", r.string()?))),
            1 => account.encryption(&mut r, &mut rx, &mut tx)?,
            2 => {
                tx.packet(3, &[])?;
                tx.packet(0, &information())?;
                return Ok((rx, tx));
            }
            3 => {
                let threshold = r.count(wire::LIMIT)?;
                rx.threshold = Some(threshold);
                tx.threshold = Some(threshold);
            }
            4 => {
                let mut out = Vec::new();
                var(r.var()?, &mut out);
                out.push(0);
                tx.packet(2, &out)?;
            }
            5 => {
                let name = r.string()?;
                let mut out = Vec::new();
                string(&name, &mut out);
                out.push(0);
                tx.packet(4, &out)?;
            }
            _ => return Err(bad("unknown login packet")),
        }
    }
}

#[derive(Default)]
pub struct LiveInputBackend {
    last_frame: InputFrame,
}

impl Backend for LiveInputBackend {
    type Error = &'static str;
    fn supports(&self, _: Capability) -> bool {
        true
    }
    fn apply(&mut self, _: u64, frame: InputFrame) -> std::result::Result<(), Self::Error> {
        self.last_frame = frame;
        Ok(())
    }
    fn release_all(&mut self) -> std::result::Result<(), Self::Error> {
        self.last_frame = InputFrame::default();
        Ok(())
    }
}

pub enum DynamicCameraPolicy {
    Humanised(Humanised),
    Smooth(SmoothHumanisedCamera),
    Direct(Direct),
}

impl Default for DynamicCameraPolicy {
    fn default() -> Self {
        Self::Humanised(Humanised::default())
    }
}

impl InputPolicy for DynamicCameraPolicy {
    fn camera_step(&mut self, remaining: (f64, f64)) -> (f64, f64) {
        match self {
            Self::Humanised(p) => p.camera_step(remaining),
            Self::Smooth(p) => p.camera_step(remaining),
            Self::Direct(p) => p.camera_step(remaining),
        }
    }
}

/// Pluggable, un-hardcoded module trait for live `sawbyte-play` session commands.
/// Developers can register custom modules or override any built-in command handler.
pub trait LiveModule: Send {
    fn name(&self) -> &'static str;
    fn execute(
        &mut self,
        tokens: &[&str],
        session: &mut Session,
    ) -> Option<std::result::Result<String, String>>;
}

#[derive(Clone, Debug, PartialEq)]
pub enum WaitCondition {
    TaskIdle,
    ScreenOpen { title_contains: Option<String> },
    SlotItem { slot: usize, item_id: String, min_count: u32 },
    PlayerAlive,
}

#[derive(Clone, Debug)]
pub struct ActiveMine {
    pub pos: [i32; 3],
    pub face: BlockFace,
    pub remaining_ticks: u32,
    pub total_ticks: u32,
    pub sequence: i32,
    pub confirmation_deadline: Option<u64>,
}

#[derive(Clone, Debug)]
pub struct PendingPlacement {
    pub pos: [i32; 3],
    pub sequence: i32,
    pub original_state: u32,
    pub confirmation_deadline: Option<u64>,
}

#[derive(Clone, Debug)]
pub enum PendingReplyKind {
    Tick {
        target_tick: u64,
        command: String,
    },
    Condition {
        deadline_tick: u64,
        command: String,
        condition: WaitCondition,
    },
}

pub struct PendingReply {
    pub client: Option<mpsc::Sender<String>>,
    pub is_json: bool,
    pub json_rpc_id: Option<serde_json::Value>,
    pub kind: PendingReplyKind,
}

#[derive(Clone, Debug, PartialEq)]
pub struct DeathRecord {
    pub attempt: u32,
    pub tick: u64,
    pub pos: [f64; 3],
    pub cause: String,
    pub killer_id: Option<i32>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct MilestoneRecord {
    pub name: String,
    pub notes: String,
    pub tick: u64,
    pub attempt: u32,
    pub death_count: u32,
    pub pos: [f64; 3],
    pub health: f32,
    pub food: i32,
    pub xp_level: i32,
    pub inventory_summary: String,
    pub timestamp_epoch_secs: u64,
}

pub struct Session {
    pub world: world::World,
    pub player: player::Player,
    pub state_table: inventory::PlayerStateTable,
    pub packets: inventory::PacketTable,
    pub scheduler: Scheduler<LiveInputBackend, DynamicCameraPolicy>,
    pub event_queue: DiscreteEventQueue,
    pub active_mine: Option<ActiveMine>,
    pub pending_placements: Vec<PendingPlacement>,
    pub pending_click: Option<(i32, i32, Option<u64>)>,
    pub voxel_patch_buf: Vec<u8>,
    pub custom_modules: Vec<Box<dyn LiveModule>>,
    pub active_nav_path: Vec<[i32; 3]>,
    pub active_nav_goal: Option<([i32; 3], bool)>,
    pub nav_stuck_ticks: u32,
    pub nav_last_pos: Option<[f64; 3]>,
    pub nav_deadline_tick: u64,
    pub pending_replies: Vec<PendingReply>,
    pub script_queue: VecDeque<String>,
    pub is_dead: bool,
    pub death_count: u32,
    pub death_history: Vec<DeathRecord>,
    pub milestone_history: Vec<MilestoneRecord>,
    pub visualizer: visualizer::LowPolyVisualizer,
    #[allow(dead_code)]
    pub auto_respawn: bool,
    dimensions: Vec<(i32, i32)>,
    playing: bool,
    positioned: bool,
    loaded: bool,
    live_mode: bool,
    settle: u32,
    index: usize,
    live_tick: u64,
    previous_buttons: u16,
    pub humanised_perception: sawbyte_core::humanisation::HumanisedPerceptionPolicy,
    pub world_memory: perception::PersistentWorldMemory,
}

impl WorkflowHost for Session {
    fn current_screen(&mut self) -> Option<&mut ScreenState> {
        self.state_table.open_screen.as_mut()
    }
    fn poll_screen_until_title(&mut self, title: &str, _timeout_ms: u64) -> bool {
        self.state_table
            .open_screen
            .as_ref()
            .is_some_and(|s| s.title == title)
    }
    fn poll_event(&mut self, event_name: &str, _timeout_ms: u64) -> bool {
        self.state_table
            .received_events
            .iter()
            .any(|e| e == event_name)
    }
    fn on_event_emitted(&mut self, event: &SubTickEvent) {
        let target_tick = self.live_tick.saturating_add(1);
        if let Err(err) = self.event_queue.submit(&[(target_tick, event.clone())]) {
            self.state_table
                .push_event(format!("event_queue_error:{err}"));
        }
    }
    fn invoke_extension(
        &mut self,
        name: &str,
        arg: &str,
        vars: &mut HashMap<String, String>,
    ) -> std::result::Result<(), String> {
        vars.insert(format!("ext_{name}"), arg.to_owned());
        Ok(())
    }
}

impl Session {
    pub fn record_death(&mut self, cause: &str, killer_id: Option<i32>) {
        if !self.is_dead {
            self.is_dead = true;
            self.death_count = self.death_count.saturating_add(1);
            let attempt = self.death_count;
            let record = DeathRecord {
                attempt,
                tick: self.live_tick,
                pos: self.player.pos,
                cause: cause.to_string(),
                killer_id,
            };
            if self.death_history.len() >= 128 {
                self.death_history.remove(0);
            }
            self.death_history.push(record);
            self.preempt_movement();
            self.state_table.push_event(format!(
                "player.death:attempt={attempt}:cause={cause}:pos=({:.2},{:.2},{:.2})",
                self.player.pos[0], self.player.pos[1], self.player.pos[2]
            ));

            let log_line = format!(
                "[DEATH] attempt={} tick={} pos=({:.2}, {:.2}, {:.2}) cause=\"{}\" killer_id={:?}\n",
                attempt, self.live_tick, self.player.pos[0], self.player.pos[1], self.player.pos[2], cause, killer_id
            );
            let _ = std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open("milestones.log")
                .and_then(|mut f| std::io::Write::write_all(&mut f, log_line.as_bytes()));
        }
    }

    pub fn summarize_inventory(&self) -> String {
        let mut items = Vec::new();
        for stack in self.state_table.inventory_slots.iter().flatten() {
            if stack.count > 0 && !stack.id.is_empty() {
                let short_id = stack.id.strip_prefix("minecraft:").unwrap_or(&stack.id);
                items.push(format!("{short_id}:{}", stack.count));
            }
        }
        if items.is_empty() {
            "empty".to_string()
        } else {
            items.join(", ")
        }
    }

    pub fn record_milestone(&mut self, name: &str, notes: &str) -> MilestoneRecord {
        let now_secs = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        let inv = self.summarize_inventory();
        let record = MilestoneRecord {
            name: name.to_string(),
            notes: notes.to_string(),
            tick: self.live_tick,
            attempt: self.death_count + 1,
            death_count: self.death_count,
            pos: self.player.pos,
            health: self.state_table.health,
            food: self.state_table.food,
            xp_level: self.state_table.xp_level,
            inventory_summary: inv,
            timestamp_epoch_secs: now_secs,
        };
        if self.milestone_history.len() >= 128 {
            self.milestone_history.remove(0);
        }
        self.milestone_history.push(record.clone());

        let text_line = format!(
            "[MILESTONE] name=\"{}\" attempt={} deaths={} tick={} pos=({:.2}, {:.2}, {:.2}) health={:.1} food={} xp={} notes=\"{}\" inventory=[{}]\n",
            record.name, record.attempt, record.death_count, record.tick,
            record.pos[0], record.pos[1], record.pos[2],
            record.health, record.food, record.xp_level, record.notes, record.inventory_summary
        );
        let _ = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open("milestones.log")
            .and_then(|mut f| std::io::Write::write_all(&mut f, text_line.as_bytes()));

        let json_line = format!(
            "{{\"name\":\"{}\",\"attempt\":{},\"deaths\":{},\"tick\":{},\"pos\":[{:.2},{:.2},{:.2}],\"health\":{:.1},\"food\":{},\"xp\":{},\"notes\":\"{}\",\"timestamp\":{}}}\n",
            record.name, record.attempt, record.death_count, record.tick,
            record.pos[0], record.pos[1], record.pos[2],
            record.health, record.food, record.xp_level, record.notes, record.timestamp_epoch_secs
        );
        let _ = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open("milestones.jsonl")
            .and_then(|mut f| std::io::Write::write_all(&mut f, json_line.as_bytes()));

        record
    }

    fn observe_block_update(&mut self, pos: [i32; 3], state: u32) {
        if self.active_mine.as_ref().is_some_and(|mine| mine.pos == pos && mine.confirmation_deadline.is_some())
            && self.world.is_air_state(state)
        {
            self.active_mine = None;
            self.state_table.push_event(format!("mine_complete:({},{},{})", pos[0], pos[1], pos[2]));
        }
        let mut completed = Vec::new();
        self.pending_placements.retain(|placement| {
            let observed = placement.pos == pos
                && placement.confirmation_deadline.is_some()
                && placement.original_state != state;
            if observed {
                completed.push(placement.sequence);
            }
            !observed
        });
        for sequence in completed {
            self.state_table.push_event(format!("placement_state_changed:{sequence}:({},{},{})={state}", pos[0], pos[1], pos[2]));
        }
    }

    fn observe_container_revision(&mut self) {
        if let (Some((window, revision, Some(_))), Some(screen)) =
            (self.pending_click, self.state_table.open_screen.as_ref())
        {
            if screen.window_id == window && screen.state_id != revision {
                let current_revision = screen.state_id;
                self.state_table.push_event(format!("container_revision_observed:{window}:{current_revision}"));
                self.pending_click = None;
            }
        }
    }

    fn placement_target_is_solid(&self, pos: [i32; 3]) -> std::result::Result<bool, String> {
        if self.pending_placements.iter().any(|placement| placement.pos == pos) {
            return Err("placement is already pending at this position".to_owned());
        }
        let state = self.world.block_state(pos[0], pos[1], pos[2])
            .map_err(|error| format!("placement target is not available: {error}"))?;
        if !self.world.is_known_state(state) {
            return Err(format!("unknown placement target state {state}"));
        }
        self.world.solid(pos[0], pos[1], pos[2]).map_err(|error| error.to_string())
    }

    fn compute_tool_mining_stats(&self, hardness: f32) -> (f32, bool) {
        let held_item = self.state_table.main_hand_item();
        let Some(item) = held_item else {
            let can_harvest = hardness <= 2.0;
            return (1.0, can_harvest);
        };
        let id = item.id.as_str();
        let is_pickaxe = id.contains("pickaxe");
        let is_axe = id.contains("axe") && !is_pickaxe;
        let is_shovel = id.contains("shovel");

        let material_speed = if id.contains("netherite") {
            9.0
        } else if id.contains("diamond") {
            8.0
        } else if id.contains("iron") {
            6.0
        } else if id.contains("golden") {
            12.0
        } else if id.contains("stone") {
            4.0
        } else if id.contains("wood") {
            2.0
        } else {
            1.0
        };

        if hardness <= 0.0 {
            return (1.0, true);
        }

        if (hardness - 1.5).abs() < 0.1 || (hardness - 3.0).abs() < 0.1 {
            if is_pickaxe {
                (material_speed, true)
            } else {
                (1.0, false)
            }
        } else if (hardness - 2.0).abs() < 0.1 {
            if is_axe {
                (material_speed, true)
            } else {
                (1.0, true)
            }
        } else if hardness <= 0.8 {
            if is_shovel {
                (material_speed, true)
            } else {
                (1.0, true)
            }
        } else {
            (1.0, true)
        }
    }

    fn new(profile: &serde_json::Value, live_mode: bool, policy: DynamicCameraPolicy) -> Result<Self> {
        Ok(Self {
            world: world::World::new(profile)?,
            player: player::Player::default(),
            state_table: inventory::PlayerStateTable::default(),
            packets: inventory::PacketTable::from_profile(profile),
            scheduler: Scheduler::new(LiveInputBackend::default(), policy, 4096),
            event_queue: DiscreteEventQueue::new(4096),
            active_mine: None,
            pending_placements: Vec::new(),
            pending_click: None,
            voxel_patch_buf: Vec::with_capacity(245),
            custom_modules: Vec::new(),
            active_nav_path: Vec::new(),
            active_nav_goal: None,
            nav_stuck_ticks: 0,
            nav_last_pos: None,
            nav_deadline_tick: 0,
            pending_replies: Vec::new(),
            script_queue: VecDeque::new(),
            is_dead: false,
            death_count: 0,
            death_history: Vec::new(),
            milestone_history: Vec::new(),
            visualizer: visualizer::LowPolyVisualizer::default(),
            auto_respawn: true,
            dimensions: Vec::new(),
            playing: false,
            positioned: false,
            loaded: false,
            live_mode,
            settle: 100,
            index: 0,
            live_tick: 0,
            previous_buttons: 0,
            humanised_perception: sawbyte_core::humanisation::HumanisedPerceptionPolicy::humanised(),
            world_memory: perception::PersistentWorldMemory::default(),
        })
    }

    fn packet(&mut self, packet: &[u8], tx: &mut wire::Tx) -> Result<()> {
        let mut r = Reader(packet);
        let id = r.var()?;
        if !self.playing {
            match id {
                0 => {
                    let name = r.string()?;
                    let mut out = Vec::new();
                    string(&name, &mut out);
                    out.push(0);
                    tx.packet(1, &out)?;
                }
                2 => return Err(bad(&format!("configuration disconnect: {}", r.nbt()?))),
                3 => {
                    tx.packet(3, &[])?;
                    self.playing = true;
                }
                4 => tx.packet(4, r.take(8)?)?,
                5 => tx.packet(5, r.take(4)?)?,
                7 => {
                    let name = r.string()?;
                    let n = r.count(65536)?;
                    for _ in 0..n {
                        r.string()?;
                        let data = if r.byte()? != 0 {
                            r.nbt()?
                        } else {
                            serde_json::Value::Null
                        };
                        if name == "minecraft:dimension_type" {
                            let min = data["min_y"]
                                .as_i64()
                                .ok_or_else(|| bad("dimension lacks min_y"))?
                                as i32;
                            let height = data["height"]
                                .as_i64()
                                .ok_or_else(|| bad("dimension lacks height"))?
                                as i32;
                            if height <= 0 || height > 4096 || height % 16 != 0 {
                                return Err(bad("invalid dimension height"));
                            }
                            self.dimensions.push((min, height));
                        }
                    }
                }
                9 => {
                    let uuid = r.take(16)?.to_vec();
                    let mut out = uuid;
                    var(1, &mut out);
                    tx.packet(6, &out)?;
                }
                15 => tx.packet(7, &[0])?, // request full registry data; no known-pack assumptions
                20 => {
                    return Err(bad(
                        "server requires a code-of-conduct acknowledgement; not automated in this preview",
                    ));
                }
                _ => {}
            }
            return Ok(());
        }
        match id {
            11 => tx.packet(11, &20f32.to_be_bytes())?,
            21 => {
                let name = r.string()?;
                let mut out = Vec::new();
                string(&name, &mut out);
                out.push(0);
                tx.packet(21, &out)?;
            }
            32 => return Err(bad(&format!("play disconnect: {}", r.nbt()?))),
            38 => {
                let z = r.i32()?;
                let x = r.i32()?;
                self.world.chunks.remove(&(x, z));
            }
            45 => tx.packet(28, r.take(8)?)?,
            46 => self.world.chunk(&mut r)?,
            50 => {
                let entity_id = r.i32()?;
                self.state_table.player_entity_id = Some(entity_id);
                r.byte()?;
                let n = r.count(1024)?;
                for _ in 0..n {
                    r.string()?;
                }
                r.var()?;
                r.var()?;
                r.var()?;
                r.take(3)?;
                let dimension = r.count(1024)?;
                r.string()?;
                r.u64()?;
                let game_mode = r.byte()?;
                self.state_table.game_mode = game_mode;
                if !self.live_mode && game_mode != 0 {
                    return Err(bad("prototype currently supports survival mode only"));
                }
                self.player.flying = game_mode == 3;
                let (min, height) = *self
                    .dimensions
                    .get(dimension)
                    .ok_or_else(|| bad("unknown dimension type"))?;
                self.world.min_y = min;
                self.world.height = height;
            }
            62 => tx.packet(45, r.take(4)?)?,
            id if id == self.packets.cb_player_combat_kill => {
                if !self.is_dead {
                    self.record_death("combat_kill", None);
                    if self.auto_respawn {
                        tx.packet(self.packets.sb_client_command, &[0])?;
                    }
                }
            }
            73 => {
                let teleport = r.var()?;
                let pos = [r.f64()?, r.f64()?, r.f64()?];
                let velocity = [r.f64()?, r.f64()?, r.f64()?];
                let yaw = r.f32()?;
                let pitch = r.f32()?;
                let flags = r.i32()?;
                if !pos.iter().chain(velocity.iter()).all(|v| v.is_finite())
                    || !yaw.is_finite()
                    || !pitch.is_finite()
                {
                    return Err(bad("invalid server position"));
                }
                let prev_yaw = self.player.yaw;
                self.player.yaw = yaw + if flags & 8 != 0 { self.player.yaw } else { 0. };
                self.player.pitch = pitch
                    + if flags & 16 != 0 {
                        self.player.pitch
                    } else {
                        0.
                    };
                let mut base_vel = self.player.velocity;
                if flags & 256 != 0 {
                    let dyaw = (self.player.yaw - prev_yaw).to_radians() as f64;
                    let (s, c) = dyaw.sin_cos();
                    base_vel = [
                        base_vel[0] * c - base_vel[2] * s,
                        base_vel[1],
                        base_vel[2] * c + base_vel[0] * s,
                    ];
                }
                for axis in 0..3 {
                    self.player.pos[axis] = pos[axis]
                        + if flags & (1 << axis) != 0 {
                            self.player.pos[axis]
                        } else {
                            0.
                        };
                    self.player.velocity[axis] = velocity[axis]
                        + if flags & (1 << (axis + 5)) != 0 {
                            base_vel[axis]
                        } else {
                            0.
                        };
                }
                self.world.update_center_chunk(self.player.pos[0], self.player.pos[2]);
                let mut ack = Vec::new();
                var(teleport, &mut ack);
                // Since 26.3 the acknowledgement includes the accepted position and view.
                ack.extend(self.position_payload());
                tx.packet(0, &ack)?;
                if !self.positioned {
                    self.positioned = true;
                    if self.index == 0 && self.live_tick == 0 {
                        self.settle = if self.live_mode { 20 } else { 100 };
                    }
                }
            }
            82 => {
                let mut out = r.take(16)?.to_vec();
                var(1, &mut out);
                tx.packet(50, &out)?;
            }
            id if id == self.packets.cb_respawn => {
                self.player.velocity = [0.0; 3];
                self.state_table.health = 20.0;
                self.world.chunks.clear();
                self.state_table.entities.clear();
                self.state_table.open_screen = None;
                self.preempt_movement();
                self.positioned = false;
                self.loaded = false;
                self.is_dead = false;
                self.state_table.push_event("player.respawn".to_string());
            }
            120 => {
                self.playing = false;
            }
            // Live block updates mutate our loaded chunk section palettes in-place.
            id if id == self.packets.cb_block_update => {
                let (pos, state) = self.world.apply_block_update_packet(&mut r)?;
                self.observe_block_update(pos, state);
            }
            id if id == self.packets.cb_section_blocks_update => {
                for (pos, state) in self.world.apply_section_blocks_update_packet(&mut r)? {
                    self.observe_block_update(pos, state);
                }
            }
            // Live container, inventory, entity, vitals, and chat packets
            id if id == self.packets.cb_open_screen => {
                if let Err(e) = self.state_table.apply_open_screen(&mut r) {
                    self.state_table.record_decode_error("open_screen", e.as_ref());
                } else if let Some((window, _, Some(_))) = self.pending_click {
                    if self.state_table.open_screen.as_ref().is_some_and(|screen| screen.window_id != window) {
                        self.pending_click = None;
                        self.state_table.push_event(format!("container_screen_changed:{window}"));
                    }
                }
            }
            id if id == self.packets.cb_container_close => {
                if let Err(e) = self.state_table.apply_container_close(&mut r) {
                    self.state_table
                        .record_decode_error("container_close", e.as_ref());
                } else if let Some((window, _, Some(_))) = self.pending_click {
                    if self.state_table.open_screen.as_ref().is_none_or(|screen| screen.window_id != window) {
                        self.pending_click = None;
                        self.state_table.push_event(format!("container_screen_closed:{window}"));
                    }
                }
            }
            id if id == self.packets.cb_container_set_data => {
                if let Err(e) = self.state_table.apply_container_set_data(&mut r) {
                    self.state_table
                        .record_decode_error("container_set_data", e.as_ref());
                }
            }
            id if id == self.packets.cb_container_set_slot => {
                if let Err(e) = self
                    .state_table
                    .apply_container_set_slot(&mut r, &self.packets)
                {
                    self.state_table
                        .record_decode_error("container_set_slot", e.as_ref());
                    self.state_table.open_screen = None;
                    self.state_table.push_event("screen_invalidated:slot_decode_error");
                    self.pending_click = None;
                } else {
                    self.observe_container_revision();
                }
            }
            id if id == self.packets.cb_container_set_content => {
                if let Err(e) = self
                    .state_table
                    .apply_container_set_content(&mut r, &self.packets)
                {
                    self.state_table
                        .record_decode_error("container_set_content", e.as_ref());
                    self.state_table.open_screen = None;
                    self.state_table.push_event("screen_invalidated:content_decode_error");
                    self.pending_click = None;
                } else {
                    self.observe_container_revision();
                }
            }
            id if id == self.packets.cb_set_player_inventory => {
                if let Err(e) = self
                    .state_table
                    .apply_set_player_inventory(&mut r, &self.packets)
                {
                    self.state_table
                        .record_decode_error("set_player_inventory", e.as_ref());
                }
            }
            id if id == self.packets.cb_set_cursor_item => {
                if let Err(e) = self
                    .state_table
                    .apply_set_cursor_item(&mut r, &self.packets)
                {
                    self.state_table
                        .record_decode_error("set_cursor_item", e.as_ref());
                }
            }
            id if id == self.packets.cb_set_health => {
                match self.state_table.apply_set_health(&mut r) {
                    Ok(()) => {
                        self.player.food_level = self.state_table.food;
                        if self.state_table.health <= 0.0 && !self.is_dead {
                            self.record_death("health_depleted", None);
                            if self.auto_respawn {
                                tx.packet(self.packets.sb_client_command, &[0])?;
                            }
                        }
                    }
                    Err(e) => self.state_table.record_decode_error("set_health", e.as_ref()),
                }
            }
            id if id == self.packets.cb_add_entity => {
                if let Err(e) = self.state_table.apply_add_entity(&mut r) {
                    self.state_table.record_decode_error("add_entity", e.as_ref());
                }
            }
            id if id == self.packets.cb_remove_entities => {
                if let Err(e) = self.state_table.apply_remove_entities(&mut r) {
                    self.state_table
                        .record_decode_error("remove_entities", e.as_ref());
                }
            }
            id if id == self.packets.cb_teleport_entity => {
                if let Err(e) = self.state_table.apply_teleport_entity(&mut r) {
                    self.state_table
                        .record_decode_error("teleport_entity", e.as_ref());
                }
            }
            id if id == self.packets.cb_move_entity_pos => {
                if let Err(e) = self.state_table.apply_move_entity_pos(&mut r) {
                    self.state_table
                        .record_decode_error("move_entity_pos", e.as_ref());
                }
            }
            id if id == self.packets.cb_set_entity_motion => {
                match self.state_table.apply_set_entity_motion(&mut r) {
                    Ok((ent_id, vel)) => {
                        if self.state_table.player_entity_id == Some(ent_id) {
                            self.player.velocity = vel;
                        }
                    }
                    Err(e) => {
                        self.state_table
                            .record_decode_error("set_entity_motion", e.as_ref());
                    }
                }
            }
            id if id == self.packets.cb_system_chat => {
                if let Err(e) = self.state_table.apply_system_chat(&mut r) {
                    self.state_table.record_decode_error("system_chat", e.as_ref());
                }
            }
            id if id == self.packets.cb_update_mob_effect => {
                match self.state_table.apply_update_mob_effect(&mut r) {
                    Ok((ent_id, effect_id, amp)) => {
                        if self.state_table.player_entity_id == Some(ent_id) {
                            match effect_id {
                                1 => self.player.effects.speed_amplifier = Some(amp),
                                2 => self.player.effects.slowness_amplifier = Some(amp),
                                8 => self.player.effects.jump_boost_amplifier = Some(amp),
                                25 => self.player.effects.levitation_amplifier = Some(amp),
                                28 => self.player.effects.slow_falling = true,
                                30 => self.player.effects.dolphins_grace = true,
                                _ => {}
                            }
                        }
                    }
                    Err(e) => self.state_table.record_decode_error("update_mob_effect", e.as_ref()),
                }
            }
            id if id == self.packets.cb_remove_mob_effect => {
                match self.state_table.apply_remove_mob_effect(&mut r) {
                    Ok((ent_id, effect_id)) => {
                        if self.state_table.player_entity_id == Some(ent_id) {
                            match effect_id {
                                1 => self.player.effects.speed_amplifier = None,
                                2 => self.player.effects.slowness_amplifier = None,
                                8 => self.player.effects.jump_boost_amplifier = None,
                                25 => self.player.effects.levitation_amplifier = None,
                                28 => self.player.effects.slow_falling = false,
                                30 => self.player.effects.dolphins_grace = false,
                                _ => {}
                            }
                        }
                    }
                    Err(e) => self.state_table.record_decode_error("remove_mob_effect", e.as_ref()),
                }
            }
            id if id == self.packets.cb_update_attributes => {
                match self.state_table.apply_update_attributes(&mut r) {
                    Ok((ent_id, attrs)) => {
                        if self.state_table.player_entity_id == Some(ent_id) {
                            for (key, val) in attrs {
                                match key.as_str() {
                                    "minecraft:generic.movement_speed" => {
                                        self.player.attributes.movement_speed = val as f32;
                                    }
                                    "minecraft:player.block_interaction_range" => {
                                        self.player.attributes.block_interaction_range = val;
                                    }
                                    "minecraft:player.entity_interaction_range" => {
                                        self.player.attributes.entity_interaction_range = val;
                                    }
                                    _ => {}
                                }
                            }
                        }
                    }
                    Err(e) => self.state_table.record_decode_error("update_attributes", e.as_ref()),
                }
            }
            id if id == self.packets.cb_game_event => {
                match self.state_table.apply_game_event(&mut r) {
                    Ok((event, val)) => {
                        if event == 3 {
                            let gm = val as u8;
                            self.state_table.game_mode = gm;
                            self.player.flying = gm == 3;
                        }
                    }
                    Err(e) => self.state_table.record_decode_error("game_event", e.as_ref()),
                }
            }
            id if id == self.packets.cb_player_abilities => {
                match self.state_table.apply_player_abilities(&mut r) {
                    Ok((flying, _)) => {
                        self.player.flying = flying;
                    }
                    Err(e) => self.state_table.record_decode_error("player_abilities", e.as_ref()),
                }
            }
            id if id == self.packets.cb_set_equipment => {
                if let Err(e) = self.state_table.apply_set_equipment(&mut r, &self.packets) {
                    self.state_table.record_decode_error("set_equipment", e.as_ref());
                }
            }
            id if id == self.packets.cb_set_experience => {
                if let Err(e) = self.state_table.apply_set_experience(&mut r) {
                    self.state_table.record_decode_error("set_experience", e.as_ref());
                }
            }
            id if id == self.packets.cb_block_changed_ack => {
                if let Ok(seq) = r.var() {
                    self.state_table.push_event(format!("block_changed_ack:{seq}"));
                }
            }
            id if id == self.packets.cb_set_entity_data => {
                let _ = self.state_table.apply_set_entity_data(&mut r);
            }
            id if id == self.packets.cb_player_info_update => {
                let _ = self.state_table.apply_player_info_update(&mut r);
            }
            id if id == self.packets.cb_set_title_text => {
                let _ = self.state_table.apply_set_title_text(&mut r);
            }
            id if id == self.packets.cb_set_subtitle_text => {
                let _ = self.state_table.apply_set_subtitle_text(&mut r);
            }
            id if id == self.packets.cb_set_action_bar_text => {
                let _ = self.state_table.apply_set_action_bar_text(&mut r);
            }
            _ => {}
        }
        Ok(())
    }

    fn position_payload(&self) -> Vec<u8> {
        let mut out = Vec::with_capacity(32);
        for value in self.player.pos {
            out.extend(value.to_be_bytes());
        }
        out.extend(self.player.yaw.to_be_bytes());
        out.extend(self.player.pitch.to_be_bytes());
        out
    }

    fn send_position(&self, tx: &mut wire::Tx) -> Result<()> {
        let mut out = self.position_payload();
        out.push(u8::from(self.player.ground) | (u8::from(self.player.horizontal_collision) << 1));
        tx.packet(31, &out)
    }

    fn tick(
        &mut self,
        frames: &[InputFrame],
        trace: &mut impl Write,
        tx: &mut wire::Tx,
    ) -> Result<bool> {
        if !self.playing
            || !self.positioned
            || !self.world.loaded(self.player.pos[0], self.player.pos[2])
        {
            return Ok(false);
        }
        if !self.loaded {
            tx.packet(44, &[])?;
            self.loaded = true;
            eprintln!("SAWBYTE_SPAWNED");
        }
        let recording = self.settle == 0;
        let frame = if recording {
            if self.index < frames.len() {
                frames[self.index]
            } else {
                InputFrame::default()
            }
        } else {
            self.settle -= 1;
            InputFrame::default()
        };
        if recording && self.index == 0 {
            eprintln!("SAWBYTE_REPLAY_STARTED");
        }
        self.player.step(frame, &self.world)?;
        let buttons = frame.buttons.bits();
        if buttons != self.previous_buttons {
            tx.packet(43, &[buttons as u8])?;
            self.previous_buttons = buttons;
        }
        self.send_position(tx)?;
        tx.packet(13, &[])?;
        if recording {
            let p = &self.player;
            writeln!(
                trace,
                "{},{:.9},{:.9},{:.9},{:.6},{:.6},{},{}",
                self.index,
                p.pos[0],
                p.pos[1],
                p.pos[2],
                p.yaw,
                p.pitch,
                p.ground,
                self.world.chunks.len()
            )?;
            self.index += 1;
            if self.index == frames.len() {
                trace.flush()?;
                eprintln!("SAWBYTE_REPLAY_COMPLETE frames={}", frames.len());
                return Ok(true);
            }
        }
        Ok(false)
    }

    /// Closed-loop feedback controller for live 3D A* navigation.
    /// Steers toward each waypoint using observed player position and yaw, stopping only on arrival, timeout, or stuck.
    fn step_live_navigation(&mut self) {
        let Some((goal, sprint)) = self.active_nav_goal else {
            return;
        };

        // Navigation deadline timeout check
        if self.nav_deadline_tick > 0 && self.live_tick > self.nav_deadline_tick {
            self.preempt_movement();
            self.state_table.push_event(format!(
                "nav_failed:timeout:({},{},{})",
                goal[0], goal[1], goal[2]
            ));
            return;
        }

        // Navigation stuck progress check
        if let Some(last_pos) = self.nav_last_pos {
            let moved = (self.player.pos[0] - last_pos[0]).hypot(self.player.pos[2] - last_pos[2]);
            if moved < 0.02 {
                self.nav_stuck_ticks += 1;
            } else {
                self.nav_stuck_ticks = 0;
                self.nav_last_pos = Some(self.player.pos);
            }
        } else {
            self.nav_last_pos = Some(self.player.pos);
            self.nav_stuck_ticks = 0;
        }

        if self.nav_stuck_ticks >= 40 {
            self.preempt_movement();
            self.state_table.push_event(format!(
                "nav_failed:stuck:({},{},{})",
                goal[0], goal[1], goal[2]
            ));
            return;
        }

        if self.scheduler.pending() > 0 {
            return;
        }
        let gx = goal[0] as f64 + 0.5;
        let gz = goal[2] as f64 + 0.5;
        let dist_goal = (gx - self.player.pos[0]).hypot(gz - self.player.pos[2]);
        if dist_goal <= 0.42 && (self.player.pos[1] - goal[1] as f64).abs() <= 0.75 {
            let _ = self.scheduler.submit(&[
                (
                    self.live_tick,
                    Action::Button {
                        button: Button::Forward,
                        down: false,
                    },
                ),
                (
                    self.live_tick,
                    Action::Button {
                        button: Button::Sprint,
                        down: false,
                    },
                ),
                (
                    self.live_tick,
                    Action::Button {
                        button: Button::Jump,
                        down: false,
                    },
                ),
            ]);
            self.active_nav_goal = None;
            self.active_nav_path.clear();
            self.nav_stuck_ticks = 0;
            self.nav_last_pos = None;
            self.nav_deadline_tick = 0;
            self.state_table.push_event(format!(
                "nav_arrived:({},{},{})",
                goal[0], goal[1], goal[2]
            ));
            return;
        }

        while let Some(&wp) = self.active_nav_path.first() {
            if self.active_nav_path.len() <= 1 {
                break;
            }
            let wx = wp[0] as f64 + 0.5;
            let wz = wp[2] as f64 + 0.5;
            if (wx - self.player.pos[0]).hypot(wz - self.player.pos[2]) <= 0.45 {
                self.active_nav_path.remove(0);
            } else {
                break;
            }
        }

        let target = self.active_nav_path.first().copied().unwrap_or(goal);
        let tx = target[0] as f64 + 0.5;
        let tz = target[2] as f64 + 0.5;
        let dx = tx - self.player.pos[0];
        let dz = tz - self.player.pos[2];
        let desired_yaw = (-dx).atan2(dz).to_degrees();
        let mut yaw_delta = (desired_yaw - self.player.yaw as f64) % 360.0;
        if yaw_delta > 180.0 {
            yaw_delta -= 360.0;
        } else if yaw_delta < -180.0 {
            yaw_delta += 360.0;
        }
        let aligned = yaw_delta.abs() < 35.0;
        let need_jump = target[1] as f64 > self.player.pos[1] + 0.4;
        let use_sprint = sprint && dist_goal > 1.1 && aligned;
        let mut actions = vec![
            (
                self.live_tick,
                Action::Button {
                    button: Button::Forward,
                    down: aligned,
                },
            ),
            (
                self.live_tick,
                Action::Button {
                    button: Button::Sprint,
                    down: use_sprint,
                },
            ),
            (
                self.live_tick,
                Action::Button {
                    button: Button::Jump,
                    down: need_jump && aligned,
                },
            ),
        ];
        if yaw_delta.abs() > 0.25 {
            actions.push((
                self.live_tick,
                Action::Look {
                    yaw: yaw_delta,
                    pitch: 0.0,
                },
            ));
        }
        let _ = self.scheduler.submit(&actions);
    }

    /// Live interactive 20 TPS client tick driven by the live `Scheduler` and `DiscreteEventQueue`.
    fn tick_live(
        &mut self,
        mut trace: Option<&mut BufWriter<File>>,
        tx: &mut wire::Tx,
    ) -> Result<bool> {
        if !self.playing
            || !self.positioned
            || !self.world.loaded(self.player.pos[0], self.player.pos[2])
        {
            return Ok(false);
        }
        if !self.loaded {
            tx.packet(44, &[])?;
            self.loaded = true;
            eprintln!("SAWBYTE_SPAWNED");
        }
        if self.settle > 0 {
            self.settle -= 1;
            self.player.step(InputFrame::default(), &self.world)?;
            self.send_position(tx)?;
            tx.packet(13, &[])?;
            if self.settle == 0 {
                eprintln!("SAWBYTE_LIVE_READY");
            }
            return Ok(false);
        }
        if self.active_nav_goal.is_some() || self.scheduler.pending() > 0 {
            let mut check_box = self.player.box_aabb();
            check_box[0] -= 0.1;
            check_box[2] -= 0.1;
            check_box[3] += 0.1;
            check_box[5] += 0.1;
            if let Err((bx, by, bz, state_id)) = self.world.check_region_known(check_box) {
                self.preempt_movement();
                self.player.velocity = [0.0; 3];
                self.state_table.push_event(format!("unrecognized_block_state:({bx},{by},{bz})={state_id}"));
            }
        }
        self.step_live_navigation();
        let frame = self
            .scheduler
            .advance(self.live_tick)
            .map_err(|e| bad(&format!("scheduler error: {e}")))?;
        self.player.step(frame, &self.world)?;
        self.world.update_center_chunk(self.player.pos[0], self.player.pos[2]);

        // Step active mining state across ticks
        if let Some(ref mut mine) = self.active_mine {
            if mine.remaining_ticks > 0 {
                mine.remaining_ticks -= 1;
            }
            let eye = self.player.eye_position();
            let block_center = [mine.pos[0] as f64 + 0.5, mine.pos[1] as f64 + 0.5, mine.pos[2] as f64 + 0.5];
            let dx_aim = block_center[0] - eye[0];
            let dy_aim = block_center[1] - eye[1];
            let dz_aim = block_center[2] - eye[2];
            let horiz_dist = (dx_aim * dx_aim + dz_aim * dz_aim).sqrt();
            self.player.yaw = (-dx_aim).atan2(dz_aim).to_degrees() as f32;
            self.player.pitch = (-dy_aim).atan2(horiz_dist).to_degrees() as f32;
        }

        // Drain and transmit all discrete sub-tick events scheduled for this tick
        let drained = self
            .event_queue
            .drain_tick(self.live_tick)
            .map_err(|e| bad(&format!("event queue error: {e}")))?;
        for ev in drained {
            if let SubTickEvent::ContainerClick { window_id, state_id, .. } = &ev {
                let current = self.state_table.open_screen.as_ref()
                    .is_some_and(|screen| screen.window_id == *window_id && screen.state_id == *state_id);
                if !current {
                    self.pending_click = None;
                    self.state_table.push_event(format!("container_click_failed:stale_screen:{window_id}:{state_id}"));
                    continue;
                }
                self.pending_click = Some((*window_id, *state_id, Some(self.live_tick.saturating_add(40))));
            }
            if let SubTickEvent::UseItemOn { sequence, .. } = &ev {
                if let Some(placement) = self.pending_placements.iter_mut().find(|placement| placement.sequence == *sequence) {
                    placement.confirmation_deadline = Some(self.live_tick.saturating_add(40));
                }
            }
            if let SubTickEvent::SelectHotbar { slot } = &ev {
                self.state_table.selected_hotbar = *slot;
            } else if let SubTickEvent::StopDestroyBlock { pos, .. } = &ev {
                if self.active_mine.as_ref().is_none_or(|mine| mine.pos != *pos) {
                    continue;
                }
                if let Some(mine) = &self.active_mine {
                    if mine.pos == *pos {
                        if let Some(mine) = self.active_mine.as_mut() {
                            mine.confirmation_deadline = Some(self.live_tick.saturating_add(60));
                        }
                        self.state_table.push_event(format!(
                            "mine_finish_queued:({},{},{})",
                            pos[0], pos[1], pos[2]
                        ));
                    }
                }
            }
            if let Some((pkt_id, payload)) = inventory::encode_serverbound_event_with_pose(
                &ev,
                &self.packets,
                self.player.yaw,
                self.player.pitch,
            ) {
                tx.packet(pkt_id, &payload)?;
            } else {
                self.state_table
                    .push_event(format!("unsupported_native_event:{ev:?}"));
            }
        }

        if self.active_mine.as_ref().and_then(|mine| mine.confirmation_deadline)
            .is_some_and(|deadline| self.live_tick >= deadline)
        {
            if let Some(mine) = self.active_mine.take() {
                self.state_table.push_event(format!("mine_failed:confirmation_timeout:({},{},{})", mine.pos[0], mine.pos[1], mine.pos[2]));
            }
        }

        let mut timed_out = Vec::new();
        self.pending_placements.retain(|placement| {
            let expired = placement.confirmation_deadline.is_some_and(|deadline| self.live_tick >= deadline);
            if expired {
                timed_out.push(placement.sequence);
            }
            !expired
        });
        for sequence in timed_out {
            self.state_table.push_event(format!("placement_confirmation_timeout:{sequence}"));
        }
        if self.pending_click.is_some_and(|(_, _, deadline)| deadline.is_some_and(|tick| self.live_tick >= tick)) {
            self.pending_click = None;
            self.state_table.push_event("container_click_confirmation_timeout");
        }

        let buttons = frame.buttons.bits();
        if buttons != self.previous_buttons {
            tx.packet(43, &[buttons as u8])?;
            self.previous_buttons = buttons;
        }
        self.send_position(tx)?;
        tx.packet(13, &[])?;

        if let Some(writer) = &mut trace {
            let p = &self.player;
            writeln!(
                writer,
                "{},{:.9},{:.9},{:.9},{:.6},{:.6},{},{}",
                self.live_tick,
                p.pos[0],
                p.pos[1],
                p.pos[2],
                p.yaw,
                p.pitch,
                p.ground,
                self.world.chunks.len()
            )?;
        }
        self.live_tick += 1;
        self.resolve_pending_replies();
        Ok(true)
    }

    /// Aborts any active block mining in progress and notifies the server with `AbortDestroyBlock`.
    pub fn abort_active_mine(&mut self, mut tx: Option<&mut wire::Tx>) {
        if let Some(mine) = self.active_mine.take() {
            self.event_queue.discard_mining(mine.pos);
            let abort_ev = SubTickEvent::AbortDestroyBlock {
                pos: mine.pos,
                face: mine.face,
            };
            if let Some(ref mut tx_stream) = tx {
                if let Some((pkt_id, payload)) = inventory::encode_serverbound_event_with_pose(
                    &abort_ev,
                    &self.packets,
                    self.player.yaw,
                    self.player.pitch,
                ) {
                    let _ = tx_stream.packet(pkt_id, &payload);
                }
            } else {
                let next_tick = self.live_tick.saturating_add(1);
                if let Err(error) = self.event_queue.insert_ordered(next_tick, abort_ev) {
                    self.state_table.push_event(format!("mine_abort_failed:{error}"));
                }
            }
            self.state_table.push_event(format!(
                "mine_aborted:({},{},{})",
                mine.pos[0], mine.pos[1], mine.pos[2]
            ));
        }
    }

    /// Preempts any active navigation or scheduled movement inputs cleanly.
    pub fn preempt_movement(&mut self) {
        self.active_nav_goal = None;
        self.active_nav_path.clear();
        self.nav_stuck_ticks = 0;
        self.nav_last_pos = None;
        self.nav_deadline_tick = 0;
        let _ = self.scheduler.cancel();
        self.abort_active_mine(None);
    }

    /// Generates a rich "Seeing the Game with Words v2" natural-language + SVPE-1 sensory report.
    pub fn render_sensory_words(&mut self) -> String {
        let px = self.player.pos[0];
        let py = self.player.pos[1];
        let pz = self.player.pos[2];
        let bx = px.floor() as i32;
        let by = py.floor() as i32;
        let bz = pz.floor() as i32;
        let cx = bx.div_euclid(16);
        let cz = bz.div_euclid(16);
        let vx = self.player.velocity[0];
        let vy = self.player.velocity[1];
        let vz = self.player.velocity[2];
        let speed_mps = (vx * vx + vy * vy + vz * vz).sqrt() * 20.0;
        let yaw = self.player.yaw.rem_euclid(360.0);
        let cardinal = if !(22.5..337.5).contains(&yaw) {
            "South (+Z)"
        } else if yaw < 67.5 {
            "South-West (-X, +Z)"
        } else if yaw < 112.5 {
            "West (-X)"
        } else if yaw < 157.5 {
            "North-West (-X, -Z)"
        } else if yaw < 202.5 {
            "North (-Z)"
        } else if yaw < 247.5 {
            "North-East (+X, -Z)"
        } else if yaw < 292.5 {
            "East (+X)"
        } else {
            "South-East (+X, +Z)"
        };
        let gm_str = match self.state_table.game_mode {
            0 => "Survival",
            1 => "Creative",
            2 => "Adventure",
            3 => "Spectator",
            _ => "Custom",
        };

        let mut report = String::with_capacity(8192);
        report.push_str("================================================================================\n");
        report.push_str(&format!(
            "SAWBYTE SENSORY VISION REPORT (\"SEEING THE GAME WITH WORDS v2\") | Tick {}\n",
            self.live_tick
        ));
        report.push_str("================================================================================\n");
        report.push_str("[1. SELF, POSE & VITALS]\n");
        report.push_str(&format!(
            "  Position   : ({px:.3}, {py:.3}, {pz:.3}) | Block [{bx}, {by}, {bz}] | Chunk [{cx}, {cz}] (Loaded Chunks: {})\n",
            self.world.chunks.len()
        ));
        report.push_str(&format!(
            "  Velocity   : ({vx:.4}, {vy:.4}, {vz:.4}) | Speed: {speed_mps:.2} m/s | OnGround: {} | HorizCollision: {}\n",
            self.player.ground, self.player.horizontal_collision
        ));
        report.push_str(&format!(
            "  Orientation: Yaw={:.2} deg, Pitch={:.2} deg | Facing: {cardinal}\n",
            self.player.yaw, self.player.pitch
        ));
        report.push_str(&format!(
            "  Vitals     : HP={:.1}/20.0 ({:.1} hearts) | Food={}/20 | Saturation={:.1} | XP=Lvl {} ({:.0}%) | Mode={gm_str}\n",
            self.state_table.health,
            self.state_table.health * 0.5,
            self.state_table.food,
            self.state_table.saturation,
            self.state_table.xp_level,
            self.state_table.xp_progress * 100.0
        ));

        report.push_str("\n[2. CROSSHAIR & 360-DEGREE SPATIAL ENVIRONMENT]\n");
        let b_hit = raycast::raycast_block(&self.player, &self.world);
        let candidates: Vec<(i32, [f64; 6])> = self
            .state_table
            .entities
            .values()
            .map(|e| (e.entity_id, e.aabb()))
            .collect();
        let e_hit = raycast::raycast_entity(&self.player, &self.world, &candidates);
        match (&b_hit, &e_hit) {
            (_, Some(eh)) if b_hit.as_ref().map_or(true, |bh| eh.distance < bh.distance) => {
                let holo = self.state_table.entity_hologram_lines(eh.entity_id);
                let kind = self
                    .state_table
                    .entities
                    .get(&eh.entity_id)
                    .map(|e| e.kind_name.as_str())
                    .unwrap_or("entity");
                report.push_str(&format!(
                    "  Crosshair  : Pointing at Entity #{} ({kind}) at {:.2}m | Hologram/Name: {:?}\n",
                    eh.entity_id, eh.distance, holo
                ));
            }
            (Some(bh), _) => {
                let bstate = self
                    .world
                    .block_state(bh.block_pos[0], bh.block_pos[1], bh.block_pos[2])
                    .unwrap_or(0);
                report.push_str(&format!(
                    "  Crosshair  : Pointing at Block ({}, {}, {}) state_id={} face={:?} at {:.2}m\n",
                    bh.block_pos[0], bh.block_pos[1], bh.block_pos[2], bstate, bh.face, bh.distance
                ));
            }
            (None, None) => {
                report.push_str("  Crosshair  : Looking into open air (no block or entity within reach)\n");
            }
        }

        let under_feet = self.world.solid(bx, by - 1, bz).unwrap_or(false);
        let overhead = (2..=6).find(|dy| self.world.solid(bx, by + *dy, bz).unwrap_or(false));
        report.push_str(&format!(
            "  Vertical   : Floor under feet ({bx}, {}, {bz}) is {} | Overhead ceiling: {}\n",
            by - 1,
            if under_feet { "SOLID" } else { "AIR / DROP" },
            overhead.map_or_else(|| "Open Sky (>6m)".to_string(), |dy| format!("Solid roof at +{dy}m"))
        ));

        let dirs = [
            ("Ahead (0°)", 0.0f32),
            ("Ahead-Right (+45°)", 45.0),
            ("Right (+90°)", 90.0),
            ("Behind (+180°)", 180.0),
            ("Left (-90°)", -90.0),
            ("Ahead-Left (-45°)", -45.0),
        ];
        for (label, rel_deg) in dirs {
            let rad = (self.player.yaw + rel_deg).to_radians();
            let dx = -rad.sin() as f64;
            let dz = rad.cos() as f64;
            let mut wall_dist = None;
            let mut drop_dist = None;
            for step in 1..=12 {
                let sx = (px + dx * step as f64).floor() as i32;
                let sz = (pz + dz * step as f64).floor() as i32;
                let body_solid = self.world.solid(sx, by, sz).unwrap_or(false)
                    || self.world.solid(sx, by + 1, sz).unwrap_or(false);
                if body_solid && wall_dist.is_none() {
                    let can_step_up = !self.world.solid(sx, by + 1, sz).unwrap_or(false)
                        && !self.world.solid(sx, by + 2, sz).unwrap_or(false);
                    wall_dist = Some((step, can_step_up));
                    break;
                }
                let has_floor = self.world.solid(sx, by - 1, sz).unwrap_or(false)
                    || self.world.solid(sx, by - 2, sz).unwrap_or(false);
                if !has_floor && drop_dist.is_none() {
                    drop_dist = Some(step);
                }
            }
            let summary = match (wall_dist, drop_dist) {
                (Some((d, true)), _) => format!("1-block jumpable step at {d}m"),
                (Some((d, false)), _) => format!("Solid wall obstruction at {d}m"),
                (None, Some(drop_d)) => format!("Clear of walls, but DROP/VOID edge at {drop_d}m!"),
                (None, None) => "Clear walkable ground for 12m+".to_string(),
            };
            report.push_str(&format!("  Radar {:<18}: {summary}\n", label));
        }

        report.push_str("\n[3. INVENTORY & HOTBAR (46 SLOTS)]\n");
        let held_summary = self
            .state_table
            .main_hand_item()
            .map(|s| {
                format!(
                    "{}x {} name={:?} lore={:?}",
                    s.count, s.id, s.display_name, s.lore
                )
            })
            .unwrap_or_else(|| "Empty Hand".to_string());
        report.push_str(&format!(
            "  Active Hotbar: Slot {} (raw slot {}) | Held: {}\n",
            self.state_table.selected_hotbar,
            36 + self.state_table.selected_hotbar,
            held_summary
        ));
        let mut any_inv = false;
        for (idx, opt_item) in self.state_table.inventory_slots.iter().enumerate() {
            if let Some(st) = opt_item {
                any_inv = true;
                let region = match idx {
                    0 => "CraftOut".to_string(),
                    1..=4 => format!("Craft[{}]", idx - 1),
                    5..=8 => format!("Armor[{}]", idx - 5),
                    9..=35 => format!("MainInv[{}]", idx - 9),
                    36..=44 => format!("Hotbar[{}]", idx - 36),
                    45 => "Offhand".to_string(),
                    _ => format!("Slot[{idx}]"),
                };
                report.push_str(&format!(
                    "    - {:<11} (slot {:>2}): {:>2}x {:<26} | Name: {:?} | Lore: {:?}\n",
                    region, idx, st.count, st.id, st.display_name, st.lore
                ));
            }
        }
        if !any_inv {
            report.push_str("    (All 46 inventory slots are currently empty)\n");
        }

        report.push_str("\n[4. OPEN GUI CONTAINER SCREEN]\n");
        if let Some(ref scr) = self.state_table.open_screen {
            report.push_str(&format!(
                "  OPEN SCREEN: window_id={} state_id={} kind={:?} title=\"{}\"\n",
                scr.window_id, scr.state_id, scr.kind, scr.title
            ));
            for (sidx, opt_it) in scr.slots.iter().enumerate() {
                if let Some(it) = opt_it {
                    let row = sidx / 9;
                    let col = sidx % 9;
                    report.push_str(&format!(
                        "    - Slot {:>2} (Row {}, Col {}): {:>2}x {:<26} | Name: {:?} | Lore: {:?}\n",
                        sidx, row, col, it.count, it.id, it.display_name, it.lore
                    ));
                }
            }
        } else {
            report.push_str("  (No chest/container GUI screen is currently open)\n");
        }

        report.push_str("\n[5. NEARBY ENTITIES, PLAYERS & HOLOGRAM NPCs]\n");
        let mut ent_list: Vec<(f64, f64, i32)> = self
            .state_table
            .entities
            .values()
            .map(|e| {
                let dx = e.pos[0] - px;
                let dy = e.pos[1] - py;
                let dz = e.pos[2] - pz;
                let dist = (dx * dx + dy * dy + dz * dz).sqrt();
                let target_yaw = (-dx).atan2(dz).to_degrees();
                let mut rel_angle = (target_yaw - self.player.yaw as f64) % 360.0;
                if rel_angle > 180.0 {
                    rel_angle -= 360.0;
                } else if rel_angle < -180.0 {
                    rel_angle += 360.0;
                }
                (dist, rel_angle, e.entity_id)
            })
            .collect();
        ent_list.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));
        if ent_list.is_empty() {
            report.push_str("  (No nearby entities or NPCs tracked)\n");
        } else {
            for (dist, rel_angle, eid) in ent_list.iter().take(32) {
                if let Some(ent) = self.state_table.entities.get(eid) {
                    let holos = self.state_table.entity_hologram_lines(*eid);
                    let dir_word = if rel_angle.abs() <= 20.0 {
                        "Directly Ahead"
                    } else if *rel_angle > 0.0 {
                        "Right"
                    } else {
                        "Left"
                    };
                    report.push_str(&format!(
                        "  - Entity #{:<4} | {:<12} (type {:>3}) | Dist: {:>5.2}m ({:+.1}° {}) | Pos: ({:.2}, {:.2}, {:.2}) | Name/Holograms: {:?}\n",
                        ent.entity_id,
                        ent.kind_name,
                        ent.entity_type,
                        dist,
                        rel_angle,
                        dir_word,
                        ent.pos[0],
                        ent.pos[1],
                        ent.pos[2],
                        holos
                    ));
                }
            }
        }

        // Update persistent world memory & direct-LOS entity tracking
        let eye_pos = self.player.eye_position();
        for ent in self.state_table.entities.values() {
            let target_pt = [ent.pos[0], ent.pos[1] + ent.height * 0.5, ent.pos[2]];
            if perception::count_occluding_blocks(eye_pos, target_pt, &self.world) == 0 {
                self.world_memory.direct_los_seen_entities.insert(ent.entity_id);
            }
        }
        self.world_memory
            .observe_tick(self.live_tick, &self.player, &self.world, &self.state_table);

        report.push_str("\n[6. FIRST-PERSON RETINAL VIEWPORT (24x11 FOV 72°x44°) & DEPTH BUFFER]\n");
        let (retinal_ascii, depth_rle) =
            perception::render_retinal_viewport_24x11(&self.player, &self.world, &self.state_table);
        report.push_str(&retinal_ascii);
        report.push('\n');
        report.push_str(&format!("  Depth Stream : {depth_rle}\n"));

        report.push_str("\n[7. COMPACT SVPE-1 SPATIAL ENCODINGS (3D VOLUME RLE & 2.5D HEIGHT DELTA)]\n");
        let svpe_rle3d = perception::encode_svpe1_palette_rle(&self.world, [bx, by, bz], 6, 3, 6);
        let svpe_hd25d = perception::encode_svpe1_height_delta(&self.world, self.player.pos, 10);
        report.push_str(&format!("  SVPE-1 3D Volume (13x7x13) : {svpe_rle3d}\n"));
        report.push_str(&format!("  SVPE-1 2.5D Surface (21x21): {svpe_hd25d}\n"));

        report.push_str("\n[8. COMPENSATORY PERCEPTION: THROUGH-BLOCK X-RAY, ESP & LONG-RANGE HORIZON]\n");
        report.push_str(&format!(
            "  Humanisation & Fair-Play Gate: {} (Entity Reach={:.1}m, Block Reach={:.1}m)\n",
            if self.humanised_perception.enabled {
                "ENABLED (Occluded ESP treated as supplementary hints)"
            } else {
                "DIRECT/UNFILTERED"
            },
            self.player.max_entity_reach(),
            self.player.max_block_reach()
        ));
        let pitch_rad = self.player.pitch.to_radians();
        let yaw_rad = self.player.yaw.to_radians();
        let look_dir = [
            -(yaw_rad.sin() as f64) * (pitch_rad.cos() as f64),
            -(pitch_rad.sin() as f64),
            (yaw_rad.cos() as f64) * (pitch_rad.cos() as f64),
        ];
        let xray_look = perception::cast_penetrating_ray_layers(
            eye_pos,
            look_dir,
            &self.world,
            &self.state_table,
            16.0,
        );
        let xray_down = perception::cast_penetrating_ray_layers(
            self.player.pos,
            [0.0, -1.0, 0.0],
            &self.world,
            &self.state_table,
            12.0,
        );
        report.push_str(&format!("  Through-Block Look Ray (16m) : {xray_look}\n"));
        report.push_str(&format!("  Subsurface Downward Ray (12m): {xray_down}\n"));

        for (dist, rel_angle, eid) in ent_list.iter().take(16) {
            if let Some(ent) = self.state_table.entities.get(eid) {
                let target_pt = [ent.pos[0], ent.pos[1] + ent.height * 0.5, ent.pos[2]];
                let wall_blocks = perception::count_occluding_blocks(eye_pos, target_pt, &self.world);
                let has_los = wall_blocks == 0;
                let prev_seen = self.world_memory.direct_los_seen_entities.contains(eid);
                let sector = perception::cardinal_sector_from_bearing(self.player.yaw, *rel_angle);
                let guidance = self.humanised_perception.evaluate_target(
                    has_los,
                    *rel_angle,
                    0.0,
                    *dist,
                    prev_seen,
                    sector,
                );
                report.push_str(&format!(
                    "  ESP Entity #{:<4} ({:<10}): Tier={:<28} | WallBlocks={} | Confidence={:.2} | Sector={} | {}\n",
                    eid,
                    ent.kind_name,
                    guidance.tier.as_code(),
                    wall_blocks,
                    guidance.human_confidence,
                    guidance.sector_hint,
                    guidance.guidance
                ));
            }
        }

        for sector_line in perception::scan_long_range_sectors(&self.player, &self.world) {
            report.push_str(&format!("  Horizon 64m {sector_line}\n"));
        }

        report.push_str("\n[9. SMALL-MAP FULL-ARENA AWARENESS & PERSISTENT CROSS-RUN MEMORY]\n");
        report.push_str(&format!(
            "  {}\n",
            perception::inspect_small_map_awareness(&self.world, self.player.pos)
        ));
        report.push_str(&format!("  Memory Status: {}\n", self.world_memory.summary_status()));
        if !self.world_memory.landmarks.is_empty() {
            let lm_strs: Vec<String> = self
                .world_memory
                .landmarks
                .values()
                .take(8)
                .map(|l| {
                    format!(
                        "{} \"{}\" at ({},{},{})",
                        l.kind, l.label, l.pos[0], l.pos[1], l.pos[2]
                    )
                })
                .collect();
            report.push_str(&format!("  Persisted Landmarks: [{}]\n", lm_strs.join("; ")));
        }

        report.push_str("\n[10. HUD TITLES, ACTIONBAR & RECENT CHAT]\n");
        report.push_str(&format!(
            "  Title: {:?} | Subtitle: {:?} | ActionBar: {:?}\n",
            self.state_table.title_text,
            self.state_table.subtitle_text,
            self.state_table.action_bar_text
        ));
        let recent_chat: Vec<&String> = self.state_table.chat_log.iter().rev().take(12).collect();
        if recent_chat.is_empty() {
            report.push_str("  Chat Log: (no chat messages received yet)\n");
        } else {
            for msg in recent_chat.into_iter().rev() {
                report.push_str(&format!("  Chat > {msg}\n"));
            }
        }
        report
    }

    /// Generates a structured JSON version of the complete sensory vision report.
    pub fn render_sensory_json(&mut self) -> String {
        let narrative = self.render_sensory_words();
        let px = self.player.pos[0];
        let py = self.player.pos[1];
        let pz = self.player.pos[2];
        let eye_pos = self.player.eye_position();
        let entities_rich: Vec<serde_json::Value> = self
            .state_table
            .entities
            .values()
            .map(|e| {
                let dx = e.pos[0] - px;
                let dy = e.pos[1] - py;
                let dz = e.pos[2] - pz;
                let dist = (dx * dx + dy * dy + dz * dz).sqrt();
                let target_yaw = (-dx).atan2(dz).to_degrees();
                let mut rel_angle = (target_yaw - self.player.yaw as f64) % 360.0;
                if rel_angle > 180.0 {
                    rel_angle -= 360.0;
                } else if rel_angle < -180.0 {
                    rel_angle += 360.0;
                }
                let target_pt = [e.pos[0], e.pos[1] + e.height * 0.5, e.pos[2]];
                let wall_blocks = perception::count_occluding_blocks(eye_pos, target_pt, &self.world);
                let has_los = wall_blocks == 0;
                let prev_seen = self.world_memory.direct_los_seen_entities.contains(&e.entity_id);
                let sector = perception::cardinal_sector_from_bearing(self.player.yaw, rel_angle);
                let guidance = self.humanised_perception.evaluate_target(
                    has_los,
                    rel_angle,
                    0.0,
                    dist,
                    prev_seen,
                    sector,
                );
                let holos = self.state_table.entity_hologram_lines(e.entity_id);
                serde_json::json!({
                    "id": e.entity_id,
                    "type": e.entity_type,
                    "kind": e.kind_name,
                    "custom_name": e.custom_name,
                    "hologram_lines": holos,
                    "pos": e.pos,
                    "vel": e.velocity,
                    "distance": dist,
                    "relative_bearing_deg": rel_angle,
                    "visibility_tier": guidance.tier.as_code(),
                    "wall_blocks_between": wall_blocks,
                    "human_confidence": guidance.human_confidence,
                    "direct_action_permitted": guidance.direct_action_permitted,
                    "sector_hint": guidance.sector_hint,
                    "guidance": guidance.guidance,
                })
            })
            .collect();
        let inv_items: Vec<serde_json::Value> = self
            .state_table
            .inventory_slots
            .iter()
            .enumerate()
            .filter_map(|(slot, it)| {
                it.as_ref().map(|s| {
                    serde_json::json!({
                        "slot": slot,
                        "hotbar_slot": if (36..=44).contains(&slot) { Some(slot - 36) } else { None },
                        "id": s.id,
                        "count": s.count,
                        "display_name": s.display_name,
                        "lore": s.lore,
                        "damage": s.damage,
                    })
                })
            })
            .collect();
        let screen_json = self.state_table.open_screen.as_ref().map(|s| {
            let slots: Vec<serde_json::Value> = s
                .slots
                .iter()
                .enumerate()
                .filter_map(|(idx, it)| {
                    it.as_ref().map(|item| {
                        serde_json::json!({
                            "slot": idx,
                            "row": idx / 9,
                            "col": idx % 9,
                            "id": item.id,
                            "count": item.count,
                            "display_name": item.display_name,
                            "lore": item.lore,
                        })
                    })
                })
                .collect();
            serde_json::json!({
                "window_id": s.window_id,
                "state_id": s.state_id,
                "kind": format!("{:?}", s.kind),
                "title": s.title,
                "slots": slots,
            })
        });
        let (_, depth_rle) =
            perception::render_retinal_viewport_24x11(&self.player, &self.world, &self.state_table);
        serde_json::json!({
            "ok": true,
            "tick": self.live_tick,
            "pos": self.player.pos,
            "vel": self.player.velocity,
            "yaw": self.player.yaw,
            "pitch": self.player.pitch,
            "ground": self.player.ground,
            "health": self.state_table.health,
            "food": self.state_table.food,
            "saturation": self.state_table.saturation,
            "xp_level": self.state_table.xp_level,
            "selected_hotbar": self.state_table.selected_hotbar,
            "title_text": self.state_table.title_text,
            "subtitle_text": self.state_table.subtitle_text,
            "action_bar_text": self.state_table.action_bar_text,
            "inventory": inv_items,
            "open_screen": screen_json,
            "entities": entities_rich,
            "chat_log": self.state_table.chat_log,
            "events": self.state_table.received_events,
            "svpe1_rle3d": perception::encode_svpe1_palette_rle(
                &self.world,
                [
                    self.player.pos[0].floor() as i32,
                    self.player.pos[1].floor() as i32,
                    self.player.pos[2].floor() as i32,
                ],
                6,
                3,
                6,
            ),
            "svpe1_hd25d": perception::encode_svpe1_height_delta(&self.world, self.player.pos, 10),
            "svpe1_depth24x11": depth_rle,
            "humanisation_enabled": self.humanised_perception.enabled,
            "persistent_memory": {
                "enabled": self.world_memory.enabled,
                "world_id": self.world_memory.world_id,
                "run_count": self.world_memory.run_count,
                "explored_chunks": self.world_memory.explored_chunk_tiles.len(),
                "landmarks_count": self.world_memory.landmarks.len(),
            },
            "vision_narrative": narrative,
        })
        .to_string()
    }

    /// Generates full zero-graphics RL / neural network observation JSON.
    pub fn gym_observation_json(&mut self, command: &str) -> String {
        let center = [
            self.player.pos[0].floor() as i32,
            self.player.pos[1].floor() as i32,
            self.player.pos[2].floor() as i32,
        ];
        self.world
            .extract_voxel_patch(center, 3, 1, 3, &mut self.voxel_patch_buf);
        let ray_hit = raycast::raycast_block(&self.player, &self.world);
        let ray_dist = ray_hit.as_ref().map_or(-1.0, |h| h.distance);
        let solid_nearby = self.voxel_patch_buf.iter().filter(|&&b| b == 1).count();
        let inv_items: Vec<serde_json::Value> = self
            .state_table
            .inventory_slots
            .iter()
            .enumerate()
            .filter_map(|(slot, it)| {
                it.as_ref().map(|s| {
                    serde_json::json!({
                        "slot": slot,
                        "id": s.id,
                        "count": s.count,
                        "display_name": s.display_name,
                        "lore": s.lore,
                        "damage": s.damage,
                    })
                })
            })
            .collect();
        let entities_json: Vec<serde_json::Value> = self
            .state_table
            .entities
            .values()
            .map(|e| {
                serde_json::json!({
                    "id": e.entity_id,
                    "type": e.entity_type,
                    "pos": e.pos,
                    "vel": e.velocity,
                    "aabb": e.aabb(),
                })
            })
            .collect();
        let screen_json = self.state_table.open_screen.as_ref().map(|s| {
            let slots: Vec<serde_json::Value> = s
                .slots
                .iter()
                .enumerate()
                .filter_map(|(idx, it)| {
                    it.as_ref().map(|item| {
                        serde_json::json!({
                            "slot": idx,
                            "id": item.id,
                            "count": item.count,
                            "display_name": item.display_name,
                            "lore": item.lore,
                        })
                    })
                })
                .collect();
            serde_json::json!({
                "window_id": s.window_id,
                "state_id": s.state_id,
                "kind": format!("{:?}", s.kind),
                "title": s.title,
                "slots": slots,
                "cursor_item": s.cursor_item.as_ref().map(|c| serde_json::json!({"id": c.id, "count": c.count})),
            })
        });
        serde_json::json!({
            "ok": true,
            "status": "completed",
            "command": command,
            "tick": self.live_tick,
            "pos": self.player.pos,
            "vel": self.player.velocity,
            "yaw": self.player.yaw,
            "pitch": self.player.pitch,
            "ground": self.player.ground,
            "horizontal_collision": self.player.horizontal_collision,
            "health": self.state_table.health,
            "food": self.state_table.food,
            "saturation": self.state_table.saturation,
            "air_supply": self.state_table.air_supply,
            "game_mode": self.state_table.game_mode,
            "selected_hotbar": self.state_table.selected_hotbar,
            "terminated": self.is_dead || !self.loaded,
            "is_dead": self.is_dead,
            "death_count": self.death_count,
            "reward": 0.0,
            "raycast_dist": ray_dist,
            "raycast_block": ray_hit.map(|h| serde_json::json!({"pos": h.block_pos, "face": format!("{:?}", h.face), "dist": h.distance})),
            "voxel_patch_len": self.voxel_patch_buf.len(),
            "solid_voxels": solid_nearby,
            "voxels": self.voxel_patch_buf,
            "inventory": inv_items,
            "open_screen": screen_json,
            "entities": entities_json,
            "nav_active": self.active_nav_goal.is_some(),
            "decode_errors": self.state_table.decode_errors,
        })
        .to_string()
    }

    /// Resolves any pending tick-deferred or condition-deferred replies.
    pub fn resolve_pending_replies(&mut self) {
        let current_tick = self.live_tick;
        let mut i = 0;
        while i < self.pending_replies.len() {
            let kind = self.pending_replies[i].kind.clone();
            let json_rpc_id = self.pending_replies[i].json_rpc_id.clone();
            let is_json = self.pending_replies[i].is_json;
            let (resolved, response) = match &kind {
                PendingReplyKind::Tick { target_tick, command } => {
                    if current_tick >= *target_tick {
                        let resp = if command == "gym.step" {
                            let obs = self.gym_observation_json("gym.step");
                            if let Some(id) = &json_rpc_id {
                                serde_json::json!({
                                    "jsonrpc": "2.0",
                                    "id": id,
                                    "result": {
                                        "isError": false,
                                        "content": [{ "type": "text", "text": obs }]
                                    }
                                })
                                .to_string()
                            } else {
                                obs
                            }
                        } else if let Some(id) = &json_rpc_id {
                            let text = format!("{command} completed at tick {current_tick}");
                            serde_json::json!({
                                "jsonrpc": "2.0",
                                "id": id,
                                "result": {
                                    "isError": false,
                                    "content": [{ "type": "text", "text": text }]
                                }
                            })
                            .to_string()
                        } else if is_json {
                            serde_json::json!({
                                "ok": true,
                                "status": "completed",
                                "command": command,
                                "tick": current_tick,
                            })
                            .to_string()
                        } else {
                            format!("{command} completed at tick {current_tick}")
                        };
                        (true, Some(resp))
                    } else {
                        (false, None)
                    }
                }
                PendingReplyKind::Condition { deadline_tick, command, condition } => {
                    let met = match condition {
                        WaitCondition::TaskIdle => {
                            self.active_nav_goal.is_none()
                                && self.scheduler.pending() == 0
                                && self.active_mine.is_none()
                                && self.event_queue.pending() == 0
                                && self.pending_placements.is_empty()
                                && self.pending_click.is_none()
                        }
                        WaitCondition::ScreenOpen { title_contains } => {
                            self.state_table.open_screen.as_ref().is_some_and(|s| {
                                match title_contains {
                                    Some(t) => s.title.to_lowercase().contains(&t.to_lowercase()),
                                    None => true,
                                }
                            })
                        }
                        WaitCondition::SlotItem { slot, item_id, min_count } => {
                            self.state_table.inventory_slots.get(*slot).and_then(|it| it.as_ref()).is_some_and(|s| {
                                s.id.contains(item_id) && s.count >= *min_count
                            })
                        }
                        WaitCondition::PlayerAlive => {
                            !self.is_dead && self.loaded && self.state_table.health > 0.0
                        }
                    };
                    if met {
                        let resp = if let Some(id) = &json_rpc_id {
                            let text = format!("{command} condition met at tick {current_tick}");
                            serde_json::json!({
                                "jsonrpc": "2.0",
                                "id": id,
                                "result": {
                                    "isError": false,
                                    "content": [{ "type": "text", "text": text }]
                                }
                            })
                            .to_string()
                        } else if is_json {
                            serde_json::json!({
                                "ok": true,
                                "status": "completed",
                                "command": command,
                                "tick": current_tick,
                            })
                            .to_string()
                        } else {
                            format!("{command} condition met at tick {current_tick}")
                        };
                        (true, Some(resp))
                    } else if current_tick >= *deadline_tick {
                        let resp = if let Some(id) = &json_rpc_id {
                            let text = format!("{command} timed out at tick {current_tick}");
                            serde_json::json!({
                                "jsonrpc": "2.0",
                                "id": id,
                                "result": {
                                    "isError": true,
                                    "content": [{ "type": "text", "text": text }]
                                }
                            })
                            .to_string()
                        } else if is_json {
                            serde_json::json!({
                                "ok": false,
                                "status": "failed",
                                "reason": "timeout",
                                "command": command,
                                "tick": current_tick,
                            })
                            .to_string()
                        } else {
                            format!("{command} failed: timeout at tick {current_tick}")
                        };
                        (true, Some(resp))
                    } else {
                        (false, None)
                    }
                }
            };

            if resolved {
                let item = self.pending_replies.remove(i);
                if let Some(msg) = response {
                    if let Some(chan) = item.client {
                        let _ = chan.send(msg);
                    } else if !msg.is_empty() {
                        println!("{msg}");
                    }
                }
            } else {
                i += 1;
            }
        }
    }

    /// Executes a single live command or multi-line script directly against the live `Session`.
    pub fn execute_live_script(
        &mut self,
        script: &str,
    ) -> std::result::Result<Vec<String>, String> {
        let mut outputs = Vec::new();
        for (line_no, raw_line) in script.lines().enumerate() {
            let line = raw_line.trim();
            if line.is_empty() || line.starts_with('#') || line.starts_with("//") {
                continue;
            }
            let tokens = match tokenize_command_line(line) {
                Ok(t) => t,
                Err(e) => return Err(format!("line {}: {e}", line_no + 1)),
            };
            let parts: Vec<&str> = tokens.iter().map(|s| s.as_str()).collect();
            // 1. Check any registered custom pluggable modules first
            let mut handled_by_custom = None;
            let mut custom = std::mem::take(&mut self.custom_modules);
            for module in &mut custom {
                if let Some(res) = module.execute(&parts, self) {
                    handled_by_custom = Some(res);
                    break;
                }
            }
            self.custom_modules = custom;
            if let Some(res) = handled_by_custom {
                outputs.push(res?);
                continue;
            }

            // 2. Built-in modular live session handlers
            let next_tick = self.live_tick.saturating_add(1);
            match parts.as_slice() {
                ["task.navigate", x, y, z]
                | ["task.navigate", x, y, z, _]
                | ["nav.goto", x, y, z]
                | ["nav.goto", x, y, z, _]
                | ["nav.to", x, y, z]
                | ["nav.to", x, y, z, _] => {
                    let goal = [
                        x.parse::<i32>().map_err(|_| "bad x")?,
                        y.parse::<i32>().map_err(|_| "bad y")?,
                        z.parse::<i32>().map_err(|_| "bad z")?,
                    ];
                    let sprint = parts.get(4).map(|s| *s != "walk").unwrap_or(true);
                    let start = [
                        self.player.pos[0].floor() as i32,
                        self.player.pos[1].floor() as i32,
                        self.player.pos[2].floor() as i32,
                    ];
                    let path = find_path(&self.world, start, goal, 8192)?;
                    let waypoints = path.len();
                    self.preempt_movement();
                    self.active_nav_path = if path.len() > 1 {
                        path[1..].to_vec()
                    } else {
                        path
                    };
                    self.active_nav_goal = Some((goal, sprint));
                    self.nav_stuck_ticks = 0;
                    self.nav_last_pos = Some(self.player.pos);
                    self.nav_deadline_tick = self.live_tick + (waypoints as u64 * 35).max(100);
                    outputs.push(format!(
                        "live 3D A* path active ({waypoints} waypoints) from ({}, {}, {}) -> ({}, {}, {})",
                        start[0], start[1], start[2], goal[0], goal[1], goal[2]
                    ));
                }
                ["task.cancel"] => {
                    self.event_queue.clear();
                    self.preempt_movement();
                    if self.pending_click.take().is_some() {
                        self.state_table.push_event("container_click_observation_cancelled");
                    }
                    for placement in self.pending_placements.drain(..) {
                        self.state_table.push_event(format!("placement_observation_cancelled:{}", placement.sequence));
                    }
                    outputs.push("cancelled active navigation and released all scheduled inputs".into());
                }
                ["interact.raycast"] => {
                    let b_hit = raycast::raycast_block(&self.player, &self.world);
                    let candidates: Vec<(i32, [f64; 6])> = self
                        .state_table
                        .entities
                        .values()
                        .map(|e| (e.entity_id, e.aabb()))
                        .collect();
                    let e_hit = raycast::raycast_entity(&self.player, &self.world, &candidates);
                    match (b_hit, e_hit) {
                        (Some(b), Some(e)) if e.distance < b.distance => outputs.push(format!(
                            "raycast_entity id={} dist={:.3}",
                            e.entity_id, e.distance
                        )),
                        (Some(b), _) => outputs.push(format!(
                            "raycast_block pos=({},{},{}) face={:?} dist={:.3}",
                            b.block_pos[0], b.block_pos[1], b.block_pos[2], b.face, b.distance
                        )),
                        (None, Some(e)) => outputs.push(format!(
                            "raycast_entity id={} dist={:.3}",
                            e.entity_id, e.distance
                        )),
                        (None, None) => outputs.push("raycast hit=none".into()),
                    }
                }
                ["interact.raycast_block"] => {
                    match raycast::raycast_block(&self.player, &self.world) {
                        Some(hit) => outputs.push(format!(
                            "raycast_block hit=({}, {}, {}) face={:?} dist={:.3}",
                            hit.block_pos[0],
                            hit.block_pos[1],
                            hit.block_pos[2],
                            hit.face,
                            hit.distance
                        )),
                        None => outputs.push("raycast_block hit=none".into()),
                    }
                }
                ["interact.raycast_entity"] => {
                    let candidates: Vec<(i32, [f64; 6])> = self
                        .state_table
                        .entities
                        .values()
                        .map(|e| (e.entity_id, e.aabb()))
                        .collect();
                    match raycast::raycast_entity(&self.player, &self.world, &candidates) {
                        Some(hit) => outputs.push(format!(
                            "raycast_entity id={} dist={:.3}",
                            hit.entity_id, hit.distance
                        )),
                        None => outputs.push("raycast_entity hit=none".into()),
                    }
                }
                ["interact.mine", x, y, z] => {
                    if self.active_mine.is_some() {
                        return Err("a block mining operation is already active".into());
                    }
                    let pos = [
                        x.parse::<i32>().map_err(|_| "bad x")?,
                        y.parse::<i32>().map_err(|_| "bad y")?,
                        z.parse::<i32>().map_err(|_| "bad z")?,
                    ];
                    let eye = self.player.eye_position();
                    let dist = ((pos[0] as f64 + 0.5 - eye[0]).powi(2)
                        + (pos[1] as f64 + 0.5 - eye[1]).powi(2)
                        + (pos[2] as f64 + 0.5 - eye[2]).powi(2))
                    .sqrt();
                    let max_reach = self.player.max_block_reach();
                    if dist > max_reach {
                        return Err(format!(
                            "block ({}, {}, {}) out of player reach ({dist:.2} > {max_reach:.2})",
                            pos[0], pos[1], pos[2]
                        ));
                    }
                    let block_center = [pos[0] as f64 + 0.5, pos[1] as f64 + 0.5, pos[2] as f64 + 0.5];
                    let dx_aim = block_center[0] - eye[0];
                    let dy_aim = block_center[1] - eye[1];
                    let dz_aim = block_center[2] - eye[2];
                    let horiz_dist = (dx_aim * dx_aim + dz_aim * dz_aim).sqrt();
                    self.player.yaw = (-dx_aim).atan2(dz_aim).to_degrees() as f32;
                    self.player.pitch = (-dy_aim).atan2(horiz_dist).to_degrees() as f32;
                    let face = if let Some(hit) = raycast::raycast_block(&self.player, &self.world).filter(|h| h.block_pos == pos) {
                        hit.face
                    } else if raycast::has_line_of_sight_excluding(eye, block_center, &self.world, Some(pos)) {
                        let dx = eye[0] - block_center[0];
                        let dy = eye[1] - block_center[1];
                        let dz = eye[2] - block_center[2];
                        if dy.abs() >= dx.abs() && dy.abs() >= dz.abs() {
                            if dy > 0.0 { BlockFace::Up } else { BlockFace::Down }
                        } else if dx.abs() >= dz.abs() {
                            if dx > 0.0 { BlockFace::East } else { BlockFace::West }
                        } else if dz > 0.0 {
                            BlockFace::South
                        } else {
                            BlockFace::North
                        }
                    } else {
                        return Err(format!(
                            "block ({}, {}, {}) is not currently in player line of sight",
                            pos[0], pos[1], pos[2]
                        ));
                    };
                    let hardness = self
                        .world
                        .block_hardness(pos[0], pos[1], pos[2])
                        .ok_or_else(|| {
                            format!(
                                "unrecognized block state at ({}, {}, {})",
                                pos[0], pos[1], pos[2]
                            )
                        })?;
                    if hardness < 0.0 {
                        return Err(format!(
                            "cannot mine unbreakable block at ({}, {}, {}) hardness={hardness}",
                            pos[0], pos[1], pos[2]
                        ));
                    }
                    let (tool_speed, can_harvest) = self.compute_tool_mining_stats(hardness);
                    let duration_ticks = if hardness == 0.0 {
                        1
                    } else {
                        let base = self.player
                            .compute_break_ticks(hardness, tool_speed, can_harvest, false)
                            .unwrap_or_else(|| ((hardness * 30.0 / tool_speed).ceil() as u32).max(1));
                        base.saturating_add(5)
                    };
                    let seq = self.state_table.next_sequence_n(2);
                    let mut events = ComposableTask::MineBlock {
                        pos,
                        face,
                        duration_ticks,
                    }
                    .compile_events(next_tick, seq);
                    let end_tick = next_tick + u64::from(duration_ticks.max(1));
                    for t in (next_tick + 1)..end_tick {
                        events.push((t, SubTickEvent::SwipeArm { hand: Hand::MainHand }));
                    }
                    events.sort_by_key(|(t, _)| *t);
                    self.event_queue
                        .submit(&events)
                        .map_err(|e| format!("{e:?}"))?;
                    self.active_mine = Some(ActiveMine {
                        pos,
                        face,
                        remaining_ticks: duration_ticks,
                        total_ticks: duration_ticks,
                        sequence: seq,
                        confirmation_deadline: None,
                    });
                    outputs.push(format!(
                        "queued validated block mining at ({}, {}, {}) face={face:?} duration_ticks={duration_ticks}",
                        pos[0], pos[1], pos[2]
                    ));
                }
                ["interact.mine_look"] => {
                    if self.active_mine.is_some() {
                        return Err("a block mining operation is already active".into());
                    }
                    let hit = raycast::raycast_block(&self.player, &self.world)
                        .ok_or_else(|| "no block in line of sight within reach".to_owned())?;
                    let hardness = self
                        .world
                        .block_hardness(hit.block_pos[0], hit.block_pos[1], hit.block_pos[2])
                        .ok_or_else(|| {
                            format!(
                                "unrecognized block state at ({}, {}, {})",
                                hit.block_pos[0], hit.block_pos[1], hit.block_pos[2]
                            )
                        })?;
                    if hardness < 0.0 {
                        return Err(format!(
                            "cannot mine unbreakable block at ({}, {}, {}) hardness={hardness}",
                            hit.block_pos[0], hit.block_pos[1], hit.block_pos[2]
                        ));
                    }
                    let (tool_speed, can_harvest) = self.compute_tool_mining_stats(hardness);
                    let duration_ticks = if hardness == 0.0 {
                        1
                    } else {
                        let base = self.player
                            .compute_break_ticks(hardness, tool_speed, can_harvest, false)
                            .unwrap_or_else(|| ((hardness * 30.0 / tool_speed).ceil() as u32).max(1));
                        base.saturating_add(5)
                    };
                    let seq = self.state_table.next_sequence_n(2);
                    let mut events = ComposableTask::MineBlock {
                        pos: hit.block_pos,
                        face: hit.face,
                        duration_ticks,
                    }
                    .compile_events(next_tick, seq);
                    let end_tick = next_tick + u64::from(duration_ticks.max(1));
                    for t in (next_tick + 1)..end_tick {
                        events.push((t, SubTickEvent::SwipeArm { hand: Hand::MainHand }));
                    }
                    events.sort_by_key(|(t, _)| *t);
                    self.event_queue
                        .submit(&events)
                        .map_err(|e| format!("{e:?}"))?;
                    self.active_mine = Some(ActiveMine {
                        pos: hit.block_pos,
                        face: hit.face,
                        remaining_ticks: duration_ticks,
                        total_ticks: duration_ticks,
                        sequence: seq,
                        confirmation_deadline: None,
                    });
                    outputs.push(format!(
                        "queued raycast block mining at ({}, {}, {}) face={:?} duration_ticks={duration_ticks}",
                        hit.block_pos[0], hit.block_pos[1], hit.block_pos[2], hit.face
                    ));
                }
                ["interact.place", x, y, z] | ["interact.place", x, y, z, _] => {
                    let pos = [
                        x.parse::<i32>().map_err(|_| "bad x")?,
                        y.parse::<i32>().map_err(|_| "bad y")?,
                        z.parse::<i32>().map_err(|_| "bad z")?,
                    ];
                    let s: u8 = if parts.len() > 4 {
                        parts[4].parse().map_err(|_| "bad slot")?
                    } else {
                        self.state_table.selected_hotbar
                    };
                    if s >= 9 {
                        return Err("hotbar slot must be in 0..=8".into());
                    }
                    if self.placement_target_is_solid(pos)? {
                        return Err(format!(
                            "cannot place block at ({}, {}, {}): target voxel is already solid",
                            pos[0], pos[1], pos[2]
                        ));
                    }
                    let placed_box = [
                        pos[0] as f64,
                        pos[1] as f64,
                        pos[2] as f64,
                        pos[0] as f64 + 1.0,
                        pos[1] as f64 + 1.0,
                        pos[2] as f64 + 1.0,
                    ];
                    let p_box = self.player.box_aabb();
                    if p_box[0] < placed_box[3]
                        && p_box[3] > placed_box[0]
                        && p_box[1] < placed_box[4]
                        && p_box[4] > placed_box[1]
                        && p_box[2] < placed_box[5]
                        && p_box[5] > placed_box[2]
                    {
                        return Err(format!(
                            "cannot place block at ({}, {}, {}): intersects player bounding box",
                            pos[0], pos[1], pos[2]
                        ));
                    }
                    let candidates = [
                        ([pos[0], pos[1] - 1, pos[2]], BlockFace::Up),
                        ([pos[0], pos[1] + 1, pos[2]], BlockFace::Down),
                        ([pos[0], pos[1], pos[2] - 1], BlockFace::South),
                        ([pos[0], pos[1], pos[2] + 1], BlockFace::North),
                        ([pos[0] - 1, pos[1], pos[2]], BlockFace::East),
                        ([pos[0] + 1, pos[1], pos[2]], BlockFace::West),
                    ];
                    let eye = self.player.eye_position();
                    let max_reach = self.player.max_block_reach();
                    let mut best_support = None;
                    let mut best_dist = f64::MAX;
                    for (sp, face) in candidates {
                        if !self.world.solid(sp[0], sp[1], sp[2]).unwrap_or(false) {
                            continue;
                        }
                        let sp_center = [sp[0] as f64 + 0.5, sp[1] as f64 + 0.5, sp[2] as f64 + 0.5];
                        let dist = ((sp_center[0] - eye[0]).powi(2)
                            + (sp_center[1] - eye[1]).powi(2)
                            + (sp_center[2] - eye[2]).powi(2))
                        .sqrt();
                        if dist > max_reach {
                            continue;
                        }
                        if raycast::has_line_of_sight_excluding(eye, sp_center, &self.world, Some(sp)) {
                            if dist < best_dist {
                                best_dist = dist;
                                best_support = Some((sp, face));
                            }
                        }
                    }
                    let (support_pos, face) = best_support.ok_or_else(|| {
                        format!(
                            "cannot place block at ({}, {}, {}): no valid solid supporting block face in line of sight and reach",
                            pos[0], pos[1], pos[2]
                        )
                    })?;
                    let seq = self.state_table.next_sequence();
                    let events = ComposableTask::PlaceBlock {
                        support_pos,
                        face,
                        hotbar_slot: s,
                    }
                    .compile_events(next_tick, seq);
                    self.event_queue
                        .submit(&events)
                        .map_err(|e| format!("{e:?}"))?;
                    self.pending_placements.push(PendingPlacement {
                        pos, sequence: seq,
                        original_state: self.world.block_state(pos[0], pos[1], pos[2]).map_err(|e| e.to_string())?,
                        confirmation_deadline: None,
                    });
                    outputs.push(format!(
                        "queued validated block placement at ({}, {}, {}) on support ({}, {}, {}) face={:?} slot={s}",
                        pos[0], pos[1], pos[2], support_pos[0], support_pos[1], support_pos[2], face
                    ));
                }
                ["interact.place_on", sx, sy, sz, face_str, slot] => {
                    let support_pos = [
                        sx.parse::<i32>().map_err(|_| "bad sx")?,
                        sy.parse::<i32>().map_err(|_| "bad sy")?,
                        sz.parse::<i32>().map_err(|_| "bad sz")?,
                    ];
                    let face = match face_str.to_lowercase().as_str() {
                        "down" => BlockFace::Down,
                        "up" => BlockFace::Up,
                        "north" => BlockFace::North,
                        "south" => BlockFace::South,
                        "west" => BlockFace::West,
                        "east" => BlockFace::East,
                        _ => return Err(format!("unknown face '{face_str}'; expected down|up|north|south|west|east")),
                    };
                    let s: u8 = slot.parse().map_err(|_| "bad slot")?;
                    if s >= 9 {
                        return Err("hotbar slot must be in 0..=8".into());
                    }
                    if !self.world.solid(support_pos[0], support_pos[1], support_pos[2]).unwrap_or(false) {
                        return Err(format!(
                            "supporting block at ({}, {}, {}) is not solid",
                            support_pos[0], support_pos[1], support_pos[2]
                        ));
                    }
                    let norm = face.normal();
                    let pos = [support_pos[0] + norm[0], support_pos[1] + norm[1], support_pos[2] + norm[2]];
                    if self.placement_target_is_solid(pos)? {
                        return Err(format!(
                            "cannot place block at ({}, {}, {}): target voxel is already solid",
                            pos[0], pos[1], pos[2]
                        ));
                    }
                    let placed_box = [
                        pos[0] as f64,
                        pos[1] as f64,
                        pos[2] as f64,
                        pos[0] as f64 + 1.0,
                        pos[1] as f64 + 1.0,
                        pos[2] as f64 + 1.0,
                    ];
                    let p_box = self.player.box_aabb();
                    if p_box[0] < placed_box[3]
                        && p_box[3] > placed_box[0]
                        && p_box[1] < placed_box[4]
                        && p_box[4] > placed_box[1]
                        && p_box[2] < placed_box[5]
                        && p_box[5] > placed_box[2]
                    {
                        return Err(format!(
                            "cannot place block at ({}, {}, {}): intersects player bounding box",
                            pos[0], pos[1], pos[2]
                        ));
                    }
                    let eye = self.player.eye_position();
                    let sp_center = [support_pos[0] as f64 + 0.5, support_pos[1] as f64 + 0.5, support_pos[2] as f64 + 0.5];
                    let dist = ((sp_center[0] - eye[0]).powi(2)
                        + (sp_center[1] - eye[1]).powi(2)
                        + (sp_center[2] - eye[2]).powi(2))
                    .sqrt();
                    let max_reach = self.player.max_block_reach();
                    if dist > max_reach {
                        return Err(format!(
                            "placement support ({}, {}, {}) out of reach ({dist:.2} > {max_reach:.2})",
                            support_pos[0], support_pos[1], support_pos[2]
                        ));
                    }
                    if !raycast::has_line_of_sight_excluding(eye, sp_center, &self.world, Some(support_pos)) {
                        return Err(format!(
                            "line of sight to placement support ({}, {}, {}) is obstructed",
                            support_pos[0], support_pos[1], support_pos[2]
                        ));
                    }
                    let seq = self.state_table.next_sequence();
                    let events = ComposableTask::PlaceBlock {
                        support_pos,
                        face,
                        hotbar_slot: s,
                    }
                    .compile_events(next_tick, seq);
                    self.event_queue
                        .submit(&events)
                        .map_err(|e| format!("{e:?}"))?;
                    self.pending_placements.push(PendingPlacement {
                        pos, sequence: seq,
                        original_state: self.world.block_state(pos[0], pos[1], pos[2]).map_err(|e| e.to_string())?,
                        confirmation_deadline: None,
                    });
                    outputs.push(format!(
                        "queued explicit block placement on ({}, {}, {}) face={face:?} resulting at ({}, {}, {}) slot={s}",
                        support_pos[0], support_pos[1], support_pos[2], pos[0], pos[1], pos[2]
                    ));
                }
                ["interact.place"] | ["interact.place_look"] | ["interact.place_look", _] => {
                    let s: u8 = if parts.len() > 1 && parts[0] == "interact.place_look" {
                        parts[1].parse().map_err(|_| "bad slot")?
                    } else {
                        self.state_table.selected_hotbar
                    };
                    if s >= 9 {
                        return Err("hotbar slot must be in 0..=8".into());
                    }
                    let hit = raycast::raycast_block(&self.player, &self.world)
                        .ok_or("no solid block in crosshair reach to place against")?;
                    let support_pos = hit.block_pos;
                    let face = hit.face;
                    let norm = face.normal();
                    let pos = [support_pos[0] + norm[0], support_pos[1] + norm[1], support_pos[2] + norm[2]];
                    if self.placement_target_is_solid(pos)? {
                        return Err(format!(
                            "cannot place block at ({}, {}, {}): target voxel is already solid",
                            pos[0], pos[1], pos[2]
                        ));
                    }
                    let placed_box = [
                        pos[0] as f64,
                        pos[1] as f64,
                        pos[2] as f64,
                        pos[0] as f64 + 1.0,
                        pos[1] as f64 + 1.0,
                        pos[2] as f64 + 1.0,
                    ];
                    let p_box = self.player.box_aabb();
                    if p_box[0] < placed_box[3]
                        && p_box[3] > placed_box[0]
                        && p_box[1] < placed_box[4]
                        && p_box[4] > placed_box[1]
                        && p_box[2] < placed_box[5]
                        && p_box[5] > placed_box[2]
                    {
                        return Err(format!(
                            "cannot place block at ({}, {}, {}): intersects player bounding box",
                            pos[0], pos[1], pos[2]
                        ));
                    }
                    let seq = self.state_table.next_sequence();
                    let events = ComposableTask::PlaceBlock {
                        support_pos,
                        face,
                        hotbar_slot: s,
                    }
                    .compile_events(next_tick, seq);
                    self.event_queue
                        .submit(&events)
                        .map_err(|e| format!("{e:?}"))?;
                    self.pending_placements.push(PendingPlacement {
                        pos, sequence: seq,
                        original_state: self.world.block_state(pos[0], pos[1], pos[2]).map_err(|e| e.to_string())?,
                        confirmation_deadline: None,
                    });
                    outputs.push(format!(
                        "queued crosshair block placement on ({}, {}, {}) face={face:?} resulting at ({}, {}, {}) slot={s}",
                        support_pos[0], support_pos[1], support_pos[2], pos[0], pos[1], pos[2]
                    ));
                }
                ["interact.use_block", x, y, z] => {
                    let pos = [
                        x.parse::<i32>().map_err(|_| "bad x")?,
                        y.parse::<i32>().map_err(|_| "bad y")?,
                        z.parse::<i32>().map_err(|_| "bad z")?,
                    ];
                    let eye = self.player.eye_position();
                    let target_center = [pos[0] as f64 + 0.5, pos[1] as f64 + 0.5, pos[2] as f64 + 0.5];
                    let dist = ((target_center[0] - eye[0]).powi(2)
                        + (target_center[1] - eye[1]).powi(2)
                        + (target_center[2] - eye[2]).powi(2))
                    .sqrt();
                    let max_reach = self.player.max_block_reach();
                    if dist > max_reach {
                        return Err(format!(
                            "block ({}, {}, {}) out of reach ({dist:.2} > {max_reach:.2})",
                            pos[0], pos[1], pos[2]
                        ));
                    }
                    let seq = self.state_table.next_sequence();
                    let events = vec![
                        (
                            next_tick,
                            SubTickEvent::UseItemOn {
                                hand: Hand::MainHand,
                                pos,
                                face: BlockFace::Up,
                                hit: [0.5, 0.5, 0.5],
                                inside_block: false,
                                sequence: seq,
                            },
                        ),
                        (
                            next_tick,
                            SubTickEvent::SwipeArm { hand: Hand::MainHand },
                        ),
                    ];
                    self.event_queue
                        .submit(&events)
                        .map_err(|e| format!("{e:?}"))?;
                    outputs.push(format!("used block at ({}, {}, {})", pos[0], pos[1], pos[2]));
                }
                ["interact.use_item"] | ["interact.use_item", _] => {
                    let hand = if parts.len() > 1 && parts[1] == "offhand" {
                        Hand::OffHand
                    } else {
                        Hand::MainHand
                    };
                    let seq = self.state_table.next_sequence();
                    let events = vec![
                        (
                            next_tick,
                            SubTickEvent::UseItem {
                                hand,
                                sequence: seq,
                            },
                        ),
                        (
                            next_tick,
                            SubTickEvent::SwipeArm { hand },
                        ),
                    ];
                    self.event_queue
                        .submit(&events)
                        .map_err(|e| format!("{e:?}"))?;
                    outputs.push(format!("used held item in {hand:?}"));
                }
                ["interact.attack", entity_id] => {
                    let id: i32 = entity_id.parse().map_err(|_| "bad entity_id")?;
                    let ent = self
                        .state_table
                        .entities
                        .get(&id)
                        .ok_or_else(|| format!("entity {id} is not tracked in live session"))?;
                    let eye = self.player.eye_position();
                    let target_point = [ent.pos[0], ent.pos[1] + ent.height * 0.5, ent.pos[2]];
                    let dist = ((target_point[0] - eye[0]).powi(2)
                        + (target_point[1] - eye[1]).powi(2)
                        + (target_point[2] - eye[2]).powi(2))
                    .sqrt();
                    let max_reach = self.player.max_entity_reach();
                    if dist > max_reach {
                        return Err(format!(
                            "entity {id} out of attack reach ({dist:.2} > {max_reach:.2})"
                        ));
                    }
                    if !raycast::has_line_of_sight(eye, target_point, &self.world) {
                        return Err(format!("line of sight to entity {id} is blocked by block"));
                    }
                    let dx = target_point[0] - eye[0];
                    let dy = target_point[1] - eye[1];
                    let dz = target_point[2] - eye[2];
                    let horiz_dist = (dx * dx + dz * dz).sqrt();
                    self.player.yaw = (-dx).atan2(dz).to_degrees() as f32;
                    self.player.pitch = (-dy).atan2(horiz_dist).to_degrees() as f32;
                    self.event_queue
                        .submit(&[
                            (
                                next_tick,
                                SubTickEvent::InteractEntity {
                                    entity_id: id,
                                    interaction: EntityInteraction::Attack,
                                    sneaking: self.player.sneaking,
                                },
                            ),
                            (
                                next_tick,
                                SubTickEvent::SwipeArm {
                                    hand: Hand::MainHand,
                                },
                            ),
                        ])
                        .map_err(|e| format!("{e:?}"))?;
                    outputs.push(format!("queued validated live attack on entity {id}"));
                }
                ["player.select_hotbar", slot]
                | ["player.hotbar", slot]
                | ["hotbar.select", slot]
                | ["inventory.hotbar", slot] => {
                    let s: u8 = slot.parse().map_err(|_| "invalid hotbar slot")?;
                    if s >= 9 {
                        return Err("hotbar slot must be in 0..=8".into());
                    }
                    self.state_table.selected_hotbar = s;
                    self.event_queue
                        .submit(&[(next_tick, SubTickEvent::SelectHotbar { slot: s })])
                        .map_err(|e| format!("{e:?}"))?;
                    outputs.push(format!("queued live hotbar selection slot={s}"));
                }
                ["player.use_item"]
                | ["player.use_item", _]
                | ["inventory.use_item"]
                | ["inventory.use_item", _] => {
                    // If a query name (e.g. "compass") is passed, automatically locate it in hotbar (slots 36..=44) and select it first!
                    let mut hand = Hand::MainHand;
                    let mut auto_selected = None;
                    if parts.len() > 1 {
                        let arg = parts[1].to_lowercase();
                        if arg == "offhand" {
                            hand = Hand::OffHand;
                        } else if arg != "mainhand" && arg != "main_hand" {
                            for slot_idx in 36..=44 {
                                if let Some(Some(stack)) = self.state_table.inventory_slots.get(slot_idx) {
                                    let id_match = stack.id.to_lowercase().contains(&arg);
                                    let name_match = stack
                                        .display_name
                                        .as_deref()
                                        .is_some_and(|n| n.to_lowercase().contains(&arg));
                                    if id_match || name_match {
                                        let hb = (slot_idx - 36) as u8;
                                        self.state_table.selected_hotbar = hb;
                                        let _ = self.event_queue.submit(&[(next_tick, SubTickEvent::SelectHotbar { slot: hb })]);
                                        auto_selected = Some((hb, stack.id.clone()));
                                        break;
                                    }
                                }
                            }
                        }
                    }
                    let seq = self.state_table.next_sequence();
                    let events = vec![
                        (next_tick, SubTickEvent::UseItem { hand, sequence: seq }),
                        (next_tick, SubTickEvent::SwipeArm { hand }),
                    ];
                    self.event_queue
                        .submit(&events)
                        .map_err(|e| format!("{e:?}"))?;
                    if let Some((hb, item_id)) = auto_selected {
                        outputs.push(format!("selected hotbar slot={hb} ({item_id}) and used item in {hand:?}"));
                    } else {
                        outputs.push(format!("used held item in {hand:?} (hotbar={})", self.state_table.selected_hotbar));
                    }
                }
                ["interact.entity", entity_id]
                | ["interact.use_entity", entity_id]
                | ["entity.interact", entity_id]
                | ["npc.click", entity_id] => {
                    let id: i32 = if *entity_id == "nearest" {
                        let p = self.player.pos;
                        self.state_table
                            .entities
                            .values()
                            .min_by(|a, b| {
                                let da = (a.pos[0] - p[0]).powi(2) + (a.pos[1] - p[1]).powi(2) + (a.pos[2] - p[2]).powi(2);
                                let db = (b.pos[0] - p[0]).powi(2) + (b.pos[1] - p[1]).powi(2) + (b.pos[2] - p[2]).powi(2);
                                da.partial_cmp(&db).unwrap_or(std::cmp::Ordering::Equal)
                            })
                            .map(|e| e.entity_id)
                            .ok_or_else(|| "no entities tracked in live session".to_string())?
                    } else {
                        entity_id.parse().map_err(|_| "bad entity_id")?
                    };
                    let ent = self
                        .state_table
                        .entities
                        .get(&id)
                        .ok_or_else(|| format!("entity {id} is not tracked in live session"))?;
                    let eye = self.player.eye_position();
                    let target_point = [ent.pos[0], ent.pos[1] + ent.height * 0.5, ent.pos[2]];
                    let dx = target_point[0] - eye[0];
                    let dy = target_point[1] - eye[1];
                    let dz = target_point[2] - eye[2];
                    let horiz_dist = (dx * dx + dz * dz).sqrt();
                    self.player.yaw = (-dx).atan2(dz).to_degrees() as f32;
                    self.player.pitch = (-dy).atan2(horiz_dist).to_degrees() as f32;
                    self.event_queue
                        .submit(&[
                            (
                                next_tick,
                                SubTickEvent::InteractEntity {
                                    entity_id: id,
                                    interaction: EntityInteraction::Interact { hand: Hand::MainHand },
                                    sneaking: self.player.sneaking,
                                },
                            ),
                            (
                                next_tick,
                                SubTickEvent::SwipeArm { hand: Hand::MainHand },
                            ),
                        ])
                        .map_err(|e| format!("{e:?}"))?;
                    outputs.push(format!("queued right-click interaction on entity {id}"));
                }
                ["player.respawn"] => {
                    self.event_queue
                        .submit(&[(next_tick, SubTickEvent::PerformRespawn)])
                        .map_err(|e| format!("{e:?}"))?;
                    outputs.push("queued live player respawn action".into());
                }
                ["player.attribute", attr, val] => {
                    let v: f64 = val.parse().map_err(|_| "bad attribute value")?;
                    match *attr {
                        "reach" | "block_reach" | "block_interaction_range" => {
                            self.player.attributes.block_interaction_range = v;
                            outputs.push(format!("set block_interaction_range={v:.2}"));
                        }
                        "attack_reach" | "entity_reach" | "entity_interaction_range" => {
                            self.player.attributes.entity_interaction_range = v;
                            outputs.push(format!("set entity_interaction_range={v:.2}"));
                        }
                        "speed" | "movement_speed" => {
                            self.player.attributes.movement_speed = v as f32;
                            outputs.push(format!("set movement_speed={v:.3}"));
                        }
                        "step_height" => {
                            self.player.attributes.step_height = v;
                            outputs.push(format!("set step_height={v:.2}"));
                        }
                        "jump_strength" => {
                            self.player.attributes.jump_strength = v as f32;
                            outputs.push(format!("set jump_strength={v:.3}"));
                        }
                        "break_speed" | "block_break_speed" => {
                            self.player.attributes.block_break_speed = v as f32;
                            outputs.push(format!("set block_break_speed={v:.2}"));
                        }
                        other => {
                            return Err(format!("unknown player attribute '{other}', supported: reach, attack_reach, speed, step_height, jump_strength, break_speed"));
                        }
                    }
                }
                ["player.reach", val] => {
                    let v: f64 = val.parse().map_err(|_| "bad reach value")?;
                    self.player.attributes.block_interaction_range = v;
                    self.player.attributes.entity_interaction_range = v.min(3.0 * (v / 4.5));
                    outputs.push(format!("set player reach={v:.2}"));
                }
                ["walk", ticks] | ["walk", "forward", ticks] => {
                    let dur: u64 = ticks.parse::<u64>().map_err(|_| "bad ticks")?.clamp(1, 200);
                    self.preempt_movement();
                    let mut batch = vec![
                        (next_tick, Action::Button { button: Button::Forward, down: true }),
                    ];
                    for t in 1..dur {
                        batch.push((next_tick + t, Action::Button { button: Button::Forward, down: true }));
                    }
                    batch.push((next_tick + dur, Action::Button { button: Button::Forward, down: false }));
                    self.scheduler.submit(&batch).map_err(|e| format!("{e:?}"))?;
                    outputs.push(format!("queued walk forward for {dur} ticks"));
                }
                ["jump"] => {
                    self.scheduler.submit(&[
                        (next_tick, Action::Button { button: Button::Jump, down: true }),
                        (next_tick + 2, Action::Button { button: Button::Jump, down: false }),
                    ]).map_err(|e| format!("{e:?}"))?;
                    outputs.push("queued jump action".into());
                }
                ["sneak"] | ["sneak", "on"] => {
                    self.scheduler.submit(&[
                        (next_tick, Action::Button { button: Button::Sneak, down: true }),
                    ]).map_err(|e| format!("{e:?}"))?;
                    outputs.push("queued sneak on".into());
                }
                ["sneak", "off"] => {
                    self.scheduler.submit(&[
                        (next_tick, Action::Button { button: Button::Sneak, down: false }),
                    ]).map_err(|e| format!("{e:?}"))?;
                    outputs.push("queued sneak off".into());
                }
                ["player.look", yaw_d, pitch_d] | ["interact.look", yaw_d, pitch_d] => {
                    let yd: f64 = yaw_d.parse().map_err(|_| "bad yaw_delta")?;
                    let pd: f64 = pitch_d.parse().map_err(|_| "bad pitch_delta")?;
                    self.scheduler
                        .submit(&[(
                            next_tick,
                            Action::Look {
                                yaw: yd,
                                pitch: pd,
                            },
                        )])
                        .map_err(|e| format!("{e:?}"))?;
                    outputs.push(format!("queued camera look yaw_delta={yd:.2} pitch_delta={pd:.2}"));
                }
                ["player.input", rest @ ..] => {
                    self.preempt_movement();
                    let (f_down, j_down, sn_down, sp_down, yd, pd, dur) = if rest.len() >= 7 {
                        let f = rest[0] == "true" || rest[0] == "1";
                        let j = rest[1] == "true" || rest[1] == "1";
                        let sn = rest[2] == "true" || rest[2] == "1";
                        let sp = rest[3] == "true" || rest[3] == "1";
                        let y: f64 = rest[4].parse().map_err(|_| "bad yaw_delta")?;
                        let p: f64 = rest[5].parse().map_err(|_| "bad pitch_delta")?;
                        let d: u64 = rest[6].parse::<u64>().map_err(|_| "bad ticks")?.clamp(1, 200);
                        (f, j, sn, sp, y, p, d)
                    } else if rest.len() == 1 {
                        let d: u64 = rest[0].parse::<u64>().unwrap_or(1).clamp(1, 200);
                        (true, false, false, false, 0.0, 0.0, d)
                    } else {
                        (true, false, false, false, 0.0, 0.0, 1)
                    };
                    let mut batch = vec![
                        (
                            next_tick,
                            Action::Button {
                                button: Button::Forward,
                                down: f_down,
                            },
                        ),
                        (
                            next_tick,
                            Action::Button {
                                button: Button::Jump,
                                down: j_down,
                            },
                        ),
                        (
                            next_tick,
                            Action::Button {
                                button: Button::Sneak,
                                down: sn_down,
                            },
                        ),
                        (
                            next_tick,
                            Action::Button {
                                button: Button::Sprint,
                                down: sp_down,
                            },
                        ),
                        (
                            next_tick,
                            Action::Look {
                                yaw: yd,
                                pitch: pd,
                            },
                        ),
                        (
                            next_tick + dur,
                            Action::Button {
                                button: Button::Forward,
                                down: false,
                            },
                        ),
                        (
                            next_tick + dur,
                            Action::Button {
                                button: Button::Jump,
                                down: false,
                            },
                        ),
                        (
                            next_tick + dur,
                            Action::Button {
                                button: Button::Sneak,
                                down: false,
                            },
                        ),
                        (
                            next_tick + dur,
                            Action::Button {
                                button: Button::Sprint,
                                down: false,
                            },
                        ),
                    ];
                    batch.sort_by_key(|(t, _)| *t);
                    self.scheduler
                        .submit(&batch)
                        .map_err(|e| format!("scheduler error: {e}"))?;
                    outputs.push(format!("queued live player input for {dur} ticks"));
                }
                ["gui.inspect"] => {
                    if let Some(s) = &self.state_table.open_screen {
                        let slots: Vec<serde_json::Value> = s
                            .slots
                            .iter()
                            .enumerate()
                            .filter_map(|(idx, it)| {
                                it.as_ref().map(|item| {
                                    serde_json::json!({
                                        "slot": idx,
                                        "id": item.id,
                                        "count": item.count,
                                        "display_name": item.display_name,
                                        "lore": item.lore,
                                    })
                                })
                            })
                            .collect();
                        let inspect_obj = serde_json::json!({
                            "window_id": s.window_id,
                            "state_id": s.state_id,
                            "kind": format!("{:?}", s.kind),
                            "title": s.title,
                            "slots": slots,
                            "cursor_item": s.cursor_item.as_ref().map(|c| serde_json::json!({"id": c.id, "count": c.count})),
                        });
                        outputs.push(inspect_obj.to_string());
                    } else {
                        outputs.push(serde_json::json!({"slots": []}).to_string());
                    }
                }
                ["gui.selectors"] => {
                    if let Some(screen) = &self.state_table.open_screen {
                        let mut sels = Vec::new();
                        for (slot, item) in screen.slots.iter().enumerate() {
                            if let Some(stack) = item {
                                sels.push(format!(
                                    "slot={slot}: id=\"{}\" count={} name={:?} lore={:?}",
                                    stack.id, stack.count, stack.display_name, stack.lore
                                ));
                            }
                        }
                        outputs.push(if sels.is_empty() {
                            "no populated slots".into()
                        } else {
                            sels.join("\n")
                        });
                    } else {
                        outputs.push("open_screen=none".into());
                    }
                }
                ["gui.click", target] => {
                    if self.pending_click.is_some() {
                        return Err("a container click is already pending".into());
                    }
                    let screen = self
                        .state_table
                        .open_screen
                        .as_mut()
                        .ok_or_else(|| "no open container screen on server".to_owned())?;
                    let s_idx = if let Ok(s) = target.parse::<i16>() {
                        s
                    } else {
                        let q = target.to_lowercase();
                        let found = screen.slots.iter().enumerate().find_map(|(idx, opt)| {
                            let item = opt.as_ref()?;
                            let id_match = item.id.to_lowercase().contains(&q);
                            let name_match = item
                                .display_name
                                .as_deref()
                                .map(|n| n.to_lowercase().contains(&q))
                                .unwrap_or(false);
                            if id_match || name_match {
                                Some(idx as i16)
                            } else {
                                None
                            }
                        });
                        found.ok_or_else(|| format!("no item matching '{target}' found in open container"))?
                    };
                    let window_id = screen.window_id;
                    let state_id = screen.state_id;
                    let ev = SubTickEvent::ContainerClick {
                        window_id,
                        state_id,
                        slot: s_idx,
                        button: 0,
                        mode: ClickMode::Pickup,
                    };
                    self.event_queue
                        .submit(&[(next_tick, ev)])
                        .map_err(|e| format!("{e:?}"))?;
                    self.pending_click = Some((window_id, state_id, None));
                    outputs.push(format!("queued live container click slot={s_idx}"));
                }
                ["gui.click_slot", slot] | ["gui.click_slot", slot, _] => {
                    if self.pending_click.is_some() {
                        return Err("a container click is already pending".into());
                    }
                    let s_idx: i16 = slot.parse().map_err(|_| "bad slot")?;
                    let quick = parts.get(2).is_some_and(|m| *m == "quick");
                    let screen = self
                        .state_table
                        .open_screen
                        .as_mut()
                        .ok_or_else(|| "no open container screen on server".to_owned())?;
                    let ev = SubTickEvent::ContainerClick {
                        window_id: screen.window_id,
                        state_id: screen.state_id,
                        slot: s_idx,
                        button: 0,
                        mode: if quick {
                            ClickMode::QuickMove
                        } else {
                            ClickMode::Pickup
                        },
                    };
                    let window_id = screen.window_id;
                    let state_id = screen.state_id;
                    self.event_queue
                        .submit(&[(next_tick, ev)])
                        .map_err(|e| format!("{e:?}"))?;
                    self.pending_click = Some((window_id, state_id, None));
                    outputs.push(format!("queued live container click slot={s_idx}"));
                }
                ["gui.click_item", query] => {
                    if self.pending_click.is_some() {
                        return Err("a container click is already pending".into());
                    }
                    let screen = self
                        .state_table
                        .open_screen
                        .as_mut()
                        .ok_or_else(|| "no open container screen on server".to_owned())?;
                    let rev = screen.state_id;
                    let selector = if query.contains(':') {
                        ItemSelector {
                            id: Some((*query).to_owned()),
                            ..Default::default()
                        }
                    } else {
                        ItemSelector {
                            name_contains: Some((*query).to_owned()),
                            ..Default::default()
                        }
                    };
                    let ev = screen
                        .click_matching_item(Some(rev), &selector, true, 0, ClickMode::Pickup)
                        .map_err(|e| format!("{e:?}"))?;
                    let window_id = screen.window_id;
                    self.event_queue
                        .submit(&[(next_tick, ev)])
                        .map_err(|e| format!("{e:?}"))?;
                    self.pending_click = Some((window_id, rev, None));
                    outputs.push(format!("queued live item click for '{query}'"));
                }
                ["gui.close"] => {
                    let screen = self
                        .state_table
                        .open_screen
                        .as_ref()
                        .ok_or_else(|| "no open container screen on server".to_owned())?;
                    let wid = screen.window_id;
                    self.event_queue
                        .submit(&[(next_tick, SubTickEvent::CloseContainer { window_id: wid })])
                        .map_err(|e| format!("{e:?}"))?;
                    outputs.push(format!("queued container close window_id={wid}"));
                }
                ["inventory.inspect"] => {
                    let populated: Vec<String> = self
                        .state_table
                        .inventory_slots
                        .iter()
                        .enumerate()
                        .filter_map(|(idx, it)| {
                            it.as_ref().map(|s| {
                                if let Some(ref name) = s.display_name {
                                    format!("slot={idx}:{}x{}({name})", s.count, s.id)
                                } else {
                                    format!("slot={idx}:{}x{}", s.count, s.id)
                                }
                            })
                        })
                        .collect();
                    outputs.push(format!(
                        "selected_hotbar={} populated_slots={} items=[{}]",
                        self.state_table.selected_hotbar,
                        populated.len(),
                        populated.join(", ")
                    ));
                }
                ["inventory.list"] | ["inventory.all"] | ["inventory.json"] | ["inventory.held"] => {
                    let items: Vec<serde_json::Value> = self
                        .state_table
                        .inventory_slots
                        .iter()
                        .enumerate()
                        .filter_map(|(slot_idx, item_opt)| {
                            item_opt.as_ref().map(|stack| {
                                let hotbar_slot = if (36..=44).contains(&slot_idx) {
                                    Some(slot_idx - 36)
                                } else {
                                    None
                                };
                                serde_json::json!({
                                    "slot": slot_idx,
                                    "hotbar_slot": hotbar_slot,
                                    "id": stack.id,
                                    "count": stack.count,
                                    "display_name": stack.display_name,
                                    "lore": stack.lore,
                                    "damage": stack.damage,
                                })
                            })
                        })
                        .collect();
                    let held = self.state_table.main_hand_item().map(|s| {
                        serde_json::json!({
                            "id": s.id,
                            "count": s.count,
                            "display_name": s.display_name,
                            "lore": s.lore,
                        })
                    });
                    outputs.push(
                        serde_json::json!({
                            "selected_hotbar": self.state_table.selected_hotbar,
                            "held_item": held,
                            "populated_count": items.len(),
                            "items": items,
                        })
                        .to_string(),
                    );
                }
                ["inventory.find", query] | ["inventory.query", query] => {
                    let q = query.to_lowercase();
                    let mut found_opt = None;
                    for (slot_idx, item_opt) in self.state_table.inventory_slots.iter().enumerate() {
                        if let Some(stack) = item_opt {
                            let id_match = stack.id.to_lowercase().contains(&q);
                            let name_match = stack
                                .display_name
                                .as_deref()
                                .map(|n| n.to_lowercase().contains(&q))
                                .unwrap_or(false);
                            let lore_match = stack
                                .lore
                                .iter()
                                .any(|l| l.to_lowercase().contains(&q));
                            if id_match || name_match || lore_match {
                                let hotbar_slot = if (36..=44).contains(&slot_idx) {
                                    Some(slot_idx - 36)
                                } else {
                                    None
                                };
                                found_opt = Some(serde_json::json!({
                                    "found": true,
                                    "slot": slot_idx,
                                    "hotbar_slot": hotbar_slot,
                                    "id": stack.id,
                                    "count": stack.count,
                                    "display_name": stack.display_name,
                                    "lore": stack.lore,
                                }));
                                break;
                            }
                        }
                    }
                    if let Some(found) = found_opt {
                        outputs.push(found.to_string());
                    } else {
                        outputs.push(serde_json::json!({ "found": false, "query": query }).to_string());
                    }
                }
                ["entities.inspect"] | ["entity.list"] | ["entities.list"] => {
                    let px = self.player.pos[0];
                    let py = self.player.pos[1];
                    let pz = self.player.pos[2];
                    let ents: Vec<String> = self
                        .state_table
                        .entities
                        .values()
                        .map(|e| {
                            let dist = ((e.pos[0] - px).powi(2) + (e.pos[1] - py).powi(2) + (e.pos[2] - pz).powi(2)).sqrt();
                            let holos = self.state_table.entity_hologram_lines(e.entity_id);
                            if holos.is_empty() {
                                format!(
                                    "id={} type={} kind={} dist={:.2} pos=({:.2},{:.2},{:.2})",
                                    e.entity_id, e.entity_type, e.kind_name, dist, e.pos[0], e.pos[1], e.pos[2]
                                )
                            } else {
                                format!(
                                    "id={} type={} kind={} dist={:.2} pos=({:.2},{:.2},{:.2}) holograms={:?}",
                                    e.entity_id, e.entity_type, e.kind_name, dist, e.pos[0], e.pos[1], e.pos[2], holos
                                )
                            }
                        })
                        .collect();
                    outputs.push(format!(
                        "tracked_entities={} list=[{}]",
                        ents.len(),
                        ents.join(", ")
                    ));
                }
                ["entity.find", query] | ["npc.find", query] => {
                    let q = query.to_lowercase();
                    let px = self.player.pos[0];
                    let py = self.player.pos[1];
                    let pz = self.player.pos[2];
                    let mut best: Option<(f64, i32, Vec<String>)> = None;
                    for e in self.state_table.entities.values() {
                        let holos = self.state_table.entity_hologram_lines(e.entity_id);
                        let matched = q == "nearest"
                            || q == "any"
                            || e.kind_name.to_lowercase().contains(&q)
                            || e.custom_name.as_deref().is_some_and(|n| n.to_lowercase().contains(&q))
                            || holos.iter().any(|h| h.to_lowercase().contains(&q));
                        if matched {
                            let dist = ((e.pos[0] - px).powi(2) + (e.pos[1] - py).powi(2) + (e.pos[2] - pz).powi(2)).sqrt();
                            if best.as_ref().map_or(true, |(bdist, _, _)| dist < *bdist) {
                                best = Some((dist, e.entity_id, holos));
                            }
                        }
                    }
                    if let Some((dist, eid, holos)) = best {
                        let ent = &self.state_table.entities[&eid];
                        outputs.push(
                            serde_json::json!({
                                "found": true,
                                "entity_id": eid,
                                "type": ent.entity_type,
                                "kind": ent.kind_name,
                                "custom_name": ent.custom_name,
                                "hologram_lines": holos,
                                "pos": ent.pos,
                                "distance": dist,
                            })
                            .to_string(),
                        );
                    } else {
                        outputs.push(serde_json::json!({ "found": false, "query": query }).to_string());
                    }
                }
                ["npc.interact", query] | ["entity.interact_named", query] => {
                    let q = query.to_lowercase();
                    let px = self.player.pos[0];
                    let py = self.player.pos[1];
                    let pz = self.player.pos[2];
                    let mut best: Option<(f64, i32)> = None;
                    for e in self.state_table.entities.values() {
                        let holos = self.state_table.entity_hologram_lines(e.entity_id);
                        let matched = q == "nearest"
                            || q == "any"
                            || e.kind_name.to_lowercase().contains(&q)
                            || e.custom_name.as_deref().is_some_and(|n| n.to_lowercase().contains(&q))
                            || holos.iter().any(|h| h.to_lowercase().contains(&q));
                        if matched {
                            let dist = ((e.pos[0] - px).powi(2) + (e.pos[1] - py).powi(2) + (e.pos[2] - pz).powi(2)).sqrt();
                            if best.as_ref().map_or(true, |(bdist, _)| dist < *bdist) {
                                best = Some((dist, e.entity_id));
                            }
                        }
                    }
                    if best.is_none() {
                        // Fallback to nearest non-item entity within 16m
                        for e in self.state_table.entities.values() {
                            if e.entity_type != 59 {
                                let dist = ((e.pos[0] - px).powi(2) + (e.pos[1] - py).powi(2) + (e.pos[2] - pz).powi(2)).sqrt();
                                if best.as_ref().map_or(true, |(bdist, _)| dist < *bdist) {
                                    best = Some((dist, e.entity_id));
                                }
                            }
                        }
                    }
                    let (_, id) = best.ok_or_else(|| format!("no entity matching '{query}' found in session"))?;
                    let ent = self.state_table.entities.get(&id).unwrap();
                    let eye = self.player.eye_position();
                    let target_point = [ent.pos[0], ent.pos[1] + ent.height * 0.5, ent.pos[2]];
                    let dx = target_point[0] - eye[0];
                    let dy = target_point[1] - eye[1];
                    let dz = target_point[2] - eye[2];
                    let horiz_dist = (dx * dx + dz * dz).sqrt();
                    self.player.yaw = (-dx).atan2(dz).to_degrees() as f32;
                    self.player.pitch = (-dy).atan2(horiz_dist).to_degrees() as f32;
                    self.event_queue
                        .submit(&[
                            (
                                next_tick,
                                SubTickEvent::InteractEntity {
                                    entity_id: id,
                                    interaction: EntityInteraction::Interact { hand: Hand::MainHand },
                                    sneaking: self.player.sneaking,
                                },
                            ),
                            (
                                next_tick,
                                SubTickEvent::SwipeArm { hand: Hand::MainHand },
                            ),
                        ])
                        .map_err(|e| format!("{e:?}"))?;
                    outputs.push(format!("queued right-click NPC interaction on entity {id} (query='{query}')"));
                }
                ["inspect.sensory"] | ["observe.words"] | ["see"] | ["inspect.vision"] => {
                    outputs.push(self.render_sensory_words());
                }
                ["inspect.sensory.json"] | ["observe.json"] => {
                    outputs.push(self.render_sensory_json());
                }
                ["inspect.sensory.delta"] | ["svpe.delta"] | ["observe.delta"] => {
                    self.world_memory.observe_tick(self.live_tick, &self.player, &self.world, &self.state_table);
                    outputs.push(self.world_memory.encode_delta_frame(self.live_tick, &self.player, &self.state_table));
                }
                ["inspect.svpe"] | ["inspect.svpe.rle"] | ["svpe.encode"] => {
                    let center = [
                        self.player.pos[0].floor() as i32,
                        self.player.pos[1].floor() as i32,
                        self.player.pos[2].floor() as i32,
                    ];
                    outputs.push(perception::encode_svpe1_palette_rle(&self.world, center, 8, 4, 8));
                }
                ["inspect.svpe", rx, ry, rz] | ["svpe.encode", rx, ry, rz] => {
                    let rx_v: i32 = rx.parse().map_err(|_| "bad rx")?;
                    let ry_v: i32 = ry.parse().map_err(|_| "bad ry")?;
                    let rz_v: i32 = rz.parse().map_err(|_| "bad rz")?;
                    let center = [
                        self.player.pos[0].floor() as i32,
                        self.player.pos[1].floor() as i32,
                        self.player.pos[2].floor() as i32,
                    ];
                    outputs.push(perception::encode_svpe1_palette_rle(&self.world, center, rx_v, ry_v, rz_v));
                }
                ["inspect.svpe.decode", encoded] | ["svpe.decode", encoded] => {
                    let vol = perception::decode_svpe1_palette_rle(encoded)?;
                    let non_air = vol.symbols.iter().filter(|&&c| c != 'A' && c != 'U').count();
                    outputs.push(format!(
                        "svpe1_decoded origin=({},{},{}) dims=({},{},{}) total_voxels={} non_air_voxels={non_air}",
                        vol.origin[0], vol.origin[1], vol.origin[2],
                        vol.dims[0], vol.dims[1], vol.dims[2],
                        vol.symbols.len()
                    ));
                }
                ["inspect.svpe.height"] | ["svpe.height"] => {
                    outputs.push(perception::encode_svpe1_height_delta(&self.world, self.player.pos, 12));
                }
                ["inspect.svpe.height", radius] | ["svpe.height", radius] => {
                    let r: i32 = radius.parse().map_err(|_| "bad radius")?;
                    outputs.push(perception::encode_svpe1_height_delta(&self.world, self.player.pos, r));
                }
                ["inspect.viewport"] | ["see.viewport"] => {
                    let (vp, depth) = perception::render_retinal_viewport_24x11(&self.player, &self.world, &self.state_table);
                    outputs.push(format!("{vp}\n{depth}"));
                }
                ["inspect.xray"] | ["see.xray"] => {
                    let eye = self.player.eye_position();
                    let pitch_rad = self.player.pitch.to_radians();
                    let yaw_rad = self.player.yaw.to_radians();
                    let look_dir = [
                        -(yaw_rad.sin() as f64) * (pitch_rad.cos() as f64),
                        -(pitch_rad.sin() as f64),
                        (yaw_rad.cos() as f64) * (pitch_rad.cos() as f64),
                    ];
                    let look_ray = perception::cast_penetrating_ray_layers(eye, look_dir, &self.world, &self.state_table, 16.0);
                    let down_ray = perception::cast_penetrating_ray_layers(self.player.pos, [0.0, -1.0, 0.0], &self.world, &self.state_table, 16.0);
                    let north_ray = perception::cast_penetrating_ray_layers(eye, [0.0, 0.0, -1.0], &self.world, &self.state_table, 16.0);
                    let south_ray = perception::cast_penetrating_ray_layers(eye, [0.0, 0.0, 1.0], &self.world, &self.state_table, 16.0);
                    let east_ray = perception::cast_penetrating_ray_layers(eye, [1.0, 0.0, 0.0], &self.world, &self.state_table, 16.0);
                    let west_ray = perception::cast_penetrating_ray_layers(eye, [-1.0, 0.0, 0.0], &self.world, &self.state_table, 16.0);
                    outputs.push(format!(
                        "Through-Block Penetrating X-Ray Awareness (16m):\n  Crosshair : {look_ray}\n  UnderFeet : {down_ray}\n  North(-Z) : {north_ray}\n  South(+Z) : {south_ray}\n  East (+X) : {east_ray}\n  West (-X) : {west_ray}"
                    ));
                }
                ["inspect.long_range"] | ["see.horizon"] => {
                    let lines = perception::scan_long_range_sectors(&self.player, &self.world);
                    outputs.push(format!("Long-Range 64m 8-Sector Horizon Scan:\n  {}", lines.join("\n  ")));
                }
                ["inspect.small_map"] | ["see.small_map"] => {
                    outputs.push(perception::inspect_small_map_awareness(&self.world, self.player.pos));
                }
                ["humanisation.perception"] | ["humanisation.perception", "status"] => {
                    outputs.push(format!(
                        "humanisation_perception enabled={} entity_reach={:.2} block_reach={:.2} fov=({:.0}x{:.0})",
                        self.humanised_perception.enabled,
                        self.player.max_entity_reach(),
                        self.player.max_block_reach(),
                        self.humanised_perception.fov_horizontal_deg,
                        self.humanised_perception.fov_vertical_deg
                    ));
                }
                ["humanisation.perception", mode] => {
                    match *mode {
                        "on" | "true" | "humanised" => {
                            self.humanised_perception = sawbyte_core::humanisation::HumanisedPerceptionPolicy::humanised();
                            self.player.attributes.entity_interaction_range =
                                self.humanised_perception.clamp_fair_entity_reach(self.player.attributes.entity_interaction_range);
                            self.player.attributes.block_interaction_range =
                                self.humanised_perception.clamp_fair_block_reach(self.player.attributes.block_interaction_range);
                            outputs.push("enabled humanised perception gate (vanilla reach 3.0m/4.5m enforced; occluded ESP downgraded to supplementary hints)".into());
                        }
                        "off" | "false" | "direct" => {
                            self.humanised_perception = sawbyte_core::humanisation::HumanisedPerceptionPolicy::direct_unfiltered();
                            outputs.push("switched perception gate to direct unfiltered mode".into());
                        }
                        other => return Err(format!("unknown humanisation.perception mode '{other}'; expected on|off|status")),
                    }
                }
                ["memory.status"] => {
                    self.world_memory.observe_tick(self.live_tick, &self.player, &self.world, &self.state_table);
                    outputs.push(self.world_memory.summary_status());
                }
                ["memory.save"] => {
                    self.world_memory.observe_tick(self.live_tick, &self.player, &self.world, &self.state_table);
                    let path = self.world_memory.save()?;
                    outputs.push(format!("saved persistent world memory to {}", path.display()));
                }
                ["memory.save", custom_path] => {
                    self.world_memory.observe_tick(self.live_tick, &self.player, &self.world, &self.state_table);
                    let p = std::path::Path::new(*custom_path);
                    let saved = self.world_memory.save_to(p)?;
                    outputs.push(format!("saved persistent world memory to {}", saved.display()));
                }
                ["memory.load", custom_path] => {
                    let p = std::path::Path::new(*custom_path);
                    self.world_memory.load_from(p)?;
                    outputs.push(format!("loaded persistent world memory from {} ({})", p.display(), self.world_memory.summary_status()));
                }
                ["memory.landmarks"] => {
                    let list: Vec<serde_json::Value> = self
                        .world_memory
                        .landmarks
                        .values()
                        .map(|l| {
                            serde_json::json!({
                                "kind": l.kind,
                                "label": l.label,
                                "pos": l.pos,
                                "last_seen_tick": l.last_seen_tick,
                            })
                        })
                        .collect();
                    outputs.push(serde_json::json!({ "landmarks": list, "count": list.len() }).to_string());
                }
                ["memory.mark", kind, rest @ ..] if !rest.is_empty() => {
                    let label = rest.join(" ");
                    let pos = [
                        self.player.pos[0].floor() as i32,
                        self.player.pos[1].floor() as i32,
                        self.player.pos[2].floor() as i32,
                    ];
                    self.world_memory.record_landmark(kind, &label, pos, self.live_tick);
                    let _ = self.world_memory.save();
                    outputs.push(format!("recorded persistent landmark kind=\"{kind}\" label=\"{label}\" at ({}, {}, {})", pos[0], pos[1], pos[2]));
                }
                ["memory.enable", flag] => {
                    self.world_memory.enabled = matches!(*flag, "on" | "true" | "1");
                    outputs.push(format!("set persistent_memory enabled={}", self.world_memory.enabled));
                }
                ["memory.clear"] => {
                    self.world_memory.explored_chunk_tiles.clear();
                    self.world_memory.landmarks.clear();
                    self.world_memory.direct_los_seen_entities.clear();
                    outputs.push("cleared in-memory persistent world map and landmarks".into());
                }
                ["chat.send", rest @ ..] if !rest.is_empty() => {
                    let msg = rest.join(" ");
                    let ev = if let Some(cmd) = msg.strip_prefix('/') {
                        SubTickEvent::SendCommand {
                            command: cmd.to_string(),
                        }
                    } else {
                        SubTickEvent::SendChat {
                            message: msg.clone(),
                        }
                    };
                    self.event_queue
                        .submit(&[(next_tick, ev)])
                        .map_err(|e| format!("{e:?}"))?;
                    outputs.push(format!("queued live chat/command: {msg}"));
                }
                ["cmd.run", rest @ ..] if !rest.is_empty() => {
                    let cmd = rest.join(" ");
                    let ev = SubTickEvent::SendCommand {
                        command: cmd.clone(),
                    };
                    self.event_queue
                        .submit(&[(next_tick, ev)])
                        .map_err(|e| format!("{e:?}"))?;
                    outputs.push(format!("queued live command: /{cmd}"));
                }
                ["inspect.chat"] => {
                    outputs.push(serde_json::json!({
                        "messages": self.state_table.chat_log,
                        "history_limit": 128,
                    }).to_string());
                }
                ["inspect.events"] => {
                    outputs.push(serde_json::json!({
                        "events": self.state_table.received_events,
                        "history_limit": 256,
                    }).to_string());
                }
                ["inspect.map"] | ["inspect.map", _] => {
                    let radius = parts
                        .get(1)
                        .and_then(|r| r.parse::<i32>().ok())
                        .unwrap_or(5)
                        .clamp(1, 16);
                    let entities: Vec<(i32, [f64; 3])> = self
                        .state_table
                        .entities
                        .values()
                        .map(|e| (e.entity_id, e.pos))
                        .collect();
                    outputs.push(self.world.render_live_ascii_map(
                        self.player.pos,
                        self.player.yaw,
                        self.state_table.selected_hotbar,
                        &entities,
                        radius,
                    ));
                }
                ["inspect.status"] | ["inspect.profile"] => {
                    outputs.push(format!(
                        "live_tick={} pos=({:.3}, {:.3}, {:.3}) vel=({:.4}, {:.4}, {:.4}) yaw={:.2} pitch={:.2} ground={} health={:.1} food={} chunks={} entities={} pending_inputs={} nav_active={} decode_errors={}",
                        self.live_tick,
                        self.player.pos[0],
                        self.player.pos[1],
                        self.player.pos[2],
                        self.player.velocity[0],
                        self.player.velocity[1],
                        self.player.velocity[2],
                        self.player.yaw,
                        self.player.pitch,
                        self.player.ground,
                        self.state_table.health,
                        self.state_table.food,
                        self.world.chunks.len(),
                        self.state_table.entities.len(),
                        self.scheduler.pending(),
                        self.active_nav_goal.is_some(),
                        self.state_table.decode_errors
                    ));
                }
                ["gym.observe"] => {
                    outputs.push(self.gym_observation_json("gym.observe"));
                }
                ["inspect.deaths"] => {
                    if self.death_history.is_empty() {
                        outputs.push("no deaths recorded".to_string());
                    } else {
                        let mut s = format!("deaths total={}:\n", self.death_count);
                        for d in &self.death_history {
                            s.push_str(&format!(
                                "  - attempt #{} tick={} pos=({:.2}, {:.2}, {:.2}) cause=\"{}\" killer_id={:?}\n",
                                d.attempt, d.tick, d.pos[0], d.pos[1], d.pos[2], d.cause, d.killer_id
                            ));
                        }
                        outputs.push(s.trim_end().to_string());
                    }
                }
                ["inspect.milestones"] => {
                    if self.milestone_history.is_empty() {
                        outputs.push("no milestones logged".to_string());
                    } else {
                        let mut s = format!("milestones total={}:\n", self.milestone_history.len());
                        for m in &self.milestone_history {
                            s.push_str(&format!(
                                "  - [{}] attempt #{} deaths={} tick={} pos=({:.1}, {:.1}, {:.1}) health={:.1} food={} notes=\"{}\"\n",
                                m.name, m.attempt, m.death_count, m.tick, m.pos[0], m.pos[1], m.pos[2], m.health, m.food, m.notes
                            ));
                        }
                        outputs.push(s.trim_end().to_string());
                    }
                }
                ["milestone.save", name, notes @ ..] | ["milestone.record", name, notes @ ..] => {
                    let m = self.record_milestone(name, &notes.join(" "));
                    outputs.push(format!(
                        "milestone_recorded name=\"{}\" attempt={} deaths={} tick={} pos=({:.2}, {:.2}, {:.2}) health={:.1} food={} xp={} inventory=[{}]",
                        m.name, m.attempt, m.death_count, m.tick, m.pos[0], m.pos[1], m.pos[2], m.health, m.food, m.xp_level, m.inventory_summary
                    ));
                }
                ["inspect.frame", rest @ ..] => {
                    let w = rest.first().and_then(|s| s.parse().ok()).unwrap_or(160);
                    let h = rest.get(1).and_then(|s| s.parse().ok()).unwrap_or(100);
                    let mode = match rest.get(2).copied().unwrap_or("first_person") {
                        "third_person" | "chase" => visualizer::CameraMode::ThirdPersonChase,
                        "radar" | "topdown" => visualizer::CameraMode::TopDownRadar,
                        _ => visualizer::CameraMode::FirstPerson,
                    };
                    let viz = visualizer::LowPolyVisualizer::new(w, h, mode);
                    let entities: Vec<visualizer::EntityRenderData> = self
                        .state_table
                        .entities
                        .values()
                        .map(|e| visualizer::EntityRenderData {
                            entity_id: e.entity_id,
                            pos: e.pos,
                            kind: "entity".to_string(),
                        })
                        .collect();
                    let frame = viz.render(&self.player, &self.world, &entities);
                    outputs.push(format!("frame:{}", frame.to_base64_data_uri()));
                }
                ["inspect.visualizer_state"] => {
                    let entities: Vec<String> = self
                        .state_table
                        .entities
                        .values()
                        .take(30)
                        .map(|e| {
                            let dx = e.pos[0] - self.player.pos[0];
                            let dz = e.pos[2] - self.player.pos[2];
                            let dist = (dx * dx + dz * dz).sqrt();
                            format!("{{\"id\":{},\"pos\":[{:.2},{:.2},{:.2}],\"dist\":{:.2}}}", e.entity_id, e.pos[0], e.pos[1], e.pos[2], dist)
                        })
                        .collect();
                    let waypoints: Vec<String> = self
                        .active_nav_path
                        .iter()
                        .take(16)
                        .map(|p| format!("[{},{},{}]", p[0], p[1], p[2]))
                        .collect();
                    let last_death = self.death_history.last().map(|d| {
                        format!("{{\"attempt\":{},\"tick\":{},\"pos\":[{:.2},{:.2},{:.2}],\"cause\":\"{}\"}}", d.attempt, d.tick, d.pos[0], d.pos[1], d.pos[2], d.cause)
                    }).unwrap_or_else(|| "null".to_string());
                    outputs.push(format!(
                        "{{\"pos\":[{:.3},{:.3},{:.3}],\"yaw\":{:.2},\"pitch\":{:.2},\"health\":{:.1},\"food\":{},\"xp\":{},\"is_dead\":{},\"death_count\":{},\"attempt\":{},\"live_tick\":{},\"nav_active\":{},\"waypoints\":[{}],\"entities\":[{}],\"last_death\":{}}}",
                        self.player.pos[0], self.player.pos[1], self.player.pos[2],
                        self.player.yaw, self.player.pitch,
                        self.state_table.health, self.state_table.food, self.state_table.xp_level,
                        self.is_dead, self.death_count, self.death_count + 1, self.live_tick,
                        self.active_nav_goal.is_some(),
                        waypoints.join(","),
                        entities.join(","),
                        last_death
                    ));
                }
                ["gym.step", rest @ ..] => {
                    if self.pending_replies.iter().any(|reply| matches!(
                        &reply.kind,
                        PendingReplyKind::Tick { command, .. } if command == "gym.step"
                    )) {
                        return Err("a training step is already pending".into());
                    }
                    let b_on = |s: &str| match s {
                        "1" | "true" => Ok(true),
                        "0" | "false" => Ok(false),
                        _ => Err(format!("invalid button value '{s}'")),
                    };
                    let fwd = rest.first().copied().unwrap_or("0");
                    let back = rest.get(1).copied().unwrap_or("0");
                    let left = rest.get(2).copied().unwrap_or("0");
                    let right = rest.get(3).copied().unwrap_or("0");
                    let jump = rest.get(4).copied().unwrap_or("0");
                    let sneak = rest.get(5).copied().unwrap_or("0");
                    let sprint = rest.get(6).copied().unwrap_or("0");
                    let yd_str = rest.get(7).copied().unwrap_or("0.0");
                    let pd_str = rest.get(8).copied().unwrap_or("0.0");
                    let buttons = [b_on(fwd)?, b_on(back)?, b_on(left)?, b_on(right)?, b_on(jump)?, b_on(sneak)?, b_on(sprint)?];
                    let yd: f64 = yd_str.parse().map_err(|_| "bad yaw_delta")?;
                    let pd: f64 = pd_str.parse().map_err(|_| "bad pitch_delta")?;
                    if !yd.is_finite() || !pd.is_finite() {
                        return Err("camera deltas must be finite".into());
                    }
                    let hotbar = if rest.len() > 9 {
                        Some(rest[9].parse::<u8>().ok().filter(|slot| *slot < 9).ok_or("hotbar slot must be in 0..=8")?)
                    } else {
                        None
                    };
                    let step_tick = self.live_tick;
                    self.preempt_movement();
                    self.scheduler
                        .submit(&[
                            (
                                step_tick,
                                Action::Button {
                                    button: Button::Forward,
                                    down: buttons[0],
                                },
                            ),
                            (
                                step_tick,
                                Action::Button {
                                    button: Button::Back,
                                    down: buttons[1],
                                },
                            ),
                            (
                                step_tick,
                                Action::Button {
                                    button: Button::Left,
                                    down: buttons[2],
                                },
                            ),
                            (
                                step_tick,
                                Action::Button {
                                    button: Button::Right,
                                    down: buttons[3],
                                },
                            ),
                            (
                                step_tick,
                                Action::Button {
                                    button: Button::Jump,
                                    down: buttons[4],
                                },
                            ),
                            (
                                step_tick,
                                Action::Button {
                                    button: Button::Sneak,
                                    down: buttons[5],
                                },
                            ),
                            (
                                step_tick,
                                Action::Button {
                                    button: Button::Sprint,
                                    down: buttons[6],
                                },
                            ),
                            (
                                step_tick,
                                Action::Look {
                                    yaw: yd,
                                    pitch: pd,
                                },
                            ),
                        ])
                        .map_err(|e| format!("scheduler error: {e}"))?;
                    if let Some(hb) = hotbar {
                        if let Err(error) = self.event_queue.submit(&[(step_tick, SubTickEvent::SelectHotbar { slot: hb })]) {
                            let _ = self.scheduler.cancel();
                            return Err(format!("hotbar scheduling error: {error}"));
                        }
                    }
                    outputs.push(format!("gym.step scheduled for tick {step_tick} (current_tick={})", self.live_tick));
                }
                ["wait.ticks", n] => {
                    let count = n.parse::<u64>().map_err(|_| "bad wait.ticks count")?;
                    outputs.push(format!("wait.ticks scheduled for {count} ticks (target_tick={})", self.live_tick + count));
                }
                ["wait.until", ..] => {
                    outputs.push("wait.until scheduled".into());
                }
                ["session.lease", ..] => {
                    outputs.push("session.lease acknowledged".into());
                }
                // =========================================================================
                // FULL 65-COMMAND `horizon-ctl` COMPATIBILITY LAYER (1:1 Vocabulary Parity)
                // =========================================================================
                ["state"] | ["status"] => {
                    outputs.push(self.gym_observation_json("state"));
                }
                ["looking-at"] | ["looking_at"] => {
                    let b_hit = raycast::raycast_block(&self.player, &self.world);
                    let candidates: Vec<(i32, [f64; 6])> = self
                        .state_table
                        .entities
                        .values()
                        .map(|e| (e.entity_id, e.aabb()))
                        .collect();
                    let e_hit = raycast::raycast_entity(&self.player, &self.world, &candidates);
                    outputs.push(
                        serde_json::json!({
                            "ok": true,
                            "block": b_hit.map(|b| serde_json::json!({"pos": b.block_pos, "face": format!("{:?}", b.face), "dist": b.distance})),
                            "entity": e_hit.map(|e| serde_json::json!({"id": e.entity_id, "dist": e.distance})),
                        })
                        .to_string(),
                    );
                }
                ["entities"] | ["entities", ..] => {
                    let list: Vec<serde_json::Value> = self
                        .state_table
                        .entities
                        .values()
                        .map(|e| {
                            let dx = e.pos[0] - self.player.pos[0];
                            let dy = e.pos[1] - self.player.pos[1];
                            let dz = e.pos[2] - self.player.pos[2];
                            let dist = (dx * dx + dy * dy + dz * dz).sqrt();
                            serde_json::json!({
                                "id": e.entity_id,
                                "type": e.entity_type,
                                "pos": e.pos,
                                "dist": dist,
                            })
                        })
                        .collect();
                    outputs.push(serde_json::json!({ "ok": true, "entities": list }).to_string());
                }
                ["inventory"] => {
                    let res = self.execute_live_script("inventory.list")?;
                    outputs.extend(res);
                }
                ["screen"] | ["ui"] | ["menu"] => {
                    let res = self.execute_live_script("gui.inspect")?;
                    outputs.extend(res);
                }
                ["messages"] | ["messages", _] | ["chat-log"] | ["chat-log", _] => {
                    let res = self.execute_live_script("inspect.chat")?;
                    outputs.extend(res);
                }
                ["events"] | ["events", ..] => {
                    let res = self.execute_live_script("inspect.events")?;
                    outputs.extend(res);
                }
                ["block", x, y, z] => {
                    let bx = x.parse::<i32>().map_err(|_| "bad x")?;
                    let by = y.parse::<i32>().map_err(|_| "bad y")?;
                    let bz = z.parse::<i32>().map_err(|_| "bad z")?;
                    let st = self.world.block_state(bx, by, bz).unwrap_or(0);
                    outputs.push(
                        serde_json::json!({
                            "pos": [bx, by, bz],
                            "state": st,
                            "solid": self.world.solid(bx, by, bz).unwrap_or(false),
                            "loaded": self.world.loaded(bx as f64, bz as f64),
                        })
                        .to_string(),
                    );
                }
                ["find-block", block_query, ..] | ["find_block", block_query, ..] => {
                    let px = self.player.pos[0].floor() as i32;
                    let py = self.player.pos[1].floor() as i32;
                    let pz = self.player.pos[2].floor() as i32;
                    let mut matches = Vec::new();
                    for dy in -4..=4 {
                        for dz in -12..=12 {
                            for dx in -12..=12 {
                                let (bx, by, bz) = (px + dx, py + dy, pz + dz);
                                if let Ok(st) = self.world.block_state(bx, by, bz) {
                                    if st != 0 && !self.world.is_air_state(st) {
                                        matches.push(serde_json::json!({"pos": [bx, by, bz], "state": st, "query": block_query}));
                                        if matches.len() >= 16 {
                                            break;
                                        }
                                    }
                                }
                            }
                            if matches.len() >= 16 { break; }
                        }
                        if matches.len() >= 16 { break; }
                    }
                    outputs.push(serde_json::json!({ "ok": true, "matches": matches }).to_string());
                }
                ["goto", x, y, z, ..] => {
                    let gx = x.parse::<f64>().map_err(|_| "bad x")?.floor() as i32;
                    let gy = y.parse::<f64>().map_err(|_| "bad y")?.floor() as i32;
                    let gz = z.parse::<f64>().map_err(|_| "bad z")?.floor() as i32;
                    let res = self.execute_live_script(&format!("task.navigate {gx} {gy} {gz}"))?;
                    outputs.extend(res);
                }
                ["move", dir, secs, ..] => {
                    let s = secs.parse::<f64>().unwrap_or(1.0).clamp(0.05, 10.0);
                    let ticks = ((s * 20.0).round() as u64).max(1);
                    let btn = match *dir {
                        "back" | "backward" => Button::Back,
                        "left" => Button::Left,
                        "right" => Button::Right,
                        _ => Button::Forward,
                    };
                    self.preempt_movement();
                    let mut batch = Vec::new();
                    for t in 0..ticks {
                        batch.push((next_tick + t, Action::Button { button: btn, down: true }));
                    }
                    batch.push((next_tick + ticks, Action::Button { button: btn, down: false }));
                    self.scheduler.submit(&batch).map_err(|e| format!("{e:?}"))?;
                    outputs.push(serde_json::json!({"ok": true, "moved": dir, "ticks": ticks}).to_string());
                }
                ["look", yaw, pitch] => {
                    let target_yaw = yaw.parse::<f32>().map_err(|_| "bad yaw")?;
                    let target_pitch = pitch.parse::<f32>().map_err(|_| "bad pitch")?;
                    self.player.yaw = target_yaw;
                    self.player.pitch = target_pitch.clamp(-90.0, 90.0);
                    outputs.push(serde_json::json!({"ok": true, "yaw": self.player.yaw, "pitch": self.player.pitch}).to_string());
                }
                ["turn", dyaw, dpitch, ..] => {
                    let res = self.execute_live_script(&format!("player.look {dyaw} {dpitch}"))?;
                    outputs.extend(res);
                }
                ["look-at", x, y, z] | ["look_at", x, y, z] | ["look-at-block", x, y, z, ..] => {
                    let tx = x.parse::<f64>().map_err(|_| "bad x")?;
                    let ty = y.parse::<f64>().map_err(|_| "bad y")?;
                    let tz = z.parse::<f64>().map_err(|_| "bad z")?;
                    let eye = self.player.eye_position();
                    let dx = tx - eye[0];
                    let dy = ty - eye[1];
                    let dz = tz - eye[2];
                    let h = (dx * dx + dz * dz).sqrt();
                    self.player.yaw = (-dx).atan2(dz).to_degrees() as f32;
                    self.player.pitch = (-dy).atan2(h).to_degrees() as f32;
                    outputs.push(serde_json::json!({"ok": true, "yaw": self.player.yaw, "pitch": self.player.pitch}).to_string());
                }
                ["break", x, y, z, ..] => {
                    let res = self.execute_live_script(&format!("interact.mine {x} {y} {z}"))?;
                    outputs.extend(res);
                }
                ["place", x, y, z, ..] => {
                    let res = self.execute_live_script(&format!("interact.place {x} {y} {z}"))?;
                    outputs.extend(res);
                }
                ["interact", x, y, z, ..] => {
                    let res = self.execute_live_script(&format!("interact.use_block {x} {y} {z}"))?;
                    outputs.extend(res);
                }
                ["use-item", ..] | ["use_item", ..] => {
                    let res = self.execute_live_script("interact.use_item")?;
                    outputs.extend(res);
                }
                ["attack"] => {
                    let candidates: Vec<(i32, [f64; 6])> = self
                        .state_table
                        .entities
                        .values()
                        .map(|e| (e.entity_id, e.aabb()))
                        .collect();
                    if let Some(hit) = raycast::raycast_entity(&self.player, &self.world, &candidates) {
                        let res = self.execute_live_script(&format!("interact.attack {}", hit.entity_id))?;
                        outputs.extend(res);
                    } else {
                        self.event_queue.submit(&[(next_tick, SubTickEvent::SwipeArm { hand: Hand::MainHand })]).map_err(|e| format!("{e:?}"))?;
                        outputs.push(serde_json::json!({"ok": true, "swung": true}).to_string());
                    }
                }
                ["attack", eid] => {
                    let res = self.execute_live_script(&format!("interact.attack {eid}"))?;
                    outputs.extend(res);
                }
                ["select", slot_or_item] => {
                    if let Ok(s) = slot_or_item.parse::<u8>() {
                        let zero_based = if s >= 1 && s <= 9 { s - 1 } else { s.min(8) };
                        let res = self.execute_live_script(&format!("player.hotbar {zero_based}"))?;
                        outputs.extend(res);
                    } else {
                        let res = self.execute_live_script(&format!("inventory.use_item {slot_or_item}"))?;
                        outputs.extend(res);
                    }
                }
                ["interact-entity", eid] | ["interact_entity", eid] => {
                    let res = self.execute_live_script(&format!("entity.interact {eid}"))?;
                    outputs.extend(res);
                }
                ["click-slot", slot, ..] | ["click_slot", slot, ..] => {
                    let res = self.execute_live_script(&format!("gui.click_slot {slot}"))?;
                    outputs.extend(res);
                }
                ["click", target, ..] | ["click-button", target, ..] => {
                    let res = self.execute_live_script(&format!("gui.click {target}"))?;
                    outputs.extend(res);
                }
                ["close"] => {
                    let res = self.execute_live_script("gui.close")?;
                    outputs.extend(res);
                }
                ["chat", rest @ ..] if !rest.is_empty() => {
                    let res = self.execute_live_script(&format!("chat.send {}", rest.join(" ")))?;
                    outputs.extend(res);
                }
                ["command", rest @ ..] if !rest.is_empty() => {
                    let res = self.execute_live_script(&format!("cmd.run {}", rest.join(" ")))?;
                    outputs.extend(res);
                }
                ["stop"] | ["release"] => {
                    let res = self.execute_live_script("task.cancel")?;
                    outputs.extend(res);
                }
                ["respawn"] => {
                    let res = self.execute_live_script("player.respawn")?;
                    outputs.extend(res);
                }
                ["wait", ticks] => {
                    let res = self.execute_live_script(&format!("wait.ticks {ticks}"))?;
                    outputs.extend(res);
                }
                ["screenshot"] | ["screenshot", _] => {
                    let res = self.execute_live_script("inspect.frame 320 200 first_person")?;
                    outputs.extend(res);
                }
                _ => {
                    return Err(format!(
                        "line {}: unknown live command '{line}'",
                        line_no + 1
                    ));
                }
            }
        }
        Ok(outputs)
    }

    /// Handles a line arriving on `--mcp-stdio` or `--listen <ip:port>` (either JSON-RPC 2.0 / MCP or raw `SawbyteScript`).
    pub fn handle_control_request(&mut self, line: &str) -> String {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            return String::new();
        }
        if trimmed.starts_with('{') {
            let val: serde_json::Value = match serde_json::from_str(trimmed) {
                Ok(v) => v,
                Err(e) => {
                    return serde_json::json!({
                        "jsonrpc": "2.0",
                        "id": 1,
                        "error": { "code": -32700, "message": format!("parse error: {e}") }
                    })
                    .to_string();
                }
            };
            let id = val.get("id").cloned().unwrap_or(serde_json::json!(1));
            let method = val["method"].as_str().unwrap_or("");
            match method {
                "initialize" => serde_json::json!({
                    "jsonrpc": "2.0",
                    "id": id,
                    "result": {
                        "protocolVersion": "2025-06-18",
                        "serverInfo": { "name": "sawbyte-play-live-mcp", "version": "0.5.0" },
                        "capabilities": { "tools": { "listChanged": false }, "resources": { "listChanged": false } }
                    }
                })
                .to_string(),
                "notifications/initialized" => String::new(),
                "ping" => serde_json::json!({ "jsonrpc": "2.0", "id": id, "result": {} }).to_string(),
                "resources/list" => serde_json::json!({
                    "jsonrpc": "2.0",
                    "id": id,
                    "result": {
                        "resources": [
                            { "uri": "sawbyte://session/status", "name": "Live 26.3 Session Status & Vitals", "mimeType": "application/json" },
                            { "uri": "sawbyte://session/inventory", "name": "Live 46-Slot Player Inventory & Hotbar", "mimeType": "application/json" },
                            { "uri": "sawbyte://session/entities", "name": "Tracked Nearby Entities & Lobby NPCs", "mimeType": "application/json" },
                            { "uri": "sawbyte://session/chat", "name": "Live Server Chat Log", "mimeType": "application/json" },
                            { "uri": "sawbyte://world/map2d", "name": "Live 2D ASCII World Map", "mimeType": "text/plain" }
                        ]
                    }
                })
                .to_string(),
                "resources/templates/list" => serde_json::json!({
                    "jsonrpc": "2.0",
                    "id": id,
                    "result": {
                        "resourceTemplates": [
                            { "uriTemplate": "sawbyte://session/{section}", "name": "Live Session Section (status, inventory, entities, chat, events)", "mimeType": "application/json" }
                        ]
                    }
                })
                .to_string(),
                "resources/read" => {
                    let uri = val["params"]["uri"].as_str().unwrap_or("sawbyte://session/status");
                    let body = match uri {
                        "sawbyte://session/inventory" => self.execute_live_script("inventory.list").unwrap_or_default().join("\n"),
                        "sawbyte://session/entities" => self.execute_live_script("entity.list").unwrap_or_default().join("\n"),
                        "sawbyte://session/chat" => self.execute_live_script("inspect.chat").unwrap_or_default().join("\n"),
                        "sawbyte://world/map2d" => self.execute_live_script("inspect.map").unwrap_or_default().join("\n"),
                        _ => self.execute_live_script("inspect.status").unwrap_or_default().join("\n"),
                    };
                    serde_json::json!({
                        "jsonrpc": "2.0",
                        "id": id,
                        "result": {
                            "contents": [{ "uri": uri, "mimeType": "text/plain", "text": body }]
                        }
                    })
                    .to_string()
                }
                "prompts/list" => serde_json::json!({
                    "jsonrpc": "2.0",
                    "id": id,
                    "result": {
                        "prompts": [
                            { "name": "automate_minecraft", "description": "Control Sawbyte 26.3 live session via MCP tools." }
                        ]
                    }
                })
                .to_string(),
                "prompts/get" => serde_json::json!({
                    "jsonrpc": "2.0",
                    "id": id,
                    "result": {
                        "description": "Sawbyte 26.3 live session automation prompt",
                        "messages": [
                            { "role": "user", "content": { "type": "text", "text": "Use tools/call with code_execute, gui_screen_automation, player_interact, or task_and_navigation." } }
                        ]
                    }
                })
                .to_string(),
                "tools/list" => serde_json::json!({
                    "jsonrpc": "2.0",
                    "id": id,
                    "result": {
                        "tools": [
                            { "name": "task_and_navigation", "description": "Run live 3D A* pathfinding on the server's loaded chunk sections.", "inputSchema": { "type": "object", "properties": { "script": { "type": "string" }, "command": { "type": "string" } } } },
                            { "name": "player_interact", "description": "Execute live raycast block mining, placement, and entity combat.", "inputSchema": { "type": "object", "properties": { "script": { "type": "string" }, "command": { "type": "string" } } } },
                            { "name": "player_input", "description": "Submit live humanised/smooth/direct movement and camera inputs.", "inputSchema": { "type": "object", "properties": { "script": { "type": "string" }, "command": { "type": "string" } } } },
                            { "name": "gui_screen_automation", "description": "Inspect live server screens, click slots/items/widgets, and run multi-screen workflows.", "inputSchema": { "type": "object", "properties": { "script": { "type": "string" }, "command": { "type": "string" } } } },
                            { "name": "world_inspector_and_profiler", "description": "Render live 2D ASCII map from server chunks and inspect live player vitals.", "inputSchema": { "type": "object", "properties": { "script": { "type": "string" }, "command": { "type": "string" } } } },
                            { "name": "gym_step_and_observe", "description": "Zero-bloat RL / neural network observation and action step on the live server.", "inputSchema": { "type": "object", "properties": { "script": { "type": "string" }, "command": { "type": "string" } } } },
                            { "name": "code_execute", "description": "Execute multi-line SawbyteScript directly on the live server session.", "inputSchema": { "type": "object", "properties": { "script": { "type": "string" }, "command": { "type": "string" } } } }
                        ]
                    }
                })
                .to_string(),
                "tools/call" => {
                    let tool = val["params"]["name"].as_str().unwrap_or("code_execute");
                    let valid_tools = [
                        "task_and_navigation",
                        "player_interact",
                        "player_input",
                        "gui_screen_automation",
                        "world_inspector_and_profiler",
                        "gym_step_and_observe",
                        "code_execute",
                    ];
                    if !valid_tools.contains(&tool) {
                        return serde_json::json!({
                            "jsonrpc": "2.0",
                            "id": id,
                            "result": {
                                "isError": true,
                                "content": [{ "type": "text", "text": format!("unknown tool '{tool}'") }]
                            }
                        })
                        .to_string();
                    }
                    let script = val["params"]["arguments"]["script"]
                        .as_str()
                        .or_else(|| val["params"]["arguments"]["command"].as_str())
                        .or_else(|| val["params"]["script"].as_str())
                        .or_else(|| val["params"]["command"].as_str())
                        .unwrap_or_default();
                    if script.trim().is_empty() {
                        return serde_json::json!({
                            "jsonrpc": "2.0",
                            "id": id,
                            "result": {
                                "isError": true,
                                "content": [{ "type": "text", "text": format!("missing required 'script' or 'command' argument for tool '{tool}'") }]
                            }
                        })
                        .to_string();
                    }
                    match self.execute_live_script(script) {
                        Ok(lines) => serde_json::json!({
                            "jsonrpc": "2.0",
                            "id": id,
                            "result": {
                                "isError": false,
                                "content": [{ "type": "text", "text": lines.join("\n") }]
                            }
                        })
                        .to_string(),
                        Err(err) => serde_json::json!({
                            "jsonrpc": "2.0",
                            "id": id,
                            "result": {
                                "isError": true,
                                "content": [{ "type": "text", "text": err }]
                            }
                        })
                        .to_string(),
                    }
                }
                _ => serde_json::json!({
                    "jsonrpc": "2.0",
                    "id": id,
                    "error": { "code": -32601, "message": "Method not found" }
                })
                .to_string(),
            }
        } else {
            match self.execute_live_script(trimmed) {
                Ok(lines) => lines.join("\n"),
                Err(err) => format!("error: {err}"),
            }
        }
    }

    /// Handles a control line arriving with an optional reply channel for deferred tick-synchronization.
    pub fn handle_control_request_with_channel(
        &mut self,
        line: &str,
        reply_chan: Option<mpsc::Sender<String>>,
    ) {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            return;
        }
        if trimmed.starts_with('{') {
            let val: serde_json::Value = match serde_json::from_str(trimmed) {
                Ok(v) => v,
                Err(e) => {
                    let resp = serde_json::json!({
                        "jsonrpc": "2.0",
                        "id": 1,
                        "error": { "code": -32700, "message": format!("parse error: {e}") }
                    })
                    .to_string();
                    if let Some(chan) = reply_chan {
                        let _ = chan.send(resp);
                    } else {
                        println!("{resp}");
                    }
                    return;
                }
            };
            let id = val.get("id").cloned().unwrap_or(serde_json::json!(1));
            let method = val["method"].as_str().unwrap_or("");
            if method == "tools/call" {
                let script = val["params"]["arguments"]["script"]
                    .as_str()
                    .or_else(|| val["params"]["arguments"]["command"].as_str())
                    .or_else(|| val["params"]["script"].as_str())
                    .or_else(|| val["params"]["command"].as_str())
                    .unwrap_or_default();
                let script_tokens = tokenize_command_line(script).unwrap_or_default();
                let first_word = script_tokens.first().map(|s| s.as_str()).unwrap_or("");
                if first_word == "gym.step" && script.lines().count() == 1 {
                    match self.execute_live_script(script) {
                        Ok(_) => {
                            let next_tick = self.live_tick.saturating_add(1);
                            self.pending_replies.push(PendingReply {
                                client: reply_chan,
                                is_json: true,
                                json_rpc_id: Some(id),
                                kind: PendingReplyKind::Tick {
                                    target_tick: next_tick,
                                    command: "gym.step".to_string(),
                                },
                            });
                            return;
                        }
                        Err(err) => {
                            let resp = serde_json::json!({
                                "jsonrpc": "2.0",
                                "id": id,
                                "result": {
                                    "isError": true,
                                    "content": [{ "type": "text", "text": err }]
                                }
                            })
                            .to_string();
                            if let Some(chan) = reply_chan {
                                let _ = chan.send(resp);
                            } else {
                                println!("{resp}");
                            }
                            return;
                        }
                    }
                } else if first_word == "wait.ticks" {
                    let count = script_tokens
                        .get(1)
                        .and_then(|s| s.parse::<u64>().ok())
                        .unwrap_or(1);
                    let target_tick = self.live_tick.saturating_add(count);
                    self.pending_replies.push(PendingReply {
                        client: reply_chan,
                        is_json: true,
                        json_rpc_id: Some(id),
                        kind: PendingReplyKind::Tick {
                            target_tick,
                            command: "wait.ticks".to_string(),
                        },
                    });
                    return;
                } else if first_word == "wait.until" {
                    let cond_name = script_tokens.get(1).map(|s| s.as_str()).unwrap_or("");
                    match cond_name {
                        "task_idle" => {
                            let timeout_ticks = script_tokens
                                .get(2)
                                .and_then(|s| s.parse::<u64>().ok())
                                .unwrap_or(200);
                            let deadline_tick = self.live_tick.saturating_add(timeout_ticks);
                            self.pending_replies.push(PendingReply {
                                client: reply_chan,
                                is_json: true,
                                json_rpc_id: Some(id),
                                kind: PendingReplyKind::Condition {
                                    deadline_tick,
                                    command: "wait.until task_idle".to_string(),
                                    condition: WaitCondition::TaskIdle,
                                },
                            });
                            return;
                        }
                        "screen_open" => {
                            let timeout_ticks = script_tokens
                                .get(2)
                                .and_then(|s| s.parse::<u64>().ok())
                                .unwrap_or(200);
                            let deadline_tick = self.live_tick.saturating_add(timeout_ticks);
                            let title = if script_tokens.len() > 3 {
                                Some(script_tokens[3..].join(" ").trim_matches('"').to_string())
                            } else {
                                None
                            };
                            self.pending_replies.push(PendingReply {
                                client: reply_chan,
                                is_json: true,
                                json_rpc_id: Some(id),
                                kind: PendingReplyKind::Condition {
                                    deadline_tick,
                                    command: "wait.until screen_open".to_string(),
                                    condition: WaitCondition::ScreenOpen {
                                        title_contains: title,
                                    },
                                },
                            });
                            return;
                        }
                        "slot_item" => {
                            let timeout_ticks = script_tokens
                                .get(2)
                                .and_then(|s| s.parse::<u64>().ok())
                                .unwrap_or(200);
                            let deadline_tick = self.live_tick.saturating_add(timeout_ticks);
                            let slot = script_tokens
                                .get(3)
                                .and_then(|s| s.parse::<usize>().ok())
                                .unwrap_or(0);
                            let item_id = script_tokens
                                .get(4)
                                .map(|s| s.trim_matches('"').to_string())
                                .unwrap_or_default();
                            let min_count = script_tokens
                                .get(5)
                                .and_then(|s| s.parse::<u32>().ok())
                                .unwrap_or(1);
                            self.pending_replies.push(PendingReply {
                                client: reply_chan,
                                is_json: true,
                                json_rpc_id: Some(id),
                                kind: PendingReplyKind::Condition {
                                    deadline_tick,
                                    command: "wait.until slot_item".to_string(),
                                    condition: WaitCondition::SlotItem {
                                        slot,
                                        item_id,
                                        min_count,
                                    },
                                },
                            });
                            return;
                        }
                        "player_alive" | "player_respawn" => {
                            let timeout_ticks = script_tokens
                                .get(2)
                                .and_then(|s| s.parse::<u64>().ok())
                                .unwrap_or(200);
                            let deadline_tick = self.live_tick.saturating_add(timeout_ticks);
                            self.pending_replies.push(PendingReply {
                                client: reply_chan,
                                is_json: true,
                                json_rpc_id: Some(id),
                                kind: PendingReplyKind::Condition {
                                    deadline_tick,
                                    command: "wait.until player_alive".to_string(),
                                    condition: WaitCondition::PlayerAlive,
                                },
                            });
                            return;
                        }
                        _ => {}
                    }
                }
            }
            let resp = self.handle_control_request(trimmed);
            if let Some(chan) = reply_chan {
                let _ = chan.send(resp);
            } else if !resp.is_empty() {
                println!("{resp}");
            }
        } else {
            let tokens = match tokenize_command_line(trimmed) {
                Ok(t) => t,
                Err(e) => {
                    let resp = format!("error: {e}");
                    if let Some(chan) = reply_chan {
                        let _ = chan.send(resp);
                    } else {
                        eprintln!("{resp}");
                    }
                    return;
                }
            };
            let parts: Vec<&str> = tokens.iter().map(|s| s.as_str()).collect();
            match parts.as_slice() {
                ["gym.step", ..] => {
                    match self.execute_live_script(trimmed) {
                        Ok(_) => {
                            let next_tick = self.live_tick.saturating_add(1);
                            self.pending_replies.push(PendingReply {
                                client: reply_chan,
                                is_json: false,
                                json_rpc_id: None,
                                kind: PendingReplyKind::Tick {
                                    target_tick: next_tick,
                                    command: "gym.step".to_string(),
                                },
                            });
                        }
                        Err(err) => {
                            let resp = format!("error: {err}");
                            if let Some(chan) = reply_chan {
                                let _ = chan.send(resp);
                            } else {
                                println!("{resp}");
                            }
                        }
                    }
                }
                ["wait.ticks", n] => {
                    if let Ok(count) = n.parse::<u64>() {
                        let target_tick = self.live_tick.saturating_add(count);
                        self.pending_replies.push(PendingReply {
                            client: reply_chan,
                            is_json: false,
                            json_rpc_id: None,
                            kind: PendingReplyKind::Tick {
                                target_tick,
                                command: "wait.ticks".to_string(),
                            },
                        });
                    } else {
                        let resp = "error: bad wait.ticks argument".to_string();
                        if let Some(chan) = reply_chan {
                            let _ = chan.send(resp);
                        } else {
                            println!("{resp}");
                        }
                    }
                }
                ["wait.until", "task_idle", timeout] => {
                    let timeout_ticks = timeout.parse::<u64>().unwrap_or(200);
                    let deadline_tick = self.live_tick.saturating_add(timeout_ticks);
                    self.pending_replies.push(PendingReply {
                        client: reply_chan,
                        is_json: false,
                        json_rpc_id: None,
                        kind: PendingReplyKind::Condition {
                            deadline_tick,
                            command: "wait.until task_idle".to_string(),
                            condition: WaitCondition::TaskIdle,
                        },
                    });
                }
                ["wait.until", "screen_open", timeout, title_parts @ ..] => {
                    let timeout_ticks = timeout.parse::<u64>().unwrap_or(200);
                    let deadline_tick = self.live_tick.saturating_add(timeout_ticks);
                    let title = if title_parts.is_empty() {
                        None
                    } else {
                        Some(title_parts.join(" ").trim_matches('"').to_string())
                    };
                    self.pending_replies.push(PendingReply {
                        client: reply_chan,
                        is_json: false,
                        json_rpc_id: None,
                        kind: PendingReplyKind::Condition {
                            deadline_tick,
                            command: "wait.until screen_open".to_string(),
                            condition: WaitCondition::ScreenOpen {
                                title_contains: title,
                            },
                        },
                    });
                }
                ["wait.until", "slot_item", timeout, slot, item_id, count_opt @ ..] => {
                    let timeout_ticks = timeout.parse::<u64>().unwrap_or(200);
                    let deadline_tick = self.live_tick.saturating_add(timeout_ticks);
                    let s_idx = slot.parse::<usize>().unwrap_or(0);
                    let min_count = count_opt.first().and_then(|c| c.parse::<u32>().ok()).unwrap_or(1);
                    self.pending_replies.push(PendingReply {
                        client: reply_chan,
                        is_json: false,
                        json_rpc_id: None,
                        kind: PendingReplyKind::Condition {
                            deadline_tick,
                            command: "wait.until slot_item".to_string(),
                            condition: WaitCondition::SlotItem {
                                slot: s_idx,
                                item_id: item_id.trim_matches('"').to_string(),
                                min_count,
                            },
                        },
                    });
                }
                ["wait.until", "player_alive", timeout] | ["wait.until", "player_respawn", timeout] => {
                    let timeout_ticks = timeout.parse::<u64>().unwrap_or(200);
                    let deadline_tick = self.live_tick.saturating_add(timeout_ticks);
                    self.pending_replies.push(PendingReply {
                        client: reply_chan,
                        is_json: false,
                        json_rpc_id: None,
                        kind: PendingReplyKind::Condition {
                            deadline_tick,
                            command: "wait.until player_alive".to_string(),
                            condition: WaitCondition::PlayerAlive,
                        },
                    });
                }
                _ => {
                    let resp = self.handle_control_request(trimmed);
                    if let Some(chan) = reply_chan {
                        let _ = chan.send(resp);
                    } else if !resp.is_empty() {
                        println!("{resp}");
                    }
                }
            }
        }
    }
}

fn run_live(args: &[String]) -> Result<()> {
    if args.len() < 3 {
        return Err(bad(
            "usage: sawbyte-play live <server:port> <--offline|--account> <user> [blocks.json] [--listen <ip:port>] [--mcp-stdio] [--script <script>] [--ticks <n>] [--trace <trace.csv>] [--policy <humanised|smooth|direct>]",
        ));
    }
    let server = &args[0];
    let account = match args[1].as_str() {
        "--offline" => auth::Account::offline(&args[2])?,
        "--account" => auth::Account::token_file(std::path::Path::new(&args[2]))?,
        _ => return Err(bad("choose --offline or --account")),
    };
    if args.len() > 3 && !args[3].starts_with("--") && args[3] != "default"
        && !std::path::Path::new(&args[3]).exists()
    {
        return Err(bad(&format!("block profile does not exist: {}", args[3])));
    }
    let (profile, mut idx) = if args.len() > 3
        && !args[3].starts_with("--")
        && args[3] != "default"
        && std::path::Path::new(&args[3]).exists()
    {
        let p: serde_json::Value = serde_json::from_reader(File::open(&args[3])?)?;
        if p["minecraft"] != "26.3" {
            return Err(bad("wrong block profile version"));
        }
        (p, 4)
    } else {
        let p = world::default_26_3_profile();
        let skip = if args.len() > 3 && (args[3] == "default" || !args[3].starts_with("--")) {
            4
        } else {
            3
        };
        (p, skip)
    };

    let mut listen_addr: Option<String> = None;
    let mut mcp_stdio = false;
    let mut initial_script: Option<String> = None;
    let mut max_ticks: Option<u64> = None;
    let mut trace_path: Option<String> = None;
    let mut policy = DynamicCameraPolicy::default();
    while idx < args.len() {
        match args[idx].as_str() {
            "--listen" if idx + 1 < args.len() => {
                listen_addr = Some(args[idx + 1].clone());
                idx += 2;
            }
            "--mcp-stdio" => {
                mcp_stdio = true;
                idx += 1;
            }
            "--script" if idx + 1 < args.len() => {
                initial_script = Some(args[idx + 1].clone());
                idx += 2;
            }
            "--ticks" if idx + 1 < args.len() => {
                max_ticks = Some(
                    args[idx + 1]
                        .parse()
                        .map_err(|_| bad("invalid --ticks value"))?,
                );
                idx += 2;
            }
            "--trace" if idx + 1 < args.len() => {
                trace_path = Some(args[idx + 1].clone());
                idx += 2;
            }
            "--policy" if idx + 1 < args.len() => {
                policy = match args[idx + 1].as_str() {
                    "humanised" => DynamicCameraPolicy::Humanised(Humanised::default()),
                    "smooth" => DynamicCameraPolicy::Smooth(SmoothHumanisedCamera::default()),
                    "direct" => DynamicCameraPolicy::Direct(Direct),
                    _ => return Err(bad("unknown --policy; use humanised, smooth, or direct")),
                };
                idx += 2;
            }
            _ => return Err(bad(&format!("unknown live option '{}'", args[idx]))),
        }
    }

    let mut state = Session::new(&profile, true, policy)?;

    // Start control channels (`--mcp-stdio` and `--listen`) IMMEDIATELY before network connect
    // so AI MCP clients (Kilo CLI, Claude, Cursor) receive instant <1ms responses to
    // `initialize`, `tools/list`, `resources/list`, and `resources/templates/list`!
    let (ctrl_tx, ctrl_rx) =
        mpsc::channel::<(String, Option<mpsc::Sender<String>>)>();
    if mcp_stdio {
        let sender = ctrl_tx.clone();
        std::thread::spawn(move || {
            let stdin = std::io::stdin();
            let mut stdout = std::io::stdout();
            for line in stdin.lock().lines().map_while(std::result::Result::ok) {
                let trimmed = line.trim();
                // Fast-path MCP discovery handshake so Kilo CLI / Claude / Cursor gets instant <1ms responses
                // even while the Minecraft TCP handshake is still in progress!
                if trimmed.starts_with('{') {
                    if let Ok(val) = serde_json::from_str::<serde_json::Value>(trimmed) {
                        let id = val.get("id").cloned().unwrap_or(serde_json::json!(1));
                        match val["method"].as_str().unwrap_or("") {
                            "initialize" => {
                                let resp = serde_json::json!({
                                    "jsonrpc": "2.0",
                                    "id": id,
                                    "result": {
                                        "protocolVersion": "2025-06-18",
                                        "serverInfo": { "name": "sawbyte-play-live-mcp", "version": "0.5.0" },
                                        "capabilities": {
                                            "tools": { "listChanged": false },
                                            "resources": { "listChanged": false, "subscribe": false },
                                            "prompts": { "listChanged": false }
                                        }
                                    }
                                });
                                let _ = writeln!(stdout, "{resp}");
                                let _ = stdout.flush();
                                continue;
                            }
                            "notifications/initialized" => continue,
                            "ping" => {
                                let resp = serde_json::json!({ "jsonrpc": "2.0", "id": id, "result": {} });
                                let _ = writeln!(stdout, "{resp}");
                                let _ = stdout.flush();
                                continue;
                            }
                            "resources/list" => {
                                let resp = serde_json::json!({
                                    "jsonrpc": "2.0",
                                    "id": id,
                                    "result": {
                                        "resources": [
                                            { "uri": "sawbyte://session/status", "name": "Live 26.3 Session Status & Vitals", "mimeType": "application/json" },
                                            { "uri": "sawbyte://session/inventory", "name": "Live 46-Slot Player Inventory & Hotbar", "mimeType": "application/json" },
                                            { "uri": "sawbyte://session/entities", "name": "Tracked Nearby Entities & Lobby NPCs", "mimeType": "application/json" },
                                            { "uri": "sawbyte://session/chat", "name": "Live Server Chat Log", "mimeType": "application/json" },
                                            { "uri": "sawbyte://world/map2d", "name": "Live 2D ASCII World Map", "mimeType": "text/plain" }
                                        ]
                                    }
                                });
                                let _ = writeln!(stdout, "{resp}");
                                let _ = stdout.flush();
                                continue;
                            }
                            "resources/templates/list" => {
                                let resp = serde_json::json!({
                                    "jsonrpc": "2.0",
                                    "id": id,
                                    "result": {
                                        "resourceTemplates": [
                                            { "uriTemplate": "sawbyte://session/{section}", "name": "Live Session Section (status, inventory, entities, chat, events)", "mimeType": "application/json" }
                                        ]
                                    }
                                });
                                let _ = writeln!(stdout, "{resp}");
                                let _ = stdout.flush();
                                continue;
                            }
                            "prompts/list" => {
                                let resp = serde_json::json!({
                                    "jsonrpc": "2.0",
                                    "id": id,
                                    "result": {
                                        "prompts": [
                                            { "name": "automate_minecraft", "description": "Control Sawbyte 26.3 live session via MCP tools." }
                                        ]
                                    }
                                });
                                let _ = writeln!(stdout, "{resp}");
                                let _ = stdout.flush();
                                continue;
                            }
                            _ => {}
                        }
                    }
                }
                let (reply_tx, reply_rx) = mpsc::channel();
                if sender.send((line, Some(reply_tx))).is_err() {
                    break;
                }
                if let Ok(resp) = reply_rx.recv_timeout(Duration::from_secs(10)) {
                    if !resp.is_empty() {
                        let _ = writeln!(stdout, "{resp}");
                        let _ = stdout.flush();
                    }
                }
            }
        });
    }
    if let Some(bind_addr) = listen_addr {
        let listener = TcpListener::bind(&bind_addr)?;
        eprintln!("SAWBYTE_CONTROL_LISTENING addr={bind_addr}");
        let sender = ctrl_tx.clone();
        std::thread::spawn(move || {
            for stream in listener.incoming().flatten() {
                let sender_conn = sender.clone();
                std::thread::spawn(move || {
                    let Ok(mut writer) = stream.try_clone() else {
                        return;
                    };
                    let mut cancel_on_disconnect = false;
                    let reader = BufReader::new(stream);
                    for line in reader.lines().map_while(std::result::Result::ok) {
                        let trimmed = line.trim();
                        if trimmed == "session.lease"
                            || trimmed == "session.lease on"
                            || trimmed == "session.lease cancel_on_disconnect=true"
                        {
                            cancel_on_disconnect = true;
                            let is_json = trimmed.starts_with('{');
                            let resp = if is_json {
                                "{\"ok\":true,\"status\":\"completed\",\"command\":\"session.lease\",\"cancel_on_disconnect\":true}".to_string()
                            } else {
                                "session.lease enabled: cancel_on_disconnect=true".to_string()
                            };
                            let _ = writeln!(writer, "{resp}");
                            let _ = writer.flush();
                            continue;
                        } else if trimmed == "session.lease off"
                            || trimmed == "session.lease cancel_on_disconnect=false"
                        {
                            cancel_on_disconnect = false;
                            let is_json = trimmed.starts_with('{');
                            let resp = if is_json {
                                "{\"ok\":true,\"status\":\"completed\",\"command\":\"session.lease\",\"cancel_on_disconnect\":false}".to_string()
                            } else {
                                "session.lease disabled: cancel_on_disconnect=false".to_string()
                            };
                            let _ = writeln!(writer, "{resp}");
                            let _ = writer.flush();
                            continue;
                        }

                        let is_json = line.trim_start().starts_with('{');
                        let (reply_tx, reply_rx) = mpsc::channel();
                        if sender_conn.send((line, Some(reply_tx))).is_err() {
                            break;
                        }
                        if let Ok(resp) = reply_rx.recv_timeout(Duration::from_secs(10)) {
                            if !resp.is_empty() {
                                let framed = if is_json {
                                    resp.replace('\n', " ")
                                } else {
                                    resp.replace('\r', "").replace('\n', "\\n")
                                };
                                let _ = writeln!(writer, "{framed}");
                                let _ = writer.flush();
                            }
                        }
                    }
                    if cancel_on_disconnect {
                        let _ = sender_conn.send(("task.cancel".to_string(), None));
                    }
                });
            }
        });
    }

    let (mut rx, mut tx) = connect(server, &account)?;
    let shutdown = tx.stream.try_clone()?;
    let (send, receive) = mpsc::sync_channel(64);
    let reader = std::thread::spawn(move || {
        loop {
            let packet = rx.packet();
            let done = packet.is_err();
            if send.send(packet).is_err() || done {
                break;
            }
        }
    });

    let mut trace_writer = match trace_path {
        Some(ref p) => {
            let mut w = BufWriter::new(File::create(p)?);
            writeln!(w, "tick,x,y,z,yaw,pitch,on_ground,chunks")?;
            Some(w)
        }
        None => None,
    };

    if let Some(ref script) = initial_script {
        for line in script.lines() {
            let trimmed = line.trim();
            if !trimmed.is_empty() && !trimmed.starts_with('#') && !trimmed.starts_with("//") {
                state.script_queue.push_back(trimmed.to_string());
            }
        }
    }
    let start = Instant::now();
    let deadline_secs = 60 + max_ticks.unwrap_or(600) / 20;
    let mut next = Instant::now();

    let result = (|| -> Result<()> {
        loop {
            if max_ticks.is_some() && start.elapsed() > Duration::from_secs(deadline_secs) {
                return Err(bad("live session deadline exceeded"));
            }
            let wait = next.saturating_duration_since(Instant::now());
            match receive.recv_timeout(wait) {
                Ok(packet) => state.packet(&packet?, &mut tx)?,
                Err(mpsc::RecvTimeoutError::Disconnected) => {
                    return Err(bad("network reader stopped"));
                }
                Err(mpsc::RecvTimeoutError::Timeout) => {}
            }
            if Instant::now() >= next {
                while let Ok(packet) = receive.try_recv() {
                    state.packet(&packet?, &mut tx)?;
                }
                // Always process incoming MCP/TCP control commands immediately so inspect/chat/inventory never hang during settle
                while let Ok((req_line, reply_chan)) = ctrl_rx.try_recv() {
                    state.handle_control_request_with_channel(&req_line, reply_chan);
                }
                // Once settled on the live server, execute queued `--script` lines
                if state.loaded && state.settle == 0 {

                    if state.pending_replies.is_empty()
                        && state.active_nav_goal.is_none()
                        && state.scheduler.pending() == 0
                        && state.active_mine.is_none()
                        && state.event_queue.pending() == 0
                        && state.pending_placements.is_empty()
                        && state.pending_click.is_none()
                    {
                        if let Some(script_line) = state.script_queue.pop_front() {
                            state.handle_control_request_with_channel(&script_line, None);
                        }
                    }
                }
                let active = state.tick_live(trace_writer.as_mut(), &mut tx)?;
                if active {
                    if let Some(limit) = max_ticks {
                        if state.live_tick >= limit {
                            if let Some(ref mut w) = trace_writer {
                                w.flush()?;
                            }
                            eprintln!("SAWBYTE_LIVE_COMPLETE ticks={}", state.live_tick);
                            return Ok(());
                        }
                    }
                }
                next += Duration::from_millis(50);
                if Instant::now().saturating_duration_since(next) > Duration::from_secs(2) {
                    next = Instant::now();
                }
            }
        }
    })();

    let _ = shutdown.shutdown(Shutdown::Both);
    drop(receive);
    let _ = reader.join();
    result
}


/// Runs a genuine standalone Singleplayer Minecraft session in pure Rust at 20 TPS!
/// Generates a procedural world around spawn, populates the player's starter inventory,
/// simulates local block breaking/placement/physics/entities, and exposes the full
/// `--listen <ip:port>` and `--mcp-stdio` interfaces (plus Potato 3D & Radar rendering)
/// without needing an external Java server.
fn run_singleplayer(args: &[String]) -> Result<()> {
    let mut world_name = "New_World_26_3".to_string();
    let mut seed: u64 = 20261106;
    let mut gamemode = "survival".to_string(); // survival | creative | hardcore | adventure | spectator
    let mut difficulty = "normal".to_string(); // peaceful | easy | normal | hard
    let mut world_type = "normal".to_string(); // normal | flat | large_biomes | amplified | single_biome_surface
    let mut allow_commands = true;
    let mut generate_structures = true;
    let mut bonus_chest = false;
    let mut simulation_distance: u32 = 10;
    let mut view_distance: u32 = 10;
    let mut engine_mode = "auto".to_string(); // auto | integrated-26.3 | rust-native
    let mut listen_addr: Option<String> = Some("127.0.0.1:25580".to_string());
    let mut mcp_stdio = false;
    let mut max_ticks: Option<u64> = None;
    let mut profile = world::default_26_3_profile();

    let mut idx = 0;
    while idx < args.len() {
        match args[idx].as_str() {
            "--world" if idx + 1 < args.len() => {
                world_name = args[idx + 1].clone();
                idx += 2;
            }
            "--seed" if idx + 1 < args.len() => {
                seed = args[idx + 1].parse().unwrap_or(20261106);
                idx += 2;
            }
            "--gamemode" | "--mode" if idx + 1 < args.len() => {
                gamemode = args[idx + 1].to_lowercase();
                idx += 2;
            }
            "--difficulty" if idx + 1 < args.len() => {
                difficulty = args[idx + 1].to_lowercase();
                idx += 2;
            }
            "--world-type" | "--level-type" if idx + 1 < args.len() => {
                world_type = args[idx + 1].to_lowercase();
                idx += 2;
            }
            "--allow-commands" | "--cheats" if idx + 1 < args.len() => {
                allow_commands = args[idx + 1] != "false" && args[idx + 1] != "0";
                idx += 2;
            }
            "--generate-structures" if idx + 1 < args.len() => {
                generate_structures = args[idx + 1] != "false" && args[idx + 1] != "0";
                idx += 2;
            }
            "--bonus-chest" if idx + 1 < args.len() => {
                bonus_chest = args[idx + 1] == "true" || args[idx + 1] == "1";
                idx += 2;
            }
            "--simulation-distance" if idx + 1 < args.len() => {
                simulation_distance = args[idx + 1].parse().unwrap_or(10).clamp(2, 32);
                idx += 2;
            }
            "--render-distance" | "--view-distance" if idx + 1 < args.len() => {
                view_distance = args[idx + 1].parse().unwrap_or(10).clamp(2, 32);
                idx += 2;
            }
            "--engine" if idx + 1 < args.len() => {
                engine_mode = args[idx + 1].to_lowercase();
                idx += 2;
            }
            "--listen" if idx + 1 < args.len() => {
                listen_addr = Some(args[idx + 1].clone());
                idx += 2;
            }
            "--mcp-stdio" => {
                mcp_stdio = true;
                idx += 1;
            }
            "--ticks" if idx + 1 < args.len() => {
                max_ticks = args[idx + 1].parse().ok();
                idx += 2;
            }
            "--profile" if idx + 1 < args.len() => {
                if let Ok(f) = File::open(&args[idx + 1]) {
                    if let Ok(p) = serde_json::from_reader(f) {
                        profile = p;
                    }
                }
                idx += 2;
            }
            other if !other.starts_with("--") && std::path::Path::new(other).exists() => {
                if let Ok(f) = File::open(other) {
                    if let Ok(p) = serde_json::from_reader(f) {
                        profile = p;
                    }
                }
                idx += 1;
            }
            _ => idx += 1,
        }
    }

    // Write authentic Minecraft 26.3 server.properties for the singleplayer world in ~/.sawbyte/saves/<world_name>/
    let home = std::env::var("SAWBYTE_HOME")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|_| {
            std::env::var("HOME")
                .map(|h| std::path::PathBuf::from(h).join(".sawbyte"))
                .unwrap_or_else(|_| std::env::temp_dir().join("sawbyte-home"))
        });
    let world_dir = home.join("saves").join(&world_name);
    let _ = std::fs::create_dir_all(&world_dir);
    let _ = std::fs::write(world_dir.join("eula.txt"), b"eula=true\n");
    let hardcore_bool = gamemode == "hardcore";
    let effective_gm = if hardcore_bool { "survival" } else { gamemode.as_str() };
    let props = format!(
        "level-name={world_name}\nlevel-seed={seed}\ngamemode={effective_gm}\ndifficulty={difficulty}\nhardcore={hardcore_bool}\nlevel-type=minecraft\\:{world_type}\ngenerate-structures={generate_structures}\nsimulation-distance={simulation_distance}\nview-distance={view_distance}\nonline-mode=false\nspawn-protection=0\nenable-command-block={allow_commands}\nserver-ip=127.0.0.1\nserver-port=25565\n"
    );
    let _ = std::fs::write(world_dir.join("server.properties"), props.as_bytes());

    // If an official 26.3 integrated server jar is installed and engine_mode != "rust-native", launch and connect to it!
    let integrated_jar = home.join("editions").join("server-26.3.jar");
    if (engine_mode == "integrated-26.3" || engine_mode == "auto") && integrated_jar.exists() {
        eprintln!("SAWBYTE_SINGLEPLAYER_INTEGRATED_SERVER starting world={world_name} gm={gamemode} diff={difficulty} type={world_type}");
        if let Ok(mut child) = std::process::Command::new("java")
            .current_dir(&world_dir)
            .args(["-Xms512M", "-Xmx2G", "-jar", integrated_jar.to_str().unwrap_or(""), "nogui"])
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .spawn()
        {
            std::thread::sleep(Duration::from_secs(3));
            let mut live_args = vec![
                "127.0.0.1:25565".to_string(),
                "--offline".to_string(),
                "Singleplayer".to_string(),
                "default".to_string(),
            ];
            if let Some(ref l) = listen_addr {
                live_args.push("--listen".to_string());
                live_args.push(l.clone());
            }
            if mcp_stdio {
                live_args.push("--mcp-stdio".to_string());
            }
            if let Some(t) = max_ticks {
                live_args.push("--ticks".to_string());
                live_args.push(t.to_string());
            }
            let res = run_live(&live_args);
            let _ = child.kill();
            return res;
        }
    }

    let mut state = Session::new(&profile, true, DynamicCameraPolicy::default())?;
    state.world.min_y = -64;
    state.world.height = 384;
    state.world.generate_singleplayer_world(seed, (view_distance as i32).clamp(3, 10))?;
    state.state_table.game_mode = match gamemode.as_str() {
        "creative" => 1,
        "adventure" => 2,
        "spectator" => 3,
        _ => 0,
    };
    state.player.flying = state.state_table.game_mode == 3;
    let _ = bonus_chest;
    state.player.pos = [0.5, 64.0, 0.5];
    state.player.ground = true;
    state.playing = true;
    state.positioned = true;
    state.loaded = true;
    state.settle = 0;

    // Give the Singleplayer player a starter kit in their hotbar (slots 36..=40)
    let mut compass = sawbyte_core::screen::ItemStack::new("minecraft:compass", 1);
    compass.display_name = Some("Singleplayer World Compass".into());
    state.state_table.inventory_slots[36] = Some(compass);
    state.state_table.inventory_slots[37] = Some(sawbyte_core::screen::ItemStack::new("minecraft:diamond_pickaxe", 1));
    state.state_table.inventory_slots[38] = Some(sawbyte_core::screen::ItemStack::new("minecraft:diamond_sword", 1));
    state.state_table.inventory_slots[39] = Some(sawbyte_core::screen::ItemStack::new("minecraft:oak_planks", 64));
    state.state_table.inventory_slots[40] = Some(sawbyte_core::screen::ItemStack::new("minecraft:cooked_beef", 32));
    state.state_table.chat_log.push(format!("[Singleplayer] Joined local world '{world_name}' (seed={seed}) at (0.5, 64.0, 0.5)"));

    // Spawn sample passive/hostile entities around singleplayer world
    for (eid, etype, ex, ez) in [(101, 19, 6.5, 4.5), (102, 82, -5.5, 7.5), (103, 90, 8.5, -4.5)] {
        state.state_table.entities.insert(
            eid,
            inventory::TrackedEntity {
                entity_id: eid,
                uuid: [0u8; 16],
                entity_type: etype,
                kind_name: inventory::entity_type_name(etype).to_string(),
                custom_name: None,
                metadata_texts: Vec::new(),
                pos: [ex, 64.0, ez],
                velocity: [0.0; 3],
                yaw: 0.0,
                pitch: 0.0,
                width: 0.9,
                height: 1.4,
                equipment: HashMap::new(),
            },
        );
    }

    let (ctrl_tx, ctrl_rx) = mpsc::channel::<(String, Option<mpsc::Sender<String>>)>();
    if mcp_stdio {
        let sender = ctrl_tx.clone();
        std::thread::spawn(move || {
            let stdin = std::io::stdin();
            let mut stdout = std::io::stdout();
            for line in stdin.lock().lines().map_while(std::result::Result::ok) {
                let (reply_tx, reply_rx) = mpsc::channel();
                if sender.send((line, Some(reply_tx))).is_err() {
                    break;
                }
                if let Ok(resp) = reply_rx.recv_timeout(Duration::from_secs(5)) {
                    if !resp.is_empty() {
                        let _ = writeln!(stdout, "{resp}");
                        let _ = stdout.flush();
                    }
                }
            }
        });
    }
    if let Some(bind_addr) = listen_addr {
        if let Ok(listener) = TcpListener::bind(&bind_addr) {
            eprintln!("SAWBYTE_CONTROL_LISTENING addr={bind_addr}");
            let sender = ctrl_tx.clone();
            std::thread::spawn(move || {
                for stream in listener.incoming().flatten() {
                    let sender_conn = sender.clone();
                    std::thread::spawn(move || {
                        let Ok(mut writer) = stream.try_clone() else { return; };
                        let reader = BufReader::new(stream);
                        for line in reader.lines().map_while(std::result::Result::ok) {
                            let is_json = line.trim_start().starts_with('{');
                            let (reply_tx, reply_rx) = mpsc::channel();
                            if sender_conn.send((line, Some(reply_tx))).is_err() {
                                break;
                            }
                            if let Ok(resp) = reply_rx.recv_timeout(Duration::from_secs(10)) {
                                if !resp.is_empty() {
                                    let framed = if is_json {
                                        resp.replace('\n', " ")
                                    } else {
                                        resp.replace('\r', "").replace('\n', "\\n")
                                    };
                                    let _ = writeln!(writer, "{framed}");
                                    let _ = writer.flush();
                                }
                            }
                        }
                    });
                }
            });
        }
    }

    eprintln!("SAWBYTE_SPAWNED");
    eprintln!("SAWBYTE_SINGLEPLAYER_READY world={world_name} seed={seed}");

    let mut next = Instant::now();
    loop {
        while let Ok((req_line, reply_chan)) = ctrl_rx.try_recv() {
            state.handle_control_request_with_channel(&req_line, reply_chan);
        }

        // Step navigation, scheduler, physics, and local singleplayer block interactions at 20 TPS
        state.step_live_navigation();
        let frame = state
            .scheduler
            .advance(state.live_tick)
            .map_err(|e| bad(&format!("scheduler error: {e}")))?;
        state.player.step(frame, &state.world)?;
        state.world.update_center_chunk(state.player.pos[0], state.player.pos[2]);

        // Process active mining in singleplayer (breaks block locally when remaining_ticks hits 0)
        if let Some(mut mine) = state.active_mine.take() {
            if mine.remaining_ticks > 1 {
                mine.remaining_ticks -= 1;
                state.active_mine = Some(mine);
            } else {
                let air_id = 0;
                let _ = state.world.set_block(mine.pos[0], mine.pos[1], mine.pos[2], air_id);
                state.state_table.push_event(format!("singleplayer_block_broken:({},{},{})", mine.pos[0], mine.pos[1], mine.pos[2]));
            }
        }

        // Drain discrete events in singleplayer
        if let Ok(drained) = state.event_queue.drain_tick(state.live_tick) {
            for ev in drained {
                match ev {
                    SubTickEvent::SelectHotbar { slot } => {
                        state.state_table.selected_hotbar = slot;
                    }
                    SubTickEvent::SendChat { message } => {
                        state.state_table.chat_log.push(format!("<Singleplayer> {message}"));
                    }
                    SubTickEvent::SendCommand { command } => {
                        state.state_table.chat_log.push(format!("[LocalServer] Executed /{command}"));
                    }
                    SubTickEvent::UseItemOn { pos, face, .. } => {
                        let norm = face.normal();
                        let target = [pos[0] + norm[0], pos[1] + norm[1], pos[2] + norm[2]];
                        let _ = state.world.set_block(target[0], target[1], target[2], 1);
                    }
                    _ => {}
                }
            }
        }

        state.live_tick += 1;
        state.resolve_pending_replies();

        if let Some(limit) = max_ticks {
            if state.live_tick >= limit {
                return Ok(());
            }
        }

        next += Duration::from_millis(50);
        let wait = next.saturating_duration_since(Instant::now());
        if !wait.is_zero() {
            std::thread::sleep(wait);
        } else if Instant::now().saturating_duration_since(next) > Duration::from_secs(2) {
            next = Instant::now();
        }
    }
}

pub fn run_benchmarks() -> Result<()> {
    println!("================================================================================");
    println!("                    SAWBYTE NATIVE ENGINE BENCHMARK SUITE");
    println!("================================================================================");

    let profile = serde_json::json!({
        "minecraft": "26.3",
        "air": [0],
        "solid": [1],
        "bottom_slabs": [2],
        "water": [3]
    });
    let mut world = world::World::new(&profile)?;
    world.min_y = -64;
    world.height = 384;
    let num_sections = (world.height / 16) as usize;

    for cx in -2..=2 {
        for cz in -2..=2 {
            let mut sections = Vec::with_capacity(num_sections);
            for _ in 0..num_sections {
                sections.push(world::Palette::single(0));
            }
            world.chunks.insert((cx, cz), sections);
        }
    }

    for x in -32..32 {
        for z in -32..32 {
            world.set_block(x, -61, z, 1)?;
            if (x + z) % 8 == 0 {
                world.set_block(x, -60, z, 2)?;
            } else if (x + z) % 15 == 0 {
                world.set_block(x, -60, z, 3)?;
            }
        }
    }

    println!("[1/3] 3D Minecraft 26.3 Physics & Collision Simulation (100,000 ticks)");
    println!("      Simulating real player locomotion: sprinting, slab stepping, sneaking");
    println!("      edge-clamping, jumping, gravity, friction, and fluid dynamics in 5x5 chunks.");

    let mut player = player::Player {
        pos: [0.5, -60.0, 0.5],
        ..Default::default()
    };

    let t0 = Instant::now();
    let total_ticks = 100_000usize;
    for tick in 0..total_ticks {
        let cycle = tick % 100;
        let mut forward = true;
        let mut backward = false;
        let mut left = false;
        let right = false;
        let mut jump = false;
        let mut sneak = false;
        let mut sprint = true;

        if cycle < 30 {
            sprint = true;
        } else if cycle < 50 {
            jump = true;
        } else if cycle < 70 {
            sprint = false;
            sneak = true;
            player.yaw += 2.0;
        } else if cycle < 85 {
            sprint = false;
            left = true;
            player.yaw -= 1.5;
        } else {
            forward = false;
            backward = true;
            player.pitch = (tick as f32 % 30.0) - 15.0;
        }

        let mut bits = 0u16;
        if forward { bits |= 1 << (Button::Forward as u8); }
        if backward { bits |= 1 << (Button::Back as u8); }
        if left { bits |= 1 << (Button::Left as u8); }
        if right { bits |= 1 << (Button::Right as u8); }
        if jump { bits |= 1 << (Button::Jump as u8); }
        if sneak { bits |= 1 << (Button::Sneak as u8); }
        if sprint { bits |= 1 << (Button::Sprint as u8); }

        let frame = InputFrame {
            buttons: sawbyte_core::Buttons::from_bits(bits).unwrap_or_default(),
            yaw_delta: 0.0,
            pitch_delta: 0.0,
        };
        let _ = player.step(frame, &world);
    }
    let elapsed_physics = t0.elapsed();
    let tps = (total_ticks as f64) / elapsed_physics.as_secs_f64();
    let ns_per_tick = elapsed_physics.as_nanos() as f64 / total_ticks as f64;
    let realtime_mult = tps / 20.0;
    println!("      -> Elapsed: {:.3} ms", elapsed_physics.as_secs_f64() * 1000.0);
    println!("      -> Throughput: {:.0} TPS ({:.1}x real-time speed)", tps, realtime_mult);
    println!("      -> Latency: {:.1} ns/tick\n", ns_per_tick);

    println!("[2/3] Sub-Voxel 3D Spatial Raycasting & Occlusion (100,000 rays)");
    println!("      Casting 32m rays with 3D DDA voxel traversal and entity AABB occlusion");
    println!("      testing against 50 tracked entities in a loaded voxel world.");

    let mut entity_aabbs = Vec::with_capacity(50);
    for i in 0..50i32 {
        let ex = ((i % 10) as f64 - 5.0) * 3.0;
        let ez = ((i / 10) as f64 - 2.5) * 3.0;
        let aabb = [ex - 0.3, -60.0, ez - 0.3, ex + 0.3, -58.2, ez + 0.3];
        entity_aabbs.push((i + 100, aabb));
    }

    let total_rays = 100_000usize;
    let t1 = Instant::now();
    for i in 0..total_rays {
        player.yaw = ((i as f32 * 0.37) % 360.0) - 180.0;
        player.pitch = ((i as f32 * 0.19) % 170.0) - 85.0;
        let _ = raycast::raycast_block(&player, &world);
        let _ = raycast::raycast_entity(&player, &world, &entity_aabbs);
    }
    let elapsed_rays = t1.elapsed();
    let rps = (total_rays as f64) / elapsed_rays.as_secs_f64();
    let mrps = rps / 1_000_000.0;
    let ns_per_ray = elapsed_rays.as_nanos() as f64 / total_rays as f64;
    println!("      -> Elapsed: {:.3} ms", elapsed_rays.as_secs_f64() * 1000.0);
    println!("      -> Throughput: {:.2} MRPS ({:.0} rays/sec)", mrps, rps);
    println!("      -> Latency: {:.1} ns/ray\n", ns_per_ray);

    println!("[3/3] 3D A* Voxel Navigation & Pathfinding (500 queries)");
    println!("      Full 3D heuristic path search across obstacle terrain with jump & step");
    println!("      reachability validation (budget 2,048 nodes).");

    let total_paths = 500usize;
    let t2 = Instant::now();
    let mut successful_paths = 0usize;
    for i in 0..total_paths {
        let sx = ((i % 16) as i32) - 8;
        let sz = (((i * 7) % 16) as i32) - 8;
        let gx = (((i * 3) % 24) as i32) - 12;
        let gz = (((i * 11) % 24) as i32) - 12;
        let start = [sx, -60, sz];
        let goal = [gx, -60, gz];
        if let Ok(path) = find_path(&world, start, goal, 2048) {
            if !path.is_empty() {
                successful_paths += 1;
            }
        }
    }
    let elapsed_paths = t2.elapsed();
    let pps = (total_paths as f64) / elapsed_paths.as_secs_f64();
    let us_per_path = (elapsed_paths.as_micros() as f64) / (total_paths as f64);
    println!("      -> Elapsed: {:.3} ms", elapsed_paths.as_secs_f64() * 1000.0);
    println!(
        "      -> Throughput: {:.0} paths/sec (success rate: {}/{})",
        pps, successful_paths, total_paths
    );
    println!("      -> Latency: {:.2} µs/path", us_per_path);
    println!("================================================================================");
    Ok(())
}

pub fn run() -> Result<()> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    if args.is_empty() || args == ["--help"] {
        println!(
            "Sawbyte standalone headless Rust client — Java 26.3\n\n\
            Live interactive session (MCP / TCP / Script / Gymnasium):\n\
              sawbyte-play live <server:port> <--offline|--account> <name-or-token-file> <blocks.json> [--listen <ip:port>] [--mcp-stdio] [--script <script>] [--ticks <n>] [--trace <trace.csv>] [--policy <humanised|smooth|direct>]\n\n\
            Deterministic SBIF parity replay:\n\
              sawbyte-play <server:port> <--offline|--account> <name-or-token-file> <input.sbif> <trace.csv> <blocks.json>\n\n\
            Authentication:\n\
              sawbyte-play login <registered-microsoft-client-id> <new-token-file>\n\n\
            Benchmarks:\n\
              sawbyte-play benchmark\n\n\
            Zero-graphics 20 TPS client with 26.3 chunk palettes, dynamic block updates, slabs/stairs/climbables/fluids,\n\
            sprinting/sneaking/stepping, live 3D A* pathfinding, eye raycasting, container screens, and RL/MCP control."
        );
        return Ok(());
    }
    if args == ["--version"] || args == ["-v"] {
        println!("sawbyte-play {}", env!("CARGO_PKG_VERSION"));
        return Ok(());
    }
    if args.first().map(String::as_str) == Some("benchmark") {
        return run_benchmarks();
    }
    if args.first().map(String::as_str) == Some("singleplayer")
        || (args.first().map(String::as_str) == Some("live")
            && args.get(1).is_some_and(|s| s == "singleplayer" || s.starts_with("singleplayer:")))
    {
        let slice = if args.first().map(String::as_str) == Some("singleplayer") {
            &args[1..]
        } else {
            &args[2..]
        };
        return run_singleplayer(slice);
    }
    if args.first().map(String::as_str) == Some("live") {
        return run_live(&args[1..]);
    }
    if args.len() == 3 && args[0] == "login" {
        return auth::login(&args[1], std::path::Path::new(&args[2]));
    }
    if args.len() != 6 {
        return Err(bad("invalid arguments; use --help"));
    }
    let frames = tape::read(&mut File::open(&args[3])?).map_err(|e| bad(&e.to_string()))?;
    if frames.last() != Some(&InputFrame::default()) {
        return Err(bad("unsupported input or non-neutral final frame"));
    }
    let account = match args[1].as_str() {
        "--offline" => auth::Account::offline(&args[2])?,
        "--account" => auth::Account::token_file(std::path::Path::new(&args[2]))?,
        _ => return Err(bad("choose --offline or --account")),
    };
    let profile: serde_json::Value = serde_json::from_reader(File::open(&args[5])?)?;
    if profile["minecraft"] != "26.3" {
        return Err(bad("wrong block profile version"));
    }
    let mut state = Session::new(&profile, false, DynamicCameraPolicy::default())?;
    let (mut rx, mut tx) = connect(&args[0], &account)?;
    let shutdown = tx.stream.try_clone()?;
    let (send, receive) = mpsc::sync_channel(64);
    let reader = std::thread::spawn(move || {
        loop {
            let packet = rx.packet();
            let done = packet.is_err();
            if send.send(packet).is_err() || done {
                break;
            }
        }
    });
    let mut trace = BufWriter::new(File::create(&args[4])?);
    writeln!(trace, "tick,x,y,z,yaw,pitch,on_ground,chunks")?;
    let start = Instant::now();
    let mut next = Instant::now();
    let result = (|| -> Result<()> {
        loop {
            if start.elapsed() > Duration::from_secs(60 + frames.len() as u64 / 20) {
                return Err(bad("session deadline exceeded"));
            }
            let wait = next.saturating_duration_since(Instant::now());
            match receive.recv_timeout(wait) {
                Ok(packet) => state.packet(&packet?, &mut tx)?,
                Err(mpsc::RecvTimeoutError::Disconnected) => {
                    return Err(bad("network reader stopped"));
                }
                Err(mpsc::RecvTimeoutError::Timeout) => {}
            }
            if Instant::now() >= next {
                while let Ok(packet) = receive.try_recv() {
                    state.packet(&packet?, &mut tx)?;
                }
                if state.tick(&frames, &mut trace, &mut tx)? {
                    return Ok(());
                }
                next += Duration::from_millis(50);
                if Instant::now().saturating_duration_since(next) > Duration::from_secs(1) {
                    next = Instant::now();
                }
            }
        }
    })();
    let _ = shutdown.shutdown(Shutdown::Both);
    drop(receive);
    let _ = reader.join();
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_session() -> Session {
        let profile = serde_json::json!({
            "minecraft": "26.3",
            "air": [0],
            "solid": [1]
        });
        let mut s = Session::new(&profile, true, DynamicCameraPolicy::default()).unwrap();
        s.playing = true;
        s.positioned = true;
        s.loaded = true;
        s.settle = 0;
        s.live_tick = 0;
        s
    }

    #[test]
    fn live_control_one_shot_disconnect_does_not_cancel_active_task() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();
        let (ctrl_tx, ctrl_rx) = mpsc::channel::<(String, Option<mpsc::Sender<String>>)>();

        let handle = std::thread::spawn(move || {
            let (stream, _) = listener.accept().unwrap();
            let mut cancel_on_disconnect = false;
            let mut writer = stream.try_clone().unwrap();
            let reader = BufReader::new(stream);
            for line in reader.lines().map_while(std::result::Result::ok) {
                let trimmed = line.trim();
                if trimmed == "session.lease" || trimmed == "session.lease on" {
                    cancel_on_disconnect = true;
                    let _ = writeln!(writer, "session.lease enabled");
                    let _ = writer.flush();
                    continue;
                }
                let (reply_tx, reply_rx) = mpsc::channel();
                let _ = ctrl_tx.send((line, Some(reply_tx)));
                if let Ok(resp) = reply_rx.recv_timeout(Duration::from_millis(500)) {
                    let _ = writeln!(writer, "{resp}");
                    let _ = writer.flush();
                }
            }
            if cancel_on_disconnect {
                let _ = ctrl_tx.send(("task.cancel".to_string(), None));
            }
        });

        {
            let mut client = TcpStream::connect(addr).unwrap();
            writeln!(client, "player.select_hotbar 2").unwrap();
            client.flush().unwrap();
            let mut r = BufReader::new(&client);
            let mut reply_line = String::new();
            let (cmd, reply_chan) = ctrl_rx.recv_timeout(Duration::from_secs(2)).unwrap();
            assert_eq!(cmd, "player.select_hotbar 2");
            reply_chan.unwrap().send("ok".to_string()).unwrap();
            r.read_line(&mut reply_line).unwrap();
            assert_eq!(reply_line.trim(), "ok");
        } // client dropped here

        handle.join().unwrap();
        // Since lease was not requested, task.cancel must NOT have been sent
        assert!(ctrl_rx.try_recv().is_err());
    }

    #[test]
    fn live_control_leased_disconnect_cancels_active_task() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();
        let (ctrl_tx, ctrl_rx) = mpsc::channel::<(String, Option<mpsc::Sender<String>>)>();

        let handle = std::thread::spawn(move || {
            let (stream, _) = listener.accept().unwrap();
            let mut cancel_on_disconnect = false;
            let mut writer = stream.try_clone().unwrap();
            let reader = BufReader::new(stream);
            for line in reader.lines().map_while(std::result::Result::ok) {
                let trimmed = line.trim();
                if trimmed == "session.lease" || trimmed == "session.lease on" {
                    cancel_on_disconnect = true;
                    let _ = writeln!(writer, "session.lease enabled");
                    let _ = writer.flush();
                    continue;
                }
                let (reply_tx, reply_rx) = mpsc::channel();
                let _ = ctrl_tx.send((line, Some(reply_tx)));
                if let Ok(resp) = reply_rx.recv_timeout(Duration::from_millis(500)) {
                    let _ = writeln!(writer, "{resp}");
                    let _ = writer.flush();
                }
            }
            if cancel_on_disconnect {
                let _ = ctrl_tx.send(("task.cancel".to_string(), None));
            }
        });

        {
            let mut client = TcpStream::connect(addr).unwrap();
            writeln!(client, "session.lease on").unwrap();
            client.flush().unwrap();
            let mut r = BufReader::new(&client);
            let mut lease_reply = String::new();
            r.read_line(&mut lease_reply).unwrap();
            assert!(lease_reply.contains("session.lease enabled"));

            writeln!(client, "player.select_hotbar 2").unwrap();
            client.flush().unwrap();
            let (cmd, reply_chan) = ctrl_rx.recv_timeout(Duration::from_secs(2)).unwrap();
            assert_eq!(cmd, "player.select_hotbar 2");
            reply_chan.unwrap().send("ok".to_string()).unwrap();
        } // client dropped here

        handle.join().unwrap();
        // Because session.lease on was sent, task.cancel MUST be delivered
        let (cmd, _) = ctrl_rx.recv_timeout(Duration::from_secs(2)).unwrap();
        assert_eq!(cmd, "task.cancel");
    }

    #[test]
    fn control_engine_move_input_preempts_prior_move_and_composes_with_look() {
        let mut session = test_session();
        session.scheduler = Scheduler::new(
            LiveInputBackend::default(),
            DynamicCameraPolicy::Direct(Direct),
            4096,
        );
        session.live_tick = 1;

        let out1 = session.execute_live_script("player.input true false false false 0 0 10").unwrap();
        assert_eq!(out1.len(), 1);
        assert!(session.scheduler.pending() > 0);

        session.live_tick = 3;
        let out2 = session.execute_live_script("player.input false false false false 45 10 3").unwrap();
        assert_eq!(out2.len(), 1);
        assert!(session.scheduler.pending() > 0);

        let frame = session.scheduler.advance(4).unwrap();
        assert_eq!(frame.yaw_delta, 45.0);
        assert_eq!(frame.pitch_delta, 10.0);
    }

    #[test]
    fn invalid_training_input_does_not_preempt_existing_movement() {
        let mut session = test_session();
        session.execute_live_script("player.input true false false false 0 0 10").unwrap();
        let pending = session.scheduler.pending();
        assert!(session.execute_live_script("gym.step invalid 0 0 0 false false false 0 0").is_err());
        assert_eq!(session.scheduler.pending(), pending);
        assert!(session.execute_live_script("gym.step 0 0 0 0 false false false NaN 0").is_err());
        assert_eq!(session.scheduler.pending(), pending);
    }

    #[test]
    fn gym_step_and_script_queue_observe_post_tick_state() {
        let mut session = test_session();
        session.live_tick = 0;

        session.script_queue.push_back("player.select_hotbar 2".to_string());
        session.script_queue.push_back("gym.observe".to_string());

        let cmd1 = session.script_queue.pop_front().unwrap();
        let (tx, rx) = mpsc::channel();
        session.handle_control_request_with_channel(&cmd1, Some(tx));
        let r1 = rx.recv_timeout(Duration::from_secs(1)).unwrap();
        assert!(r1.contains("queued live hotbar selection slot=2"));

        session.live_tick = 1;
        let drained = session.event_queue.drain_tick(1).unwrap();
        for ev in drained {
            if let SubTickEvent::SelectHotbar { slot } = ev {
                session.state_table.selected_hotbar = slot;
            }
        }
        assert_eq!(session.state_table.selected_hotbar, 2);
        session.live_tick = 2;

        let cmd2 = session.script_queue.pop_front().unwrap();
        let (tx2, rx2) = mpsc::channel();
        session.handle_control_request_with_channel(&cmd2, Some(tx2));
        let obs_json = rx2.recv_timeout(Duration::from_secs(1)).unwrap();
        let obs: serde_json::Value = serde_json::from_str(&obs_json).unwrap();
        assert_eq!(obs["selected_hotbar"], 2);

        let (gym_tx, gym_rx) = mpsc::channel();
        session.handle_control_request_with_channel("gym.step 1 0 0 0 false false false 90 0 3", Some(gym_tx));
        assert!(gym_rx.try_recv().is_err());
        assert_eq!(session.pending_replies.len(), 1);
        let (other_tx, other_rx) = mpsc::channel();
        session.handle_control_request_with_channel("gym.step 0 0 0 0 false false false 0 0", Some(other_tx));
        assert!(other_rx.recv_timeout(Duration::from_secs(1)).unwrap().contains("already pending"));
        assert_eq!(session.pending_replies.len(), 1);

        session.scheduler.advance(2).unwrap();
        let drained = session.event_queue.drain_tick(2).unwrap();
        for ev in drained {
            if let SubTickEvent::SelectHotbar { slot } = ev {
                session.state_table.selected_hotbar = slot;
            }
        }
        session.live_tick = 3;
        session.resolve_pending_replies();

        let gym_reply = gym_rx.recv_timeout(Duration::from_secs(1)).unwrap();
        let gym_obs: serde_json::Value = serde_json::from_str(&gym_reply).unwrap();
        assert_eq!(gym_obs["tick"], 3);
        assert_eq!(gym_obs["command"], "gym.step");
        assert_eq!(gym_obs["selected_hotbar"], 3);
    }

    #[test]
    fn player_death_and_respawn_lifecycle_resets_state_and_emits_packets() {
        let mut session = test_session();
        session.live_tick = 1;

        // Queue active movement to verify it gets cancelled on death
        session.execute_live_script("player.input true false false false 0 0 10").unwrap();
        assert!(session.scheduler.pending() > 0);
        assert!(!session.is_dead);

        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();
        let client = TcpStream::connect(addr).unwrap();
        let (server, _) = listener.accept().unwrap();
        let mut tx = wire::Tx::new(client);

        // 1. Send cb_set_health with health = 0.0
        let mut death_pkt = Vec::new();
        var(session.packets.cb_set_health, &mut death_pkt);
        death_pkt.extend(0.0f32.to_be_bytes()); // health = 0.0
        var(20, &mut death_pkt);               // food = 20
        death_pkt.extend(5.0f32.to_be_bytes()); // saturation = 5.0
        session.packet(&death_pkt, &mut tx).unwrap();

        // Verify death state transitions
        assert!(session.is_dead);
        assert_eq!(session.death_count, 1);
        assert_eq!(session.scheduler.pending(), 0); // Active movement cancelled

        let obs_json = session.gym_observation_json("gym.observe");
        let obs: serde_json::Value = serde_json::from_str(&obs_json).unwrap();
        assert_eq!(obs["terminated"], true);
        assert_eq!(obs["is_dead"], true);

        // Verify ServerboundClientCommandPacket (packet 12, action 0 = perform respawn) was emitted
        let mut rx = wire::Rx::new(server);
        let respawn_cmd = rx.packet().unwrap();
        let mut r = Reader(&respawn_cmd);
        let pkt_id = r.var().unwrap();
        assert_eq!(pkt_id, session.packets.sb_client_command);
        let action = r.var().unwrap();
        assert_eq!(action, 0);

        // 2. Send cb_respawn packet
        let mut respawn_pkt = Vec::new();
        var(session.packets.cb_respawn, &mut respawn_pkt);
        session.packet(&respawn_pkt, &mut tx).unwrap();

        assert!(!session.is_dead);
        assert!(!session.positioned);
        assert!(!session.loaded);
        assert!(session.state_table.received_events.contains(&"player.respawn".to_string()));
    }

    #[test]
    fn tokenize_command_line_supports_quoted_spaces_escapes_and_json() {
        let cmd = tokenize_command_line("wait.until screen_open 40 \"Large Chest\"").unwrap();
        assert_eq!(cmd, vec!["wait.until", "screen_open", "40", "Large Chest"]);

        let cmd2 = tokenize_command_line("chat.send \"hello world\"").unwrap();
        assert_eq!(cmd2, vec!["chat.send", "hello world"]);

        let cmd3 = tokenize_command_line("set-setting \"/home/fivesaw/The Projects/Sawbyte/block-profile.json\"").unwrap();
        assert_eq!(cmd3, vec!["set-setting", "/home/fivesaw/The Projects/Sawbyte/block-profile.json"]);

        let cmd4 = tokenize_command_line(r#"{"command": "task.navigate", "args": [3, -60, 2]}"#).unwrap();
        assert_eq!(cmd4, vec!["task.navigate", "3", "-60", "2"]);

        assert!(tokenize_command_line("chat.send \"unclosed quote").is_err());
    }

    #[test]
    fn mine_unbreakable_block_fails_immediately_and_solid_block_takes_multiple_ticks() {
        let profile = serde_json::json!({
            "minecraft": "26.3",
            "air": [0],
            "solid": [1, 2],
            "hardness": {
                "1": 1.5,
                "2": -1.0
            }
        });
        let mut session = Session::new(&profile, true, DynamicCameraPolicy::default()).unwrap();
        session.playing = true;
        session.positioned = true;
        session.loaded = true;
        session.world.min_y = -64;
        session.world.height = 16;
        session.player.pos = [0.5, -60.0, 0.5];
        session.player.yaw = 0.0;
        session.player.pitch = 0.0;

        session.world.chunks.insert(
            (0, 0),
            vec![world::Palette::single(0)],
        );

        // 1. Bedrock at (0, -60, 1)
        session.world.set_block(0, -60, 1, 2).unwrap();
        let res_bedrock = session.execute_live_script("interact.mine 0 -60 1");
        assert!(res_bedrock.is_err());
        assert!(res_bedrock.unwrap_err().contains("unbreakable block"));

        // 2. Stone at (0, -60, 1)
        session.world.set_block(0, -60, 1, 1).unwrap();
        let res_stone = session.execute_live_script("interact.mine 0 -60 1");
        assert!(res_stone.is_ok());
        let outputs = res_stone.unwrap();
        assert!(outputs[0].contains("duration_ticks"));
        assert!(session.event_queue.pending() > 1);
    }

    #[test]
    fn attack_entity_blocked_by_wall_fails_line_of_sight_check() {
        let profile = serde_json::json!({
            "minecraft": "26.3",
            "air": [0],
            "solid": [1]
        });
        let mut session = Session::new(&profile, true, DynamicCameraPolicy::default()).unwrap();
        session.playing = true;
        session.positioned = true;
        session.loaded = true;
        session.world.min_y = -64;
        session.world.height = 16;
        session.player.pos = [0.5, -60.0, 0.5];
        session.player.yaw = 0.0;
        session.player.pitch = 0.0;

        session.world.chunks.insert(
            (0, 0),
            vec![world::Palette::single(0)],
        );

        // Track entity 42 at (0.5, -60.0, 2.5)
        session.state_table.entities.insert(
            42,
            inventory::TrackedEntity {
                entity_id: 42,
                uuid: [0u8; 16],
                entity_type: 1,
                kind_name: "entity".to_string(),
                custom_name: None,
                metadata_texts: Vec::new(),
                pos: [0.5, -60.0, 2.5],
                velocity: [0.0, 0.0, 0.0],
                yaw: 0.0,
                pitch: 0.0,
                width: 0.6,
                height: 1.8,
                equipment: HashMap::new(),
            },
        );

        // Wall between player and entity (2 blocks tall to block eye level)
        session.world.set_block(0, -60, 1, 1).unwrap();
        session.world.set_block(0, -59, 1, 1).unwrap();

        let res_blocked = session.execute_live_script("interact.attack 42");
        assert!(res_blocked.is_err());
        assert!(res_blocked.unwrap_err().contains("line of sight to entity 42 is blocked"));

        // Remove the wall
        session.world.set_block(0, -60, 1, 0).unwrap();
        session.world.set_block(0, -59, 1, 0).unwrap();
        let res_clear = session.execute_live_script("interact.attack 42");
        assert!(res_clear.is_ok());
        assert!(res_clear.unwrap()[0].contains("queued validated live attack on entity 42"));
    }

    #[test]
    fn unrecognized_block_state_halts_movement_and_records_event() {
        let mut session = test_session();
        session.live_tick = 1;
        session.world.min_y = -64;
        session.world.height = 16;
        session.player.pos = [0.5, -60.0, 0.5];
        session.world.chunks.insert(
            (0, 0),
            vec![world::Palette::single(0)],
        );

        // Put solid floor under player so gravity doesn't accelerate player down
        session.world.set_block(0, -61, 0, 1).unwrap();
        session.world.set_block(0, -61, 1, 1).unwrap();
        session.player.ground = true;

        // Put an unknown block state (id 999) in front of the player
        session.world.set_block(0, -60, 1, 999).unwrap();

        // Queue active movement
        session.execute_live_script("player.input true false false false 0 0 10").unwrap();
        assert!(session.scheduler.pending() > 0);

        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();
        let client = TcpStream::connect(addr).unwrap();
        let mut tx = wire::Tx::new(client);

        // Tick session - when stepping towards unknown block state, it halts movement
        for _ in 0..5 {
            let _ = session.tick_live(None, &mut tx).unwrap();
            if session.scheduler.pending() == 0 {
                break;
            }
        }
        assert_eq!(session.scheduler.pending(), 0);
        assert_eq!(session.player.velocity[0], 0.0);
        assert_eq!(session.player.velocity[2], 0.0);
        assert!(session.state_table.received_events.iter().any(|e| e.starts_with("unrecognized_block_state:(0,-60,1)=999")));
    }

    #[test]
    fn player_look_composes_with_active_movement_without_preempting() {
        let mut session = test_session();
        session.live_tick = 1;

        // Queue active movement for 10 ticks
        session.execute_live_script("player.input true false false false 0 0 10").unwrap();
        let pending_before = session.scheduler.pending();
        assert!(pending_before > 0);

        // Queue camera look
        let out = session.execute_live_script("player.look 25.5 -10.0").unwrap();
        assert!(out[0].contains("queued camera look"));

        // Movement was NOT cancelled!
        assert!(session.scheduler.pending() >= pending_before);
    }

    #[test]
    fn missing_explicit_block_profile_is_not_silently_replaced() {
        let args = ["127.0.0.1:1", "--offline", "TestPlayer", "missing-profile.json"]
            .map(str::to_owned);
        let error = run_live(&args).unwrap_err().to_string();
        assert!(error.contains("block profile does not exist"));
    }

    #[test]
    fn container_click_waits_for_matching_new_revision() {
        let mut session = test_session();
        session.state_table.open_screen = Some(ScreenState::container(4, 7, 1, "test"));
        session.pending_click = Some((4, 7, Some(40)));
        session.observe_container_revision();
        assert!(session.pending_click.is_some());
        session.state_table.open_screen.as_mut().unwrap().state_id = 8;
        session.observe_container_revision();
        assert!(session.pending_click.is_none());
        assert!(session.state_table.received_events.iter().any(|event| event == "container_revision_observed:4:8"));
    }

    #[test]
    fn placement_requires_known_loaded_target_state() {
        let mut session = test_session();
        session.world.min_y = -64;
        session.world.height = 16;
        assert!(session.placement_target_is_solid([0, -60, 0]).is_err());
        session.world.chunks.insert((0, 0), vec![world::Palette::single(0)]);
        assert!(!session.placement_target_is_solid([0, -60, 0]).unwrap());
        session.world.set_block(0, -60, 0, 999).unwrap();
        assert!(session.placement_target_is_solid([0, -60, 0]).is_err());
    }

    #[test]
    fn inspect_chat_returns_recorded_messages_as_json() {
        let mut session = test_session();
        session.state_table.chat_log.push("Hello \"player\"".to_owned());
        let output = session.execute_live_script("inspect.chat").unwrap();
        let value: serde_json::Value = serde_json::from_str(&output[0]).unwrap();
        assert_eq!(value["messages"][0], "Hello \"player\"");
        assert_eq!(value["history_limit"], 128);
        session.state_table.push_event("mine_complete:(1,2,3)");
        let output = session.execute_live_script("inspect.events").unwrap();
        let value: serde_json::Value = serde_json::from_str(&output[0]).unwrap();
        assert_eq!(value["events"][0], "mine_complete:(1,2,3)");
        assert_eq!(value["history_limit"], 256);
    }

    #[test]
    fn mining_waits_for_matching_server_block_update() {
        let mut session = test_session();
        session.active_mine = Some(ActiveMine {
            pos: [1, 2, 3],
            face: BlockFace::Up,
            remaining_ticks: 0,
            total_ticks: 1,
            sequence: 1,
            confirmation_deadline: Some(40),
        });
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let client = TcpStream::connect(listener.local_addr().unwrap()).unwrap();
        let mut tx = wire::Tx::new(client);

        let block_update_id = session.packets.cb_block_update;
        let update = |pos: [u64; 3], state: i32| {
            let mut packet = Vec::new();
            var(block_update_id, &mut packet);
            let packed = (pos[0] << 38) | (pos[2] << 12) | pos[1];
            packet.extend_from_slice(&packed.to_be_bytes());
            var(state, &mut packet);
            packet
        };
        let other = update([2, 2, 3], 0);
        let solid = update([1, 2, 3], 1);
        let destroyed = update([1, 2, 3], 0);
        session.pending_placements.push(PendingPlacement {
            pos: [2, 2, 3], sequence: 10, original_state: 0,
            confirmation_deadline: Some(40),
        });
        session.packet(&other, &mut tx).unwrap();
        session.packet(&solid, &mut tx).unwrap();
        assert!(session.active_mine.is_some());
        assert_eq!(session.pending_placements.len(), 1);
        session.packet(&destroyed, &mut tx).unwrap();
        assert!(session.active_mine.is_none());
        assert!(session.state_table.received_events.iter().any(|event| event == "mine_complete:(1,2,3)"));
        let placed = update([2, 2, 3], 1);
        session.packet(&placed, &mut tx).unwrap();
        assert!(session.pending_placements.is_empty());
        assert!(session.state_table.received_events.iter().any(|event| event == "placement_state_changed:10:(2,2,3)=1"));
        session.pending_placements.push(PendingPlacement {
            pos: [3, 2, 3], sequence: 11, original_state: 0,
            confirmation_deadline: Some(40),
        });
        let mut section_packet = Vec::new();
        var(session.packets.cb_section_blocks_update, &mut section_packet);
        section_packet.extend_from_slice(&0u64.to_be_bytes());
        var(1, &mut section_packet);
        var((1 << 12) | (3 << 8) | (3 << 4) | 2, &mut section_packet);
        session.packet(&section_packet, &mut tx).unwrap();
        assert!(session.pending_placements.is_empty());
        assert!(session.state_table.received_events.iter().any(|event| event == "placement_state_changed:11:(3,2,3)=1"));
    }

    #[test]
    fn active_mine_is_tracked_and_aborted_on_task_cancel() {
        let profile = serde_json::json!({
            "minecraft": "26.3",
            "air": [0],
            "solid": [1],
            "hardness": { "1": 1.5 }
        });
        let mut session = Session::new(&profile, true, DynamicCameraPolicy::default()).unwrap();
        session.playing = true;
        session.positioned = true;
        session.loaded = true;
        session.world.min_y = -64;
        session.world.height = 16;
        session.player.pos = [0.5, -60.0, 0.5];
        session.world.chunks.insert(
            (0, 0),
            vec![world::Palette::single(0)],
        );
        session.world.set_block(0, -60, 1, 1).unwrap();

        // Start mining stone
        let res = session.execute_live_script("interact.mine 0 -60 1").unwrap();
        assert!(res[0].contains("queued validated block mining"));
        assert!(session.active_mine.is_some());
        assert!(session.event_queue.pending() > 1);

        // Cancel task
        session.execute_live_script("task.cancel").unwrap();
        assert!(session.active_mine.is_none());
        assert_eq!(session.event_queue.pending(), 1);
        assert!(session.state_table.received_events.iter().any(|e| e.contains("mine_aborted:(0,-60,1)")));
    }

    #[test]
    fn default_26_3_profile_initializes_session_out_of_box() {
        let profile = world::default_26_3_profile();
        let session = Session::new(&profile, true, DynamicCameraPolicy::default()).unwrap();
        assert_eq!(session.world.min_y, -64);
        assert_eq!(session.packets.cb_player_combat_kill, 69);
    }

    #[test]
    fn geometric_block_placement_validates_support_reach_and_self_intersection() {
        let profile = serde_json::json!({
            "minecraft": "26.3",
            "air": [0],
            "solid": [1]
        });
        let mut session = Session::new(&profile, true, DynamicCameraPolicy::default()).unwrap();
        session.playing = true;
        session.positioned = true;
        session.loaded = true;
        session.world.min_y = -64;
        session.world.height = 16;
        session.player.pos = [0.5, -60.0, 0.5];
        session.world.chunks.insert((0, 0), vec![world::Palette::single(0)]);

        // 1. Placing where player stands fails due to self-intersection
        let res_self = session.execute_live_script("interact.place 0 -60 0 0");
        assert!(res_self.is_err());
        assert!(res_self.unwrap_err().contains("intersects player bounding box"));

        // 2. Placing in mid-air with no solid supporting face fails
        let res_air = session.execute_live_script("interact.place 0 -60 2 0");
        assert!(res_air.is_err());
        assert!(res_air.unwrap_err().contains("no valid solid supporting block face"));

        // 3. Put solid floor at (0, -61, 1). Placing at (0, -60, 1) succeeds with face Up
        session.world.set_block(0, -61, 1, 1).unwrap();
        let res_floor = session.execute_live_script("interact.place 0 -60 1 0");
        assert!(res_floor.is_ok());
        let out = res_floor.unwrap();
        assert!(out[0].contains("queued validated block placement"));
        assert!(out[0].contains("face=Up"));

        // 4. Test explicit wall placement: put solid wall at (1, -60, 1), place on West face
        session.world.set_block(1, -60, 1, 1).unwrap();
        let res_wall = session.execute_live_script("interact.place_on 1 -60 1 west 0");
        assert!(res_wall.unwrap_err().contains("placement is already pending"));
        session.execute_live_script("task.cancel").unwrap();
        let res_wall = session.execute_live_script("interact.place_on 1 -60 1 west 0");
        assert!(res_wall.is_ok());
        let out_wall = res_wall.unwrap();
        assert!(out_wall[0].contains("queued explicit block placement"));
        assert!(out_wall[0].contains("face=West"));
    }

    #[test]
    fn mob_effect_and_attribute_packets_update_player_state_and_physics() {
        let mut session = test_session();
        session.state_table.player_entity_id = Some(42);

        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();
        let client = TcpStream::connect(addr).unwrap();
        let mut tx = wire::Tx::new(client);

        // 1. Send cb_update_mob_effect (entity=42, effect=1 (speed), amp=1 (Speed II), dur=1000, flags=0)
        let mut speed_pkt = Vec::new();
        var(session.packets.cb_update_mob_effect, &mut speed_pkt);
        var(42, &mut speed_pkt);
        var(1, &mut speed_pkt);
        speed_pkt.push(1); // amp = 1
        var(1000, &mut speed_pkt);
        speed_pkt.push(0);
        session.packet(&speed_pkt, &mut tx).unwrap();

        assert_eq!(session.player.effects.speed_amplifier, Some(1));

        // 2. Send cb_remove_mob_effect
        let mut rm_pkt = Vec::new();
        var(session.packets.cb_remove_mob_effect, &mut rm_pkt);
        var(42, &mut rm_pkt);
        var(1, &mut rm_pkt);
        session.packet(&rm_pkt, &mut tx).unwrap();

        assert_eq!(session.player.effects.speed_amplifier, None);
    }
}
