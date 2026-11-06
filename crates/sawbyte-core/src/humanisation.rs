//! Configurable, seedable humanisation policies for camera aiming, GUI cursor travel,
//! post-revision reaction delays, click cadence, text typing, and compensatory headless perception.
//! Enabled by default with an explicit `Direct` bypass and custom `InputPolicy` hooks.

use crate::InputPolicy;

/// Vanilla survival interaction reach limits (strictly enforced when humanisation / fair-play is active).
pub const VANILLA_ENTITY_INTERACT_REACH: f64 = 3.0;
pub const VANILLA_BLOCK_INTERACT_REACH: f64 = 4.5;

/// Deterministic SplitMix64 PRNG so seeded runs are 100% reproducible across platforms without external crates.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DeterministicRng {
    state: u64,
}

impl DeterministicRng {
    pub const fn new(seed: u64) -> Self {
        Self {
            state: seed ^ 0x5341_5742_5954_4501,
        }
    }

    pub fn next_u64(&mut self) -> u64 {
        self.state = self.state.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.state;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    /// Uniform float in `[0.0, 1.0)`.
    pub fn next_f64(&mut self) -> f64 {
        ((self.next_u64() >> 11) as f64) * (1.0 / ((1u64 << 53) as f64))
    }
}

/// Epistemic classification of how a headless agent perceives a block, structure, or entity.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VisibilityTier {
    /// Unobstructed line-of-sight inside the player's current camera frustum (FOV).
    DirectFovLos,
    /// Unobstructed line-of-sight or within acoustic footstep/sound radius (<=16m), but outside current camera FOV.
    PeripheralAcousticLos,
    /// Previously observed with direct line-of-sight earlier in the session or recalled from persistent memory.
    MemoryRecalled,
    /// Detected exclusively via compensatory through-wall ESP or through-block X-ray raycasting with no prior LOS.
    PrivilegedThroughWallHint,
}

impl VisibilityTier {
    pub const fn as_code(&self) -> &'static str {
        match self {
            Self::DirectFovLos => "DIRECT_FOV_LOS",
            Self::PeripheralAcousticLos => "PERIPHERAL_ACOUSTIC_LOS",
            Self::MemoryRecalled => "MEMORY_RECALLED",
            Self::PrivilegedThroughWallHint => "PRIVILEGED_THROUGH_WALL_HINT",
        }
    }
}

/// Result of filtering a perceived entity or block through `HumanisedPerceptionPolicy`.
#[derive(Clone, Debug, PartialEq)]
pub struct PerceivedTargetGuidance {
    pub tier: VisibilityTier,
    /// Epistemic weight in `[0.0, 1.0]` indicating how strongly an agent should act on this target when humanised.
    pub human_confidence: f64,
    /// Whether immediate camera lock / direct interaction is natural and permitted.
    pub direct_action_permitted: bool,
    /// Recommended human-like action before engaging the target.
    pub guidance: &'static str,
    /// Coarse compass sector hint (e.g. `"North-East (~8m)"`) used when exact coordinates are masked by humanisation.
    pub sector_hint: String,
    /// Reaction delay ticks to wait after acquiring line of sight before attacking or clicking.
    pub reaction_delay_ticks: u32,
}

/// Governs how compensatory headless perception (through-wall ESP, through-block awareness, long-range scans)
/// interacts with humanisation and fair-play mechanics.
///
/// When `enabled == true` (default):
/// 1. Enforces strict vanilla survival reach (`3.0` blocks for entity combat, `4.5` blocks for block interaction).
/// 2. Prohibits snapping the crosshair onto or attacking `PrivilegedThroughWallHint` targets through walls.
/// 3. Treats occluded through-wall detections as low-weight supplementary hints (`human_confidence = 0.18`),
///    guiding the agent to explore or peek around corners naturally before acting.
#[derive(Clone, Copy, Debug)]
pub struct HumanisedPerceptionPolicy {
    pub enabled: bool,
    pub fov_horizontal_deg: f64,
    pub fov_vertical_deg: f64,
    pub acoustic_awareness_radius: f64,
    pub max_entity_reach: f64,
    pub max_block_reach: f64,
    pub visual_acquisition_reaction_ticks: u32,
}

impl Default for HumanisedPerceptionPolicy {
    fn default() -> Self {
        Self::humanised()
    }
}

impl HumanisedPerceptionPolicy {
    pub const fn humanised() -> Self {
        Self {
            enabled: true,
            fov_horizontal_deg: 75.0,
            fov_vertical_deg: 55.0,
            acoustic_awareness_radius: 16.0,
            max_entity_reach: VANILLA_ENTITY_INTERACT_REACH,
            max_block_reach: VANILLA_BLOCK_INTERACT_REACH,
            visual_acquisition_reaction_ticks: 3,
        }
    }

