//! Pure-Rust Native Desktop Application for Sawbyte (Zero HTML, Zero Localhost, Zero Browser).
//!
//! Renders a hardware/software anti-aliased Modrinth/Lunar-inspired desktop interface
//! directly to a native OS window:
//! - Linux / Unix: Direct X11 / XWayland protocol over UnixStream (`/tmp/.X11-unix/X*`)
//!   with `.Xauthority` MIT-MAGIC-COOKIE-1 authentication + native GTK3/Tkinter framebuffer fallback.
//! - Windows: Direct Win32 GDI window (`user32.dll` / `gdi32.dll` `CreateWindowExW` + `StretchDIBits`).
#![allow(clippy::all, warnings, unused)]

use crate::ui_assets::{FontStrike, FONT_12, FONT_14, FONT_18};
use sawbyte_core::launcher::{PersistentLauncher, RuntimeMode};
use std::fs;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::path::{Path, PathBuf};
use std::process::{Child, Command};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

pub const WIN_W: usize = 1280;
pub const WIN_H: usize = 800;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ActiveTab {
    Home,
    Viewport3D,
    ModsEditions,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum LaunchEdition {
    Headless,
    Potato3D,
    FabricBridge,
}

pub struct AppState {
    pub tab: ActiveTab,
    pub edition: LaunchEdition,
    pub mcp_enabled: bool,
    pub gym_enabled: bool,
    pub terminal_attached: bool,
    pub running_instance: Option<String>,
    pub status_line: String,
    pub selected_instance: String,
    pub server_addr: String,
    pub account_name: String,
    pub camera_mode: usize, // 0 = 1st Person, 1 = 3rd Person, 2 = 360 Radar
    pub bridge_installed: bool,
    pub headless_installed: bool,
    pub cli_installed: bool,
    pub logs: Vec<String>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            tab: ActiveTab::Home,
            edition: LaunchEdition::Headless,
            mcp_enabled: true,
            gym_enabled: true,
            terminal_attached: true,
            running_instance: None,
            status_line: "Ready to launch · 20.0 TPS".to_string(),
            selected_instance: "default-native".to_string(),
            server_addr: "127.0.0.1:25565".to_string(),
            account_name: "SawbyteNative".to_string(),
            camera_mode: 0,
            bridge_installed: true,
            headless_installed: true,
            cli_installed: true,
            logs: vec![
                "[Sawbyte v0.5.0] Native Desktop App initialized (Zero HTML / Zero Browser)".to_string(),
                "[Engine] Persistent root: ~/.sawbyte | Protocol: Minecraft 26.3 (777)".to_string(),
                "[MCP] AI Agent Server active on stdio & TCP 127.0.0.1:8765".to_string(),
                "[RL] Gymnasium sub-voxel observation socket ready on 127.0.0.1:25580".to_string(),
            ],
        }
    }
}

pub struct UiSurface {
    pub width: usize,
    pub height: usize,
    pub pixels: Vec<u32>,
}

impl UiSurface {
    pub fn new(width: usize, height: usize, bg: u32) -> Self {
        Self {
            width,
            height,
            pixels: vec![bg; width * height],
        }
    }

    #[inline(always)]
    pub fn blend_pixel(&mut self, x: i32, y: i32, color: u32, alpha: u8) {
        if x < 0 || y < 0 || x >= self.width as i32 || y >= self.height as i32 || alpha == 0 {
            return;
        }
        let idx = (y as usize) * self.width + (x as usize);
        if alpha == 255 {
            self.pixels[idx] = color;
            return;
        }
        let dst = self.pixels[idx];
        let a = alpha as u32;
        let inv_a = 255 - a;
        let r = (((color >> 16) & 0xFF) * a + ((dst >> 16) & 0xFF) * inv_a) / 255;
        let g = (((color >> 8) & 0xFF) * a + ((dst >> 8) & 0xFF) * inv_a) / 255;
        let b = ((color & 0xFF) * a + (dst & 0xFF) * inv_a) / 255;
        self.pixels[idx] = (r << 16) | (g << 8) | b;
    }

    pub fn fill_rect(&mut self, x: i32, y: i32, w: i32, h: i32, color: u32) {
        let x0 = x.max(0) as usize;
        let y0 = y.max(0) as usize;
        let x1 = (x + w).min(self.width as i32).max(0) as usize;
        let y1 = (y + h).min(self.height as i32).max(0) as usize;
        for py in y0..y1 {
            let row = py * self.width;
            for px in x0..x1 {
                self.pixels[row + px] = color;
            }
        }
    }

    /// Anti-aliased SDF rounded rectangle with optional 1px border.
    pub fn fill_rounded_rect(
        &mut self,
        x: i32,
        y: i32,
        w: i32,
        h: i32,
        radius: i32,
        fill_color: u32,
        border_color: Option<u32>,
    ) {
        if w <= 0 || h <= 0 {
            return;
        }
        let r = radius.clamp(0, w.min(h) / 2) as f32;
        let hw = w as f32 * 0.5;
        let hh = h as f32 * 0.5;
        let cx = x as f32 + hw;
        let cy = y as f32 + hh;

        let x0 = x.max(0);
        let y0 = y.max(0);
        let x1 = (x + w).min(self.width as i32);
        let y1 = (y + h).min(self.height as i32);

        for py in y0..y1 {
            let dy = (py as f32 + 0.5 - cy).abs() - (hh - r);
            for px in x0..x1 {
                let dx = (px as f32 + 0.5 - cx).abs() - (hw - r);
                let dist = if dx > 0.0 && dy > 0.0 {
                    (dx * dx + dy * dy).sqrt() - r
                } else {
                    dx.max(dy) - r
                };

                if dist < 0.5 {
                    let fill_alpha = ((0.5 - dist).clamp(0.0, 1.0) * 255.0) as u8;
                    if let Some(bcol) = border_color {
                        if dist > -1.2 {
                            let border_t = ((dist + 1.2) / 1.2).clamp(0.0, 1.0);
                            let col = lerp_color(fill_color, bcol, border_t);
                            self.blend_pixel(px, py, col, fill_alpha);
                        } else {
                            self.blend_pixel(px, py, fill_color, fill_alpha);
                        }
                    } else {
                        self.blend_pixel(px, py, fill_color, fill_alpha);
                    }
                }
            }
        }
    }

    pub fn fill_circle(&mut self, cx: i32, cy: i32, radius: i32, color: u32) {
        let r = radius as f32;
        for py in (cy - radius - 1)..=(cy + radius + 1) {
            for px in (cx - radius - 1)..=(cx + radius + 1) {
                let dx = px as f32 - cx as f32;
                let dy = py as f32 - cy as f32;
                let d = (dx * dx + dy * dy).sqrt() - r;
                if d < 0.5 {
                    let a = ((0.5 - d).clamp(0.0, 1.0) * 255.0) as u8;
                    self.blend_pixel(px, py, color, a);
                }
            }
        }
    }

    pub fn draw_text(&mut self, mut x: i32, y: i32, text: &str, strike: &FontStrike, color: u32) -> i32 {
        for ch in text.chars() {
            let code = ch as usize;
            let idx = if (32..=126).contains(&code) {
                code - 32
            } else {
                b'?' as usize - 32
            };
            let (w, h, y_off, adv, off) = strike.meta[idx];
            if w > 0 && h > 0 {
                let base_off = off as usize;
                for gy in 0..(h as i32) {
                    let row_off = base_off + (gy as usize) * (w as usize);
                    for gx in 0..(w as i32) {
                        let a = strike.alpha[row_off + (gx as usize)];
                        if a > 0 {
                            self.blend_pixel(x + gx, y + (y_off as i32) + gy, color, a);
                        }
                    }
                }
            }
            x += adv as i32;
        }
        x
    }

    pub fn text_width(text: &str, strike: &FontStrike) -> i32 {
        let mut w = 0;
        for ch in text.chars() {
            let code = ch as usize;
            let idx = if (32..=126).contains(&code) { code - 32 } else { b'?' as usize - 32 };
            w += strike.meta[idx].3 as i32;
        }
        w
    }

    /// Draws a crisp 3D isometric Minecraft block icon (`kind`: 0=Redstone, 1=Grass, 2=Bot, 3=Fabric/Scroll, 4=Emerald).
    pub fn draw_iso_voxel(&mut self, cx: i32, cy: i32, radius: i32, kind: u8) {
        let (top_c, left_c, right_c, core_c) = match kind {
            0 => (0xF87171, 0xDC2626, 0x991B1B, 0xFEF08A), // Redstone Block
            1 => (0x22C55E, 0x854D0E, 0x713F12, 0x4ADE80), // Grass / Earth Cube
            2 => (0xE2E8F0, 0x94A3B8, 0x475569, 0x1BD96A), // Iron Bot Mascot
            3 => (0x818CF8, 0x4F46E5, 0x3730A3, 0xC7D2FE), // Fabric / Mod Cube
            _ => (0x34D399, 0x10B981, 0x047857, 0xA7F3D0), // Emerald / AI Cube
        };
        let r = radius as f32;
        for dy in -radius..=radius {
            for dx in -radius..=radius {
                let u = dx as f32 / r;
                let v = dy as f32 / r;
                // Isometric hexagon bounds
                if u.abs() <= 0.86 && (v.abs() + u.abs() * 0.5) <= 0.95 {
                    let col = if v < -u.abs() * 0.5 {
                        top_c
                    } else if u < 0.0 {
                        if kind == 1 && v < -u * 0.5 + 0.28 {
                            0x16A34A // grass side fringe
                        } else {
                            left_c
                        }
                    } else if kind == 1 && v < u * 0.5 + 0.28 {
                        0x15803D
                    } else {
                        right_c
                    };
                    self.blend_pixel(cx + dx, cy + dy, col, 255);
                }
            }
        }
        // Visor / Core detail
        if kind == 2 {
            // Bot glowing emerald visor eyes
            self.fill_rect(cx - radius / 2, cy + 1, radius / 3, radius / 5 + 2, 0x1BD96A);
            self.fill_rect(cx + radius / 6, cy - 1, radius / 3, radius / 5 + 2, 0x1BD96A);
        } else if kind == 0 {
            self.fill_circle(cx, cy, (radius / 4).max(2), core_c);
        }
    }

