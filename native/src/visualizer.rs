//! Pure-Rust ultra-optimized software 3D low-poly rasterizer and retro visualizer.
//!
//! Provides a zero-JVM, zero-graphics-driver, stripped-down visual client mode
//! that directly renders the voxel world, player orientation, and entities
//! into a low-resolution retro buffer (e.g. 160x100 or 320x200) with flat shading.

#![allow(dead_code)]

use crate::player::Player;
use crate::world::World;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CameraMode {
    FirstPerson,
    ThirdPersonChase,
    TopDownRadar,
}

#[derive(Clone, Debug)]
pub struct EntityRenderData {
    pub entity_id: i32,
    pub pos: [f64; 3],
    pub kind: String,
}

pub struct LowPolyVisualizer {
    pub width: usize,
    pub height: usize,
    pub max_dist: f64,
    pub camera_mode: CameraMode,
}

impl Default for LowPolyVisualizer {
    fn default() -> Self {
        Self {
            width: 160,
            height: 100,
            max_dist: 36.0,
            camera_mode: CameraMode::FirstPerson,
        }
    }
}

pub struct RenderedFrame {
    pub width: usize,
    pub height: usize,
    pub pixels: Vec<u32>, // 0x00RRGGBB
}

impl RenderedFrame {
    pub fn new(width: usize, height: usize, clear_color: u32) -> Self {
        Self {
            width,
            height,
            pixels: vec![clear_color; width * height],
        }
    }

    #[inline(always)]
    pub fn set_pixel(&mut self, x: usize, y: usize, color: u32) {
        if x < self.width && y < self.height {
            self.pixels[y * self.width + x] = color;
        }
    }

    /// Encodes the pixel buffer into an uncompressed BMP image (Windows Bitmap 24-bpp)
    /// which is natively displayable in all web browsers without external dependencies.
    pub fn to_bmp_bytes(&self) -> Vec<u8> {
        let row_size = (self.width * 3 + 3) & !3; // padded to 4 bytes
        let pixel_data_size = row_size * self.height;
        let file_size = 54 + pixel_data_size;

        let mut buf = Vec::with_capacity(file_size);

        // BITMAPFILEHEADER (14 bytes)
        buf.extend_from_slice(b"BM");
        buf.extend_from_slice(&(file_size as u32).to_le_bytes());
        buf.extend_from_slice(&[0, 0, 0, 0]); // reserved
        buf.extend_from_slice(&54u32.to_le_bytes()); // offset to pixel array

        // BITMAPINFOHEADER (40 bytes)
        buf.extend_from_slice(&40u32.to_le_bytes()); // header size
        buf.extend_from_slice(&(self.width as i32).to_le_bytes());
        // Bottom-up bitmap: positive height
        buf.extend_from_slice(&(self.height as i32).to_le_bytes());
        buf.extend_from_slice(&1u16.to_le_bytes()); // color planes
        buf.extend_from_slice(&24u16.to_le_bytes()); // bits per pixel (24bpp BGR)
        buf.extend_from_slice(&0u32.to_le_bytes()); // compression (BI_RGB)
        buf.extend_from_slice(&(pixel_data_size as u32).to_le_bytes());
        buf.extend_from_slice(&2835u32.to_le_bytes()); // ~72 DPI horiz
        buf.extend_from_slice(&2835u32.to_le_bytes()); // ~72 DPI vert
        buf.extend_from_slice(&0u32.to_le_bytes()); // colors in palette
        buf.extend_from_slice(&0u32.to_le_bytes()); // important colors

        // Pixel Array (Bottom-up, BGR order)
        let pad_len = row_size - (self.width * 3);
        let pad = [0u8; 4];
        for y in (0..self.height).rev() {
            let row_offset = y * self.width;
            for x in 0..self.width {
                let color = self.pixels[row_offset + x];
                let r = ((color >> 16) & 0xFF) as u8;
                let g = ((color >> 8) & 0xFF) as u8;
                let b = (color & 0xFF) as u8;
                buf.push(b);
                buf.push(g);
                buf.push(r);
            }
            if pad_len > 0 {
                buf.extend_from_slice(&pad[..pad_len]);
            }
        }
        buf
    }

