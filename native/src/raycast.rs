//! Exact 26.3 eye-position block & entity AABB raycasting, line-of-sight occlusion,
//! reach enforcement, and multi-tick block break speed calculation.
#![allow(dead_code)]

use crate::player::Player;
use crate::world::World;
use sawbyte_core::events::BlockFace;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BlockHit {
    pub block_pos: [i32; 3],
    pub face: BlockFace,
    pub hit_vec: [f64; 3],
    pub local_hit: [f32; 3],
    pub inside_block: bool,
    pub distance: f64,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct EntityHit {
    pub entity_id: i32,
    pub hit_vec: [f64; 3],
    pub distance: f64,
}

pub fn look_direction(yaw: f32, pitch: f32) -> [f64; 3] {
    let yaw_rad = (-yaw - 180.0).to_radians();
    let pitch_rad = (-pitch).to_radians();
    let cos_pitch = -pitch_rad.cos();
    let sin_pitch = pitch_rad.sin();
    let sin_yaw = yaw_rad.sin();
    let cos_yaw = yaw_rad.cos();
    [
        (sin_yaw * cos_pitch) as f64,
        sin_pitch as f64,
        (cos_yaw * cos_pitch) as f64,
    ]
}

/// Intersects a ray `origin + t * dir` (`t in [0, max_dist]`) against an axis-aligned bounding box.
pub fn ray_intersect_aabb(
    origin: [f64; 3],
    dir: [f64; 3],
    aabb: [f64; 6],
    max_dist: f64,
) -> Option<(f64, BlockFace, bool)> {
    let min = [aabb[0], aabb[1], aabb[2]];
    let max = [aabb[3], aabb[4], aabb[5]];
    if (0..3).all(|i| origin[i] > min[i] + 1e-7 && origin[i] < max[i] - 1e-7) {
        return Some((0.0, BlockFace::Up, true));
    }
    let mut t_min = 0.0_f64;
    let mut t_max = max_dist;
    let mut hit_face = BlockFace::Up;

    let faces = [
        (BlockFace::West, BlockFace::East),
        (BlockFace::Down, BlockFace::Up),
        (BlockFace::North, BlockFace::South),
    ];

    for axis in 0..3 {
        if dir[axis].abs() < 1e-12 {
            if origin[axis] < min[axis] || origin[axis] > max[axis] {
                return None;
            }
        } else {
            let inv = 1.0 / dir[axis];
            let (mut t1, mut t2, f_near) = if dir[axis] >= 0.0 {
                (
                    (min[axis] - origin[axis]) * inv,
                    (max[axis] - origin[axis]) * inv,
                    faces[axis].0,
                )
            } else {
                (
                    (max[axis] - origin[axis]) * inv,
                    (min[axis] - origin[axis]) * inv,
                    faces[axis].1,
                )
            };
            if t1 > t2 {
                std::mem::swap(&mut t1, &mut t2);
            }
            if t1 > t_min {
                t_min = t1;
                hit_face = f_near;
            }
            t_max = t_max.min(t2);
            if t_min > t_max {
                return None;
            }
        }
    }
    (t_min <= max_dist && t_min >= 0.0).then_some((t_min, hit_face, false))
}

/// Raycasts from the player's eye position through loaded world blocks up to `block_interaction_range`.
pub fn raycast_block(player: &Player, world: &World) -> Option<BlockHit> {
    let origin = [
        player.pos[0],
        player.pos[1] + player.eye_height(),
        player.pos[2],
    ];
    let dir = look_direction(player.yaw, player.pitch);
    let max_dist = player.attributes.block_interaction_range;

    let end = [
        origin[0] + dir[0] * max_dist,
        origin[1] + dir[1] * max_dist,
        origin[2] + dir[2] * max_dist,
    ];
    let min_v = [
        origin[0].min(end[0]).floor() as i32,
        origin[1].min(end[1]).floor() as i32,
        origin[2].min(end[2]).floor() as i32,
    ];
    let max_v = [
        origin[0].max(end[0]).floor() as i32,
        origin[1].max(end[1]).floor() as i32,
        origin[2].max(end[2]).floor() as i32,
    ];

    let mut best: Option<BlockHit> = None;
    for x in min_v[0]..=max_v[0] {
        for y in min_v[1]..=max_v[1] {
            for z in min_v[2]..=max_v[2] {
                let Ok(boxes) = world.collision_boxes(x, y, z) else {
                    continue;
                };
                for b in boxes {
                    let limit = best.map_or(max_dist, |h| h.distance);
                    if let Some((dist, face, inside)) = ray_intersect_aabb(origin, dir, b, limit) {
                        let hit_vec = [
                            origin[0] + dir[0] * dist,
                            origin[1] + dir[1] * dist,
                            origin[2] + dir[2] * dist,
                        ];
                        let local_hit = [
                            (hit_vec[0] - x as f64).clamp(0.0, 1.0) as f32,
                            (hit_vec[1] - y as f64).clamp(0.0, 1.0) as f32,
                            (hit_vec[2] - z as f64).clamp(0.0, 1.0) as f32,
                        ];
                        best = Some(BlockHit {
                            block_pos: [x, y, z],
                            face,
                            hit_vec,
                            local_hit,
                            inside_block: inside,
                            distance: dist,
                        });
                    }
                }
            }
        }
    }
    best
}

/// Raycasts against candidate entity AABBs `(entity_id, aabb)`, enforcing both `entity_interaction_range`
/// and line-of-sight block occlusion (treating unloaded chunks conservatively as occluders).
pub fn raycast_entity(
    player: &Player,
    world: &World,
    entities: &[(i32, [f64; 6])],
) -> Option<EntityHit> {
    let origin = [
        player.pos[0],
        player.pos[1] + player.eye_height(),
        player.pos[2],
    ];
    let dir = look_direction(player.yaw, player.pitch);
    let mut max_dist = player.attributes.entity_interaction_range;

    let end = [
        origin[0] + dir[0] * max_dist,
        origin[1] + dir[1] * max_dist,
        origin[2] + dir[2] * max_dist,
    ];
    let min_v = [
        origin[0].min(end[0]).floor() as i32,
        origin[1].min(end[1]).floor() as i32,
        origin[2].min(end[2]).floor() as i32,
    ];
    let max_v = [
        origin[0].max(end[0]).floor() as i32,
        origin[1].max(end[1]).floor() as i32,
        origin[2].max(end[2]).floor() as i32,
    ];

    let mut occlusion_dist = max_dist;
    for x in min_v[0]..=max_v[0] {
        for y in min_v[1]..=max_v[1] {
            for z in min_v[2]..=max_v[2] {
                match world.collision_boxes(x, y, z) {
                    Ok(boxes) => {
                        for b in boxes {
                            if let Some((dist, _, _)) = ray_intersect_aabb(origin, dir, b, occlusion_dist) {
                                occlusion_dist = occlusion_dist.min(dist);
                            }
                        }
                    }
                    Err(_) => {
                        // Conservative occlusion: unloaded or unknown block treated as solid unit cube
                        let unit_box = [x as f64, y as f64, z as f64, (x + 1) as f64, (y + 1) as f64, (z + 1) as f64];
                        if let Some((dist, _, _)) = ray_intersect_aabb(origin, dir, unit_box, occlusion_dist) {
                            occlusion_dist = occlusion_dist.min(dist);
                        }
                    }
                }
            }
        }
    }
    max_dist = max_dist.min(occlusion_dist);

    let mut best = None;
    for &(entity_id, aabb) in entities {
        if let Some((dist, _, _)) = ray_intersect_aabb(origin, dir, aabb, max_dist) {
            max_dist = dist;
            best = Some(EntityHit {
                entity_id,
                hit_vec: [
                    origin[0] + dir[0] * dist,
                    origin[1] + dir[1] * dist,
                    origin[2] + dir[2] * dist,
                ],
                distance: dist,
            });
        }
    }
    best
}

/// Checks whether an unobstructed line of sight exists between `origin` and `target` in `world`.
/// Returns `false` if any solid block collision box intersects the segment.
pub fn has_line_of_sight(origin: [f64; 3], target: [f64; 3], world: &World) -> bool {
    has_line_of_sight_excluding(origin, target, world, None)
}

/// Returns `false` if any solid block collision box (excluding an optional target block) intersects the segment.
pub fn has_line_of_sight_excluding(
    origin: [f64; 3],
    target: [f64; 3],
    world: &World,
    exclude_block: Option<[i32; 3]>,
) -> bool {
    let delta = [target[0] - origin[0], target[1] - origin[1], target[2] - origin[2]];
    let dist = (delta[0] * delta[0] + delta[1] * delta[1] + delta[2] * delta[2]).sqrt();
    if dist < 1e-4 {
        return true;
    }
    let dir = [delta[0] / dist, delta[1] / dist, delta[2] / dist];
    let min_v = [
        origin[0].min(target[0]).floor() as i32,
        origin[1].min(target[1]).floor() as i32,
        origin[2].min(target[2]).floor() as i32,
    ];
    let max_v = [
        origin[0].max(target[0]).floor() as i32,
        origin[1].max(target[1]).floor() as i32,
        origin[2].max(target[2]).floor() as i32,
    ];

    for x in min_v[0]..=max_v[0] {
        for y in min_v[1]..=max_v[1] {
            for z in min_v[2]..=max_v[2] {
                if exclude_block == Some([x, y, z]) {
                    continue;
                }
                match world.collision_boxes(x, y, z) {
                    Ok(boxes) => {
                        for b in boxes {
                            if let Some((hit_dist, _, _)) = ray_intersect_aabb(origin, dir, b, dist) {
                                if hit_dist < dist - 0.05 {
                                    return false;
                                }
                            }
                        }
                    }
                    Err(_) => {
                        let unit_box = [
                            x as f64,
                            y as f64,
                            z as f64,
                            (x + 1) as f64,
                            (y + 1) as f64,
                            (z + 1) as f64,
                        ];
                        if let Some((hit_dist, _, _)) = ray_intersect_aabb(origin, dir, unit_box, dist) {
                            if hit_dist < dist - 0.05 {
                                return false;
                            }
                        }
                    }
                }
            }
        }
    }
    true
}

/// Computes per-tick block breaking progress (`0.0..=1.0`) matching Minecraft 26.3 rules.
pub fn break_progress_per_tick(
    player: &Player,
    block_hardness: f32,
    can_harvest: bool,
    tool_speed: f32,
    efficiency_level: u8,
    in_water_without_aqua_affinity: bool,
) -> f32 {
    if block_hardness < 0.0 {
        return 0.0;
    }
    if block_hardness == 0.0 {
        return 1.0;
    }
    let mut speed = tool_speed;
    if speed > 1.0 && efficiency_level > 0 {
        speed += f32::from(efficiency_level) * f32::from(efficiency_level) + 1.0;
    }
    if let Some(haste) = player.effects.haste_amplifier {
        speed *= 1.0 + 0.2 * f32::from(haste + 1);
    }
    if let Some(fatigue) = player.effects.mining_fatigue_amplifier {
        let factor = match fatigue {
            0 => 0.3,
            1 => 0.09,
            2 => 0.0027,
            _ => 0.00081,
        };
        speed *= factor;
    }
    speed *= player.attributes.block_break_speed;
    if in_water_without_aqua_affinity {
        speed /= 5.0;
    }
    if !player.ground {
        speed /= 5.0;
    }
    let divisor = if can_harvest { 30.0 } else { 100.0 };
    speed / block_hardness / divisor
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn raycast_finds_block_face_and_enforces_occlusion_on_entities() {
        let mut world = World::new(&json!({"minecraft":"26.3","air":[0],"solid":[1]})).unwrap();
        world.min_y = -64;
        world.height = 32;
        world.chunks.insert(
            (0, 0),
            vec![
                crate::world::Palette::single(1),
                crate::world::Palette::single(0),
            ],
        );
        // Place solid block at (0, -47, 2) in front of the player at (0.5, -48.0, 0.5) looking +Z (yaw = 0)
        world.set_block(0, -47, 2, 1).unwrap();
        let player = Player {
            pos: [0.5, -48.0, 0.5],
            yaw: 0.0,
            pitch: 0.0,
            ground: true,
            ..Default::default()
        };
        let hit = raycast_block(&player, &world).unwrap();
        assert_eq!(hit.block_pos, [0, -47, 2]);
        assert_eq!(hit.face, BlockFace::North);

        // Entity behind the wall at z = 3.0 is occluded by the block at z = 2.0
        let occluded_entity = [(42, [0.2, -48.0, 2.8, 0.8, -46.2, 3.4])];
        assert!(raycast_entity(&player, &world, &occluded_entity).is_none());

        // Entity in front of the wall at z = 1.5 is hit
        let visible_entity = [(43, [0.2, -48.0, 1.2, 0.8, -46.2, 1.8])];
        let ehit = raycast_entity(&player, &world, &visible_entity).unwrap();
        assert_eq!(ehit.entity_id, 43);
    }
}
