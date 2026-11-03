#![allow(clippy::collapsible_if)]

mod auth;
mod inventory;
mod player;
mod raycast;
mod wire;
mod world;
use sawbyte_core::{
    Action, Backend, Button, Capability, Direct, Humanised, InputFrame, InputPolicy, Scheduler,
    events::{BlockFace, ClickMode, DiscreteEventQueue, EntityInteraction, Hand, SubTickEvent},
    humanisation::SmoothHumanisedCamera,
    screen::{ItemSelector, ScreenState},
    tape,
    tasks::{
        ComposableTask, WorkflowHost, WorkflowOutcome, execute_workflow, find_path,
        shop_purchase_workflow,
    },
};
use std::{
    collections::HashMap,
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
    var(777, &mut hello);
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

pub struct Session {
    pub world: world::World,
    pub player: player::Player,
    pub state_table: inventory::PlayerStateTable,
    pub packets: inventory::PacketTable,
    pub scheduler: Scheduler<LiveInputBackend, DynamicCameraPolicy>,
    pub event_queue: DiscreteEventQueue,
    pub voxel_patch_buf: Vec<u8>,
    pub custom_modules: Vec<Box<dyn LiveModule>>,
    pub active_nav_path: Vec<[i32; 3]>,
    pub active_nav_goal: Option<([i32; 3], bool)>,
    dimensions: Vec<(i32, i32)>,
    playing: bool,
    positioned: bool,
    loaded: bool,
    live_mode: bool,
    settle: u32,
    index: usize,
    live_tick: u64,
    previous_buttons: u16,
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
    fn new(profile: &serde_json::Value, live_mode: bool, policy: DynamicCameraPolicy) -> Result<Self> {
        Ok(Self {
            world: world::World::new(profile)?,
            player: player::Player::default(),
            state_table: inventory::PlayerStateTable::default(),
            packets: inventory::PacketTable::from_profile(profile),
            scheduler: Scheduler::new(LiveInputBackend::default(), policy, 4096),
            event_queue: DiscreteEventQueue::new(4096),
            voxel_patch_buf: Vec::with_capacity(245),
            custom_modules: Vec::new(),
            active_nav_path: Vec::new(),
            active_nav_goal: None,
            dimensions: Vec::new(),
            playing: false,
            positioned: false,
            loaded: false,
            live_mode,
            settle: 100,
            index: 0,
            live_tick: 0,
            previous_buttons: 0,
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
            69 => return Err(bad("player died; stopping")),
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
                let mut ack = Vec::new();
                var(teleport, &mut ack);
                // Since 26.3 the acknowledgement includes the accepted position and view.
                ack.extend(self.position_payload());
                tx.packet(0, &ack)?;
                self.positioned = true;
                if self.index == 0 && self.live_tick == 0 {
                    self.settle = if self.live_mode { 20 } else { 100 };
                }
            }
            82 => {
                let mut out = r.take(16)?.to_vec();
                var(1, &mut out);
                tx.packet(50, &out)?;
            }
            84 => {
                self.player.velocity = [0.0; 3];
                self.state_table.health = 20.0;
            }
            120 => {
                self.playing = false;
            }
            // Live block updates mutate our loaded chunk section palettes in-place.
            id if id == self.packets.cb_block_update => {
                self.world.apply_block_update_packet(&mut r)?;
            }
            id if id == self.packets.cb_section_blocks_update => {
                self.world.apply_section_blocks_update_packet(&mut r)?;
            }
            // Live container, inventory, entity, vitals, and chat packets
            id if id == self.packets.cb_open_screen => {
                if let Err(e) = self.state_table.apply_open_screen(&mut r) {
                    self.state_table.record_decode_error("open_screen", e.as_ref());
                }
            }
            id if id == self.packets.cb_container_close => {
                if let Err(e) = self.state_table.apply_container_close(&mut r) {
                    self.state_table
                        .record_decode_error("container_close", e.as_ref());
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
                }
            }
            id if id == self.packets.cb_container_set_content => {
                if let Err(e) = self
                    .state_table
                    .apply_container_set_content(&mut r, &self.packets)
                {
                    self.state_table
                        .record_decode_error("container_set_content", e.as_ref());
                }
            }
            id if id == self.packets.cb_set_health => {
                match self.state_table.apply_set_health(&mut r) {
                    Ok(()) => {
                        self.player.food_level = self.state_table.food;
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
            id if id == self.packets.cb_system_chat => {
                if let Err(e) = self.state_table.apply_system_chat(&mut r) {
                    self.state_table.record_decode_error("system_chat", e.as_ref());
                }
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
            frames[self.index]
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
    /// Steers toward each waypoint using observed player position and yaw, stopping only on arrival.
    fn step_live_navigation(&mut self) {
        let Some((goal, sprint)) = self.active_nav_goal else {
            return;
        };
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
        self.step_live_navigation();
        let frame = self
            .scheduler
            .advance(self.live_tick)
            .map_err(|e| bad(&format!("scheduler error: {e}")))?;
        self.player.step(frame, &self.world)?;

        // Drain and transmit all discrete sub-tick events scheduled for this tick
        let drained = self
            .event_queue
            .drain_tick(self.live_tick)
            .map_err(|e| bad(&format!("event queue error: {e}")))?;
        for ev in drained {
            if let SubTickEvent::SelectHotbar { slot } = &ev {
                self.state_table.selected_hotbar = *slot;
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
        Ok(true)
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
            let parts: Vec<&str> = line.split_whitespace().collect();
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
                ["task.navigate", x, y, z] | ["task.navigate", x, y, z, _] => {
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
                    let path = find_path(&self.world, start, goal, 2048)?;
                    let waypoints = path.len();
                    let _ = self.scheduler.cancel();
                    self.active_nav_path = if path.len() > 1 {
                        path[1..].to_vec()
                    } else {
                        path
                    };
                    self.active_nav_goal = Some((goal, sprint));
                    outputs.push(format!(
                        "live 3D A* path active ({waypoints} waypoints) from ({}, {}, {}) -> ({}, {}, {})",
                        start[0], start[1], start[2], goal[0], goal[1], goal[2]
                    ));
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
                    let max_reach = self.player.attributes.block_interaction_range;
                    if dist > max_reach {
                        return Err(format!(
                            "block ({}, {}, {}) out of player reach ({dist:.2} > {max_reach:.2})",
                            pos[0], pos[1], pos[2]
                        ));
                    }
                    let face = raycast::raycast_block(&self.player, &self.world)
                        .filter(|h| h.block_pos == pos)
                        .map(|h| h.face)
                        .ok_or_else(|| {
                            format!(
                                "block ({}, {}, {}) is not currently in player line of sight",
                                pos[0], pos[1], pos[2]
                            )
                        })?;
                    let duration_ticks = self
                        .player
                        .compute_break_ticks(1.5, 1.0, true, false)
                        .unwrap_or(6);
                    let seq = self.state_table.next_sequence_n(2);
                    let events = ComposableTask::MineBlock {
                        pos,
                        face,
                        duration_ticks,
                    }
                    .compile_events(next_tick, seq);
                    self.event_queue
                        .submit(&events)
                        .map_err(|e| format!("{e:?}"))?;
                    outputs.push(format!(
                        "queued validated block mining at ({}, {}, {}) face={face:?} duration_ticks={duration_ticks}",
                        pos[0], pos[1], pos[2]
                    ));
                }
                ["interact.mine_look"] => {
                    let hit = raycast::raycast_block(&self.player, &self.world)
                        .ok_or_else(|| "no block in line of sight within reach".to_owned())?;
                    let duration_ticks = self
                        .player
                        .compute_break_ticks(1.5, 1.0, true, false)
                        .unwrap_or(6);
                    let seq = self.state_table.next_sequence_n(2);
                    let events = ComposableTask::MineBlock {
                        pos: hit.block_pos,
                        face: hit.face,
                        duration_ticks,
                    }
                    .compile_events(next_tick, seq);
                    self.event_queue
                        .submit(&events)
                        .map_err(|e| format!("{e:?}"))?;
                    outputs.push(format!(
                        "queued raycast block mining at ({}, {}, {}) face={:?} duration_ticks={duration_ticks}",
                        hit.block_pos[0], hit.block_pos[1], hit.block_pos[2], hit.face
                    ));
                }
                ["interact.place", x, y, z, slot] => {
                    let pos = [
                        x.parse::<i32>().map_err(|_| "bad x")?,
                        y.parse::<i32>().map_err(|_| "bad y")?,
                        z.parse::<i32>().map_err(|_| "bad z")?,
                    ];
                    let s: u8 = slot.parse().map_err(|_| "bad slot")?;
                    if s >= 9 {
                        return Err("hotbar slot must be in 0..=8".into());
                    }
                    let support_pos = [pos[0], pos[1] - 1, pos[2]];
                    let eye = self.player.eye_position();
                    let dist = ((support_pos[0] as f64 + 0.5 - eye[0]).powi(2)
                        + (support_pos[1] as f64 + 0.5 - eye[1]).powi(2)
                        + (support_pos[2] as f64 + 0.5 - eye[2]).powi(2))
                    .sqrt();
                    let max_reach = self.player.attributes.block_interaction_range;
                    if dist > max_reach {
                        return Err(format!(
                            "placement support ({}, {}, {}) out of player reach ({dist:.2} > {max_reach:.2})",
                            support_pos[0], support_pos[1], support_pos[2]
                        ));
                    }
                    let seq = self.state_table.next_sequence();
                    let events = ComposableTask::PlaceBlock {
                        support_pos,
                        face: BlockFace::Up,
                        hotbar_slot: s,
                    }
                    .compile_events(next_tick, seq);
                    self.event_queue
                        .submit(&events)
                        .map_err(|e| format!("{e:?}"))?;
                    outputs.push(format!(
                        "queued validated block placement at ({}, {}, {}) slot={s}",
                        pos[0], pos[1], pos[2]
                    ));
                }
                ["interact.attack", entity_id] => {
                    let id: i32 = entity_id.parse().map_err(|_| "bad entity_id")?;
                    let ent = self
                        .state_table
                        .entities
                        .get(&id)
                        .ok_or_else(|| format!("entity {id} is not tracked in live session"))?;
                    let eye = self.player.eye_position();
                    let dist = ((ent.pos[0] - eye[0]).powi(2)
                        + (ent.pos[1] + ent.height * 0.5 - eye[1]).powi(2)
                        + (ent.pos[2] - eye[2]).powi(2))
                    .sqrt();
                    let max_reach = self.player.attributes.entity_interaction_range + 0.5;
                    if dist > max_reach {
                        return Err(format!(
                            "entity {id} out of attack reach ({dist:.2} > {max_reach:.2})"
                        ));
                    }
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
                ["player.select_hotbar", slot] => {
                    let s: u8 = slot.parse().map_err(|_| "invalid hotbar slot")?;
                    self.event_queue
                        .submit(&[(next_tick, SubTickEvent::SelectHotbar { slot: s })])
                        .map_err(|e| format!("{e:?}"))?;
                    outputs.push(format!("queued live hotbar selection slot={s}"));
                }
                ["player.input", fwd, jump, sneak, sprint, yaw_d, pitch_d, ticks] => {
                    let f_down = *fwd == "true" || *fwd == "1";
                    let j_down = *jump == "true" || *jump == "1";
                    let sn_down = *sneak == "true" || *sneak == "1";
                    let sp_down = *sprint == "true" || *sprint == "1";
                    let yd: f64 = yaw_d.parse().map_err(|_| "bad yaw_delta")?;
                    let pd: f64 = pitch_d.parse().map_err(|_| "bad pitch_delta")?;
                    let dur: u64 = ticks.parse::<u64>().map_err(|_| "bad ticks")?.clamp(1, 200);
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
                        let populated = s.slots.iter().filter(|it| it.is_some()).count();
                        outputs.push(format!(
                            "open_screen window_id={} state_id={} kind={:?} title=\"{}\" slots={}/{} cursor={} widgets={} fields={}",
                            s.window_id,
                            s.state_id,
                            s.kind,
                            s.title,
                            populated,
                            s.slots.len(),
                            s.cursor_item.as_ref().map_or("none", |c| c.id.as_str()),
                            s.widgets.len(),
                            s.fields.len()
                        ));
                    } else {
                        outputs.push("open_screen=none".into());
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
                ["gui.click_slot", slot] | ["gui.click_slot", slot, _] => {
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
                    self.event_queue
                        .submit(&[(next_tick, ev)])
                        .map_err(|e| format!("{e:?}"))?;
                    outputs.push(format!("queued live container click slot={s_idx}"));
                }
                ["gui.click_item", query] => {
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
                    self.event_queue
                        .submit(&[(next_tick, ev)])
                        .map_err(|e| format!("{e:?}"))?;
                    outputs.push(format!("queued live item click for '{query}'"));
                }
                ["gui.close"] => {
                    let screen = self
                        .state_table
                        .open_screen
                        .take()
                        .ok_or_else(|| "no open container screen on server".to_owned())?;
                    let wid = screen.window_id;
                    self.event_queue
                        .submit(&[(next_tick, SubTickEvent::CloseContainer { window_id: wid })])
                        .map_err(|e| format!("{e:?}"))?;
                    outputs.push(format!("closed container window_id={wid}"));
                }
                ["gui.run_shop_workflow", item_id, amount] => {
                    if self.state_table.open_screen.is_none() {
                        return Err(
                            "cannot run screen workflow: no container screen is open on the live server"
                                .into(),
                        );
                    }
                    let wf = shop_purchase_workflow(item_id, amount);
                    match execute_workflow(&wf, self) {
                        WorkflowOutcome::Completed {
                            emitted_events,
                            final_revision,
                            ..
                        } => outputs.push(format!(
                            "live screen workflow completed ({} serverbound events queued, final_rev={final_revision})",
                            emitted_events.len()
                        )),
                        other => {
                            return Err(format!("live screen workflow did not complete: {other:?}"))
                        }
                    }
                }
                ["inventory.inspect"] => {
                    let populated: Vec<String> = self
                        .state_table
                        .inventory_slots
                        .iter()
                        .enumerate()
                        .filter_map(|(idx, it)| {
                            it.as_ref().map(|s| format!("slot={idx}:{}x{}", s.count, s.id))
                        })
                        .collect();
                    outputs.push(format!(
                        "selected_hotbar={} populated_slots={} items=[{}]",
                        self.state_table.selected_hotbar,
                        populated.len(),
                        populated.join(", ")
                    ));
                }
                ["entities.inspect"] => {
                    let ents: Vec<String> = self
                        .state_table
                        .entities
                        .values()
                        .map(|e| {
                            format!(
                                "id={} type={} pos=({:.2},{:.2},{:.2})",
                                e.entity_id, e.entity_type, e.pos[0], e.pos[1], e.pos[2]
                            )
                        })
                        .collect();
                    outputs.push(format!(
                        "tracked_entities={} list=[{}]",
                        ents.len(),
                        ents.join(", ")
                    ));
                }
                ["chat.send", rest @ ..] if !rest.is_empty() => {
                    let msg = rest.join(" ");
                    let ev = SubTickEvent::SendChat {
                        message: msg.clone(),
                    };
                    self.event_queue
                        .submit(&[(next_tick, ev)])
                        .map_err(|e| format!("{e:?}"))?;
                    outputs.push(format!("queued live chat: {msg}"));
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
                    let obs = serde_json::json!({
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
                    });
                    outputs.push(obs.to_string());
                }
                ["gym.step", fwd, back, left, right, jump, sneak, sprint, yaw_d, pitch_d] => {
                    let b_on = |s: &str| s == "1" || s == "true";
                    let yd: f64 = yaw_d.parse().map_err(|_| "bad yaw_delta")?;
                    let pd: f64 = pitch_d.parse().map_err(|_| "bad pitch_delta")?;
                    self.scheduler
                        .submit(&[
                            (
                                next_tick,
                                Action::Button {
                                    button: Button::Forward,
                                    down: b_on(fwd),
                                },
                            ),
                            (
                                next_tick,
                                Action::Button {
                                    button: Button::Back,
                                    down: b_on(back),
                                },
                            ),
                            (
                                next_tick,
                                Action::Button {
                                    button: Button::Left,
                                    down: b_on(left),
                                },
                            ),
                            (
                                next_tick,
                                Action::Button {
                                    button: Button::Right,
                                    down: b_on(right),
                                },
                            ),
                            (
                                next_tick,
                                Action::Button {
                                    button: Button::Jump,
                                    down: b_on(jump),
                                },
                            ),
                            (
                                next_tick,
                                Action::Button {
                                    button: Button::Sneak,
                                    down: b_on(sneak),
                                },
                            ),
                            (
                                next_tick,
                                Action::Button {
                                    button: Button::Sprint,
                                    down: b_on(sprint),
                                },
                            ),
                            (
                                next_tick,
                                Action::Look {
                                    yaw: yd,
                                    pitch: pd,
                                },
                            ),
                        ])
                        .map_err(|e| format!("scheduler error: {e}"))?;
                    outputs.push(format!("gym.step scheduled for tick {next_tick}"));
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
                        "serverInfo": { "name": "sawbyte-play-live-mcp", "version": "0.2.0" },
                        "capabilities": { "tools": { "listChanged": false }, "resources": { "listChanged": false } }
                    }
                })
                .to_string(),
                "notifications/initialized" => String::new(),
                "ping" => serde_json::json!({ "jsonrpc": "2.0", "id": id, "result": {} }).to_string(),
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
}

fn run_live(args: &[String]) -> Result<()> {
    if args.len() < 4 {
        return Err(bad(
            "usage: sawbyte-play live <server:port> <--offline|--account> <user> <blocks.json> [--listen <ip:port>] [--mcp-stdio] [--script <script>] [--ticks <n>] [--trace <trace.csv>] [--policy <humanised|smooth|direct>]",
        ));
    }
    let server = &args[0];
    let account = match args[1].as_str() {
        "--offline" => auth::Account::offline(&args[2])?,
        "--account" => auth::Account::token_file(std::path::Path::new(&args[2]))?,
        _ => return Err(bad("choose --offline or --account")),
    };
    let profile: serde_json::Value = serde_json::from_reader(File::open(&args[3])?)?;
    if profile["minecraft"] != "26.3" {
        return Err(bad("wrong block profile version"));
    }

    let mut listen_addr: Option<String> = None;
    let mut mcp_stdio = false;
    let mut initial_script: Option<String> = None;
    let mut max_ticks: Option<u64> = None;
    let mut trace_path: Option<String> = None;
    let mut policy = DynamicCameraPolicy::default();

    let mut idx = 4;
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

    // Optional control channels (stdio or TCP listener)
    let (ctrl_tx, ctrl_rx) =
        mpsc::channel::<(String, Option<mpsc::Sender<String>>)>();
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
                    let reader = BufReader::new(stream);
                    for line in reader.lines().map_while(std::result::Result::ok) {
                        let is_json = line.trim_start().starts_with('{');
                        let (reply_tx, reply_rx) = mpsc::channel();
                        if sender_conn.send((line, Some(reply_tx))).is_err() {
                            break;
                        }
                        if let Ok(resp) = reply_rx.recv_timeout(Duration::from_secs(5)) {
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

    let mut trace_writer = match trace_path {
        Some(ref p) => {
            let mut w = BufWriter::new(File::create(p)?);
            writeln!(w, "tick,x,y,z,yaw,pitch,on_ground,chunks")?;
            Some(w)
        }
        None => None,
    };

    let mut script_dispatched = false;
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
                // Once settled on the live server, execute `--script` or incoming MCP/TCP commands
                if state.loaded && state.settle == 0 {
                    if !script_dispatched {
                        script_dispatched = true;
                        if let Some(ref script) = initial_script {
                            match state.execute_live_script(script) {
                                Ok(lines) => {
                                    for l in lines {
                                        println!("{l}");
                                    }
                                }
                                Err(e) => return Err(bad(&format!("live script error: {e}"))),
                            }
                        }
                    }
                    while let Ok((req_line, reply_chan)) = ctrl_rx.try_recv() {
                        let resp = state.handle_control_request(&req_line);
                        if let Some(reply) = reply_chan {
                            let _ = reply.send(resp);
                        } else if !resp.is_empty() {
                            println!("{resp}");
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

fn run() -> Result<()> {
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
            Zero-graphics 20 TPS client with 26.3 chunk palettes, dynamic block updates, slabs/stairs/climbables/fluids,\n\
            sprinting/sneaking/stepping, live 3D A* pathfinding, eye raycasting, container screens, and RL/MCP control."
        );
        return Ok(());
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
                    return Err(bad("tick budget exceeded"));
                }
            }
        }
    })();
    let _ = shutdown.shutdown(Shutdown::Both);
    drop(receive);
    let _ = reader.join();
    result
}
fn main() -> std::process::ExitCode {
    match run() {
        Ok(()) => std::process::ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("sawbyte-play: {error}");
            std::process::ExitCode::FAILURE
        }
    }
}