    /// Converts the rendered BMP into a base64 Data URI (`data:image/bmp;base64,...`)
    pub fn to_base64_data_uri(&self) -> String {
        let bytes = self.to_bmp_bytes();
        let b64 = base64_encode(&bytes);
        format!("data:image/bmp;base64,{b64}")
    }
}

impl LowPolyVisualizer {
    pub fn new(width: usize, height: usize, camera_mode: CameraMode) -> Self {
        Self {
            width,
            height,
            max_dist: 36.0,
            camera_mode,
        }
    }

    /// Renders a full frame from player position and loaded world chunks.
    pub fn render(
        &self,
        player: &Player,
        world: &World,
        entities: &[EntityRenderData],
    ) -> RenderedFrame {
        let mut frame = RenderedFrame::new(self.width, self.height, 0x090D16); // deep dark sky

        match self.camera_mode {
            CameraMode::TopDownRadar => {
                self.render_topdown(player, world, entities, &mut frame);
            }
            CameraMode::FirstPerson | CameraMode::ThirdPersonChase => {
                self.render_3d(player, world, entities, &mut frame);
            }
        }

        // Draw crosshair at center
        let cx = self.width / 2;
        let cy = self.height / 2;
        for dx in -2..=2 {
            frame.set_pixel(
                (cx as isize + dx).clamp(0, self.width as isize - 1) as usize,
                cy,
                0x58A6FF,
            );
        }
        for dy in -2..=2 {
            frame.set_pixel(
                cx,
                (cy as isize + dy).clamp(0, self.height as isize - 1) as usize,
                0x58A6FF,
            );
        }

        frame
    }