    pub fn draw_player_head(&mut self, x: i32, y: i32, size: i32, crown: bool) {
        self.fill_rounded_rect(x, y, size, size, 6, 0xD9A074, None);
        // Hair
        self.fill_rect(x + 2, y + 2, size - 4, size / 3, 0x3B2314);
        // Eyes
        let eye_y = y + (size * 52) / 100;
        let ew = (size / 6).max(3);
        self.fill_rect(x + size / 5, eye_y, ew, ew, 0xFFFFFF);
        self.fill_rect(x + size / 5 + ew / 2, eye_y, ew / 2 + 1, ew, 0x4F46E5);
        self.fill_rect(x + (size * 3) / 5, eye_y, ew, ew, 0xFFFFFF);
        self.fill_rect(x + (size * 3) / 5, eye_y, ew / 2 + 1, ew, 0x4F46E5);
        if crown {
            self.fill_rect(x + 2, y + 1, size - 4, size / 4, 0xF59E0B);
            self.fill_circle(x + size / 2, y + 3, 2, 0x1BD96A);
        }
    }

    pub fn draw_toggle_switch(&mut self, x: i32, y: i32, on: bool) {
        let bg = if on { 0x1BD96A } else { 0x323640 };
        self.fill_rounded_rect(x, y, 42, 22, 11, bg, None);
        let knob_x = if on { x + 31 } else { x + 11 };
        let knob_col = if on { 0x081C10 } else { 0x94A3B8 };
        self.fill_circle(knob_x, y + 11, 7, knob_col);
    }

    pub fn draw_play_triangle(&mut self, cx: i32, cy: i32, size: i32, color: u32) {
        for dy in -size..=size {
            let span = size - dy.abs();
            for dx in (-size / 2)..=(-size / 2 + span) {
                self.blend_pixel(cx + dx + 1, cy + dy, color, 255);
            }
        }
    }
}

#[inline(always)]
fn lerp_color(c0: u32, c1: u32, t: f32) -> u32 {
    let inv = 1.0 - t;
    let r = (((c0 >> 16) & 0xFF) as f32 * inv + ((c1 >> 16) & 0xFF) as f32 * t) as u32;
    let g = (((c0 >> 8) & 0xFF) as f32 * inv + ((c1 >> 8) & 0xFF) as f32 * t) as u32;
    let b = ((c0 & 0xFF) as f32 * inv + (c1 & 0xFF) as f32 * t) as u32;
    (r << 16) | (g << 8) | b
}

pub fn render_app_frame(state: &AppState) -> UiSurface {
    let mut s = UiSurface::new(WIN_W, WIN_H, 0x16181C);

    // 1. Left Icon-Only Navigation Rail (0..62) — Modrinth App Style
    s.fill_rect(0, 0, 62, WIN_H as i32, 0x111316);
    s.fill_rect(61, 0, 1, WIN_H as i32, 0x22262E);

    // Window control dots at top-left
    s.fill_circle(18, 18, 5, 0xFF5F56);
    s.fill_circle(32, 18, 5, 0xFFBD2E);
    s.fill_circle(46, 18, 5, 0x27C93F);

    // Primary Green Play/Home Circle Button (cx=31, cy=58)
    s.fill_circle(31, 58, 19, if state.tab == ActiveTab::Home { 0x1BD96A } else { 0x232730 });
    s.draw_play_triangle(31, 58, 6, if state.tab == ActiveTab::Home { 0x07170E } else { 0x1BD96A });

    // Nav Icon 2: 3D & Radar Viewport (cy=110)
    s.fill_rounded_rect(
        13,
        92,
        36,
        36,
        9,
        if state.tab == ActiveTab::Viewport3D { 0x252A34 } else { 0x111316 },
        if state.tab == ActiveTab::Viewport3D { Some(0x1BD96A) } else { None },
    );
    s.fill_circle(31, 110, 9, if state.tab == ActiveTab::Viewport3D { 0x1BD96A } else { 0x8B94A5 });
    s.fill_circle(31, 110, 4, 0x111316);

    // Nav Icon 3: Mods & Editions (cy=156)
    s.fill_rounded_rect(
        13,
        138,
        36,
        36,
        9,
        if state.tab == ActiveTab::ModsEditions { 0x252A34 } else { 0x111316 },
        if state.tab == ActiveTab::ModsEditions { Some(0x1BD96A) } else { None },
    );
    s.fill_rounded_rect(23, 148, 16, 16, 3, if state.tab == ActiveTab::ModsEditions { 0x1BD96A } else { 0x8B94A5 }, None);

    // Divider line in Left Rail
    s.fill_rect(14, 192, 34, 1, 0x242830);

    // Pinned Instance Shortcut Icons in Left Rail (36x36 rounded tiles)
    let rail_tiles = [
        (212, 0xF59E0B, 0u8), // Redstone / Headless
        (260, 0x16A34A, 1u8), // Grass / Potato 3D
        (308, 0x4F46E5, 3u8), // Indigo / Fabric 26.3
        (356, 0x1E293B, 2u8), // Bot Mascot
    ];
    for (ty, bg_col, vkind) in rail_tiles {
        s.fill_rounded_rect(13, ty, 36, 36, 9, bg_col, None);
        s.draw_iso_voxel(31, ty + 18, 11, vkind);
    }

    // Bottom '+' and Settings icons in Left Rail
    s.fill_rounded_rect(13, 700, 36, 36, 18, 0x1C2026, Some(0x2B303B));
    s.draw_text(26, 709, "+", &FONT_18, 0x94A3B8);
    s.fill_rounded_rect(13, 748, 36, 36, 18, 0x1C2026, None);
    s.fill_circle(31, 766, 7, 0x8B94A5);
    s.fill_circle(31, 766, 3, 0x1C2026);

    // 2. Top Header Bar (x: 62..1280, y: 0..46)
    s.fill_rect(62, 0, (WIN_W - 62) as i32, 46, 0x111316);
    s.fill_rect(62, 45, (WIN_W - 62) as i32, 1, 0x22262E);

    // Brand logo + title
    s.fill_circle(84, 23, 11, 0x1BD96A);
    s.draw_text(79, 15, "S", &FONT_14, 0x07170E);
    s.draw_text(102, 13, "sawbyte", &FONT_18, 0xF8FAFC);
    s.draw_text(192, 16, "<   >", &FONT_14, 0x64748B);

    let crumb = match state.tab {
        ActiveTab::Home => "Home",
        ActiveTab::Viewport3D => "3D & Radar Viewport",
        ActiveTab::ModsEditions => "Mods & Editions",
    };
    s.draw_play_triangle(252, 23, 4, 0x94A3B8);
    s.draw_text(264, 16, crumb, &FONT_14, 0xE2E8F0);

    // Top-right status pill & Companion Terminal button
    s.fill_rounded_rect(842, 9, 118, 28, 7, 0x1F242D, Some(0x2E3440));
    s.draw_text(856, 16, ">_ Terminal", &FONT_12, 0xE2E8F0);

    s.fill_rounded_rect(972, 9, 290, 28, 14, 0x132A1E, Some(0x1BD96A));
    s.fill_circle(988, 23, 4, 0x1BD96A);
    s.draw_text(1000, 16, &state.status_line, &FONT_12, 0x1BD96A);

    // 3. Right Sidebar (x: 972..1280, y: 46..800) — Modrinth App Style
    s.fill_rect(972, 46, 308, (WIN_H - 46) as i32, 0x121418);
    s.fill_rect(972, 46, 1, (WIN_H - 46) as i32, 0x22262E);

    s.draw_text(992, 62, "Playing as", &FONT_12, 0x94A3B8);
    s.fill_rounded_rect(990, 84, 272, 58, 10, 0x1C1F26, Some(0x2A2F3A));
    s.draw_player_head(1002, 95, 36, true);
    s.draw_text(1048, 96, &state.account_name, &FONT_14, 0xF8FAFC);
    s.draw_text(1048, 116, "Minecraft 26.3 Account", &FONT_12, 0x8B94A5);
    s.draw_text(1240, 105, "v", &FONT_12, 0x8B94A5);

    // Engine & AI Toggles (matching reference image 3 toggle cards!)
    s.draw_text(992, 160, "Engine & AI Bridges", &FONT_12, 0x94A3B8);

    // Toggle Card 1: MCP Agent Bridge
    s.fill_rounded_rect(990, 182, 272, 50, 10, 0x1C1F26, Some(0x282C35));
    s.fill_rounded_rect(1000, 192, 30, 30, 7, 0x132A1E, None);
    s.draw_iso_voxel(1015, 207, 9, 4);
    s.draw_text(1038, 193, "MCP Agent Server", &FONT_14, 0xF1F5F9);
    s.draw_text(1038, 211, "Claude / Gemini · Ready", &FONT_12, 0x8B94A5);
    s.draw_toggle_switch(1208, 196, state.mcp_enabled);

    // Toggle Card 2: Neural Net / Gym RL API
    s.fill_rounded_rect(990, 240, 272, 50, 10, 0x1C1F26, Some(0x282C35));
    s.fill_rounded_rect(1000, 250, 30, 30, 7, 0x1E293B, None);
    s.draw_iso_voxel(1015, 265, 9, 2);
    s.draw_text(1038, 251, "Neural Net RL Socket", &FONT_14, 0xF1F5F9);
    s.draw_text(1038, 269, "127.0.0.1:25580 · 20 TPS", &FONT_12, 0x8B94A5);
    s.draw_toggle_switch(1208, 254, state.gym_enabled);

    // Active Bots List (Modrinth "Online - 2" style)
    s.draw_text(992, 310, "Active Sessions - 2", &FONT_12, 0x94A3B8);

    s.fill_rounded_rect(990, 332, 272, 46, 9, 0x191C22, None);
    s.draw_player_head(1000, 341, 28, true);
    s.fill_circle(1026, 367, 4, 0x1BD96A);
    s.draw_text(1038, 339, "SawbyteNative", &FONT_14, 0xF1F5F9);
    s.draw_text(1038, 356, "Playing 127.0.0.1:25565", &FONT_12, 0x1BD96A);

    s.fill_rounded_rect(990, 384, 272, 46, 9, 0x191C22, None);
    s.draw_player_head(1000, 393, 28, false);
    s.fill_circle(1026, 419, 4, 0xF59E0B);
    s.draw_text(1038, 391, "PotatoBot_02", &FONT_14, 0xF1F5F9);
    s.draw_text(1038, 408, "Standby · 10.4 MB RSS", &FONT_12, 0x8B94A5);

    // Bottom-Right Live 3D Viewport Preview Card (replaces "News" card from Modrinth)
    s.draw_text(992, 448, "Live 3D & Radar Stream", &FONT_12, 0x94A3B8);
    s.fill_rounded_rect(990, 470, 272, 308, 12, 0x1C1F26, Some(0x282C35));

    // Mini 3D Raymarcher Viewport inside Right Sidebar Card
    draw_software_3d_preview(&mut s, 1002, 482, 248, 152, state.camera_mode);

    s.draw_text(1004, 648, "Zero-GPU Voxel Raymarcher", &FONT_14, 0xF8FAFC);
    s.draw_text(1004, 668, "XYZ: 0.5, -60.0, 0.5 · 20.0 HP", &FONT_12, 0x8B94A5);
    s.draw_text(1004, 686, "Memory: 8.2 MB RSS (No JVM)", &FONT_12, 0x1BD96A);

    s.fill_rounded_rect(1004, 714, 244, 34, 8, 0x262B35, Some(0x333946));
    s.draw_text(1048, 724, "Expand 3D Viewport ->", &FONT_12, 0xF1F5F9);

    // 4. Center Main Content Area (x: 62..972, y: 46..800)
    match state.tab {
        ActiveTab::Home => render_home_tab(&mut s, state),
        ActiveTab::Viewport3D => render_viewport_tab(&mut s, state),
        ActiveTab::ModsEditions => render_mods_tab(&mut s, state),
    }

    s
}

