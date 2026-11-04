//! Typed input scheduling shared by future native and graphical backends.
//! No networking, serialization, sleeping, rendering, or game physics lives here.

use std::collections::VecDeque;
use std::fmt;

pub mod events;
pub mod humanisation;
pub mod json;
pub mod launcher;
pub mod mcp;
pub mod screen;
pub mod tape;
pub mod tasks;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Button {
    Forward,
    Back,
    Left,
    Right,
    Jump,
    Sneak,
    Sprint,
    Attack,
    Use,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Buttons(u16);

impl Buttons {
    pub fn bits(self) -> u16 {
        self.0
    }

    pub fn from_bits(bits: u16) -> Option<Self> {
        (bits & !0x01ff == 0).then_some(Self(bits))
    }

    pub fn pressed(self, button: Button) -> bool {
        self.0 & (1 << button as u8) != 0
    }

    fn set(&mut self, button: Button, down: bool) {
        let mask = 1 << button as u8;
        if down {
            self.0 |= mask;
        } else {
            self.0 &= !mask;
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Capability {
    Buttons,
    Camera,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Action {
    Button {
        button: Button,
        down: bool,
    },
    /// Relative degrees; adapters must preserve the selected version's input semantics.
    Look {
        yaw: f64,
        pitch: f64,
    },
}

impl Action {
    fn capability(self) -> Capability {
        match self {
            Self::Button { .. } => Capability::Buttons,
            Self::Look { .. } => Capability::Camera,
        }
    }

    fn valid(self) -> bool {
        match self {
            Self::Button { .. } => true,
            Self::Look { yaw, pitch } => {
                yaw.is_finite() && pitch.is_finite() && yaw.abs() <= 360.0 && pitch.abs() <= 180.0
            }
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct InputFrame {
    pub buttons: Buttons,
    pub yaw_delta: f64,
    pub pitch_delta: f64,
}

/// A failed apply may have partially affected the backend. `release_all` must
/// release controls independently of normal scheduling (or report failure).
pub trait Backend {
    type Error: fmt::Display;
    fn supports(&self, capability: Capability) -> bool;
    fn apply(&mut self, tick: u64, frame: InputFrame) -> Result<(), Self::Error>;
    fn release_all(&mut self) -> Result<(), Self::Error>;
}

/// First humanisation primitive: bounded camera motion. This is not a model of
/// human behaviour or a claim of indistinguishability. More policies can implement this trait.
pub trait InputPolicy {
    fn camera_step(&mut self, remaining: (f64, f64)) -> (f64, f64);
}

#[derive(Clone, Copy, Debug)]
pub struct Humanised {
    max_degrees_per_tick: f64,
}

impl Default for Humanised {
    fn default() -> Self {
        Self {
            max_degrees_per_tick: 12.0,
        }
    }
}

impl Humanised {
    pub fn new(max_degrees_per_tick: f64) -> Option<Self> {
        (max_degrees_per_tick.is_finite() && max_degrees_per_tick > 0.0).then_some(Self {
            max_degrees_per_tick,
        })
    }
}

impl InputPolicy for Humanised {
    fn camera_step(&mut self, (yaw, pitch): (f64, f64)) -> (f64, f64) {
        let length = yaw.hypot(pitch);
        let scale = if length > self.max_degrees_per_tick {
            self.max_degrees_per_tick / length
        } else {
            1.0
        };
        (yaw * scale, pitch * scale)
    }
}

#[derive(Default)]
pub struct Direct;

impl InputPolicy for Direct {
    fn camera_step(&mut self, remaining: (f64, f64)) -> (f64, f64) {
        remaining
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum Error {
    QueueFull,
    InvalidAction,
    Unsupported(Capability),
    OutOfOrder,
    Halted,
    InvalidPolicy,
    Backend {
        cause: String,
        release_error: Option<String>,
    },
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}

impl std::error::Error for Error {}

/// Single-owner scheduler. Queue length and work per tick are bounded by capacity.
/// Ticks are supplied by the backend clock, never by sleeps inside the scheduler.
pub struct Scheduler<B: Backend, P: InputPolicy = Humanised> {
    backend: B,
    policy: P,
    queue: VecDeque<(u64, Action)>,
    capacity: usize,
    last_tick: Option<u64>,
    buttons: Buttons,
    remaining: (f64, f64),
    halted: bool,
}

impl<B: Backend, P: InputPolicy> Scheduler<B, P> {
    pub fn new(backend: B, policy: P, capacity: usize) -> Self {
        Self {
            backend,
            policy,
            queue: VecDeque::new(),
            capacity,
            last_tick: None,
            buttons: Buttons::default(),
            remaining: (0.0, 0.0),
            halted: false,
        }
    }

    pub fn pending(&self) -> usize {
        self.queue.len()
    }

    /// Batch admission is atomic: a rejected sequence enqueues nothing.
    pub fn submit(&mut self, actions: &[(u64, Action)]) -> Result<(), Error> {
        if self.halted {
            return Err(Error::Halted);
        }
        if actions.len() > self.capacity.saturating_sub(self.queue.len()) {
            return Err(Error::QueueFull);
        }
        let mut previous = None;
        for &(tick, action) in actions {
            if self.last_tick.is_some_and(|last| tick <= last)
                || previous.is_some_and(|prev| tick < prev)
            {
                return Err(Error::OutOfOrder);
            }
            if !action.valid() {
                return Err(Error::InvalidAction);
            }
            if !self.backend.supports(action.capability()) {
                return Err(Error::Unsupported(action.capability()));
            }
            previous = Some(tick);
        }
        self.queue.extend(actions.iter().copied());
        self.queue.make_contiguous().sort_by_key(|(tick, _)| *tick);
        Ok(())
    }

    fn clear(&mut self) {
        self.queue.clear();
        self.buttons = Buttons::default();
        self.remaining = (0.0, 0.0);
    }

    /// Used for cancellation and manual handoff. Does not reset the monotonic clock.
    pub fn cancel(&mut self) -> Result<(), Error> {
        self.clear();
        match self.backend.release_all() {
            Ok(()) => {
                self.halted = false;
                Ok(())
            }
            Err(error) => {
                self.halted = true;
                Err(Error::Backend {
                    cause: error.to_string(),
                    release_error: None,
                })
            }
        }
    }

    pub fn advance(&mut self, tick: u64) -> Result<InputFrame, Error> {
        if self.halted {
            return Err(Error::Halted);
        }
        if self.last_tick.is_some_and(|last| tick <= last) {
            return Err(Error::OutOfOrder);
        }
        // Skipping an input tick can erase a press/release pair. Require sequential
        // ticks once running; the host must choose explicit catch-up or cancellation.
        if self
            .last_tick
            .is_some_and(|last| last.checked_add(1) != Some(tick))
        {
            return Err(Error::OutOfOrder);
        }
        if self.last_tick.is_none() && self.queue.front().is_some_and(|(at, _)| *at < tick) {
            return Err(Error::OutOfOrder);
        }
        self.last_tick = Some(tick);
        while self.queue.front().is_some_and(|(at, _)| *at <= tick) {
            let (_, action) = self.queue.pop_front().expect("front was checked");
            match action {
                Action::Button { button, down } => self.buttons.set(button, down),
                // New camera intent replaces the previous unfinished target.
                Action::Look { yaw, pitch } => self.remaining = (yaw, pitch),
            }
        }
        let step = self.policy.camera_step(self.remaining);
        let valid_axis = |value: f64, target: f64| {
            value.is_finite()
                && value.abs() <= target.abs()
                && (value == 0.0 || value.signum() == target.signum())
        };
        if !valid_axis(step.0, self.remaining.0) || !valid_axis(step.1, self.remaining.1) {
            self.clear();
            self.halted = true;
            return match self.backend.release_all() {
                Ok(()) => Err(Error::InvalidPolicy),
                Err(error) => Err(Error::Backend {
                    cause: "invalid policy output".into(),
                    release_error: Some(error.to_string()),
                }),
            };
        }
        self.remaining.0 -= step.0;
        self.remaining.1 -= step.1;
        let frame = InputFrame {
            buttons: self.buttons,
            yaw_delta: step.0,
            pitch_delta: step.1,
        };
        if let Err(error) = self.backend.apply(tick, frame) {
            self.clear();
            self.halted = true;
            let release_error = self.backend.release_all().err().map(|e| e.to_string());
            return Err(Error::Backend {
                cause: error.to_string(),
                release_error,
            });
        }
        Ok(frame)
    }
}

impl<B: Backend, P: InputPolicy> Drop for Scheduler<B, P> {
    fn drop(&mut self) {
        // Best effort only. Hosts must call cancel explicitly to observe errors.
        let _ = self.backend.release_all();
    }
}

/// Tokenizes a raw command line into string arguments, supporting whitespace separation,
/// double quotes ("..."), single quotes ('...'), backslash escaping (\", \', \\, \n, \t),
/// and structured JSON commands (e.g. {"command": "task.navigate", "args": [3, -60, 2]}).
pub fn tokenize_command_line(line: &str) -> Result<Vec<String>, String> {
    let trimmed = line.trim();
    if trimmed.is_empty() {
        return Ok(Vec::new());
    }

    if trimmed.starts_with('{') {
        if let Ok(val) = json::JsonValue::parse(trimmed) {
            if let Some(cmd) = val.get("command").and_then(|v| v.as_str()) {
                let mut tokens = vec![cmd.to_string()];
                if let Some(args) = val.get("args").and_then(|v| v.as_array()) {
                    for arg in args {
                        tokens.push(arg.to_string_lossy());
                    }
                }
                return Ok(tokens);
            }
        }
    }

    let mut tokens = Vec::new();
    let mut current = String::new();
    let mut in_quote: Option<char> = None;
    let mut token_has_content = false;
    let mut chars = trimmed.chars().peekable();

    while let Some(c) = chars.next() {
        if let Some(q) = in_quote {
            if c == q {
                in_quote = None;
            } else if c == '\\' {
                if let Some(next_c) = chars.next() {
                    match next_c {
                        'n' => current.push('\n'),
                        'r' => current.push('\r'),
                        't' => current.push('\t'),
                        '\\' => current.push('\\'),
                        '"' => current.push('"'),
                        '\'' => current.push('\''),
                        other => {
                            current.push('\\');
                            current.push(other);
                        }
                    }
                } else {
                    current.push('\\');
                }
            } else {
                current.push(c);
            }
        } else if c == '"' || c == '\'' {
            in_quote = Some(c);
            token_has_content = true;
        } else if c.is_whitespace() {
            if token_has_content {
                tokens.push(std::mem::take(&mut current));
                token_has_content = false;
            }
        } else if c == '\\' {
            token_has_content = true;
            if let Some(next_c) = chars.next() {
                match next_c {
                    ' ' => current.push(' '),
                    'n' => current.push('\n'),
                    'r' => current.push('\r'),
                    't' => current.push('\t'),
                    '\\' => current.push('\\'),
                    '"' => current.push('"'),
                    '\'' => current.push('\''),
                    other => {
                        current.push('\\');
                        current.push(other);
                    }
                }
            } else {
                current.push('\\');
            }
        } else {
            token_has_content = true;
            current.push(c);
        }
    }

    if in_quote.is_some() {
        return Err("unterminated quote in command line".to_owned());
    }

    if token_has_content {
        tokens.push(current);
    }

    Ok(tokens)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tokenize_quoted_and_structured_commands() {
        assert_eq!(
            tokenize_command_line("task.navigate 3 -60 2").unwrap(),
            vec!["task.navigate", "3", "-60", "2"]
        );
        assert_eq!(
            tokenize_command_line("chat.send \"hello world\"").unwrap(),
            vec!["chat.send", "hello world"]
        );
        assert_eq!(
            tokenize_command_line("wait.until screen_open 40 \"Large Chest\"").unwrap(),
            vec!["wait.until", "screen_open", "40", "Large Chest"]
        );
        assert_eq!(
            tokenize_command_line("chat.send 'foo \\'bar\\' baz'").unwrap(),
            vec!["chat.send", "foo 'bar' baz"]
        );
        assert_eq!(
            tokenize_command_line("\"/home/fivesaw/The Projects/Sawbyte/block-profile.json\"").unwrap(),
            vec!["/home/fivesaw/The Projects/Sawbyte/block-profile.json"]
        );
        assert_eq!(
            tokenize_command_line(r#"{"command": "task.navigate", "args": [3, -60, 2]}"#).unwrap(),
            vec!["task.navigate", "3", "-60", "2"]
        );
        assert!(tokenize_command_line("chat.send \"unterminated").is_err());
    }
}