    fn render_3d(
        &self,
        player: &Player,
        world: &World,
        entities: &[EntityRenderData],
        frame: &mut RenderedFrame,
    ) {
        let (cam_x, cam_y, cam_z) = if self.camera_mode == CameraMode::ThirdPersonChase {
            let yaw_rad = (-player.yaw - 180.0).to_radians() as f64;
            let dir_x = yaw_rad.sin();
            let dir_z = yaw_rad.cos();
            (
                player.pos[0] - dir_x * 3.2,
                player.pos[1] + 2.0,
                player.pos[2] - dir_z * 3.2,
            )
        } else {
            (player.pos[0], player.pos[1] + 1.62, player.pos[2])
        };

        let fov_rad = 70.0_f64.to_radians();
        let aspect = self.width as f64 / self.height as f64;
        let half_w = (fov_rad / 2.0).tan();
        let half_h = half_w / aspect;

        let yaw_rad = (-player.yaw - 180.0).to_radians() as f64;
        let pitch_rad = (-player.pitch).to_radians() as f64;

        let forward = [
            yaw_rad.sin() * pitch_rad.cos(),
            pitch_rad.sin(),
            yaw_rad.cos() * pitch_rad.cos(),
        ];
        let right = [yaw_rad.cos(), 0.0, -yaw_rad.sin()];
        let up = [
            -yaw_rad.sin() * pitch_rad.sin(),
            pitch_rad.cos(),
            -yaw_rad.cos() * pitch_rad.sin(),
        ];

        // Raymarch voxels per pixel
        for py in 0..self.height {
            let ndc_y = 1.0 - (2.0 * (py as f64 + 0.5) / self.height as f64);
            let vy = ndc_y * half_h;
            for px in 0..self.width {
                let ndc_x = (2.0 * (px as f64 + 0.5) / self.width as f64) - 1.0;
                let vx = ndc_x * half_w;

                let dir = [
                    forward[0] + vx * right[0] + vy * up[0],
                    forward[1] + vx * right[1] + vy * up[1],
                    forward[2] + vx * right[2] + vy * up[2],
                ];
                let len = (dir[0] * dir[0] + dir[1] * dir[1] + dir[2] * dir[2]).sqrt();
                let dir_norm = [dir[0] / len, dir[1] / len, dir[2] / len];

                if let Some((hit_pos, normal, dist, block_id)) =
                    self.raycast_voxel([cam_x, cam_y, cam_z], dir_norm, world, self.max_dist)
                {
                    let raw_color = get_block_base_color(block_id, hit_pos[1]);
                    let shaded = apply_flat_shading(raw_color, normal, dist, self.max_dist);
                    frame.set_pixel(px, py, shaded);
                }
            }
        }

        // Project nearby entities as low-poly billboards
        for ent in entities {
            let dx = ent.pos[0] - cam_x;
            let dy = ent.pos[1] + 0.8 - cam_y;
            let dz = ent.pos[2] - cam_z;

            // Transform into camera space
            let p_forward = dx * forward[0] + dy * forward[1] + dz * forward[2];
            if p_forward < 0.5 || p_forward > self.max_dist {
                continue;
            }
            let p_right = dx * right[0] + dy * right[1] + dz * right[2];
            let p_up = dx * up[0] + dy * up[1] + dz * up[2];

            let screen_x = ((p_right / (p_forward * half_w)) + 1.0) * 0.5 * self.width as f64;
            let screen_y = ((1.0 - (p_up / (p_forward * half_h)))) * 0.5 * self.height as f64;

            let size = ((1.2 / p_forward) * self.height as f64).clamp(2.0, 30.0) as isize;
            let color = match ent.kind.as_str() {
                "zombie" | "skeleton" | "creeper" | "spider" => 0xEF4444, // Hostile (Red)
                "cow" | "pig" | "sheep" | "chicken" => 0x22C55E,          // Passive (Green)
                "item" => 0xF59E0B,                                       // Dropped item (Gold)
                _ => 0xBC8CFF,                                            // Other entity (Purple)
            };

            let ix = screen_x as isize;
            let iy = screen_y as isize;
            for ey in (iy - size / 2)..=(iy + size / 2) {
                for ex in (ix - size / 2)..=(ix + size / 2) {
                    if ex >= 0 && ex < self.width as isize && ey >= 0 && ey < self.height as isize {
                        frame.set_pixel(ex as usize, ey as usize, color);
                    }
                }
            }
        }
    }

    fn render_topdown(
        &self,
        player: &Player,
        world: &World,
        entities: &[EntityRenderData],
        frame: &mut RenderedFrame,
    ) {
        let center_x = player.pos[0];
        let center_y = player.pos[1].floor() as i32;
        let center_z = player.pos[2];
        let radius = (self.width.min(self.height) / 2) as i32;

        for py in 0..self.height {
            let rel_z = (py as i32) - (self.height as i32 / 2);
            let world_z = center_z.floor() as i32 + rel_z;
            for px in 0..self.width {
                let rel_x = (px as i32) - (self.width as i32 / 2);
                let world_x = center_x.floor() as i32 + rel_x;

                // Scan down from player height for top solid block
                let mut found_color = 0x111827; // void dark
                for dy in (-3..=3).rev() {
                    let y = center_y + dy;
                    if let Ok(state) = world.block_state(world_x, y, world_z) {
                        if state != 0 {
                            let base = get_block_base_color(state, y);
                            let depth_factor = match dy {
                                1..=3 => 1.15,
                                0 => 1.0,
                                _ => 0.85,
                            };
                            found_color = scale_color(base, depth_factor);
                            break;
                        }
                    }
                }
                frame.set_pixel(px, py, found_color);
            }
        }

        // Draw entities as radar pings
        for ent in entities {
            let rel_x = (ent.pos[0] - center_x) as isize + (self.width as isize / 2);
            let rel_z = (ent.pos[2] - center_z) as isize + (self.height as isize / 2);
            if rel_x >= 1
                && rel_x < (self.width as isize - 1)
                && rel_z >= 1
                && rel_z < (self.height as isize - 1)
            {
                let col = if ent.kind.contains("zombie") || ent.kind.contains("creeper") {
                    0xEF4444
                } else {
                    0x22C55E
                };
                frame.set_pixel(rel_x as usize, rel_z as usize, col);
                frame.set_pixel((rel_x + 1) as usize, rel_z as usize, col);
                frame.set_pixel(rel_x as usize, (rel_z + 1) as usize, col);
            }
        }

        // Draw player indicator in center with direction pointer
        let cx = self.width / 2;
        let cy = self.height / 2;
        let yaw_rad = (-player.yaw - 180.0).to_radians() as f64;
        let dir_x = yaw_rad.sin();
        let dir_z = yaw_rad.cos();

        for r in 0..5 {
            let px = (cx as f64 + dir_x * (r as f64)) as usize;
            let py = (cy as f64 + dir_z * (r as f64)) as usize;
            frame.set_pixel(px, py, 0x38BDF8);
        }
        frame.set_pixel(cx, cy, 0xFFFFFF);
    }