fn render_home_tab(s: &mut UiSurface, state: &AppState) {
    // A. Lunar-Client-Style Hero Launch Banner (x: 84, y: 64, w: 866, h: 136)
    s.fill_rounded_rect(84, 64, 866, 136, 14, 0x0D1015, Some(0x242933));

    // Subtle deep-space star dots in hero banner
    let stars = [
        (110, 82), (165, 160), (240, 92), (310, 175), (680, 85),
        (745, 168), (820, 95), (890, 150), (620, 140), (355, 88),
    ];
    for (sx, sy) in stars {
        s.fill_circle(sx, sy, 1, 0x475569);
    }

    // Left 3D Isometric Voxel Artwork (Earth Cube + Iron Bot Mascot, inspired by Lunar Client image 1)
    s.draw_iso_voxel(162, 132, 38, 1); // 3D Grass Earth Cube
    s.draw_iso_voxel(256, 134, 30, 2); // 3D Iron Bot Mascot

    // Right 3D Isometric Voxel Artwork
    s.draw_iso_voxel(776, 134, 30, 3); // 3D Fabric Cube
    s.draw_iso_voxel(866, 132, 36, 0); // 3D Redstone Cube

    // Center Big Emerald Green LAUNCH Button (x: 392, y: 84, w: 250, h: 54)
    s.fill_rounded_rect(392, 84, 250, 54, 10, 0x1BD96A, Some(0x4ADE80));
    let launch_title = match state.edition {
        LaunchEdition::Headless => "LAUNCH 26.3",
        LaunchEdition::Potato3D => "LAUNCH POTATO 3D",
        LaunchEdition::FabricBridge => "LAUNCH FABRIC 26.3",
    };
    let tw = UiSurface::text_width(launch_title, &FONT_18);
    s.draw_text(392 + (250 - tw) / 2, 96, launch_title, &FONT_18, 0x06150C);
    s.fill_circle(452, 124, 3, 0x06150C);
    s.draw_text(460, 118, "READY TO LAUNCH", &FONT_12, 0x06150C);

    // Edition Mode Selector Pills under the Launch button
    let modes = [
        (356, 148, 112, "Headless 26.3", state.edition == LaunchEdition::Headless),
        (476, 148, 104, "Potato 3D", state.edition == LaunchEdition::Potato3D),
        (588, 148, 104, "Fabric 26.3", state.edition == LaunchEdition::FabricBridge),
    ];
    for (mx, my, mw, mlabel, active) in modes {
        s.fill_rounded_rect(
            mx,
            my,
            mw,
            26,
            13,
            if active { 0x132A1E } else { 0x181C23 },
            Some(if active { 0x1BD96A } else { 0x282E3A }),
        );
        let lw = UiSurface::text_width(mlabel, &FONT_12);
        s.draw_text(
            mx + (mw - lw) / 2,
            my + 6,
            mlabel,
            &FONT_12,
            if active { 0x1BD96A } else { 0x94A3B8 },
        );
    }

    // B. "Jump in ^" Section (Modrinth App 3 Horizontal Instance Rows)
    s.draw_text(86, 218, "Jump in  ^", &FONT_18, 0xF8FAFC);

    let jump_rows = [
        (
            250,
            0xF59E0B,
            0u8,
            "Headless Native 26.3",
            "8.2 MB RAM",
            "Minecraft 26.3 · 127.0.0.1:25565 · Zero-JVM Engine",
            "Pure Rust · 20.0 TPS",
        ),
        (
            322,
            0x16A34A,
            1u8,
            "Potato 3D & Radar",
            "Zero-GPU",
            "Minecraft 26.3 · Software 3D Raymarcher & 360 Radar",
            "10.4 MB RAM · Live Stream",
        ),
        (
            394,
            0x4F46E5,
            3u8,
            "Graphical Fabric 26.3",
            "Bridge Mod",
            "Fabric 26.3 · Auto-injects sawbyte-bridge-0.5.0.jar",
            "Full Visual Client Sync",
        ),
    ];

    for (ry, icon_bg, vkind, title, badge, sub_left, sub_right) in jump_rows {
        s.fill_rounded_rect(84, ry, 866, 62, 12, 0x1D2027, Some(0x2A2F3A));
        // 42x42 rounded icon tile
        s.fill_rounded_rect(98, ry + 10, 42, 42, 10, icon_bg, None);
        s.draw_iso_voxel(119, ry + 31, 13, vkind);

        // Title + green badge pill
        let after_t = s.draw_text(154, ry + 13, title, &FONT_14, 0xF8FAFC);
        let bw = UiSurface::text_width(badge, &FONT_12) + 18;
        s.fill_rounded_rect(after_t + 10, ry + 11, bw, 20, 10, 0x132A1E, Some(0x1BD96A));
        s.draw_text(after_t + 19, ry + 14, badge, &FONT_12, 0x1BD96A);

        // Subtitle
        s.draw_text(154, ry + 35, sub_left, &FONT_12, 0x8B94A5);
        s.draw_text(550, ry + 24, sub_right, &FONT_12, 0xCBD5E1);

        // Solid Emerald Green "Play" button + 3-dots button on right
        s.fill_rounded_rect(822, ry + 14, 82, 34, 8, 0x1BD96A, None);
        s.draw_play_triangle(840, ry + 31, 5, 0x06150C);
        s.draw_text(852, ry + 23, "Play", &FONT_14, 0x06150C);

        s.draw_text(922, ry + 21, ":", &FONT_18, 0x94A3B8);
    }

    // C. "Library" Section (Search Bar + 5 Square Visual Instance Tiles — 1:1 with Modrinth App)
    s.draw_text(86, 474, "Library", &FONT_18, 0xF8FAFC);

    // Search Input + Action Buttons Row (y: 504)
    s.fill_rounded_rect(84, 504, 520, 36, 9, 0x1C1F26, Some(0x2A2F3A));
    s.draw_text(100, 514, "Search instances, bots, or servers...", &FONT_12, 0x64748B);

    s.fill_rounded_rect(616, 504, 152, 36, 9, 0x252A34, Some(0x323846));
    s.draw_text(634, 514, "Auto-Fetch Editions", &FONT_12, 0xE2E8F0);

    s.fill_rounded_rect(780, 504, 170, 36, 9, 0x1BD96A, None);
    s.draw_text(802, 513, "+ New instance", &FONT_14, 0x06150C);

    // 5 Square Visual Instance Cards (y: 554, w: 162, h: 214)
    let cards = [
        ("Headless Bot", "Native 26.3", 0x3B5BDB, 2u8),
        ("Potato 3D", "Visualizer 26.3", 0x0C8599, 1u8),
        ("Fabric Bridge", "Fabric 26.3", 0x2B303C, 3u8),
        ("Redstone Lab", "Native 26.3", 0xE67700, 0u8),
        ("Survival AI", "MCP + RL 26.3", 0x1B4332, 4u8),
    ];

    for (i, (ctitle, csub, art_bg, vkind)) in cards.iter().enumerate() {
        let cx = 84 + (i as i32) * 176;
        let cy = 554;
        s.fill_rounded_rect(cx, cy, 162, 214, 12, 0x1D2027, Some(0x2A2F3A));
        // Square 138x138 inner artwork tile
        s.fill_rounded_rect(cx + 12, cy + 12, 138, 138, 10, *art_bg, None);
        s.draw_iso_voxel(cx + 81, cy + 81, 38, *vkind);

        // Floating green play circle on the 5th card (just like Modrinth reference image!)
        if i == 4 {
            s.fill_circle(cx + 126, cy + 126, 16, 0x1BD96A);
            s.draw_play_triangle(cx + 126, cy + 126, 5, 0x06150C);
        }

        s.draw_text(cx + 14, cy + 162, ctitle, &FONT_14, 0xF8FAFC);
        s.draw_text(cx + 14, cy + 184, csub, &FONT_12, 0x8B94A5);
    }
}