    pub const fn direct_unfiltered() -> Self {
        Self {
            enabled: false,
            fov_horizontal_deg: 360.0,
            fov_vertical_deg: 180.0,
            acoustic_awareness_radius: 128.0,
            max_entity_reach: VANILLA_ENTITY_INTERACT_REACH,
            max_block_reach: VANILLA_BLOCK_INTERACT_REACH,
            visual_acquisition_reaction_ticks: 0,
        }
    }

    /// Classifies a target's `VisibilityTier` given line-of-sight status, relative bearing angles,
    /// distance, and whether the target was previously seen with direct LOS.
    pub fn classify_visibility(
        &self,
        has_line_of_sight: bool,
        rel_yaw_deg: f64,
        rel_pitch_deg: f64,
        distance: f64,
        previously_seen_los: bool,
    ) -> VisibilityTier {
        if has_line_of_sight {
            if rel_yaw_deg.abs() <= self.fov_horizontal_deg * 0.5
                && rel_pitch_deg.abs() <= self.fov_vertical_deg * 0.5
            {
                VisibilityTier::DirectFovLos
            } else {
                VisibilityTier::PeripheralAcousticLos
            }
        } else if previously_seen_los {
            VisibilityTier::MemoryRecalled
        } else if distance <= self.acoustic_awareness_radius * 0.5 {
            VisibilityTier::PrivilegedThroughWallHint
        } else {
            VisibilityTier::PrivilegedThroughWallHint
        }
    }

    /// Evaluates how an agent should treat a perceived target under the current humanisation mode.
    pub fn evaluate_target(
        &self,
        has_line_of_sight: bool,
        rel_yaw_deg: f64,
        rel_pitch_deg: f64,
        distance: f64,
        previously_seen_los: bool,
        cardinal_sector: &str,
    ) -> PerceivedTargetGuidance {
        let tier = self.classify_visibility(
            has_line_of_sight,
            rel_yaw_deg,
            rel_pitch_deg,
            distance,
            previously_seen_los,
        );
        let approx_dist = ((distance / 4.0).round() as i32 * 4).max(4);
        let sector_hint = format!("{cardinal_sector} (~{approx_dist}m)");

        if !self.enabled {
            return PerceivedTargetGuidance {
                tier,
                human_confidence: 1.0,
                direct_action_permitted: has_line_of_sight,
                guidance: if has_line_of_sight {
                    "Direct mode: immediate interaction permitted within vanilla reach."
                } else {
                    "Direct mode: target occluded by solid blocks; navigate to clear LOS before interacting."
                },
                sector_hint,
                reaction_delay_ticks: 0,
            };
        }

        match tier {
            VisibilityTier::DirectFovLos => PerceivedTargetGuidance {
                tier,
                human_confidence: 1.00,
                direct_action_permitted: true,
                guidance: "Visible in camera FOV with clear line of sight; safe to aim and interact.",
                sector_hint,
                reaction_delay_ticks: 0,
            },
            VisibilityTier::PeripheralAcousticLos => PerceivedTargetGuidance {
                tier,
                human_confidence: 0.78,
                direct_action_permitted: true,
                guidance: "Clear LOS or audible in periphery outside camera frustum; rotate camera smoothly first.",
                sector_hint,
                reaction_delay_ticks: self.visual_acquisition_reaction_ticks,
            },
            VisibilityTier::MemoryRecalled => PerceivedTargetGuidance {
                tier,
                human_confidence: 0.65,
                direct_action_permitted: false,
                guidance: "Recalled from prior visual contact; navigate to last known vantage point to re-verify.",
                sector_hint,
                reaction_delay_ticks: self.visual_acquisition_reaction_ticks,
            },
            VisibilityTier::PrivilegedThroughWallHint => PerceivedTargetGuidance {
                tier,
                human_confidence: 0.18,
                direct_action_permitted: false,
                guidance: "SUPPLEMENTARY HINT ONLY (occluded behind wall): Do NOT pre-aim or track through walls; explore natural corridors toward sector if relevant.",
                sector_hint,
                reaction_delay_ticks: self.visual_acquisition_reaction_ticks + 2,
            },
        }
    }

    /// Validates that a requested reach value does not exceed fair vanilla survival limits when humanisation is active.
    pub fn clamp_fair_block_reach(&self, requested: f64) -> f64 {
        if self.enabled {
            requested.clamp(1.0, VANILLA_BLOCK_INTERACT_REACH)
        } else {
            requested.clamp(1.0, 6.0)
        }
    }

    pub fn clamp_fair_entity_reach(&self, requested: f64) -> f64 {
        if self.enabled {
            requested.clamp(1.0, VANILLA_ENTITY_INTERACT_REACH)
        } else {
            requested.clamp(1.0, 6.0)
        }
    }
}