    /// Fast Amanatides & Woo 3D integer voxel DDA raymarcher.
    fn raycast_voxel(
        &self,
        origin: [f64; 3],
        dir: [f64; 3],
        world: &World,
        max_dist: f64,
    ) -> Option<([i32; 3], [i8; 3], f64, u32)> {
        let mut x = origin[0].floor() as i32;
        let mut y = origin[1].floor() as i32;
        let mut z = origin[2].floor() as i32;

        let step_x = if dir[0] > 0.0 { 1 } else { -1 };
        let step_y = if dir[1] > 0.0 { 1 } else { -1 };
        let step_z = if dir[2] > 0.0 { 1 } else { -1 };

        let inv_dx = if dir[0].abs() > 1e-6 {
            1.0 / dir[0].abs()
        } else {
            1e9
        };
        let inv_dy = if dir[1].abs() > 1e-6 {
            1.0 / dir[1].abs()
        } else {
            1e9
        };
        let inv_dz = if dir[2].abs() > 1e-6 {
            1.0 / dir[2].abs()
        } else {
            1e9
        };

        let mut t_max_x = if dir[0] > 0.0 {
            ((x + 1) as f64 - origin[0]) * inv_dx
        } else {
            (origin[0] - x as f64) * inv_dx
        };
        let mut t_max_y = if dir[1] > 0.0 {
            ((y + 1) as f64 - origin[1]) * inv_dy
        } else {
            (origin[1] - y as f64) * inv_dy
        };
        let mut t_max_z = if dir[2] > 0.0 {
            ((z + 1) as f64 - origin[2]) * inv_dz
        } else {
            (origin[2] - z as f64) * inv_dz
        };

        let delta_x = inv_dx;
        let delta_y = inv_dy;
        let delta_z = inv_dz;

        let mut dist = 0.0;
        let mut normal = [0i8, 1, 0];

        while dist < max_dist {
            if let Ok(block_id) = world.block_state(x, y, z) {
                if block_id != 0 {
                    return Some(([x, y, z], normal, dist, block_id));
                }
            }

            if t_max_x < t_max_y {
                if t_max_x < t_max_z {
                    dist = t_max_x;
                    t_max_x += delta_x;
                    x += step_x;
                    normal = [-step_x as i8, 0, 0];
                } else {
                    dist = t_max_z;
                    t_max_z += delta_z;
                    z += step_z;
                    normal = [0, 0, -step_z as i8];
                }
            } else if t_max_y < t_max_z {
                dist = t_max_y;
                t_max_y += delta_y;
                y += step_y;
                normal = [0, -step_y as i8, 0];
            } else {
                dist = t_max_z;
                t_max_z += delta_z;
                z += step_z;
                normal = [0, 0, -step_z as i8];
            }
        }
        None
    }
}

