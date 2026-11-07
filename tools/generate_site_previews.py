#!/usr/bin/env python3
"""Generates visual preview assets for the Sawbyte GitHub Pages site."""

import math
from pathlib import Path
import struct
from PIL import Image, ImageDraw, ImageFont

ROOT = Path(__file__).resolve().parents[1]
ASSETS_DIR = ROOT / "docs" / "assets"
ASSETS_DIR.mkdir(parents=True, exist_ok=True)

FONT_SANS_BOLD = "/usr/share/fonts/truetype/liberation/LiberationSans-Bold.ttf"
FONT_SANS_REG = "/usr/share/fonts/truetype/liberation/LiberationSans-Regular.ttf"
FONT_MONO_BOLD = "/usr/share/fonts/truetype/liberation/LiberationMono-Bold.ttf"
FONT_MONO_REG = "/usr/share/fonts/truetype/liberation/LiberationMono-Regular.ttf"


def get_font(path: str, size: int):
    try:
        return ImageFont.truetype(path, size)
    except Exception:
        return ImageFont.load_default()


def create_bmp_24bpp(width: int, height: int, pixel_fn) -> bytes:
    """Generates an uncompressed standard 24-bpp BMP image."""
    row_size = (width * 3 + 3) & ~3
    pixel_data_size = row_size * height
    file_size = 54 + pixel_data_size

    header = bytearray(b"BM")
    header.extend(struct.pack("<IHHI", file_size, 0, 0, 54))
    header.extend(struct.pack("<IIIHHIIIIII", 40, width, height, 1, 24, 0, pixel_data_size, 2835, 2835, 0, 0))

    pad_len = row_size - (width * 3)
    pad = b"\x00" * pad_len
    pixels = bytearray()

    for y in range(height):
        row = bytearray()
        for x in range(width):
            r, g, b = pixel_fn(x, height - 1 - y)
            row.extend(bytes([b, g, r]))
        if pad_len > 0:
            row.extend(pad)
        pixels.extend(row)

    return bytes(header + pixels)


def generate_potato_visualizer_frame():
    """Renders a simulated retro 3D voxel raymarched frame (first-person)."""
    w, h = 320, 200

    def get_color(x, y):
        if y < 100:
            sky_factor = y / 100.0
            r = int(9 + sky_factor * 12)
            g = int(13 + sky_factor * 18)
            b = int(22 + sky_factor * 35)
            if (abs(x - 160) <= 3 and abs(y - 100) <= 1) or (abs(y - 100) <= 3 and abs(x - 160) <= 1):
                return (88, 166, 255)
            return (r, g, b)
        else:
            depth = (y - 99) / 100.0
            persp_x = (x - 160) / depth

            grid_x = int((persp_x + 1000) / 20) % 2
            grid_z = int((1.0 / (depth + 0.05)) * 10) % 2

            if (x - 160) ** 2 + (y - 140) ** 2 < 400:
                if y < 135:
                    return (34, 197, 94)  # Zombie green head
                return (37, 99, 235)      # Blue shirt

            if grid_x ^ grid_z:
                shade = 0.8 + 0.2 * math.sin(x * 0.1)
                return (int(34 * shade), int(197 * shade), int(94 * shade))
            else:
                shade = 0.7 + 0.2 * math.cos(y * 0.1)
                return (int(146 * shade), int(64 * shade), int(14 * shade))

    bmp_data = create_bmp_24bpp(w, h, get_color)
    (ASSETS_DIR / "preview_potato.bmp").write_bytes(bmp_data)
    Image.open(ASSETS_DIR / "preview_potato.bmp").save(ASSETS_DIR / "preview_potato.png")
    print("Generated preview_potato.png & .bmp")


