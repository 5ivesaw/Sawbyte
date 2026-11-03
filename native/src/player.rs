//! Initial ordinary-ground motion model. Unknown mechanics fail explicitly.
use crate::{wire::{bad, Result}, world::World};
use sawbyte_core::{Button, InputFrame};

#[derive(Default)]
pub struct Player {
    pub pos: [f64; 3], pub velocity: [f64; 3], pub yaw: f32, pub pitch: f32,
    pub ground: bool, pub horizontal_collision: bool,
}
impl Player {
    pub fn step(&mut self, frame: InputFrame, world: &World) -> Result<()> {
        if frame.buttons.bits() & !31 != 0 { return Err(bad("this ground-motion prototype does not support sneak, sprint, attack or use yet")); }
        self.yaw += frame.yaw_delta as f32;
        self.pitch = (self.pitch + frame.pitch_delta as f32).clamp(-90.0,90.0);
        if self.velocity[0].powi(2) + self.velocity[2].powi(2) < 9e-6 { self.velocity[0] = 0.; self.velocity[2] = 0.; }
        if self.velocity[1].abs() < 0.003 { self.velocity[1] = 0.; }
        if self.ground && frame.buttons.pressed(Button::Jump) { self.velocity[1] = 0.42f32 as f64; }
        let b = frame.buttons;
        let mut forward = (i32::from(b.pressed(Button::Forward)) - i32::from(b.pressed(Button::Back))) as f32;
        let mut side = (i32::from(b.pressed(Button::Left)) - i32::from(b.pressed(Button::Right))) as f32;
        let magnitude = forward.hypot(side);
        if magnitude > 1. { forward /= magnitude; side /= magnitude; }
        forward *= 0.98; side *= 0.98;
        let friction: f32 = if self.ground { 0.6 * 0.91 } else { 0.91 };
        let acceleration: f32 = if self.ground { 0.1 * (0.21600002 / (0.6f32 * 0.6 * 0.6)) } else { 0.02 };
        // Minecraft's angle table has 65536 entries. Compute just the two entries used.
        let phase = (self.yaw * std::f32::consts::PI / 180. * 10430.378) as i32;
        let trig = |index: i32| ((index & 65535) as f64 * std::f64::consts::TAU / 65536.).sin() as f32;
        let sin = trig(phase); let cos = trig(phase + 16384);
        self.velocity[0] += ((side * acceleration) * cos - (forward * acceleration) * sin) as f64;
        self.velocity[2] += ((forward * acceleration) * cos + (side * acceleration) * sin) as f64;
        let wanted = self.velocity;
        let mut moved = wanted;
        // Sweep the player's box along each axis, clipping against occupied cubes.
        for axis in [1, 0, 2] {
            let mut low = [self.pos[0] - 0.3, self.pos[1], self.pos[2] - 0.3];
            let mut high = [self.pos[0] + 0.3, self.pos[1] + 1.8, self.pos[2] + 0.3];
            low[axis] += wanted[axis].min(0.); high[axis] += wanted[axis].max(0.);
            for x in low[0].floor() as i32..=high[0].floor() as i32 {
                for y in low[1].floor() as i32..=high[1].floor() as i32 {
                    for z in low[2].floor() as i32..=high[2].floor() as i32 {
                        if !world.solid(x,y,z)? { continue; }
                        let block = [x as f64, y as f64, z as f64];
                        let min = [self.pos[0]-0.3,self.pos[1],self.pos[2]-0.3];
                        let max = [self.pos[0]+0.3,self.pos[1]+1.8,self.pos[2]+0.3];
                        if (0..3).any(|a| a != axis && (max[a] <= block[a] + 1e-7 || min[a] >= block[a] + 1. - 1e-7)) { continue; }
                        if moved[axis] > 0. && max[axis] <= block[axis] + 1e-7 { moved[axis] = moved[axis].min(block[axis]-max[axis]); }
                        if moved[axis] < 0. && min[axis] >= block[axis]+1.-1e-7 { moved[axis] = moved[axis].max(block[axis]+1.-min[axis]); }
                    }
                }
            }
            self.pos[axis] += moved[axis];
        }
        self.ground = wanted[1] < 0. && moved[1] != wanted[1];
        self.horizontal_collision = moved[0] != wanted[0] || moved[2] != wanted[2];
        for (axis, value) in moved.iter().enumerate() { if *value != wanted[axis] { self.velocity[axis] = 0.; } }
        self.velocity[0] *= friction as f64;
        self.velocity[2] *= friction as f64;
        self.velocity[1] = (self.velocity[1] - 0.08) * 0.98f32 as f64;
        Ok(())
    }
}