/// Smooth acceleration/deceleration camera trajectory policy with deterministic seedable micro-variation.
/// Guarantees monotonic sign and bounded step magnitude per tick so `Scheduler` invariants always hold.
#[derive(Clone, Copy, Debug)]
pub struct SmoothHumanisedCamera {
    pub max_degrees_per_tick: f64,
    pub acceleration_gain: f64,
    pub deceleration_window_deg: f64,
    pub jitter_fraction: f64,
    current_speed: f64,
    rng: DeterministicRng,
}

impl Default for SmoothHumanisedCamera {
    fn default() -> Self {
        Self::seeded(0x2603_2026)
    }
}

impl SmoothHumanisedCamera {
    pub const fn seeded(seed: u64) -> Self {
        Self {
            max_degrees_per_tick: 14.0,
            acceleration_gain: 4.5,
            deceleration_window_deg: 24.0,
            jitter_fraction: 0.08,
            current_speed: 0.0,
            rng: DeterministicRng::new(seed),
        }
    }
}

impl InputPolicy for SmoothHumanisedCamera {
    fn camera_step(&mut self, (yaw, pitch): (f64, f64)) -> (f64, f64) {
        let distance = yaw.hypot(pitch);
        if distance <= 1e-9 || !distance.is_finite() {
            self.current_speed = 0.0;
            return (0.0, 0.0);
        }
        if distance <= 0.35 {
            self.current_speed = 0.0;
            return (yaw, pitch);
        }
        let target_speed = if distance < self.deceleration_window_deg {
            (self.max_degrees_per_tick * (distance / self.deceleration_window_deg)).max(0.5)
        } else {
            self.max_degrees_per_tick
        };
        if self.current_speed < target_speed {
            self.current_speed = (self.current_speed + self.acceleration_gain).min(target_speed);
        } else {
            self.current_speed = target_speed;
        }
        let variation = 1.0 + (self.rng.next_f64() - 0.5) * 2.0 * self.jitter_fraction;
        let step_len = (self.current_speed * variation).clamp(0.25, distance);
        let scale = (step_len / distance).min(1.0);
        (yaw * scale, pitch * scale)
    }
}

/// GUI cursor, reaction delay, and click cadence policy for inventory and menu screens.
#[derive(Clone, Copy, Debug)]
pub struct GuiTimingPolicy {
    pub enabled: bool,
    pub cursor_pos: (f64, f64),
    pub pixels_per_tick: f64,
    pub screen_open_reaction_ticks: u32,
    pub min_ticks_between_clicks: u32,
    rng: DeterministicRng,
}

impl Default for GuiTimingPolicy {
    fn default() -> Self {
        Self::humanised(0x5348_4F50)
    }
}

impl GuiTimingPolicy {
    pub const fn humanised(seed: u64) -> Self {
        Self {
            enabled: true,
            cursor_pos: (88.0, 83.0), // Standard 176x166 container center
            pixels_per_tick: 42.0,
            screen_open_reaction_ticks: 3,
            min_ticks_between_clicks: 2,
            rng: DeterministicRng::new(seed),
        }
    }

    pub const fn direct() -> Self {
        Self {
            enabled: false,
            cursor_pos: (0.0, 0.0),
            pixels_per_tick: f64::INFINITY,
            screen_open_reaction_ticks: 0,
            min_ticks_between_clicks: 0,
            rng: DeterministicRng::new(0),
        }
    }

    /// Maps a standard 9-column container slot index to approximate 2D virtual pixel coordinates.
    pub fn slot_coordinates(slot: i16) -> (f64, f64) {
        if slot < 0 {
            return (88.0, 20.0);
        }
        let col = (slot % 9) as f64;
        let row = (slot / 9) as f64;
        (8.0 + col * 18.0 + 9.0, 18.0 + row * 18.0 + 9.0)
    }

    /// Computes how many ticks to wait before clicking `target_coords` from the current virtual cursor.
    pub fn move_to_and_delay_ticks(
        &mut self,
        target_coords: (f64, f64),
        screen_just_changed: bool,
    ) -> u32 {
        if !self.enabled {
            self.cursor_pos = target_coords;
            return 0;
        }
        let dx = target_coords.0 - self.cursor_pos.0;
        let dy = target_coords.1 - self.cursor_pos.1;
        let dist = dx.hypot(dy);
        self.cursor_pos = target_coords;
        let travel_ticks = (dist / self.pixels_per_tick.max(1.0)).ceil() as u32;
        let base_reaction = if screen_just_changed {
            self.screen_open_reaction_ticks
        } else {
            self.min_ticks_between_clicks
        };
        let jitter = (self.rng.next_u64() % 2) as u32;
        base_reaction + travel_ticks + jitter
    }
}

/// Typing cadence policy for chat, commands, anvil renames, signs, books, and dialog prompts.
#[derive(Clone, Copy, Debug)]
pub struct TypingPolicy {
    pub enabled: bool,
    pub chars_per_tick: usize,
    pub submit_pause_ticks: u32,
}

