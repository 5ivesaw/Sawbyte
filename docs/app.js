/**
 * Sawbyte Official Portal Interactive Engine
 * - Soft Procedural Web Audio Minecraft UI Sound Synthesizer (with Mute Toggle)
 * - Exact §0–§f Minecraft Color Code Parser
 * - 3D Software Voxel Raymarcher & 2D Radar Simulator + Hotbar Slot Control
 * - Live Terminal Shell & Dynamic GitHub Release Downloader
 */

let sfxEnabled = true;
let audioCtx = null;

document.addEventListener('DOMContentLoaded', () => {
  initSoundEngine();
  initParticleCanvas();
  initReleaseDownloader();
  initVisualizerSimulator();
  initTerminalConsole();
  initBenchmarkTabs();
  initRespectsButton();
});

// ==============================================================================
// 1. Soft Procedural Minecraft Web Audio Synthesizer
// ==============================================================================

function ensureAudioCtx() {
  if (!sfxEnabled) return null;
  if (!audioCtx) {
    const AudioContextClass = window.AudioContext || window.webkitAudioContext;
    if (AudioContextClass) audioCtx = new AudioContextClass();
  }
  if (audioCtx && audioCtx.state === 'suspended') {
    audioCtx.resume().catch(() => {});
  }
  return audioCtx;
}

function playMcClick(pitch = 1.0) {
  const ctx = ensureAudioCtx();
  if (!ctx) return;
  const now = ctx.currentTime;

  // Soft wooden/stone Minecraft UI click
  const osc = ctx.createOscillator();
  const gain = ctx.createGain();
  const filter = ctx.createBiquadFilter();

  osc.type = 'triangle';
  osc.frequency.setValueAtTime(190 * pitch, now);
  osc.frequency.exponentialRampToValueAtTime(65 * pitch, now + 0.045);

  filter.type = 'lowpass';
  filter.frequency.setValueAtTime(950, now);

  gain.gain.setValueAtTime(0.08, now);
  gain.gain.exponentialRampToValueAtTime(0.001, now + 0.045);

  osc.connect(filter);
  filter.connect(gain);
  gain.connect(ctx.destination);

  osc.start(now);
  osc.stop(now + 0.05);
}

function playMcHoverTick() {
  const ctx = ensureAudioCtx();
  if (!ctx) return;
  const now = ctx.currentTime;

  const osc = ctx.createOscillator();
  const gain = ctx.createGain();
  osc.type = 'sine';
  osc.frequency.setValueAtTime(320, now);
  osc.frequency.exponentialRampToValueAtTime(220, now + 0.02);

  gain.gain.setValueAtTime(0.02, now);
  gain.gain.exponentialRampToValueAtTime(0.001, now + 0.02);

  osc.connect(gain);
  gain.connect(ctx.destination);
  osc.start(now);
  osc.stop(now + 0.025);
}

function playMcXpChime() {
  const ctx = ensureAudioCtx();
  if (!ctx) return;
  const now = ctx.currentTime;

  const freqs = [587.33, 783.99, 987.77, 1174.66];
  freqs.forEach((f, idx) => {
    const osc = ctx.createOscillator();
    const gain = ctx.createGain();
    osc.type = 'sine';
    const start = now + idx * 0.045;
    osc.frequency.setValueAtTime(f, start);
    gain.gain.setValueAtTime(0.06, start);
    gain.gain.exponentialRampToValueAtTime(0.001, start + 0.22);
    osc.connect(gain);
    gain.connect(ctx.destination);
    osc.start(start);
    osc.stop(start + 0.24);
  });
}

function initSoundEngine() {
  const toggleBtn = document.getElementById('sfx-toggle-btn');
  if (toggleBtn) {
    toggleBtn.addEventListener('click', () => {
      sfxEnabled = !sfxEnabled;
      toggleBtn.classList.toggle('muted', !sfxEnabled);
      toggleBtn.innerHTML = sfxEnabled ? '<span>[SFX: ON]</span>' : '<span>[SFX: OFF]</span>';
      if (sfxEnabled) playMcClick(1.15);
    });
  }

  document.querySelectorAll('.sfx-trigger').forEach((el) => {
    el.addEventListener('click', () => playMcClick(1.0));
    el.addEventListener('mouseenter', () => playMcHoverTick());
  });

  document.querySelectorAll('.sfx-hover').forEach((el) => {
    el.addEventListener('mouseenter', () => playMcHoverTick());
  });

  document.querySelectorAll('.sfx-xp').forEach((el) => {
    el.addEventListener('click', () => playMcXpChime());
  });
}