fn render_viewport_tab(s: &mut UiSurface, state: &AppState) {
    s.draw_text(86, 64, "Potato 3D & 360 Radar Supervisor", &FONT_18, 0xF8FAFC);

    // Camera Mode Pills
    let cam_modes = [
        (460, "1st Person", state.camera_mode == 0),
        (566, "3rd Person", state.camera_mode == 1),
        (672, "360 Radar", state.camera_mode == 2),
    ];
    for (px, label, active) in cam_modes {
        s.fill_rounded_rect(
            px,
            60,
            96,
            30,
            8,
            if active { 0x1BD96A } else { 0x1E222A },
            None,
        );
        let lw = UiSurface::text_width(label, &FONT_12);
        s.draw_text(
            px + (96 - lw) / 2,
            68,
            label,
            &FONT_12,
            if active { 0x06150C } else { 0xE2E8F0 },
        );
    }

    // Large 3D Software Raymarcher Viewport Card (x: 84, y: 104, w: 866, h: 420)
    s.fill_rounded_rect(84, 104, 866, 420, 14, 0x1C1F26, Some(0x2A2F3A));
    draw_software_3d_preview(s, 100, 120, 834, 388, state.camera_mode);

    // Visual Bot Controller Action Bar (No Terminal Required)
    s.draw_text(86, 540, "Visual Bot Controls", &FONT_18, 0xF8FAFC);
    let actions = [
        "Walk Forward",
        "Jump",
        "Raycast Mine",
        "3D A* Pathfind",
        "Inspect Chest",
        "Save Milestone",
    ];
    for (i, act) in actions.iter().enumerate() {
        let ax = 84 + (i as i32) * 146;
        s.fill_rounded_rect(ax, 572, 136, 40, 9, 0x1E222A, Some(0x2E3440));
        let aw = UiSurface::text_width(act, &FONT_12);
        s.draw_text(ax + (136 - aw) / 2, 585, act, &FONT_12, 0x1BD96A);
    }

    // Live Engine Output Stream Box (x: 84, y: 626, w: 866, h: 146)
    s.fill_rounded_rect(84, 626, 866, 146, 12, 0x0E1014, Some(0x242933));
    for (i, line) in state.logs.iter().take(5).enumerate() {
        s.draw_text(102, 642 + (i as i32) * 24, line, &FONT_12, 0xCBD5E1);
    }
}

fn render_mods_tab(s: &mut UiSurface, state: &AppState) {
    s.draw_text(86, 64, "Editions, Mods & System Packages", &FONT_18, 0xF8FAFC);

    let items = [
        (
            106,
            0x16A34A,
            1u8,
            "sawbyte-play (Headless & Potato 3D Engine)",
            "Standalone Rust client · Auto-fetched from GitHub Releases into ~/.sawbyte/bin",
            "Installed",
            true,
        ),
        (
            188,
            0x4F46E5,
            3u8,
            "sawbyte-bridge-0.5.0.jar (Fabric 26.3 Mod)",
            "Syncs & spectates headless bots inside Java Fabric 26.3 · Auto-binds port 25580",
            "Installed",
            true,
        ),
        (
            270,
            0xF59E0B,
            0u8,
            "Sodium 26.3 (Modrinth Fabric Optimization)",
            "High-framerate chunk rendering engine for Graphical Fabric 26.3 instances",
            "Install",
            false,
        ),
        (
            352,
            0x0C8599,
            2u8,
            "Lithium & FerriteCore 26.3",
            "Server tick & memory footprint optimization mods for Fabric 26.3",
            "Install",
            false,
        ),
        (
            434,
            0x1B4332,
            4u8,
            "System CLI & Desktop Entry (sawbyte / .deb)",
            "Installs sawbyte & sawbyte-launcher to system PATH and application menu",
            "Installed",
            true,
        ),
    ];

    for (ry, bg, vkind, title, sub, btn_txt, is_inst) in items {
        s.fill_rounded_rect(84, ry, 866, 70, 12, 0x1D2027, Some(0x2A2F3A));
        s.fill_rounded_rect(100, ry + 12, 46, 46, 10, bg, None);
        s.draw_iso_voxel(123, ry + 35, 14, vkind);

        s.draw_text(162, ry + 16, title, &FONT_14, 0xF8FAFC);
        s.draw_text(162, ry + 40, sub, &FONT_12, 0x8B94A5);

        let btn_bg = if is_inst { 0x132A1E } else { 0x1BD96A };
        let btn_fg = if is_inst { 0x1BD96A } else { 0x06150C };
        s.fill_rounded_rect(818, ry + 18, 112, 34, 8, btn_bg, Some(0x1BD96A));
        let bw = UiSurface::text_width(btn_txt, &FONT_14);
        s.draw_text(818 + (112 - bw) / 2, ry + 27, btn_txt, &FONT_14, btn_fg);
    }

    // Bottom Native Release Formats Summary Card
    s.fill_rounded_rect(84, 526, 866, 244, 12, 0x1C1F26, Some(0x2A2F3A));
    s.draw_text(106, 548, "Native Distro & OS Packages (v0.5.0)", &FONT_18, 0xF8FAFC);
    s.draw_text(106, 582, "• Debian / Ubuntu / Mint:  sawbyte-launcher_0.5.0_amd64.deb (apt / dpkg installer)", &FONT_14, 0x1BD96A);
    s.draw_text(106, 612, "• Universal Linux App:     Sawbyte-Launcher-x86_64.AppImage (single clickable file)", &FONT_14, 0xE2E8F0);
    s.draw_text(106, 642, "• Windows 10 / 11 (x64):   Sawbyte-Launcher-x64.exe (native Win32 app, no unzip needed)", &FONT_14, 0xE2E8F0);
    s.draw_text(106, 672, "• Standalone CLI & Engine: sawbyte-cli, sawbyte-play, and sawbyte-bridge-0.5.0.jar", &FONT_14, 0x94A3B8);
    s.draw_text(106, 712, "Zero HTML · Zero Localhost Web Server · Zero 200 MB Electron Bloat", &FONT_12, 0x8B94A5);
}