def generate_radar_visualizer_frame():
    """Renders a simulated top-down radar view."""
    w, h = 200, 200

    def get_color(x, y):
        cx, cy = 100, 100
        dist_sq = (x - cx) ** 2 + (y - cy) ** 2

        if dist_sq > 90 ** 2:
            return (15, 23, 42)

        dist = math.sqrt(dist_sq)
        if abs(dist - 30) < 1.2 or abs(dist - 60) < 1.2 or abs(dist - 85) < 1.2:
            return (56, 189, 248)

        if dist < 4:
            return (255, 255, 255)
        if abs(x - cx) < 2 and y < cy and (cy - y) < 15:
            return (56, 189, 248)

        if (x - 70) ** 2 + (y - 60) ** 2 < 12:
            return (239, 68, 68)
        if (x - 130) ** 2 + (y - 140) ** 2 < 12:
            return (34, 197, 94)
        if (x - 120) ** 2 + (y - 80) ** 2 < 9:
            return (245, 158, 11)

        grid = (x // 10) % 2 ^ (y // 10) % 2
        return (24, 32, 47) if grid else (19, 26, 38)

    bmp_data = create_bmp_24bpp(w, h, get_color)
    (ASSETS_DIR / "preview_radar.bmp").write_bytes(bmp_data)
    Image.open(ASSETS_DIR / "preview_radar.bmp").save(ASSETS_DIR / "preview_radar.png")
    print("Generated preview_radar.png & .bmp")


def generate_launcher_preview():
    """Renders high fidelity desktop launcher UI preview with real features."""
    w, h = 900, 560
    img = Image.new("RGB", (w, h), color=(15, 20, 28))
    draw = ImageDraw.Draw(img)

    f_title = get_font(FONT_SANS_BOLD, 15)
    f_body_b = get_font(FONT_SANS_BOLD, 13)
    f_body = get_font(FONT_SANS_REG, 13)
    f_mono_b = get_font(FONT_MONO_BOLD, 12)
    f_mono = get_font(FONT_MONO_REG, 11)
    f_small = get_font(FONT_MONO_REG, 10)

    # Window Chrome Header
    draw.rectangle([(0, 0), (w, 38)], fill=(10, 14, 21), outline=(38, 51, 69), width=1)
    # Window buttons
    draw.ellipse([(14, 13), (24, 23)], fill=(239, 68, 68))
    draw.ellipse([(32, 13), (42, 23)], fill=(245, 158, 11))
    draw.ellipse([(50, 13), (60, 23)], fill=(34, 197, 94))
    draw.text((80, 11), "Sawbyte Desktop Launcher v0.3.2  |  Universal Headless & GUI Automation Suite", fill=(148, 163, 184), font=f_title)

    # Left Navigation Sidebar
    sidebar_w = 200
    draw.rectangle([(0, 39), (sidebar_w, h)], fill=(12, 16, 24), outline=(38, 51, 69), width=1)

    nav_items = [
        ("Instances", True),
        ("Headless Engine", False),
        ("Fabric Bridge GUI", False),
        ("Modrinth Store", False),
        ("CurseForge Store", False),
        ("System CLI Setup", False),
        ("Settings", False),
    ]
    cur_y = 56
    for item, active in nav_items:
        if active:
            draw.rectangle([(8, cur_y), (sidebar_w - 8, cur_y + 34)], fill=(30, 41, 59), outline=(56, 189, 248), width=1)
            draw.text((24, cur_y + 9), item, fill=(56, 189, 248), font=f_body_b)
        else:
            draw.text((24, cur_y + 9), item, fill=(148, 163, 184), font=f_body)
        cur_y += 42

    # Bottom Sidebar: System CLI Install Button
    draw.rectangle([(10, h - 68), (sidebar_w - 10, h - 20)], fill=(20, 83, 45), outline=(34, 197, 94), width=1)
    draw.text((20, h - 56), "Install CLI to PATH", fill=(255, 255, 255), font=f_mono_b)
    draw.text((20, h - 38), "/usr/local/bin/sawbyte", fill=(134, 239, 172), font=f_small)

    # Main Workspace Area
    main_x = sidebar_w + 16
    draw.text((main_x, 56), "Active Instance: Vanilla 26.3 + Fabric Bridge", fill=(255, 255, 255), font=f_title)
    draw.text((main_x, 78), "Profile: Headless Core + Fabric Bridge Mod  |  Protocol: 774 (Minecraft 26.3)", fill=(148, 163, 184), font=f_mono)

    # 2 Launch Cards: Zero JVM Headless vs Graphical Fabric
    card_w = (w - main_x - 32) // 2

    # Card 1: Headless Rust Client
    c1_x1, c1_y1 = main_x, 110
    c1_x2, c1_y2 = main_x + card_w, 320
    draw.rectangle([(c1_x1, c1_y1), (c1_x2, c1_y2)], fill=(21, 27, 38), outline=(56, 189, 248), width=2)
    draw.text((c1_x1 + 16, c1_y1 + 16), "Headless Rust Engine", fill=(56, 189, 248), font=f_body_b)
    draw.text((c1_x1 + 16, c1_y1 + 38), "High-throughput, zero JVM, native physics.", fill=(148, 163, 184), font=f_body)
    features_c1 = [
        "- Memory Footprint: ~12 MB RSS",
        "- Tick Rate: 20.00 TPS locked",
        "- MCP AI Protocol: Port :8765 ready",
        "- Raymarched Potato Rasterizer enabled",
        "- Full Sub-voxel Raycasting & 3D A*",
    ]
    fy = c1_y1 + 72
    for feat in features_c1:
        draw.text((c1_x1 + 16, fy), feat, fill=(203, 213, 225), font=f_mono)
        fy += 22

    # Launch Headless Button
    draw.rectangle([(c1_x1 + 16, c1_y2 - 50), (c1_x2 - 16, c1_y2 - 16)], fill=(30, 58, 138), outline=(59, 130, 246), width=1)
    draw.text((c1_x1 + 80, c1_y2 - 40), "Launch Headless Engine", fill=(255, 255, 255), font=f_mono_b)

    # Card 2: Fabric Graphical Bridge
    c2_x1, c2_y1 = main_x + card_w + 16, 110
    c2_x2, c2_y2 = main_x + card_w * 2 + 16, 320
    draw.rectangle([(c2_x1, c2_y1), (c2_x2, c2_y2)], fill=(21, 27, 38), outline=(34, 197, 94), width=2)
    draw.text((c2_x1 + 16, c2_y1 + 16), "Fabric Graphical Client", fill=(34, 197, 94), font=f_body_b)
    draw.text((c2_x1 + 16, c2_y1 + 38), "Visual rendering, official Fabric 26.3 mod.", fill=(148, 163, 184), font=f_body)
    features_c2 = [
        "- Mod: sawbyte-bridge-0.3.2.jar",
        "- Auto Modrinth / CurseForge dependency fetch",
        "- Fabric API 26.3 automatically managed",
        "- Synchronized Live Bridge WebSocket",
        "- Real-time HUD Telemetry Overlays",
    ]
    fy = c2_y1 + 72
    for feat in features_c2:
        draw.text((c2_x1 + 16, fy), feat, fill=(203, 213, 225), font=f_mono)
        fy += 22

    # Launch Graphical Button
    draw.rectangle([(c2_x1 + 16, c2_y2 - 50), (c2_x2 - 16, c2_y2 - 16)], fill=(20, 83, 45), outline=(34, 197, 94), width=1)
    draw.text((c2_x1 + 80, c2_y2 - 40), "Launch Fabric GUI Client", fill=(255, 255, 255), font=f_mono_b)

    # Bottom Section: Mod & Addon Manager with Search
    box_y1 = 340
    box_y2 = h - 20
    draw.rectangle([(main_x, box_y1), (w - 16, box_y2)], fill=(10, 14, 21), outline=(38, 51, 69), width=1)
    draw.text((main_x + 16, box_y1 + 14), "Integrated Mod Store & Dependency Resolver", fill=(255, 255, 255), font=f_body_b)

    # Search bar
    draw.rectangle([(main_x + 360, box_y1 + 10), (w - 32, box_y1 + 36)], fill=(15, 20, 28), outline=(56, 189, 248), width=1)
    draw.text((main_x + 372, box_y1 + 16), "Search Modrinth & CurseForge: 'fabric-api' ...", fill=(148, 163, 184), font=f_mono)

    # Mod list items
    mods = [
        ("Sawbyte Fabric Bridge", "v0.3.2", "Core automation bridge & telemetry sync", "ACTIVE"),
        ("Fabric API", "v0.96.4+1.21", "Official Fabric lifecycle & rendering hooks", "INSTALLED"),
        ("Sodium / Embeddium", "v0.5.8", "Voxel pipeline optimization & frame pacing", "INSTALLED"),
    ]
    my = box_y1 + 52
    for name, ver, desc, status in mods:
        draw.rectangle([(main_x + 16, my), (w - 32, my + 38)], fill=(18, 24, 34), outline=(38, 51, 69), width=1)
        draw.text((main_x + 28, my + 11), name, fill=(255, 255, 255), font=f_mono_b)
        draw.text((main_x + 230, my + 11), ver, fill=(56, 189, 248), font=f_mono)
        draw.text((main_x + 320, my + 11), desc, fill=(148, 163, 184), font=f_mono)
        draw.rectangle([(w - 130, my + 8), (w - 44, my + 30)], fill=(20, 83, 45) if status == "ACTIVE" else (30, 41, 59))
        draw.text((w - 118, my + 12), status, fill=(34, 197, 94) if status == "ACTIVE" else (148, 163, 184), font=f_small)
        my += 46

    img.save(ASSETS_DIR / "preview_launcher.png")
    print("Generated preview_launcher.png")


def generate_graphical_bridge_preview():
    """Renders graphical Fabric client view with Sawbyte HUD telemetry."""
    w, h = 900, 560
    img = Image.new("RGB", (w, h), color=(15, 20, 28))
    draw = ImageDraw.Draw(img)

    f_mono_b = get_font(FONT_MONO_BOLD, 12)
    f_mono = get_font(FONT_MONO_REG, 11)
    f_small = get_font(FONT_MONO_REG, 10)

    # 3D Minecraft Scene (Simulated vibrant landscape)
    for y in range(280):
        factor = y / 280.0
        r = int(56 + factor * 60)
        g = int(140 + factor * 60)
        b = int(240 - factor * 30)
        draw.line([(0, y), (w, y)], fill=(r, g, b))

    # Sun
    draw.rectangle([(w - 180, 40), (w - 100, 120)], fill=(254, 240, 138), outline=(253, 224, 71), width=2)

    # Mountain & voxel landscape terrain
    for x in range(w):
        ground_y = int(280 + math.sin(x * 0.015) * 40 + math.cos(x * 0.04) * 15)
        draw.line([(x, ground_y), (x, ground_y + 8)], fill=(34, 197, 94))
        draw.line([(x, ground_y + 9), (x, ground_y + 45)], fill=(120, 53, 15))
        draw.line([(x, ground_y + 46), (x, h)], fill=(71, 85, 105))

    # Oak trees
    for tx in [120, 280, 540, 780]:
        ty = int(280 + math.sin(tx * 0.015) * 40 + math.cos(tx * 0.04) * 15)
        draw.rectangle([(tx - 8, ty - 60), (tx + 8, ty)], fill=(120, 53, 15))
        draw.rectangle([(tx - 36, ty - 120), (tx + 36, ty - 60)], fill=(22, 101, 52), outline=(21, 128, 61), width=2)

    # Crosshair
    cx, cy = w // 2, h // 2
    draw.line([(cx - 10, cy), (cx + 10, cy)], fill=(255, 255, 255), width=2)
    draw.line([(cx, cy - 10), (cx, cy + 10)], fill=(255, 255, 255), width=2)

    # Sawbyte Telemetry HUD (Top-Left)
    hud_w, hud_h = 360, 160
    draw.rectangle([(20, 20), (20 + hud_w, 20 + hud_h)], fill=(15, 20, 28), outline=(56, 189, 248), width=2)
    draw.rectangle([(20, 20), (20 + hud_w, 52)], fill=(21, 27, 38))
    draw.text((32, 28), "SAWBYTE FABRIC BRIDGE v0.3.2", fill=(56, 189, 248), font=f_mono_b)

    hud_data = [
        "Status: ACTIVE | Mode: Live Agent Autonomous",
        "Position: X: 142.38  Y: 64.00  Z: -89.44",
        "Heading: Yaw 184.2 deg | Pitch -4.8 deg",
        "Health: 20.0 / 20.0 | Food: 20 | Vitals: OK",
        "Active Task: Tech-Tree Mining (MineRL Spec)",
        "MCP Control: CONNECTED (tcp://127.0.0.1:8765)",
    ]
    hy = 60
    for line in hud_data:
        draw.text((32, hy), line, fill=(203, 213, 225), font=f_mono)
        hy += 16

    # Milestone Notifications Overlay (Top-Right)
    ms_w, ms_h = 340, 80
    ms_x = w - ms_w - 20
    draw.rectangle([(ms_x, 20), (ms_x + ms_w, 20 + ms_h)], fill=(15, 20, 28), outline=(34, 197, 94), width=2)
    draw.text((ms_x + 16, 30), "MILESTONE PERSISTED", fill=(34, 197, 94), font=f_mono_b)
    draw.text((ms_x + 16, 48), "Attempt #1 | Deaths: 0 | Ticks: 2,410", fill=(255, 255, 255), font=f_mono)
    draw.text((ms_x + 16, 66), "Crafted: diamond_pickaxe (recipe: 3x3 verified)", fill=(148, 163, 184), font=f_small)

    # Hotbar HUD (Bottom-Center)
    hb_w = 400
    hb_x = (w - hb_w) // 2
    hb_y = h - 64
    draw.rectangle([(hb_x, hb_y), (hb_x + hb_w, hb_y + 48)], fill=(10, 14, 21), outline=(71, 85, 105), width=2)
    slot_w = hb_w // 9
    for i in range(9):
        sx = hb_x + i * slot_w
        if i == 0:
            draw.rectangle([(sx, hb_y), (sx + slot_w, hb_y + 48)], outline=(56, 189, 248), width=3)
            draw.text((sx + 10, hb_y + 16), "SWD", fill=(56, 189, 248), font=f_small)
        elif i == 1:
            draw.text((sx + 10, hb_y + 16), "PCK", fill=(255, 255, 255), font=f_small)
        elif i == 2:
            draw.text((sx + 10, hb_y + 16), "BRD", fill=(245, 158, 11), font=f_small)
        else:
            draw.rectangle([(sx, hb_y), (sx + slot_w, hb_y + 48)], outline=(38, 51, 69), width=1)

    img.save(ASSETS_DIR / "preview_graphical_bridge.png")
    print("Generated preview_graphical_bridge.png")


if __name__ == "__main__":
    generate_potato_visualizer_frame()
    generate_radar_visualizer_frame()
    generate_launcher_preview()
    generate_graphical_bridge_preview()
    print("All preview assets successfully generated.")