// ==============================================================================
// 2. Soft Starry Void & Pixel Particle Canvas
// ==============================================================================

function initParticleCanvas() {
  const canvas = document.getElementById('bg-canvas');
  if (!canvas) return;
  const ctx = canvas.getContext('2d');

  let width = (canvas.width = window.innerWidth);
  let height = (canvas.height = window.innerHeight);

  window.addEventListener('resize', () => {
    width = canvas.width = window.innerWidth;
    height = canvas.height = window.innerHeight;
  });

  const particles = [];
  const COUNT = Math.floor((width * height) / 16000);
  const palette = [
    'rgba(85, 255, 255, ', // §b Aqua
    'rgba(85, 255, 85, ',  // §a Green
    'rgba(255, 170, 0, ',  // §6 Gold
    'rgba(170, 170, 170, ' // §7 Gray
  ];

  for (let i = 0; i < COUNT; i++) {
    particles.push({
      x: Math.random() * width,
      y: Math.random() * height,
      size: Math.random() > 0.65 ? 3 : 2,
      vy: -(Math.random() * 0.3 + 0.08),
      vx: (Math.random() - 0.5) * 0.15,
      alpha: Math.random() * 0.5 + 0.15,
      pulse: Math.random() * 0.015 + 0.005,
      color: palette[Math.floor(Math.random() * palette.length)],
    });
  }

  function render() {
    ctx.clearRect(0, 0, width, height);
    for (const p of particles) {
      p.y += p.vy;
      p.x += p.vx;
      p.alpha += Math.sin(Date.now() * p.pulse) * 0.006;
      const clampedAlpha = Math.max(0.08, Math.min(0.65, p.alpha));

      if (p.y < 0) {
        p.y = height;
        p.x = Math.random() * width;
      }
      if (p.x < 0) p.x = width;
      if (p.x > width) p.x = 0;

      ctx.fillStyle = p.color + clampedAlpha + ')';
      ctx.fillRect(Math.floor(p.x), Math.floor(p.y), p.size, p.size);
    }
    requestAnimationFrame(render);
  }
  requestAnimationFrame(render);
}

// ==============================================================================
// 3. Dynamic GitHub Release Fetcher & Copy Command
// ==============================================================================

function initReleaseDownloader() {
  const LATEST_TAG = "v0.3.2";
  const BASE_URL = `https://github.com/5ivesaw/Sawbyte/releases/download/${LATEST_TAG}`;

  const platformMap = {
    "linux-launcher": `${BASE_URL}/sawbyte-launcher-ubuntu-24.04-X64.tar.gz`,
    "linux-engine": `${BASE_URL}/sawbyte-ubuntu-24.04-X64.tar.gz`,
    "linux-play": `${BASE_URL}/sawbyte-play-linux-x64.tar.gz`,
    "win-launcher": `${BASE_URL}/sawbyte-launcher-windows-2022-X64.zip`,
    "win-engine": `${BASE_URL}/sawbyte-windows-2022-X64.zip`,
    "win-play": `${BASE_URL}/sawbyte-play-windows-x64.zip`,
    "mac-launcher": `${BASE_URL}/sawbyte-launcher-macos-14-ARM64.tar.gz`,
    "mac-engine": `${BASE_URL}/sawbyte-macos-14-ARM64.tar.gz`,
    "mac-play": `${BASE_URL}/sawbyte-play-macos-arm64.tar.gz`,
    "bridge-jar": `${BASE_URL}/sawbyte-bridge-0.3.2.jar`,
  };

  for (const [id, url] of Object.entries(platformMap)) {
    const el = document.getElementById(`dl-${id}`);
    if (el) el.href = url;
  }

  fetch('https://api.github.com/repos/5ivesaw/Sawbyte/releases/latest')
    .then((res) => (res.ok ? res.json() : null))
    .then((release) => {
      if (!release || !release.assets) return;
      const releaseTag = release.tag_name || LATEST_TAG;
      document.querySelectorAll('.release-tag-badge').forEach((b) => (b.innerText = releaseTag));

      for (const asset of release.assets) {
        const name = asset.name;
        const url = asset.browser_download_url;
        if (name.includes('launcher-ubuntu')) setHref('dl-linux-launcher', url);
        else if (name.includes('launcher-windows')) setHref('dl-win-launcher', url);
        else if (name.includes('launcher-macos')) setHref('dl-mac-launcher', url);
        else if (name.includes('ubuntu-24.04')) setHref('dl-linux-engine', url);
        else if (name.includes('windows-2022')) setHref('dl-win-engine', url);
        else if (name.includes('macos-14')) setHref('dl-mac-engine', url);
        else if (name.includes('bridge')) setHref('dl-bridge-jar', url);
      }
    })
    .catch(() => {});

  function setHref(id, url) {
    const el = document.getElementById(id);
    if (el) el.href = url;
  }
}