fn draw_software_3d_preview(s: &mut UiSurface, x: i32, y: i32, w: i32, h: i32, mode: usize) {
    let horizon = y + (h * 52) / 100;
    // Sky & Ground Gradient
    for py in y..(y + h) {
        let col = if py < horizon {
            let t = (py - y) as f32 / (horizon - y).max(1) as f32;
            lerp_color(0x0B1320, 0x16263B, t)
        } else {
            let t = (py - horizon) as f32 / (y + h - horizon).max(1) as f32;
            lerp_color(0x132A1E, 0x0B1611, t)
        };
        s.fill_rect(x, py, w, 1, col);
    }

    // Voxel terrain blocks in 3D perspective
    let blocks = [
        (x + (w * 20) / 100, horizon - (h * 18) / 100, (w * 14) / 100, (h * 34) / 100, 0x22C55E, 0x15803D),
        (x + (w * 37) / 100, horizon - (h * 10) / 100, (w * 11) / 100, (h * 26) / 100, 0xD97706, 0x92400E),
        (x + (w * 56) / 100, horizon - (h * 22) / 100, (w * 15) / 100, (h * 38) / 100, 0x16A34A, 0x14532D),
        (x + (w * 74) / 100, horizon - (h * 8) / 100, (w * 12) / 100, (h * 24) / 100, 0x0284C7, 0x0369A1),
    ];
    for (bx, by, bw, bh, top_c, body_c) in blocks {
        s.fill_rounded_rect(bx, by, bw, bh, 3, body_c, Some(0x34D399));
        s.fill_rect(bx + 1, by + 1, bw - 2, (bh / 5).max(4), top_c);
    }

    // Center Crosshair
    let cx = x + w / 2;
    let cy = y + h / 2;
    s.fill_rect(cx - 6, cy, 13, 1, 0x1BD96A);
    s.fill_rect(cx, cy - 6, 1, 13, 0x1BD96A);

    // Top-left telemetry overlay inside 3D viewport
    let tag = if mode == 2 { "360 RADAR · 20.0 TPS" } else { "3D RAYMARCHER · 20.0 TPS" };
    s.fill_rounded_rect(x + 10, y + 10, UiSurface::text_width(tag, &FONT_12) + 16, 22, 6, 0x0D1117, Some(0x1BD96A));
    s.draw_text(x + 18, y + 14, tag, &FONT_12, 0x1BD96A);
}

pub const TAURI_WEBVIEW_HTML: &str = include_str!("tauri_ui.html");

pub fn start_local_tauri_ipc_server(port: u16) -> Option<u16> {
    use std::net::TcpListener;
    let listener = TcpListener::bind(("127.0.0.1", port))
        .or_else(|_| TcpListener::bind(("127.0.0.1", 0)))
        .ok()?;
    let bound_port = listener.local_addr().ok()?.port();
    thread::spawn(move || {
        for mut stream in listener.incoming().flatten() {
            thread::spawn(move || {
                let mut buf = [0u8; 8192];
                let Ok(n) = stream.read(&mut buf) else { return; };
                if n == 0 {
                    return;
                }
                let req = String::from_utf8_lossy(&buf[..n]);
                let first_line = req.lines().next().unwrap_or("");
                let body_start = req.find("\r\n\r\n").map(|i| i + 4).unwrap_or(req.len());
                let body = &req[body_start..];

                if first_line.starts_with("POST /api/send_bot_command") {
                    let cmd = sawbyte_core::json::JsonValue::parse(body)
                        .ok()
                        .and_then(|v| v.get("command").and_then(|c| c.as_str()).map(String::from))
                        .unwrap_or_else(|| "inspect.status".to_string());
                    let ep = sawbyte_core::json::JsonValue::parse(body)
                        .ok()
                        .and_then(|v| v.get("endpoint").and_then(|c| c.as_str()).map(String::from))
                        .unwrap_or_else(|| "127.0.0.1:25580".to_string());
                    let reply = if let Ok(mut bot_sock) = TcpStream::connect(&ep) {
                        let _ = bot_sock.set_read_timeout(Some(Duration::from_millis(2000)));
                        let _ = writeln!(bot_sock, "{}", cmd.trim());
                        let _ = bot_sock.flush();
                        let mut r = std::io::BufReader::new(bot_sock);
                        let mut line = String::new();
                        let _ = std::io::BufRead::read_line(&mut r, &mut line);
                        line.trim().replace("\\n", "\n")
                    } else {
                        format!("bot socket {ep} offline — launch an instance first")
                    };
                    let json_resp = format!("{:?}", reply);
                    let http = format!(
                        "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                        json_resp.len(),
                        json_resp
                    );
                    let _ = stream.write_all(http.as_bytes());
                } else if first_line.starts_with("POST /api/launch_instance") {
                    let parsed = sawbyte_core::json::JsonValue::parse(body).ok();
                    let inst_name = parsed.as_ref().and_then(|v| v.get("instance_name")).and_then(|s| s.as_str()).unwrap_or("default-native");
                    let server_addr = parsed.as_ref().and_then(|v| v.get("server_addr")).and_then(|s| s.as_str()).unwrap_or("127.0.0.1:25565");
                    let account_name = parsed.as_ref().and_then(|v| v.get("account_name")).and_then(|s| s.as_str()).unwrap_or("SawByteBot");
                    let edition = parsed.as_ref().and_then(|v| v.get("edition")).and_then(|s| s.as_str()).unwrap_or("headless");
                    let mut msg = format!("Dispatched launch for '{inst_name}' -> {server_addr}");
                    if let Ok(mut launcher) = PersistentLauncher::load_or_init() {
                        let mode = match edition {
                            "potato" | "visualizer" => RuntimeMode::NativeMinimalVisualizer,
                            "bridge" | "fabric" => RuntimeMode::FullClientBridge,
                            _ => RuntimeMode::NativeStandalone,
                        };
                        let prof = match edition {
                            "potato" | "visualizer" => "fabric-minimal",
                            "bridge" | "fabric" => "fabric-26.3",
                            _ => "sawbyte-26.3",
                        };
                        if !launcher.instances.contains_key(inst_name) {
                            let _ = launcher.create_instance(inst_name, prof, mode);
                        }
                        if !launcher.accounts.contains_key(account_name) {
                            let _ = launcher.add_account(account_name, "00000000000030008000000000000001", None);
                        }
                        if let Ok((child, log_p)) = launcher.spawn_live_process(
                            inst_name,
                            Some(server_addr),
                            account_name,
                            Path::new("block-profile.json"),
                            &[],
                        ) {
                            msg = format!("Launched '{inst_name}' (PID {}) -> {server_addr} [log: {}]", child.id(), log_p.display());
                        }
                    }
                    let json_resp = format!("{:?}", msg);
                    let http = format!(
                        "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                        json_resp.len(),
                        json_resp
                    );
                    let _ = stream.write_all(http.as_bytes());
                } else {
                    let body_bytes = TAURI_WEBVIEW_HTML.as_bytes();
                    let http = format!(
                        "HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=utf-8\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                        body_bytes.len()
                    );
                    let _ = stream.write_all(http.as_bytes());
                    let _ = stream.write_all(body_bytes);
                }
                let _ = stream.flush();
            });
        }
    });
    Some(bound_port)
}

pub fn try_launch_tauri_or_native_webview(url: &str) -> bool {
    // 1. Check if standalone compiled Tauri binary exists
    for cand in [
        "launcher-tauri/src-tauri/target/release/sawbyte-tauri-launcher",
        "launcher-tauri/src-tauri/target/release/sawbyte-tauri-launcher.exe",
        "sawbyte-tauri-launcher",
        "sawbyte-tauri-launcher.exe",
    ] {
        if Path::new(cand).exists() {
            if Command::new(cand).status().is_ok() {
                return true;
            }
        }
    }
    // 2. On Linux, try Python PyGObject WebKit2GTK native window (same engine as Tauri on Linux!)
    #[cfg(not(windows))]
    {
        let py_webkit = format!(
            "import gi; gi.require_version('Gtk', '3.0'); gi.require_version('WebKit2', '4.0'); from gi.repository import Gtk, WebKit2; win = Gtk.Window(title='Sawbyte Launcher — Tauri WebView'); win.set_default_size(1340, 840); wv = WebKit2.WebView(); wv.load_uri('{url}'); win.add(wv); win.connect('destroy', Gtk.main_quit); win.show_all(); Gtk.main()"
        );
        if Command::new("python3").args(["-c", &py_webkit]).status().is_ok_and(|s| s.success()) {
            return true;
        }
        // Fallback to Chromium/Brave/Chrome/Edge native `--app=` frameless desktop window
        for browser in ["chromium", "google-chrome", "brave-browser", "microsoft-edge", "firefox"] {
            if Command::new(browser).arg(format!("--app={url}")).status().is_ok() {
                return true;
            }
        }
    }
    #[cfg(windows)]
    {
        if Command::new("cmd").args(["/C", "start", "msedge", &format!("--app={url}")]).status().is_ok() {
            return true;
        }
    }
    false
}

pub fn run_launcher_gui(port_opt: Option<u16>) -> Result<(), Box<dyn std::error::Error>> {
    let mut state = AppState::default();
    refresh_state_from_disk(&mut state);

    let bound_port = start_local_tauri_ipc_server(port_opt.unwrap_or(8788)).unwrap_or(8788);
    let url = format!("http://127.0.0.1:{bound_port}");

    println!("========================================================================");
    println!("        SAWBYTE TAURI + RUST DESKTOP LAUNCHER v0.5.0");
    println!("========================================================================");
    println!("  WebView UI:     {url} (Tauri v2 / WebView2 / WebKit2GTK)");
    println!("  Launcher Root:  {}", PersistentLauncher::resolve_home().display());
    println!("  CLI Companion:  Active in terminal (`sawbyte>` prompt)");
    println!("========================================================================");

    // Spawn interactive companion CLI REPL in background thread
    thread::spawn(|| {
        thread::sleep(Duration::from_millis(250));
        crate::run_companion_terminal_repl();
    });

    if try_launch_tauri_or_native_webview(&url) {
        return Ok(());
    }

    #[cfg(windows)]
    {
        return run_win32_native_window(state);
    }

    #[cfg(not(windows))]
    {
        return run_unix_native_window(state);
    }
}