impl Default for TypingPolicy {
    fn default() -> Self {
        Self {
            enabled: true,
            chars_per_tick: 2,
            submit_pause_ticks: 2,
        }
    }
}

impl TypingPolicy {
    pub const fn direct() -> Self {
        Self {
            enabled: false,
            chars_per_tick: usize::MAX,
            submit_pause_ticks: 0,
        }
    }

    /// Splits `text` into per-tick progressive string snapshots followed by the final submission tick offset.
    pub fn schedule_typing(&self, text: &str) -> Vec<(u32, String)> {
        if !self.enabled || text.is_empty() {
            return vec![(0, text.to_owned())];
        }
        let chars: Vec<char> = text.chars().collect();
        let step = self.chars_per_tick.max(1);
        let mut schedule = Vec::new();
        let mut tick = 0u32;
        let mut idx = 0;
        while idx < chars.len() {
            idx = (idx + step).min(chars.len());
            let prefix: String = chars[..idx].iter().collect();
            schedule.push((tick, prefix));
            tick += 1;
        }
        if self.submit_pause_ticks > 0 {
            if let Some(last) = schedule.last_mut() {
                last.0 += self.submit_pause_ticks;
            }
        }
        schedule
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Action, Backend, Capability, InputFrame, Scheduler};

    struct DummyBackend;
    impl Backend for DummyBackend {
        type Error = &'static str;
        fn supports(&self, _: Capability) -> bool {
            true
        }
        fn apply(&mut self, _: u64, _: InputFrame) -> Result<(), Self::Error> {
            Ok(())
        }
        fn release_all(&mut self) -> Result<(), Self::Error> {
            Ok(())
        }
    }

    #[test]
    fn smooth_camera_converges_deterministically_and_satisfies_scheduler() {
        let mut s1 = Scheduler::new(DummyBackend, SmoothHumanisedCamera::seeded(42), 16);
        let mut s2 = Scheduler::new(DummyBackend, SmoothHumanisedCamera::seeded(42), 16);
        s1.submit(&[(
            0,
            Action::Look {
                yaw: 120.0,
                pitch: -35.0,
            },
        )])
        .unwrap();
        s2.submit(&[(
            0,
            Action::Look {
                yaw: 120.0,
                pitch: -35.0,
            },
        )])
        .unwrap();

        let mut total_yaw = 0.0;
        let mut total_pitch = 0.0;
        for t in 0..30 {
            let f1 = s1.advance(t).unwrap();
            let f2 = s2.advance(t).unwrap();
            assert_eq!(f1, f2);
            total_yaw += f1.yaw_delta;
            total_pitch += f1.pitch_delta;
        }
        assert!((total_yaw - 120.0).abs() < 1e-6);
        assert!((total_pitch - -35.0).abs() < 1e-6);
    }

    #[test]
    fn gui_and_typing_policies_respect_direct_off_switch() {
        let mut human_gui = GuiTimingPolicy::humanised(7);
        let mut direct_gui = GuiTimingPolicy::direct();
        let d_human =
            human_gui.move_to_and_delay_ticks(GuiTimingPolicy::slot_coordinates(26), true);
        let d_direct =
            direct_gui.move_to_and_delay_ticks(GuiTimingPolicy::slot_coordinates(26), true);
        assert!(d_human >= 3);
        assert_eq!(d_direct, 0);

        let human_type = TypingPolicy::default().schedule_typing("6400");
        let direct_type = TypingPolicy::direct().schedule_typing("6400");
        assert_eq!(human_type.len(), 2);
        assert_eq!(human_type.last().unwrap().1, "6400");
        assert_eq!(direct_type, vec![(0, "6400".to_owned())]);
    }

    #[test]
    fn humanised_perception_policy_gates_through_wall_esp_and_clamps_vanilla_reach() {
        let policy = HumanisedPerceptionPolicy::humanised();
        assert_eq!(policy.clamp_fair_entity_reach(6.0), 3.0);
        assert_eq!(policy.clamp_fair_block_reach(6.0), 4.5);

        let visible = policy.evaluate_target(true, 10.0, -5.0, 2.8, true, "North");
        assert_eq!(visible.tier, VisibilityTier::DirectFovLos);
        assert!(visible.direct_action_permitted);
        assert_eq!(visible.human_confidence, 1.0);

        let occluded = policy.evaluate_target(false, 5.0, 0.0, 7.5, false, "East");
        assert_eq!(occluded.tier, VisibilityTier::PrivilegedThroughWallHint);
        assert!(!occluded.direct_action_permitted);
        assert!(occluded.human_confidence < 0.25);
        assert!(occluded.guidance.contains("SUPPLEMENTARY HINT ONLY"));
    }
}