window.copyCommand = function (text) {
  navigator.clipboard.writeText(text).then(() => {
    const btn = document.getElementById('copy-install-btn');
    if (btn) {
      btn.innerText = 'COPIED!';
      btn.className = 'copy-btn mc-a';
      setTimeout(() => {
        btn.innerText = 'COPY';
        btn.className = 'copy-btn';
      }, 2000);
    }
  });
};

// ==============================================================================
// 4. Interactive 3D Software Rasterizer & Tactical Radar Simulator
// ==============================================================================

window.switchPreviewMode = function (mode, btnElem) {
  const overlay = document.getElementById('image-preview-overlay');
  const hud = document.getElementById('mc-hud-overlay');
  const img = document.getElementById('preview-img-elem');

  document.querySelectorAll('#viewport .mode-tab').forEach((b) => b.classList.remove('active'));
  if (btnElem) btnElem.classList.add('active');

  if (mode === 'launcher') {
    overlay.style.display = 'block';
    if (hud) hud.style.display = 'none';
    img.src = 'assets/preview_launcher.png';
  } else if (mode === 'bridge') {
    overlay.style.display = 'block';
    if (hud) hud.style.display = 'none';
    img.src = 'assets/preview_graphical_bridge.png';
  }
};

function initVisualizerSimulator() {
  const canvas = document.getElementById('viz-canvas');
  if (!canvas) return;
  const ctx = canvas.getContext('2d');

  let mode = 'first_person';
  let yaw = 0;
  let pitch = 0;
  let posX = 8.5, posY = 64.0, posZ = 8.5;
  let tick = 1420;

  const modeButtons = document.querySelectorAll('.viz-mode-btn');
  modeButtons.forEach((btn) => {
    btn.addEventListener('click', () => {
      document.querySelectorAll('#viewport .mode-tab').forEach((b) => b.classList.remove('active'));
      btn.classList.add('active');
      mode = btn.dataset.mode;
      const overlay = document.getElementById('image-preview-overlay');
      const hud = document.getElementById('mc-hud-overlay');
      if (overlay) overlay.style.display = 'none';
      if (hud) hud.style.display = mode === 'first_person' ? 'flex' : 'none';
    });
  });

  // Hotbar slot clicks & number keys 1-9
  const slots = document.querySelectorAll('.mc-hotbar-slot');
  slots.forEach((slot) => {
    slot.addEventListener('click', () => {
      slots.forEach((s) => s.classList.remove('selected'));
      slot.classList.add('selected');
      playMcClick(1.2);
    });
  });

  window.addEventListener('keydown', (e) => {
    if (document.activeElement && document.activeElement.tagName === 'INPUT') return;
    if (e.key >= '1' && e.key <= '9') {
      const idx = parseInt(e.key, 10) - 1;
      if (slots[idx]) {
        slots.forEach((s) => s.classList.remove('selected'));
        slots[idx].classList.add('selected');
        playMcClick(1.2);
      }
      return;
    }

    const rad = (yaw * Math.PI) / 180;
    const speed = 0.4;
    if (e.key === 'w' || e.key === 'W') {
      posX -= Math.sin(rad) * speed;
      posZ += Math.cos(rad) * speed;
      tick++;
      updateTelemetry();
    } else if (e.key === 's' || e.key === 'S') {
      posX += Math.sin(rad) * speed;
      posZ -= Math.cos(rad) * speed;
      tick++;
      updateTelemetry();
    } else if (e.key === 'a' || e.key === 'A') {
      yaw -= 5;
      updateTelemetry();
    } else if (e.key === 'd' || e.key === 'D') {
      yaw += 5;
      updateTelemetry();
    }
  });

  function updateTelemetry() {
    const hud = document.getElementById('telemetry-display');
    if (hud) {
      hud.innerHTML = `
        <div class="mc-b">Sawbyte 0.3.2 (26.3 / Vanilla / Rust Headless)</div>
        <div>XYZ: <span class="mc-a">${posX.toFixed(2)} / ${posY.toFixed(2)} / ${posZ.toFixed(2)}</span></div>
        <div>Facing: <span class="mc-e">Yaw ${yaw.toFixed(1)}° / Pitch ${pitch.toFixed(1)}°</span></div>
        <div>Tick: <span class="mc-f">${tick}</span> <span class="mc-8">|</span> TPS: <span class="mc-a">20.00</span> <span class="mc-8">|</span> Deaths: <span class="mc-a">0</span></div>
      `;
    }
  }

  function renderVisualizer() {
    const w = canvas.width;
    const h = canvas.height;
    if (mode === 'radar') {
      renderRadar(ctx, w, h);
    } else {
      render3DVoxels(ctx, w, h);
    }
    requestAnimationFrame(renderVisualizer);
  }

  function renderRadar(ctx, w, h) {
    ctx.fillStyle = '#090b10';
    ctx.fillRect(0, 0, w, h);
    const cx = w / 2;
    const cy = h / 2;

    ctx.strokeStyle = '#1e2430';
    ctx.lineWidth = 2;
    for (let r = 40; r <= 160; r += 40) {
      ctx.beginPath();
      ctx.arc(cx, cy, r, 0, Math.PI * 2);
      ctx.stroke();
    }

    const sweepAngle = (Date.now() * 0.002) % (Math.PI * 2);
    ctx.strokeStyle = 'rgba(85, 255, 255, 0.45)';
    ctx.beginPath();
    ctx.moveTo(cx, cy);
    ctx.lineTo(cx + Math.cos(sweepAngle) * 160, cy + Math.sin(sweepAngle) * 160);
    ctx.stroke();

    const rad = (-yaw * Math.PI) / 180;
    ctx.fillStyle = '#FFFFFF';
    ctx.fillRect(cx - 3, cy - 3, 6, 6);

    ctx.strokeStyle = '#55FFFF';
    ctx.lineWidth = 2;
    ctx.beginPath();
    ctx.moveTo(cx, cy);
    ctx.lineTo(cx + Math.sin(rad) * 22, cy - Math.cos(rad) * 22);
    ctx.stroke();

    // Entities (Hostile Red, Passive Green, Diamond Drop Gold)
    ctx.fillStyle = '#FF5555';
    ctx.fillRect(cx + 45, cy - 60, 8, 8);
    ctx.fillStyle = '#55FF55';
    ctx.fillRect(cx - 55, cy + 30, 8, 8);
    ctx.fillStyle = '#FFAA00';
    ctx.fillRect(cx + 80, cy + 40, 6, 6);
  }

  function render3DVoxels(ctx, w, h) {
    // Crisp Night Sky
    ctx.fillStyle = '#0b0e17';
    ctx.fillRect(0, 0, w, h / 2);

    // Pixel Stars in Sky
    ctx.fillStyle = '#555555';
    ctx.fillRect(90, 40, 3, 3);
    ctx.fillRect(240, 70, 3, 3);
    ctx.fillRect(480, 35, 3, 3);
    ctx.fillRect(550, 95, 3, 3);

    // Blocky Moon
    ctx.fillStyle = '#FFFF55';
    ctx.fillRect(500, 36, 28, 28);
    ctx.fillStyle = '#FFFFFF';
    ctx.fillRect(504, 40, 20, 20);

    // Grass Ground Plane
    ctx.fillStyle = '#14381c';
    ctx.fillRect(0, h / 2, w, h / 2);

    const cx = w / 2;
    const cy = h / 2;
    const shift = ((yaw % 360) / 360) * 120 + Math.sin(Date.now() * 0.0012) * 8;

    // Voxel Grass Blocks
    for (let i = -5; i <= 5; i++) {
      const bx = Math.floor(cx + i * 56 - shift);
      const by = cy + 12;
      const bSize = 40;

      // Top Grass Face (§a)
      ctx.fillStyle = '#00AA00';
      ctx.fillRect(bx - bSize / 2, by - bSize / 2, bSize, 12);
      ctx.fillStyle = '#55FF55';
      ctx.fillRect(bx - bSize / 2 + 2, by - bSize / 2 + 2, bSize - 4, 6);

      // Side Dirt Face
      ctx.fillStyle = '#6e4c30';
      ctx.fillRect(bx - bSize / 2, by - bSize / 2 + 12, bSize, bSize - 4);
      ctx.fillStyle = '#4f3520';
      ctx.fillRect(bx - bSize / 2 + 6, by + 4, 8, 8);
      ctx.fillRect(bx + 4, by + 12, 6, 6);

      // Crisp Voxel Outline
      ctx.strokeStyle = '#060709';
      ctx.lineWidth = 2;
      ctx.strokeRect(bx - bSize / 2, by - bSize / 2, bSize, bSize + 8);
    }

    // Crisp Minecraft Crosshair
    ctx.fillStyle = '#FFFFFF';
    ctx.fillRect(cx - 8, cy - 1, 16, 2);
    ctx.fillRect(cx - 1, cy - 8, 2, 16);
  }

  updateTelemetry();
  requestAnimationFrame(renderVisualizer);
}