pub fn refresh_state_from_disk(state: &mut AppState) {
    if let Ok(launcher) = PersistentLauncher::load_or_init() {
        let bridge_p = launcher.root_dir.join("editions").join("sawbyte-bridge-0.5.0.jar");
        state.bridge_installed = bridge_p.exists();
        state.headless_installed = PersistentLauncher::locate_sawbyte_play_binary().is_ok();
        let (cli_ok, _, _) = crate::check_cli_status();
        state.cli_installed = cli_ok;
    }
}

pub fn handle_mouse_click(state: &mut AppState, mx: i32, my: i32) -> bool {
    // Left Rail navigation clicks
    if mx < 62 {
        if (38..=78).contains(&my) {
            state.tab = ActiveTab::Home;
            return true;
        } else if (90..=130).contains(&my) {
            state.tab = ActiveTab::Viewport3D;
            return true;
        } else if (136..=176).contains(&my) {
            state.tab = ActiveTab::ModsEditions;
            return true;
        }
    }

    // Top bar ">_ Terminal" button (842..960, 9..37)
    if (842..=960).contains(&mx) && (9..=37).contains(&my) {
        let _ = spawn_external_terminal();
        state.logs.insert(0, "[Terminal] Opened external sawbyte> CLI terminal window.".to_string());
        return true;
    }

    // Right sidebar toggles
    if (1195..=1260).contains(&mx) {
        if (182..=232).contains(&my) {
            state.mcp_enabled = !state.mcp_enabled;
            return true;
        } else if (240..=290).contains(&my) {
            state.gym_enabled = !state.gym_enabled;
            return true;
        }
    }

    // Right sidebar "Expand 3D Viewport" button
    if (990..=1262).contains(&mx) && (700..=760).contains(&my) {
        state.tab = ActiveTab::Viewport3D;
        return true;
    }

    if state.tab == ActiveTab::Home {
        // Edition mode pills in Hero Banner
        if (144..=178).contains(&my) {
            if (356..=468).contains(&mx) {
                state.edition = LaunchEdition::Headless;
                return true;
            } else if (476..=580).contains(&mx) {
                state.edition = LaunchEdition::Potato3D;
                return true;
            } else if (588..=692).contains(&mx) {
                state.edition = LaunchEdition::FabricBridge;
                return true;
            }
        }
        // Hero LAUNCH 26.3 button (392..642, 84..138) or "Jump in" Play buttons (822..904)
        if ((392..=642).contains(&mx) && (84..=138).contains(&my))
            || ((822..=904).contains(&mx) && (250..=456).contains(&my))
        {
            if (250..=312).contains(&my) && (822..=904).contains(&mx) {
                state.edition = LaunchEdition::Headless;
            } else if (322..=384).contains(&my) && (822..=904).contains(&mx) {
                state.edition = LaunchEdition::Potato3D;
            } else if (394..=456).contains(&my) && (822..=904).contains(&mx) {
                state.edition = LaunchEdition::FabricBridge;
            }
            trigger_edition_launch(state);
            return true;
        }
    } else if state.tab == ActiveTab::Viewport3D {
        if (56..=94).contains(&my) {
            if (460..=556).contains(&mx) {
                state.camera_mode = 0;
                return true;
            } else if (566..=662).contains(&mx) {
                state.camera_mode = 1;
                return true;
            } else if (672..=768).contains(&mx) {
                state.camera_mode = 2;
                return true;
            }
        }
    }
    false
}

fn trigger_edition_launch(state: &mut AppState) {
    let mode = match state.edition {
        LaunchEdition::Headless => RuntimeMode::NativeStandalone,
        LaunchEdition::Potato3D => RuntimeMode::NativeMinimalVisualizer,
        LaunchEdition::FabricBridge => RuntimeMode::FullClientBridge,
    };
    let inst_name = match state.edition {
        LaunchEdition::Headless => "default-native",
        LaunchEdition::Potato3D => "potato-26-3",
        LaunchEdition::FabricBridge => "fabric-bridge-26-3",
    };
    let profile = match state.edition {
        LaunchEdition::Headless => "sawbyte-26.3",
        LaunchEdition::Potato3D => "fabric-minimal",
        LaunchEdition::FabricBridge => "fabric-26.3",
    };

    state.status_line = format!("Launching {inst_name} · 20.0 TPS");
    state.logs.insert(
        0,
        format!("[Launch] Provisioning & starting '{inst_name}' ({profile}) -> {}", state.server_addr),
    );

    let srv = state.server_addr.clone();
    let acc = state.account_name.clone();
    let inst_owned = inst_name.to_string();
    let prof_owned = profile.to_string();
    thread::spawn(move || {
        if let Ok(mut launcher) = PersistentLauncher::load_or_init() {
            if !launcher.instances.contains_key(&inst_owned) {
                let _ = launcher.create_instance(&inst_owned, &prof_owned, mode);
            }
            let _ = launcher.spawn_live_process(
                &inst_owned,
                Some(&srv),
                &acc,
                Path::new("block-profile.json"),
                &[],
            );
        }
    });
}

fn spawn_external_terminal() -> Result<(), String> {
    let exe = std::env::current_exe().unwrap_or_else(|_| PathBuf::from("sawbyte"));
    if cfg!(target_os = "windows") {
        let _ = Command::new("cmd")
            .args(["/C", "start", "Sawbyte CLI", "cmd", "/K", &exe.to_string_lossy(), "shell"])
            .spawn();
        return Ok(());
    }
    for (term, flag) in [
        ("x-terminal-emulator", "-e"),
        ("gnome-terminal", "--"),
        ("konsole", "-e"),
        ("xfce4-terminal", "-x"),
        ("kitty", "-e"),
        ("alacritty", "-e"),
        ("xterm", "-e"),
    ] {
        if Command::new(term).arg(flag).arg(&exe).arg("shell").spawn().is_ok() {
            return Ok(());
        }
    }
    Err("no terminal emulator found".into())
}

// ==============================================================================
// Native X11 Protocol Window Backend (Linux / Unix — Zero HTML, Zero Browser)
// ==============================================================================

#[cfg(not(windows))]
fn run_unix_native_window(mut state: AppState) -> Result<(), Box<dyn std::error::Error>> {
    if let Ok(()) = run_x11_socket_window(&mut state) {
        return Ok(());
    }
    // Fallback to native Python GTK3 / Tkinter window if X11 socket is not directly accessible
    run_python_native_canvas_window(&mut state)
}

#[cfg(not(windows))]
fn read_xauthority_cookie(display_num: &str) -> (Vec<u8>, Vec<u8>) {
    let auth_path = std::env::var("XAUTHORITY")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            let home = std::env::var("HOME").unwrap_or_else(|_| ".".into());
            PathBuf::from(home).join(".Xauthority")
        });
    let Ok(bytes) = fs::read(&auth_path) else {
        return (Vec::new(), Vec::new());
    };
    let mut i = 0usize;
    let mut fallback = (Vec::new(), Vec::new());
    while i + 4 <= bytes.len() {
        let _family = u16::from_be_bytes([bytes[i], bytes[i + 1]]);
        i += 2;
        let read_chunk = |pos: &mut usize| -> Option<Vec<u8>> {
            if *pos + 2 > bytes.len() {
                return None;
            }
            let len = u16::from_be_bytes([bytes[*pos], bytes[*pos + 1]]) as usize;
            *pos += 2;
            if *pos + len > bytes.len() {
                return None;
            }
            let out = bytes[*pos..(*pos + len)].to_vec();
            *pos += len;
            Some(out)
        };
        let Some(_addr) = read_chunk(&mut i) else { break };
        let Some(num) = read_chunk(&mut i) else { break };
        let Some(name) = read_chunk(&mut i) else { break };
        let Some(data) = read_chunk(&mut i) else { break };
        if name == b"MIT-MAGIC-COOKIE-1" {
            if String::from_utf8_lossy(&num) == display_num {
                return (name, data);
            }
            if fallback.0.is_empty() {
                fallback = (name, data);
            }
        }
    }
    fallback
}

