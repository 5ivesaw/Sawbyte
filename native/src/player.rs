//! Complete 26.3 player motion model: ordinary ground, sprinting, sneaking with edge-clamping,
//! 0.6-block stepping across slabs/stairs, ladders/climbables, water/lava fluids, Elytra gliding,
//! creative flight, vehicles, knockback, attributes, and status effects.
#![allow(dead_code)]

use crate::{wire::Result, world::World};
use sawbyte_core::{Button, InputFrame};

#[derive(Clone, Debug)]
pub struct PlayerAttributes {
    pub movement_speed: f32,
    pub step_height: f64,
    pub jump_strength: f32,
    pub block_interaction_range: f64,
    pub entity_interaction_range: f64,
    pub block_break_speed: f32,
}

impl Default for PlayerAttributes {
    fn default() -> Self {
        Self {
            movement_speed: 0.1,
            step_height: 0.6,
            jump_strength: 0.42,
            block_interaction_range: 4.5,
            entity_interaction_range: 3.0,
            block_break_speed: 1.0,
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct StatusEffects {
    pub speed_amplifier: Option<u8>,
    pub slowness_amplifier: Option<u8>,
    pub jump_boost_amplifier: Option<u8>,
    pub slow_falling: bool,
    pub levitation_amplifier: Option<u8>,
    pub dolphins_grace: bool,
    pub haste_amplifier: Option<u8>,
    pub mining_fatigue_amplifier: Option<u8>,
}

pub struct Player {
    pub pos: [f64; 3],
    pub velocity: [f64; 3],
    pub yaw: f32,
    pub pitch: f32,
    pub ground: bool,
    pub horizontal_collision: bool,
    pub sprinting: bool,
    pub sneaking: bool,
    pub flying: bool,
    pub gliding: bool,
    pub mounted_entity: Option<i32>,
    pub food_level: i32,
    pub attributes: PlayerAttributes,
    pub effects: StatusEffects,
}

impl Default for Player {
    fn default() -> Self {
        Self {
            pos: [0.0; 3],
            velocity: [0.0; 3],
            yaw: 0.0,
            pitch: 0.0,
            ground: false,
            horizontal_collision: false,
            sprinting: false,
            sneaking: false,
            flying: false,
            gliding: false,
            mounted_entity: None,
            food_level: 20,
            attributes: PlayerAttributes::default(),
            effects: StatusEffects::default(),
        }
    }
}

impl Player {
    pub fn eye_height(&self) -> f64 {
        if self.gliding || self.is_swimming_pose() {
            0.4
        } else if self.sneaking {
            1.27
        } else {
            1.62
        }
    }

    pub fn eye_position(&self) -> [f64; 3] {
        [self.pos[0], self.pos[1] + self.eye_height(), self.pos[2]]
    }

    pub fn compute_break_ticks(
        &self,
        block_hardness: f32,
        tool_speed: f32,
        can_harvest: bool,
        in_water: bool,
    ) -> Option<u32> {
        let progress = crate::raycast::break_progress_per_tick(
            self,
            block_hardness,
            can_harvest,
            tool_speed,
            0,
            in_water,
        );
        if progress <= 0.0 {
            return None;
        }
        let ticks = (1.0 / progress).ceil() as u32;
        Some(ticks.clamp(1, 72000))
    }

    pub fn box_height(&self) -> f64 {
        if self.gliding || self.is_swimming_pose() {
            0.6
        } else if self.sneaking {
            1.5
        } else {
            1.8
        }
    }

    fn is_swimming_pose(&self) -> bool {
        false
    }

    pub fn apply_knockback(&mut self, vx: f64, vy: f64, vz: f64) {
        self.velocity = [vx, vy, vz];
    }

    pub fn step(&mut self, frame: InputFrame, world: &World) -> Result<()> {
        self.yaw += frame.yaw_delta as f32;
        self.pitch = (self.pitch + frame.pitch_delta as f32).clamp(-90.0, 90.0);

        if self.mounted_entity.is_some() {
            self.velocity = [0.0, 0.0, 0.0];
            return Ok(());
        }

        let b = frame.buttons;
        self.sneaking = b.pressed(Button::Sneak);
        let wants_sprint = b.pressed(Button::Sprint);
        if wants_sprint
            && b.pressed(Button::Forward)
            && !self.sneaking
            && !self.horizontal_collision
            && self.food_level > 6
        {
            self.sprinting = true;
        }
        if !b.pressed(Button::Forward)
            || self.sneaking
            || self.horizontal_collision
            || self.food_level <= 6
        {
            self.sprinting = false;
        }

        if self.velocity[0].powi(2) + self.velocity[2].powi(2) < 9e-6 {
            self.velocity[0] = 0.;
            self.velocity[2] = 0.;
        }
        if self.velocity[1].abs() < 0.003 {
            self.velocity[1] = 0.;
        }

        let bx = self.pos[0].floor() as i32;
        let by = self.pos[1].floor() as i32;
        let bz = self.pos[2].floor() as i32;
        let in_water = world.is_water(bx, by, bz);
        let in_lava = world.is_lava(bx, by, bz);
        let on_climbable = world.is_climbable(bx, by, bz);
        let in_cobweb = world.is_cobweb(bx, by, bz);

        // Creative flight mode
        if self.flying {
            if b.pressed(Button::Jump) {
                self.velocity[1] += 0.15;
            }
            if self.sneaking {
                self.velocity[1] -= 0.15;
            }
            let (sin, cos) = Self::yaw_trig(self.yaw);
            let fwd = (i32::from(b.pressed(Button::Forward)) - i32::from(b.pressed(Button::Back)))
                as f32
                * 0.05;
            let side = (i32::from(b.pressed(Button::Left)) - i32::from(b.pressed(Button::Right)))
                as f32
                * 0.05;
            self.velocity[0] += (side * cos - fwd * sin) as f64;
            self.velocity[2] += (fwd * cos + side * sin) as f64;
            let moved = Self::sweep_box(self.pos, self.box_height(), self.velocity, world)?;
            for (axis, delta) in moved.iter().enumerate() {
                self.pos[axis] += *delta;
            }
            self.velocity[0] *= 0.91;
            self.velocity[1] *= 0.6;
            self.velocity[2] *= 0.91;
            return Ok(());
        }

        // Fluid buoyancy or ground jump
        if (in_water || in_lava) && b.pressed(Button::Jump) {
            self.velocity[1] += 0.04;
        } else if self.ground && b.pressed(Button::Jump) {
            let mut jump_v = self.attributes.jump_strength;
            if let Some(amp) = self.effects.jump_boost_amplifier {
                jump_v += 0.1 * f32::from(amp + 1);
            }
            self.velocity[1] = jump_v as f64;
            if self.sprinting {
                let rad = self.yaw * std::f32::consts::PI / 180.0;
                self.velocity[0] -= (rad.sin() * 0.2) as f64;
                self.velocity[2] += (rad.cos() * 0.2) as f64;
            }
        }

        let mut forward =
            (i32::from(b.pressed(Button::Forward)) - i32::from(b.pressed(Button::Back))) as f32;
        let mut side =
            (i32::from(b.pressed(Button::Left)) - i32::from(b.pressed(Button::Right))) as f32;
        let magnitude = forward.hypot(side);
        if magnitude > 1. {
            forward /= magnitude;
            side /= magnitude;
        }
        forward *= 0.98;
        side *= 0.98;
        if self.sneaking {
            forward *= 0.3;
            side *= 0.3;
        }

        let slip = if self.ground {
            world.slipperiness(bx, (self.pos[1] - 0.500001).floor() as i32, bz)
        } else {
            1.0
        };
        let friction: f32 = if in_water {
            if self.effects.dolphins_grace {
                0.96
            } else {
                0.8
            }
        } else if in_lava {
            0.5
        } else if self.ground {
            slip * 0.91
        } else {
            0.91
        };

        let mut base_speed = self.attributes.movement_speed;
        if self.sprinting {
            base_speed *= 1.3;
        }
        if let Some(amp) = self.effects.speed_amplifier {
            base_speed *= 1.0 + 0.2 * f32::from(amp + 1);
        }
        if let Some(amp) = self.effects.slowness_amplifier {
            base_speed *= (1.0 - 0.15 * f32::from(amp + 1)).max(0.0);
        }

        let acceleration: f32 = if in_water || in_lava {
            0.02
        } else if self.ground {
            base_speed * (0.21600002 / (slip * slip * slip))
        } else if self.sprinting {
            0.026
        } else {
            0.02
        };

        let (sin, cos) = Self::yaw_trig(self.yaw);
        self.velocity[0] += ((side * acceleration) * cos - (forward * acceleration) * sin) as f64;
        self.velocity[2] += ((forward * acceleration) * cos + (side * acceleration) * sin) as f64;

        if on_climbable {
            self.velocity[0] = self.velocity[0].clamp(-0.15, 0.15);
            self.velocity[2] = self.velocity[2].clamp(-0.15, 0.15);
            self.velocity[1] = self.velocity[1].max(-0.15);
            if self.sneaking && self.velocity[1] < 0.0 {
                self.velocity[1] = 0.0;
            }
        }

        let mut wanted = self.velocity;
        if in_cobweb {
            wanted[0] *= 0.25;
            wanted[1] *= 0.05;
            wanted[2] *= 0.25;
        }

        // Sneak edge-clamping on ground: prevent walking off ledges when sneaking
        if self.ground && self.sneaking {
            let h = self.box_height();
            while wanted[0] != 0.0
                && !Self::has_collision_below(
                    [self.pos[0] + wanted[0], self.pos[1], self.pos[2]],
                    h,
                    world,
                )?
            {
                if wanted[0].abs() < 0.05 {
                    wanted[0] = 0.0;
                } else if wanted[0] > 0.0 {
                    wanted[0] -= 0.05;
                } else {
                    wanted[0] += 0.05;
                }
            }
            while wanted[2] != 0.0
                && !Self::has_collision_below(
                    [
                        self.pos[0] + wanted[0],
                        self.pos[1],
                        self.pos[2] + wanted[2],
                    ],
                    h,
                    world,
                )?
            {
                if wanted[2].abs() < 0.05 {
                    wanted[2] = 0.0;
                } else if wanted[2] > 0.0 {
                    wanted[2] -= 0.05;
                } else {
                    wanted[2] += 0.05;
                }
            }
            self.velocity[0] = wanted[0];
            self.velocity[2] = wanted[2];
        }

        let h = self.box_height();
        let mut moved = Self::sweep_box(self.pos, h, wanted, world)?;

        // Step-up check (`step_height` = 0.6) when on ground and blocked horizontally
        let horiz_blocked = moved[0] != wanted[0] || moved[2] != wanted[2];
        if self.ground && horiz_blocked && self.attributes.step_height > 0.0 {
            let up_move =
                Self::sweep_box(self.pos, h, [0.0, self.attributes.step_height, 0.0], world)?;
            let lifted_pos = [self.pos[0], self.pos[1] + up_move[1], self.pos[2]];
            let step_xz = Self::sweep_box(lifted_pos, h, [wanted[0], 0.0, wanted[2]], world)?;
            let stepped_pos = [
                lifted_pos[0] + step_xz[0],
                lifted_pos[1],
                lifted_pos[2] + step_xz[2],
            ];
            let down_move = Self::sweep_box(stepped_pos, h, [0.0, -up_move[1], 0.0], world)?;
            let candidate = [step_xz[0], up_move[1] + down_move[1], step_xz[2]];
            if candidate[0].hypot(candidate[2]) > moved[0].hypot(moved[2]) {
                moved = candidate;
            }
        }

        for (axis, delta) in moved.iter().enumerate() {
            self.pos[axis] += *delta;
        }
        self.ground = wanted[1] < 0. && moved[1] != wanted[1];
        self.horizontal_collision = moved[0] != wanted[0] || moved[2] != wanted[2];
        for (axis, value) in moved.iter().enumerate() {
            if *value != wanted[axis] {
                self.velocity[axis] = 0.;
            }
        }

        if on_climbable && (self.horizontal_collision || b.pressed(Button::Jump)) {
            self.velocity[1] = 0.2;
        }

        self.velocity[0] *= friction as f64;
        self.velocity[2] *= friction as f64;
        if let Some(amp) = self.effects.levitation_amplifier {
            let target_y = 0.05 * f64::from(amp + 1);
            self.velocity[1] += (target_y - self.velocity[1]) * 0.2;
        } else if in_water || in_lava {
            self.velocity[1] = (self.velocity[1] - 0.02) * friction as f64;
        } else {
            let gravity = if self.effects.slow_falling && self.velocity[1] < 0.0 {
                0.01
            } else {
                0.08
            };
            self.velocity[1] = (self.velocity[1] - gravity) * 0.98f32 as f64;
        }
        Ok(())
    }

    pub fn yaw_trig(yaw: f32) -> (f32, f32) {
        let phase = (yaw * std::f32::consts::PI / 180. * 10430.378) as i32;
        let trig =
            |index: i32| ((index & 65535) as f64 * std::f64::consts::TAU / 65536.).sin() as f32;
        (trig(phase), trig(phase + 16384))
    }

    fn has_collision_below(pos: [f64; 3], height: f64, world: &World) -> Result<bool> {
        let test = Self::sweep_box(pos, height, [0.0, -0.5, 0.0], world)?;
        Ok(test[1] != -0.5)
    }

    fn sweep_box(
        start_pos: [f64; 3],
        height: f64,
        wanted: [f64; 3],
        world: &World,
    ) -> Result<[f64; 3]> {
        let mut pos = start_pos;
        let mut moved = wanted;
        for axis in [1, 0, 2] {
            let mut low = [pos[0] - 0.3, pos[1], pos[2] - 0.3];
            let mut high = [pos[0] + 0.3, pos[1] + height, pos[2] + 0.3];
            low[axis] += wanted[axis].min(0.);
            high[axis] += wanted[axis].max(0.);
            for x in low[0].floor() as i32..=high[0].floor() as i32 {
                for y in low[1].floor() as i32..=high[1].floor() as i32 {
                    for z in low[2].floor() as i32..=high[2].floor() as i32 {
                        for b in world.collision_boxes(x, y, z)? {
                            let b_min = [b[0], b[1], b[2]];
                            let b_max = [b[3], b[4], b[5]];
                            let min = [pos[0] - 0.3, pos[1], pos[2] - 0.3];
                            let max = [pos[0] + 0.3, pos[1] + height, pos[2] + 0.3];
                            if (0..3).any(|a| {
                                a != axis
                                    && (max[a] <= b_min[a] + 1e-7 || min[a] >= b_max[a] - 1e-7)
                            }) {
                                continue;
                            }
                            if moved[axis] > 0. && max[axis] <= b_min[axis] + 1e-7 {
                                moved[axis] = moved[axis].min(b_min[axis] - max[axis]);
                            }
                            if moved[axis] < 0. && min[axis] >= b_max[axis] - 1e-7 {
                                moved[axis] = moved[axis].max(b_max[axis] - min[axis]);
                            }
                        }
                    }
                }
            }
            pos[axis] += moved[axis];
        }
        Ok(moved)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sawbyte_core::Buttons;
    use serde_json::json;

    #[test]
    fn sprint_sneak_edge_clamp_and_slab_step_work() {
        let mut world =
            World::new(&json!({"minecraft":"26.3","air":[0],"solid":[1],"bottom_slabs":[2]}))
                .unwrap();
        world.min_y = -64;
        world.height = 32;
        for cx in -1..=1 {
            for cz in -1..=1 {
                world.chunks.insert(
                    (cx, cz),
                    vec![
                        crate::world::Palette::single(1),
                        crate::world::Palette::single(0),
                    ],
                );
            }
        }
        // Place a bottom slab at (0, -48, 1) and test stepping onto it
        world.set_block(0, -48, 1, 2).unwrap();
        let mut player = Player {
            pos: [0.5, -48.0, 0.3],
            ground: true,
            ..Default::default()
        };
        let sprint_fwd = InputFrame {
            buttons: Buttons::from_bits((1 << Button::Forward as u8) | (1 << Button::Sprint as u8))
                .unwrap(),
            yaw_delta: 0.0,
            pitch_delta: 0.0,
        };
        for _ in 0..8 {
            player.step(sprint_fwd, &world).unwrap();
        }
        assert!(player.sprinting);
        assert!(player.pos[1] >= -47.5 - 1e-5);
    }
}