// ==============================================================================
// 5. Interactive Terminal Shell & Exact §0-§f Color Code Parser
// ==============================================================================

const MC_CODE_MAP = {
  '0': 'mc-0', '1': 'mc-1', '2': 'mc-2', '3': 'mc-3',
  '4': 'mc-4', '5': 'mc-5', '6': 'mc-6', '7': 'mc-7',
  '8': 'mc-8', '9': 'mc-9', 'a': 'mc-a', 'b': 'mc-b',
  'c': 'mc-c', 'd': 'mc-d', 'e': 'mc-e', 'f': 'mc-f'
};

function parseMinecraftColorText(rawText, defaultClass = 'mc-7') {
  const container = document.createElement('div');
  container.style.marginBottom = '6px';

  const parts = rawText.split(/(§[0-9a-frA-FR])/g);
  let currentClass = defaultClass;

  for (const part of parts) {
    if (part.startsWith('§') && part.length === 2) {
      const code = part[1].toLowerCase();
      if (code === 'r') currentClass = defaultClass;
      else if (MC_CODE_MAP[code]) currentClass = MC_CODE_MAP[code];
    } else if (part.length > 0) {
      const span = document.createElement('span');
      span.className = currentClass;
      span.textContent = part;
      container.appendChild(span);
    }
  }
  return container;
}