fn get_block_base_color(block_id: u32, y: i32) -> u32 {
    // Color mapping by block type or depth
    match block_id {
        // Dirt / Grass
        b if b == 1 || b == 2 => 0x22C55E, // Grass green
        b if b == 3 || b == 4 => 0x92400E, // Dirt brown
        // Stone / Cobblestone / Deepslate
        b if (5..=20).contains(&b) => {
            if y < 0 {
                0x334155 // Deepslate dark slate
            } else {
                0x64748B // Stone gray
            }
        }
        // Wood / Logs
        b if (21..=40).contains(&b) => 0xB45309, // Oak wood
        // Leaves
        b if (41..=60).contains(&b) => 0x15803D, // Leaves green
        // Water
        b if b == 80 || b == 81 => 0x0284C7, // Water cyan
        // Lava
        b if b == 90 || b == 91 => 0xEA580C, // Lava orange
        _ => {
            if y < -30 {
                0x1E293B
            } else {
                0x475569
            }
        }
    }
}

fn apply_flat_shading(base_color: u32, normal: [i8; 3], dist: f64, max_dist: f64) -> u32 {
    // Directional shading factor
    let light = if normal[1] > 0 {
        1.15 // top face illuminated by sun
    } else if normal[1] < 0 {
        0.55 // bottom face in shadow
    } else if normal[0] != 0 {
        0.80 // side face
    } else {
        0.95 // other side
    };

    // Distance fog towards dark night sky (#090D16)
    let fog = (1.0 - (dist / max_dist).clamp(0.0, 1.0)).powf(1.4);
    let r = (((base_color >> 16) & 0xFF) as f64 * light * fog + 9.0 * (1.0 - fog)) as u32;
    let g = (((base_color >> 8) & 0xFF) as f64 * light * fog + 13.0 * (1.0 - fog)) as u32;
    let b = ((base_color & 0xFF) as f64 * light * fog + 22.0 * (1.0 - fog)) as u32;

    (r.min(255) << 16) | (g.min(255) << 8) | b.min(255)
}

fn scale_color(color: u32, scale: f64) -> u32 {
    let r = (((color >> 16) & 0xFF) as f64 * scale).clamp(0.0, 255.0) as u32;
    let g = (((color >> 8) & 0xFF) as f64 * scale).clamp(0.0, 255.0) as u32;
    let b = ((color & 0xFF) as f64 * scale).clamp(0.0, 255.0) as u32;
    (r << 16) | (g << 8) | b
}

fn base64_encode(data: &[u8]) -> String {
    const TABLE: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::with_capacity((data.len() + 2) / 3 * 4);
    let mut chunks = data.chunks_exact(3);
    for chunk in &mut chunks {
        let n = ((chunk[0] as u32) << 16) | ((chunk[1] as u32) << 8) | (chunk[2] as u32);
        out.push(TABLE[((n >> 18) & 63) as usize] as char);
        out.push(TABLE[((n >> 12) & 63) as usize] as char);
        out.push(TABLE[((n >> 6) & 63) as usize] as char);
        out.push(TABLE[(n & 63) as usize] as char);
    }
    let rem = chunks.remainder();
    if rem.len() == 1 {
        let n = (rem[0] as u32) << 16;
        out.push(TABLE[((n >> 18) & 63) as usize] as char);
        out.push(TABLE[((n >> 12) & 63) as usize] as char);
        out.push('=');
        out.push('=');
    } else if rem.len() == 2 {
        let n = ((rem[0] as u32) << 16) | ((rem[1] as u32) << 8);
        out.push(TABLE[((n >> 18) & 63) as usize] as char);
        out.push(TABLE[((n >> 12) & 63) as usize] as char);
        out.push(TABLE[((n >> 6) & 63) as usize] as char);
        out.push('=');
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn visualizer_renders_valid_bmp() {
        let player = Player::default();
        let block_profile: serde_json::Value = serde_json::json!({
            "air": [0],
            "solid": [1, 2, 3]
        });
        let world = World::new(&block_profile).unwrap();
        let viz = LowPolyVisualizer::new(80, 50, CameraMode::FirstPerson);
        let frame = viz.render(&player, &world, &[]);
        let bmp = frame.to_bmp_bytes();
        assert_eq!(&bmp[0..2], b"BM");
        assert!(bmp.len() > 54);
    }
}