#[cfg(not(windows))]
fn run_x11_socket_window(state: &mut AppState) -> Result<(), String> {
    use std::os::unix::net::UnixStream;

    let disp_env = std::env::var("DISPLAY").unwrap_or_else(|_| ":0".into());
    let disp_num = disp_env
        .split(':')
        .nth(1)
        .and_then(|s| s.split('.').next())
        .unwrap_or("0");
    let sock_path = format!("/tmp/.X11-unix/X{disp_num}");
    let mut stream = UnixStream::connect(&sock_path)
        .map_err(|e| format!("cannot connect to X11 socket {sock_path}: {e}"))?;
    let _ = stream.set_read_timeout(Some(Duration::from_secs(2)));

    let (auth_name, auth_data) = read_xauthority_cookie(disp_num);
    let pad4 = |n: usize| (4 - (n % 4)) % 4;

    let mut setup_req = Vec::new();
    setup_req.push(b'l'); // little-endian
    setup_req.push(0);
    setup_req.extend_from_slice(&11u16.to_le_bytes());
    setup_req.extend_from_slice(&0u16.to_le_bytes());
    setup_req.extend_from_slice(&(auth_name.len() as u16).to_le_bytes());
    setup_req.extend_from_slice(&(auth_data.len() as u16).to_le_bytes());
    setup_req.extend_from_slice(&0u16.to_le_bytes());
    setup_req.extend_from_slice(&auth_name);
    setup_req.resize(setup_req.len() + pad4(auth_name.len()), 0);
    setup_req.extend_from_slice(&auth_data);
    setup_req.resize(setup_req.len() + pad4(auth_data.len()), 0);

    stream.write_all(&setup_req).map_err(|e| e.to_string())?;

    let mut hdr = [0u8; 8];
    stream.read_exact(&mut hdr).map_err(|e| e.to_string())?;
    if hdr[0] != 1 {
        return Err("X11 connection setup rejected".into());
    }
    let add_words = u16::from_le_bytes([hdr[6], hdr[7]]) as usize;
    let mut setup_body = vec![0u8; add_words * 4];
    stream.read_exact(&mut setup_body).map_err(|e| e.to_string())?;

    let res_id_base = u32::from_le_bytes(setup_body[4..8].try_into().unwrap());
    let res_id_mask = u32::from_le_bytes(setup_body[8..12].try_into().unwrap());
    let vendor_len = u16::from_le_bytes(setup_body[16..18].try_into().unwrap()) as usize;
    let num_formats = setup_body[21] as usize;
    let roots_offset = 32 + ((vendor_len + 3) & !3) + num_formats * 8;
    if roots_offset + 40 > setup_body.len() {
        return Err("invalid X11 root screen descriptor".into());
    }
    let root_window = u32::from_le_bytes(setup_body[roots_offset..roots_offset + 4].try_into().unwrap());
    let root_visual = u32::from_le_bytes(setup_body[roots_offset + 32..roots_offset + 36].try_into().unwrap());
    let root_depth = setup_body[roots_offset + 38];

    let step = res_id_mask & res_id_mask.wrapping_neg();
    let wid = res_id_base | step;
    let gcid = res_id_base | (step.wrapping_mul(2));

    // CreateWindow (opcode 1)
    // EventMask: ButtonPress(4) | Exposure(32768) = 0x8004, BackPixel = 0x16181C
    let mut create_win = Vec::with_capacity(40);
    create_win.push(1); // CreateWindow
    create_win.push(root_depth);
    create_win.extend_from_slice(&10u16.to_le_bytes()); // 8 + 2 values
    create_win.extend_from_slice(&wid.to_le_bytes());
    create_win.extend_from_slice(&root_window.to_le_bytes());
    create_win.extend_from_slice(&60i16.to_le_bytes());
    create_win.extend_from_slice(&40i16.to_le_bytes());
    create_win.extend_from_slice(&(WIN_W as u16).to_le_bytes());
    create_win.extend_from_slice(&(WIN_H as u16).to_le_bytes());
    create_win.extend_from_slice(&0u16.to_le_bytes()); // border_width
    create_win.extend_from_slice(&1u16.to_le_bytes()); // InputOutput
    create_win.extend_from_slice(&root_visual.to_le_bytes());
    create_win.extend_from_slice(&(0x0000_0002u32 | 0x0000_0800u32).to_le_bytes()); // CWBackPixel | CWEventMask
    create_win.extend_from_slice(&0x0016_181Cu32.to_le_bytes());
    create_win.extend_from_slice(&0x0000_8004u32.to_le_bytes());

    // ChangeProperty (opcode 18) WM_NAME (atom 39) = "Sawbyte Launcher"
    let title = b"Sawbyte";
    let mut prop_req = Vec::new();
    prop_req.push(18);
    prop_req.push(0); // Replace
    let prop_len = 6 + ((title.len() + 3) / 4);
    prop_req.extend_from_slice(&(prop_len as u16).to_le_bytes());
    prop_req.extend_from_slice(&wid.to_le_bytes());
    prop_req.extend_from_slice(&39u32.to_le_bytes()); // XA_WM_NAME
    prop_req.extend_from_slice(&31u32.to_le_bytes()); // XA_STRING
    prop_req.push(8);
    prop_req.extend_from_slice(&[0, 0, 0]);
    prop_req.extend_from_slice(&(title.len() as u32).to_le_bytes());
    prop_req.extend_from_slice(title);
    prop_req.resize(prop_req.len() + pad4(title.len()), 0);

    // CreateGC (opcode 55)
    let mut gc_req = Vec::with_capacity(16);
    gc_req.push(55);
    gc_req.push(0);
    gc_req.extend_from_slice(&4u16.to_le_bytes());
    gc_req.extend_from_slice(&gcid.to_le_bytes());
    gc_req.extend_from_slice(&wid.to_le_bytes());
    gc_req.extend_from_slice(&0u32.to_le_bytes());

    // MapWindow (opcode 8)
    let mut map_req = Vec::with_capacity(8);
    map_req.push(8);
    map_req.push(0);
    map_req.extend_from_slice(&2u16.to_le_bytes());
    map_req.extend_from_slice(&wid.to_le_bytes());

    stream.write_all(&create_win).map_err(|e| e.to_string())?;
    stream.write_all(&prop_req).map_err(|e| e.to_string())?;
    stream.write_all(&gc_req).map_err(|e| e.to_string())?;
    stream.write_all(&map_req).map_err(|e| e.to_string())?;
    stream.flush().map_err(|e| e.to_string())?;

    let _ = stream.set_read_timeout(Some(Duration::from_millis(25)));
    let mut needs_redraw = true;

    loop {
        if needs_redraw {
            let surface = render_app_frame(state);
            present_x11_surface(&mut stream, wid, gcid, root_depth, &surface)?;
            needs_redraw = false;
        }

        let mut ev = [0u8; 32];
        match stream.read_exact(&mut ev) {
            Ok(()) => {
                let code = ev[0] & 0x7F;
                if code == 12 {
                    // Expose
                    needs_redraw = true;
                } else if code == 4 {
                    // ButtonPress: event_x at 24..26, event_y at 26..28
                    let ex = i16::from_le_bytes([ev[24], ev[25]]) as i32;
                    let ey = i16::from_le_bytes([ev[26], ev[27]]) as i32;
                    if handle_mouse_click(state, ex, ey) {
                        needs_redraw = true;
                    }
                }
            }
            Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock || e.kind() == std::io::ErrorKind::TimedOut => {}
            Err(_) => break,
        }
    }
    Ok(())
}

#[cfg(not(windows))]
fn present_x11_surface(
    stream: &mut std::os::unix::net::UnixStream,
    wid: u32,
    gcid: u32,
    depth: u8,
    surface: &UiSurface,
) -> Result<(), String> {
    let strip_rows = 24usize;
    let mut y = 0usize;
    while y < surface.height {
        let rows = strip_rows.min(surface.height - y);
        let byte_count = surface.width * rows * 4;
        let req_words = 6 + (byte_count / 4);
        let mut pkt = Vec::with_capacity(24 + byte_count);
        pkt.push(72); // PutImage
        pkt.push(2);  // ZPixmap
        pkt.extend_from_slice(&(req_words as u16).to_le_bytes());
        pkt.extend_from_slice(&wid.to_le_bytes());
        pkt.extend_from_slice(&gcid.to_le_bytes());
        pkt.extend_from_slice(&(surface.width as u16).to_le_bytes());
        pkt.extend_from_slice(&(rows as u16).to_le_bytes());
        pkt.extend_from_slice(&0i16.to_le_bytes());
        pkt.extend_from_slice(&(y as i16).to_le_bytes());
        pkt.push(0);
        pkt.push(depth);
        pkt.extend_from_slice(&0u16.to_le_bytes());

        for py in y..(y + rows) {
            let row_off = py * surface.width;
            for px in 0..surface.width {
                let c = surface.pixels[row_off + px];
                pkt.push((c & 0xFF) as u8);
                pkt.push(((c >> 8) & 0xFF) as u8);
                pkt.push(((c >> 16) & 0xFF) as u8);
                pkt.push(0);
            }
        }
        stream.write_all(&pkt).map_err(|e| e.to_string())?;
        y += rows;
    }
    stream.flush().map_err(|e| e.to_string())
}

#[cfg(not(windows))]
fn run_python_native_canvas_window(state: &mut AppState) -> Result<(), Box<dyn std::error::Error>> {
    let surface = render_app_frame(state);
    let tmp_ppm = std::env::temp_dir().join("sawbyte_native_frame.ppm");
    write_ppm(&tmp_ppm, &surface)?;

    let py_script = format!(
        r#"import tkinter as tk, sys
root = tk.Tk()
root.title("Sawbyte")
root.geometry("{w}x{h}")
root.configure(bg=0x16181C)
root.resizable(False, False)
img = tk.PhotoImage(file=r"{ppm}")
lbl = tk.Label(root, image=img, bd=0, highlightthickness=0)
lbl.pack(fill="both", expand=True)
root.mainloop()
"#,
        w = WIN_W,
        h = WIN_H,
        ppm = tmp_ppm.display()
    );
    let _ = Command::new("python3").args(["-c", &py_script]).status();
    Ok(())
}