window.insertColorCode = function (code) {
  const input = document.getElementById('term-input');
  if (!input) return;
  input.value += code;
  input.focus();
  playMcClick(1.25);
};

function initTerminalConsole() {
  const input = document.getElementById('term-input');
  const body = document.getElementById('term-body');
  if (!input || !body) return;

  input.addEventListener('keydown', (e) => {
    if (e.key === 'Enter') {
      const cmd = input.value.trim();
      input.value = '';
      if (!cmd) return;

      playMcClick(1.1);
      appendFormattedLine(`§a$ §f${cmd}`);
      handleCommand(cmd);
    }
  });

  function appendFormattedLine(text, defaultClass = 'mc-7') {
    const lineElem = parseMinecraftColorText(text, defaultClass);
    body.appendChild(lineElem);
    body.scrollTop = body.scrollHeight;
  }

  function handleCommand(cmd) {
    if (cmd.includes('§')) {
      appendFormattedLine(`§8[Chat Preview] §r${cmd}`);
      return;
    }

    const parts = cmd.split(' ');
    const root = parts[0].toLowerCase();

    switch (root) {
      case 'help':
        appendFormattedLine('§6Available Sawbyte Live Commands:');
        appendFormattedLine('  §binspect.status                 §7- Query coordinates, vitals & tick');
        appendFormattedLine('  §binspect.milestones             §7- View recorded autonomous milestones');
        appendFormattedLine('  §binspect.deaths                 §7- View death count and exact causes');
        appendFormattedLine('  §bmilestone.save <name> [notes]  §7- Record new milestone to disk');
        appendFormattedLine('  §binventory.find <keyword>       §7- Search 45 inventory slots');
        appendFormattedLine('  §btask.navigate <x> <y> <z>      §7- 3D A* pathfinding to coordinates');
        appendFormattedLine('  §bcolors                         §7- Preview all 16 official §0-§f color codes');
        appendFormattedLine('  §bclear                          §7- Clear terminal console');
        break;
      case 'colors':
        appendFormattedLine('§0§0Black §1§1DarkBlue §2§2DarkGreen §3§3DarkAqua §4§4DarkRed §5§5Purple §6§6Gold §7§7Gray');
        appendFormattedLine('§8§8DarkGray §9§9Blue §a§aGreen §b§bAqua §c§cRed §d§dLightPurple §e§eYellow §f§fWhite');
        break;
      case 'inspect.status':
        appendFormattedLine('§apos=(8.50, 64.00, 8.50) §eyaw=0.0 pitch=0.0 §chealth=20.0 §6food=20 §btick=1420 status=live');
        break;
      case 'inspect.milestones':
        appendFormattedLine('§6milestones total=3:');
        appendFormattedLine('  §a[OK] [World Spawn] §7attempt #1 deaths=0 tick=40 pos=(8.5, 64.0, 8.5)');
        appendFormattedLine('  §a[OK] [Crafted Wooden Pickaxe] §7attempt #1 deaths=0 tick=320 pos=(12.0, 64.0, 9.2)');
        appendFormattedLine('  §a[OK] [Iron Ingot Smelted] §7attempt #1 deaths=0 tick=840 pos=(18.4, 52.0, 4.0)');
        break;
      case 'inspect.deaths':
        appendFormattedLine('§adeath_records total=0 §7(attempt #1 active; zero lethal encounters)');
        break;
      case 'milestone.save':
      case 'milestone.record':
        const name = parts[1] || 'Custom_Milestone';
        playMcXpChime();
        appendFormattedLine(`§amilestone_recorded §fname="${name}" §7attempt=1 deaths=0 tick=1420 health=20.0`);
        break;
      case 'inventory.find':
        const q = parts[1] || 'diamond_pickaxe';
        appendFormattedLine(`§b{"ok": true, "match": {"slot": 36, "id": "minecraft:${q}", "count": 1}}`);
        break;
      case 'task.navigate':
        appendFormattedLine(`§dlive 3D A* path active (14 waypoints) -> (${parts[1] || 120}, ${parts[2] || 64}, ${parts[3] || -30})`);
        break;
      case 'clear':
        body.innerHTML = '';
        break;
      default:
        appendFormattedLine(`§cUnknown command '${cmd}'. Type §ehelp §cfor command reference.`);
    }
  }
}

