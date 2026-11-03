mod auth;
mod player;
mod wire;
mod world;
use sawbyte_core::{InputFrame, tape};
use std::{
    fs::File,
    io::{BufWriter, Write},
    net::{Shutdown, TcpStream, ToSocketAddrs},
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

struct Session {
    world: world::World,
    player: player::Player,
    dimensions: Vec<(i32, i32)>,
    playing: bool,
    positioned: bool,
    loaded: bool,
    settle: u32,
    index: usize,
    previous_buttons: u16,
}
impl Session {
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
                r.i32()?;
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
                if r.byte()? != 0 {
                    return Err(bad("prototype currently supports survival mode only"));
                }
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
                if self.index > 0 {
                    return Err(bad("server corrected position during replay; comparison aborted"));
                }
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
                if flags & 256 != 0 {
                    return Err(bad("rotated velocity correction unsupported"));
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
                            self.player.velocity[axis]
                        } else {
                            0.
                        };
                }
                self.player.yaw = yaw + if flags & 8 != 0 { self.player.yaw } else { 0. };
                self.player.pitch = pitch
                    + if flags & 16 != 0 {
                        self.player.pitch
                    } else {
                        0.
                    };
                let mut ack = Vec::new();
                var(teleport, &mut ack);
                // Since 26.3 the acknowledgement includes the accepted position and view.
                ack.extend(self.position_payload());
                tx.packet(0, &ack)?;
                self.positioned = true;
                self.send_position(tx)?;
            }
            82 => {
                let mut out = r.take(16)?.to_vec();
                var(1, &mut out);
                tx.packet(50, &out)?;
            }
            84 | 120 => {
                return Err(bad(
                    "dimension/session transition unsupported in this prototype",
                ));
            }
            // Changes to collision data invalidate our snapshot. Never walk through stale geometry.
            8 | 86 => {
                if self.loaded {
                    return Err(bad(
                        "world changed during replay; stopping until block-update support is implemented",
                    ));
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
}

fn run() -> Result<()> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    if args.is_empty() || args == ["--help"] {
        println!(
            "Sawbyte standalone player prototype — Java 26.3\n\n\
            sawbyte-play <server:port> <--offline|--account> <name-or-token-file> <input.sbif> <trace.csv> <blocks.json>\n\
            sawbyte-play login <registered-microsoft-client-id> <new-token-file>\n\n\
            Current movement scope: ordinary full-cube ground, walk/turn/jump/stop.\n\
            Unknown collision mechanics stop execution. Not a general playable client yet."
        );
        return Ok(());
    }
    if args.len() == 3 && args[0] == "login" {
        return auth::login(&args[1], std::path::Path::new(&args[2]));
    }
    if args.len() != 6 {
        return Err(bad("invalid arguments; use --help"));
    }
    let frames = tape::read(&mut File::open(&args[3])?).map_err(|e| bad(&e.to_string()))?;
    if frames.iter().any(|f| f.buttons.bits() & !31 != 0)
        || frames.last() != Some(&InputFrame::default())
    {
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
    let mut state = Session {
        world: world::World::new(&profile)?,
        player: player::Player::default(),
        dimensions: Vec::new(),
        playing: false,
        positioned: false,
        loaded: false,
        settle: 100,
        index: 0,
        previous_buttons: 0,
    };
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