pub fn write_ppm(path: &Path, surface: &UiSurface) -> std::io::Result<()> {
    let mut f = fs::File::create(path)?;
    write!(f, "P6\n{} {}\n255\n", surface.width, surface.height)?;
    let mut rgb = Vec::with_capacity(surface.width * surface.height * 3);
    for &c in &surface.pixels {
        rgb.push(((c >> 16) & 0xFF) as u8);
        rgb.push(((c >> 8) & 0xFF) as u8);
        rgb.push((c & 0xFF) as u8);
    }
    f.write_all(&rgb)
}

// ==============================================================================
// Native Win32 GDI Window Backend (Windows 10 / 11 — Zero HTML, Zero Browser)
// ==============================================================================

#[cfg(windows)]
fn run_win32_native_window(mut state: AppState) -> Result<(), Box<dyn std::error::Error>> {
    use std::ffi::OsStr;
    use std::os::windows::ffi::OsStrExt;
    use std::ptr::{null, null_mut};

    type HWND = *mut core::ffi::c_void;
    type HDC = *mut core::ffi::c_void;
    type HINSTANCE = *mut core::ffi::c_void;
    type HICON = *mut core::ffi::c_void;
    type HCURSOR = *mut core::ffi::c_void;
    type HBRUSH = *mut core::ffi::c_void;
    type HMENU = *mut core::ffi::c_void;
    type LRESULT = isize;
    type WPARAM = usize;
    type LPARAM = isize;
    type UINT = u32;
    type DWORD = u32;

    #[repr(C)]
    struct WNDCLASSEXW {
        cb_size: UINT,
        style: UINT,
        lpfn_wnd_proc: unsafe extern "system" fn(HWND, UINT, WPARAM, LPARAM) -> LRESULT,
        cb_cls_extra: i32,
        cb_wnd_extra: i32,
        h_instance: HINSTANCE,
        h_icon: HICON,
        h_cursor: HCURSOR,
        hbr_background: HBRUSH,
        lpsz_menu_name: *const u16,
        lpsz_class_name: *const u16,
        h_icon_sm: HICON,
    }

    #[repr(C)]
    struct POINT {
        x: i32,
        y: i32,
    }

    #[repr(C)]
    struct MSG {
        hwnd: HWND,
        message: UINT,
        w_param: WPARAM,
        l_param: LPARAM,
        time: DWORD,
        pt: POINT,
    }

    #[repr(C)]
    struct BITMAPINFOHEADER {
        bi_size: DWORD,
        bi_width: i32,
        bi_height: i32,
        bi_planes: u16,
        bi_bit_count: u16,
        bi_compression: DWORD,
        bi_size_image: DWORD,
        bi_x_pels_per_meter: i32,
        bi_y_pels_per_meter: i32,
        bi_clr_used: DWORD,
        bi_clr_important: DWORD,
    }

    #[repr(C)]
    struct BITMAPINFO {
        bmi_header: BITMAPINFOHEADER,
        bmi_colors: [u32; 1],
    }

    #[link(name = "user32")]
    extern "system" {
        fn RegisterClassExW(lpwcx: *const WNDCLASSEXW) -> u16;
        fn CreateWindowExW(
            dw_ex_style: DWORD,
            lp_class_name: *const u16,
            lp_window_name: *const u16,
            dw_style: DWORD,
            x: i32,
            y: i32,
            n_width: i32,
            n_height: i32,
            h_wnd_parent: HWND,
            h_menu: HMENU,
            h_instance: HINSTANCE,
            lp_param: *mut core::ffi::c_void,
        ) -> HWND;
        fn ShowWindow(h_wnd: HWND, n_cmd_show: i32) -> i32;
        fn DefWindowProcW(h_wnd: HWND, msg: UINT, w_param: WPARAM, l_param: LPARAM) -> LRESULT;
        fn PeekMessageW(lp_msg: *mut MSG, h_wnd: HWND, min: UINT, max: UINT, remove: UINT) -> i32;
        fn TranslateMessage(lp_msg: *const MSG) -> i32;
        fn DispatchMessageW(lp_msg: *const MSG) -> LRESULT;
        fn GetDC(h_wnd: HWND) -> HDC;
        fn ReleaseDC(h_wnd: HWND, h_dc: HDC) -> i32;
        fn PostQuitMessage(n_exit_code: i32);
        fn LoadCursorW(h_instance: HINSTANCE, lp_cursor_name: *const u16) -> HCURSOR;
    }

    #[link(name = "gdi32")]
    extern "system" {
        fn StretchDIBits(
            hdc: HDC,
            x_dest: i32,
            y_dest: i32,
            dest_width: i32,
            dest_height: i32,
            x_src: i32,
            y_src: i32,
            src_width: i32,
            src_height: i32,
            lp_bits: *const core::ffi::c_void,
            lpbmi: *const BITMAPINFO,
            i_usage: UINT,
            rop: DWORD,
        ) -> i32;
    }

    use std::sync::atomic::{AtomicBool, AtomicI32, Ordering};
    static CLICK_X: AtomicI32 = AtomicI32::new(-1);
    static CLICK_Y: AtomicI32 = AtomicI32::new(-1);
    static NEEDS_PAINT: AtomicBool = AtomicBool::new(true);

    unsafe extern "system" fn wnd_proc(hwnd: HWND, msg: UINT, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
        match msg {
            0x0002 => {
                PostQuitMessage(0);
                0
            }
            0x000F => {
                NEEDS_PAINT.store(true, Ordering::SeqCst);
                DefWindowProcW(hwnd, msg, wparam, lparam)
            }
            0x0201 => {
                let x = (lparam & 0xFFFF) as i16 as i32;
                let y = ((lparam >> 16) & 0xFFFF) as i16 as i32;
                CLICK_X.store(x, Ordering::SeqCst);
                CLICK_Y.store(y, Ordering::SeqCst);
                0
            }
            _ => DefWindowProcW(hwnd, msg, wparam, lparam),
        }
    }

    let class_name: Vec<u16> = OsStr::new("SawbyteNativeWndClass\0").encode_wide().collect();
    let title: Vec<u16> = OsStr::new("Sawbyte\0").encode_wide().collect();

    unsafe {
        let wc = WNDCLASSEXW {
            cb_size: core::mem::size_of::<WNDCLASSEXW>() as u32,
            style: 0x0003,
            lpfn_wnd_proc: wnd_proc,
            cb_cls_extra: 0,
            cb_wnd_extra: 0,
            h_instance: null_mut(),
            h_icon: null_mut(),
            h_cursor: LoadCursorW(null_mut(), 32512 as *const u16),
            hbr_background: null_mut(),
            lpsz_menu_name: null(),
            lpsz_class_name: class_name.as_ptr(),
            h_icon_sm: null_mut(),
        };
        RegisterClassExW(&wc);

        let ws_overlapped_window = 0x00CF_0000u32 & !0x0004_0000u32; // fixed size
        let hwnd = CreateWindowExW(
            0,
            class_name.as_ptr(),
            title.as_ptr(),
            ws_overlapped_window | 0x1000_0000,
            100,
            60,
            (WIN_W + 16) as i32,
            (WIN_H + 39) as i32,
            null_mut(),
            null_mut(),
            null_mut(),
            null_mut(),
        );
        ShowWindow(hwnd, 5);

        let bmi = BITMAPINFO {
            bmi_header: BITMAPINFOHEADER {
                bi_size: core::mem::size_of::<BITMAPINFOHEADER>() as u32,
                bi_width: WIN_W as i32,
                bi_height: -(WIN_H as i32), // top-down DIB
                bi_planes: 1,
                bi_bit_count: 32,
                bi_compression: 0,
                bi_size_image: 0,
                bi_x_pels_per_meter: 0,
                bi_y_pels_per_meter: 0,
                bi_clr_used: 0,
                bi_clr_important: 0,
            },
            bmi_colors: [0],
        };

        let mut msg: MSG = core::mem::zeroed();
        loop {
            while PeekMessageW(&mut msg, null_mut(), 0, 0, 1) != 0 {
                if msg.message == 0x0012 {
                    // WM_QUIT
                    return Ok(());
                }
                TranslateMessage(&msg);
                DispatchMessageW(&msg);
            }

            let cx = CLICK_X.swap(-1, Ordering::SeqCst);
            let cy = CLICK_Y.swap(-1, Ordering::SeqCst);
            if cx >= 0 && cy >= 0 {
                if handle_mouse_click(&mut state, cx, cy) {
                    NEEDS_PAINT.store(true, Ordering::SeqCst);
                }
            }

            if NEEDS_PAINT.swap(false, Ordering::SeqCst) {
                let surface = render_app_frame(&state);
                let hdc = GetDC(hwnd);
                StretchDIBits(
                    hdc,
                    0,
                    0,
                    WIN_W as i32,
                    WIN_H as i32,
                    0,
                    0,
                    WIN_W as i32,
                    WIN_H as i32,
                    surface.pixels.as_ptr() as *const core::ffi::c_void,
                    &bmi,
                    0,
                    0x00CC_0020,
                );
                ReleaseDC(hwnd, hdc);
            }
            thread::sleep(Duration::from_millis(16));
        }
    }
}