// ==============================================================================
// 6. Benchmark Tabs & Personal Tribute Respects Button
// ==============================================================================

function initBenchmarkTabs() {
  const tabs = document.querySelectorAll('.bench-tab-btn');
  const sections = document.querySelectorAll('.bench-content-section');
  tabs.forEach((tab) => {
    tab.addEventListener('click', () => {
      tabs.forEach((t) => t.classList.remove('active'));
      sections.forEach((s) => (s.style.display = 'none'));
      tab.classList.add('active');
      const target = document.getElementById(tab.dataset.target);
      if (target) target.style.display = 'block';
    });
  });
}

function initRespectsButton() {
  const btn = document.getElementById('pay-respects-btn');
  const label = document.getElementById('respects-label');
  const countEl = document.getElementById('tribute-count');
  const cooldownBar = document.getElementById('tribute-cooldown-bar');
  const cooldownFill = document.getElementById('tribute-cooldown-fill');
  const statusHint = document.getElementById('tribute-status-hint');
  const wrap = document.querySelector('.tribute-actions-wrap');

  if (!btn || !label) return;

  const STORAGE_KEY_COUNT = 'sawbyte_techno_tributes';
  const STORAGE_KEY_LAST_TIME = 'sawbyte_techno_last_tribute_ts';
  const COOLDOWN_MS = 4000;

  function safeStorageGet(key, fallback) {
    try {
      const v = localStorage.getItem(key);
      return v !== null ? v : fallback;
    } catch (_) {
      return fallback;
    }
  }

  function safeStorageSet(key, value) {
    try {
      localStorage.setItem(key, String(value));
    } catch (_) {}
  }

  function formatNumber(num) {
    return Number(num).toLocaleString('en-US');
  }

  let tributeCount = parseInt(safeStorageGet(STORAGE_KEY_COUNT, '0'), 10) || 0;
  let lastTributeTime = parseInt(safeStorageGet(STORAGE_KEY_LAST_TIME, '0'), 10) || 0;
  let cooldownTimer = null;
  let isCooldown = false;

  function updateDisplay() {
    if (countEl) countEl.textContent = formatNumber(tributeCount);

    if (tributeCount > 0) {
      label.innerHTML = `<span class="mc-e">[POPPY] RESPECTS PAID (o7 × ${tributeCount}) — TECHNOBLADE NEVER DIES</span>`;
    } else {
      label.innerHTML = 'LEAVE A TRIBUTE POPPY [o7]';
    }
  }

  function spawnFloatingPoppy() {
    if (!wrap) return;
    const poppy = document.createElement('div');
    poppy.className = 'tribute-floating-poppy';
    poppy.textContent = '+1 Poppy [o7]';

    const offset = (Math.random() - 0.5) * 80;
    poppy.style.transform = `translateX(${offset}px)`;
    poppy.style.top = '10px';

    wrap.appendChild(poppy);
    setTimeout(() => {
      if (poppy.parentNode) poppy.parentNode.removeChild(poppy);
    }, 1600);
  }

  function startCooldown(remainingMs) {
    isCooldown = true;
    btn.classList.add('tribute-btn-disabled');
    btn.setAttribute('aria-disabled', 'true');
    if (cooldownBar) cooldownBar.style.display = 'block';

    const startTime = Date.now() - (COOLDOWN_MS - remainingMs);

    if (cooldownTimer) clearInterval(cooldownTimer);

    cooldownTimer = setInterval(() => {
      const elapsed = Date.now() - startTime;
      const left = Math.max(0, COOLDOWN_MS - elapsed);
      const pct = Math.min(100, (elapsed / COOLDOWN_MS) * 100);

      if (cooldownFill) cooldownFill.style.width = pct + '%';

      const secsLeft = Math.ceil(left / 1000);
      label.innerHTML = `<span class="mc-8">[COOLDOWN: ${secsLeft}s]</span> <span class="mc-7">• TECHNOBLADE NEVER DIES</span>`;
      if (statusHint) statusHint.textContent = `Cooldown active (${secsLeft}s remaining) • Anti-spam protection`;

      if (left <= 0) {
        clearInterval(cooldownTimer);
        cooldownTimer = null;
        isCooldown = false;
        btn.classList.remove('tribute-btn-disabled');
        btn.removeAttribute('aria-disabled');
        if (cooldownBar) cooldownBar.style.display = 'none';
        if (cooldownFill) cooldownFill.style.width = '0%';
        if (statusHint) statusHint.textContent = 'Saved in browser storage • Anti-spam cooldown enabled';
        updateDisplay();
      }
    }, 80);
  }

  // Restore initial state and check if cooldown is still active from prior visit/refresh
  updateDisplay();
  const elapsedSinceLast = Date.now() - lastTributeTime;
  if (lastTributeTime > 0 && elapsedSinceLast < COOLDOWN_MS) {
    startCooldown(COOLDOWN_MS - elapsedSinceLast);
  }

  btn.addEventListener('click', (e) => {
    e.preventDefault();

    const now = Date.now();
    const elapsed = now - lastTributeTime;

    // Anti-spam guard: reject clicks during cooldown
    if (isCooldown || (lastTributeTime > 0 && elapsed < COOLDOWN_MS)) {
      if (typeof playMcClick === 'function') playMcClick(0.65);
      if (statusHint) {
        statusHint.innerHTML = '<span class="mc-c">[ANTI-SPAM] Please wait for cooldown to expire before paying respects again.</span>';
        setTimeout(() => {
          if (isCooldown) {
            statusHint.textContent = 'Cooldown active • Anti-spam protection';
          } else {
            statusHint.textContent = 'Saved in browser storage • Anti-spam cooldown enabled';
          }
        }, 1500);
      }
      return;
    }

    // Valid tribute
    tributeCount++;
    lastTributeTime = now;

    safeStorageSet(STORAGE_KEY_COUNT, tributeCount);
    safeStorageSet(STORAGE_KEY_LAST_TIME, lastTributeTime);

    if (typeof playMcXpChime === 'function') {
      playMcXpChime();
    }
    spawnFloatingPoppy();
    updateDisplay();

    startCooldown(COOLDOWN_MS);
  });
}
