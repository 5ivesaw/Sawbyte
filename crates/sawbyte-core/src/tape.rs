//! Versioned, bounded input fixture shared with the graphical reference bridge.
//! This is a test/replay interchange format, not a commitment to an external API.
use crate::{Action, Backend, Button, Buttons, Capability, InputFrame, InputPolicy, Scheduler};
use std::io::{self, Read, Write};

pub const MAGIC: &[u8; 8] = b"SBIF0001";
pub const MAX_FRAMES: usize = 12_000;

fn invalid() -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, "invalid Sawbyte input tape")
}

fn valid(frame: &InputFrame) -> bool {
    frame.yaw_delta.is_finite() && frame.pitch_delta.is_finite()
        && frame.yaw_delta.abs() <= 360.0 && frame.pitch_delta.abs() <= 180.0
}

pub fn write(writer: &mut impl Write, frames: &[InputFrame]) -> io::Result<()> {
    if frames.is_empty() || frames.len() > MAX_FRAMES || !frames.iter().all(valid) {
        return Err(invalid());
    }
    writer.write_all(MAGIC)?;
    writer.write_all(&(frames.len() as u32).to_le_bytes())?;
    for frame in frames {
        writer.write_all(&frame.buttons.bits().to_le_bytes())?;
        writer.write_all(&frame.yaw_delta.to_le_bytes())?;
        writer.write_all(&frame.pitch_delta.to_le_bytes())?;
    }
    Ok(())
}

pub fn read(reader: &mut impl Read) -> io::Result<Vec<InputFrame>> {
    let mut magic = [0; 8];
    reader.read_exact(&mut magic)?;
    let mut count = [0; 4];
    reader.read_exact(&mut count)?;
    let count = u32::from_le_bytes(count) as usize;
    if &magic != MAGIC || count == 0 || count > MAX_FRAMES { return Err(invalid()); }
    let mut frames = Vec::with_capacity(count);
    for _ in 0..count {
        let mut bits = [0; 2];
        let mut yaw = [0; 8];
        let mut pitch = [0; 8];
        reader.read_exact(&mut bits)?;
        reader.read_exact(&mut yaw)?;
        reader.read_exact(&mut pitch)?;
        let frame = InputFrame {
            buttons: Buttons::from_bits(u16::from_le_bytes(bits)).ok_or_else(invalid)?,
            yaw_delta: f64::from_le_bytes(yaw),
            pitch_delta: f64::from_le_bytes(pitch),
        };
        if !valid(&frame) { return Err(invalid()); }
        frames.push(frame);
    }
    let mut trailing = [0];
    if reader.read(&mut trailing)? != 0 { return Err(invalid()); }
    Ok(frames)
}

struct Capture;
impl Backend for Capture {
    type Error = &'static str;
    fn supports(&self, _: Capability) -> bool { true }
    fn apply(&mut self, _: u64, _: InputFrame) -> Result<(), Self::Error> { Ok(()) }
    fn release_all(&mut self) -> Result<(), Self::Error> { Ok(()) }
}

/// Stationary, walk, turn, jump, release, settle. Both backends consume identical frames.
pub fn movement_fixture(policy: impl InputPolicy) -> Vec<InputFrame> {
    let mut scheduler = Scheduler::new(Capture, policy, 8);
    scheduler.submit(&[
        (20, Action::Button { button: Button::Forward, down: true }),
        (60, Action::Look { yaw: 90.0, pitch: 0.0 }),
        (80, Action::Button { button: Button::Jump, down: true }),
        (81, Action::Button { button: Button::Jump, down: false }),
        (100, Action::Button { button: Button::Forward, down: false }),
    ]).expect("fixed valid fixture");
    (0..160).map(|tick| scheduler.advance(tick).expect("capture cannot fail")).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Humanised;

    #[test]
    fn fixture_survives_interchange_without_changing_inputs() {
        let frames = movement_fixture(Humanised::default());
        let mut bytes = Vec::new();
        write(&mut bytes, &frames).unwrap();
        assert_eq!(bytes.len(), 12 + 160 * 18);
        assert_eq!(read(&mut bytes.as_slice()).unwrap(), frames);
        assert!(frames[80].buttons.pressed(Button::Jump));
        assert!(!frames[81].buttons.pressed(Button::Jump));
        assert_eq!(frames[159], InputFrame::default());
    }

    #[test]
    fn rejects_unknown_bits_excessive_count_trailing_and_truncated_data() {
        let mut bytes = Vec::new();
        write(&mut bytes, &[InputFrame::default()]).unwrap();
        for broken in [bytes[..20].to_vec(), [bytes.as_slice(), &[1]].concat()] {
            assert!(read(&mut broken.as_slice()).is_err());
        }
        bytes[13] = 0x80;
        assert!(read(&mut bytes.as_slice()).is_err());
        bytes[8..12].copy_from_slice(&u32::MAX.to_le_bytes());
        assert!(read(&mut bytes.as_slice()).is_err());
    }
}
