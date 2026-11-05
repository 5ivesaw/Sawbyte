/**
 * Sawbyte Official Portal Interactive Application Engine
 * Starry Void Canvas, 3D Voxel Raymarcher Simulator, Live Terminal & Benchmarks
 */

document.addEventListener('DOMContentLoaded', () => {
  initParticleCanvas();
  initReleaseDownloader();
  initVisualizerSimulator();
  initTerminalConsole();
  initBenchmarkTabs();
});

// ==============================================================================
// 1. Starry Night & Floating Void Particle Canvas
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
  const COUNT = Math.floor((width * height) / 14000);

  for (let i = 0; i < COUNT; i++) {
    particles.push({
      x: Math.random() * width,
      y: Math.random() * height,
      size: Math.random() * 2 + 1,
      vy: -(Math.random() * 0.4 + 0.1),
      vx: (Math.random() - 0.5) * 0.2,
      alpha: Math.random() * 0.6 + 0.2,
      pulse: Math.random() * 0.02 + 0.01,
      color: Math.random() > 0.4 ? 'rgba(56, 189, 248, ' : 'rgba(34, 197, 94, ',
    });
  }

  function render() {
    ctx.clearRect(0, 0, width, height);
    for (const p of particles) {
      p.y += p.vy;
      p.x += p.vx;
      p.alpha += Math.sin(Date.now() * p.pulse) * 0.01;
      const clampedAlpha = Math.max(0.1, Math.min(0.85, p.alpha));

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
// 2. Dynamic GitHub Release Fetcher & Fallback
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

  // Populate download button hrefs
  for (const [id, url] of Object.entries(platformMap)) {
    const el = document.getElementById(`dl-${id}`);
    if (el) el.href = url;
  }

  // Attempt live API query for up-to-date downloads
  fetch('https://api.github.com/repos/5ivesaw/Sawbyte/releases/latest')
    .then((res) => (res.ok ? res.json() : null))
    .then((release) => {
      if (!release || !release.assets) return;
      const releaseTag = release.tag_name || LATEST_TAG;
      const tagBadges = document.querySelectorAll('.release-tag-badge');
      tagBadges.forEach((b) => (b.innerText = releaseTag));

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
    .catch(() => {
      // Offline or rate-limited; fallback intact
    });

  function setHref(id, url) {
    const el = document.getElementById(id);
    if (el) el.href = url;
  }
}

// Copy Command Functionality
window.copyCommand = function (text) {
  navigator.clipboard.writeText(text).then(() => {
    const btn = document.getElementById('copy-install-btn');
    if (btn) {
      btn.innerText = 'COPIED!';
      btn.style.color = '#4ade80';
      setTimeout(() => {
        btn.innerText = 'COPY';
        btn.style.color = '';
      }, 2000);
    }
  });
};

// ==============================================================================
// 3. Interactive Pure-Rust 3D Software Rasterizer Simulator
// ==============================================================================

function initVisualizerSimulator() {
  const canvas = document.getElementById('viz-canvas');
  if (!canvas) return;
  const ctx = canvas.getContext('2d');

  let mode = 'first_person'; // first_person, third_person, radar
  let yaw = 0;
  let pitch = 0;
  let posX = 8.5, posY = 64.0, posZ = 8.5;
  let tick = 1420;
  let health = 20.0;
  let food = 20;

  const modeButtons = document.querySelectorAll('.viz-mode-btn');
  modeButtons.forEach((btn) => {
    btn.addEventListener('click', () => {
      modeButtons.forEach((b) => b.classList.remove('active'));
      btn.classList.add('active');
      mode = btn.dataset.mode;
    });
  });

  // Keyboard navigation & look controls
  window.addEventListener('keydown', (e) => {
    if (document.activeElement && document.activeElement.tagName === 'INPUT') return;
    const rad = (yaw * Math.PI) / 180;
    const speed = 0.4;
    if (e.key === 'w' || e.key === 'ArrowUp') {
      posX -= Math.sin(rad) * speed;
      posZ += Math.cos(rad) * speed;
      tick++;
    } else if (e.key === 's' || e.key === 'ArrowDown') {
      posX += Math.sin(rad) * speed;
      posZ -= Math.cos(rad) * speed;
      tick++;
    } else if (e.key === 'a' || e.key === 'ArrowLeft') {
      yaw -= 4;
    } else if (e.key === 'd' || e.key === 'ArrowRight') {
      yaw += 4;
    }
    updateTelemetry();
  });

  function updateTelemetry() {
    const hud = document.getElementById('telemetry-display');
    if (hud) {
      hud.innerHTML = `
        <div>POS: (${posX.toFixed(2)}, ${posY.toFixed(1)}, ${posZ.toFixed(2)})</div>
        <div>YAW: ${yaw.toFixed(1)}&deg; | PITCH: ${pitch.toFixed(1)}&deg;</div>
        <div>HEALTH: ${health.toFixed(1)} | FOOD: ${food}</div>
        <div>TICK: ${tick} | ATTEMPT: #1 | DEATHS: 0</div>
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
    ctx.fillStyle = '#07090e';
    ctx.fillRect(0, 0, w, h);

    const cx = w / 2;
    const cy = h / 2;

    // Grid circles
    ctx.strokeStyle = '#1e293b';
    ctx.lineWidth = 1;
    for (let r = 40; r <= 160; r += 40) {
      ctx.beginPath();
      ctx.arc(cx, cy, r, 0, Math.PI * 2);
      ctx.stroke();
    }

    // Radar sweep line
    const sweepAngle = (Date.now() * 0.002) % (Math.PI * 2);
    ctx.strokeStyle = 'rgba(56, 189, 248, 0.4)';
    ctx.beginPath();
    ctx.moveTo(cx, cy);
    ctx.lineTo(cx + Math.cos(sweepAngle) * 160, cy + Math.sin(sweepAngle) * 160);
    ctx.stroke();

    // Player heading arrow
    const rad = (-yaw * Math.PI) / 180;
    ctx.fillStyle = '#ffffff';
    ctx.beginPath();
    ctx.arc(cx, cy, 4, 0, Math.PI * 2);
    ctx.fill();

    ctx.strokeStyle = '#38bdf8';
    ctx.lineWidth = 2;
    ctx.beginPath();
    ctx.moveTo(cx, cy);
    ctx.lineTo(cx + Math.sin(rad) * 20, cy - Math.cos(rad) * 20);
    ctx.stroke();

    // Entity Blips
    // Zombie (Red)
    ctx.fillStyle = '#ef4444';
    ctx.fillRect(cx + 45, cy - 60, 6, 6);
    // Sheep (Green)
    ctx.fillStyle = '#22c55e';
    ctx.fillRect(cx - 55, cy + 30, 6, 6);
    // Diamond Item Drop (Gold)
    ctx.fillStyle = '#f59e0b';
    ctx.fillRect(cx + 80, cy + 40, 5, 5);
  }

  function render3DVoxels(ctx, w, h) {
    // Sky
    const skyGrad = ctx.createLinearGradient(0, 0, 0, h / 2);
    skyGrad.addColorStop(0, '#090d16');
    skyGrad.addColorStop(1, '#1e293b');
    ctx.fillStyle = skyGrad;
    ctx.fillRect(0, 0, w, h / 2);

    // Ground Plane
    const groundGrad = ctx.createLinearGradient(0, h / 2, 0, h);
    groundGrad.addColorStop(0, '#15803d');
    groundGrad.addColorStop(1, '#052e16');
    ctx.fillStyle = groundGrad;
    ctx.fillRect(0, h / 2, w, h / 2);

    // Voxel Blocks Raymarching Simulation
    const cx = w / 2;
    const cy = h / 2;
    const timeFactor = Math.sin(Date.now() * 0.001) * 10;

    // Draw voxel perspective pillars
    for (let i = -5; i <= 5; i++) {
      const bx = cx + (i * 55) + timeFactor;
      const by = cy + 20;
      const bSize = 36;

      // Top face (Grass green)
      ctx.fillStyle = '#22c55e';
      ctx.fillRect(bx - bSize / 2, by - bSize / 2, bSize, bSize / 2);

      // Side face (Dirt brown)
      ctx.fillStyle = '#92400e';
      ctx.fillRect(bx - bSize / 2, by, bSize, bSize);

      ctx.strokeStyle = '#0a0d14';
      ctx.lineWidth = 1;
      ctx.strokeRect(bx - bSize / 2, by - bSize / 2, bSize, bSize * 1.5);
    }

    // Hostile Mob Billboard (Zombie)
    const zx = cx + 80;
    const zy = cy - 10;
    ctx.fillStyle = '#16a34a'; // head
    ctx.fillRect(zx - 8, zy - 24, 16, 16);
    ctx.fillStyle = '#2563eb'; // shirt
    ctx.fillRect(zx - 10, zy - 8, 20, 20);

    // Crosshair
    ctx.strokeStyle = '#38bdf8';
    ctx.lineWidth = 2;
    ctx.beginPath();
    ctx.moveTo(cx - 8, cy);
    ctx.lineTo(cx + 8, cy);
    ctx.moveTo(cx, cy - 8);
    ctx.lineTo(cx, cy + 8);
    ctx.stroke();
  }

  updateTelemetry();
  requestAnimationFrame(renderVisualizer);
}

// ==============================================================================
// 4. Interactive In-Browser Live Terminal Shell
// ==============================================================================

function initTerminalConsole() {
  const input = document.getElementById('term-input');
  const body = document.getElementById('term-body');
  if (!input || !body) return;

  const history = [];
  let histIdx = 0;

  input.addEventListener('keydown', (e) => {
    if (e.key === 'Enter') {
      const cmd = input.value.trim();
      input.value = '';
      if (!cmd) return;

      history.push(cmd);
      histIdx = history.length;
      appendLine(`$ ${cmd}`, '#ffffff');
      handleCommand(cmd);
    }
  });

  function appendLine(text, color = '#38bdf8') {
    const div = document.createElement('div');
    div.style.color = color;
    div.style.marginBottom = '4px';
    div.innerText = text;
    body.appendChild(div);
    body.scrollTop = body.scrollHeight;
  }

  function handleCommand(cmd) {
    const parts = cmd.split(' ');
    const root = parts[0].toLowerCase();

    switch (root) {
      case 'help':
        appendLine('Available Sawbyte live commands:');
        appendLine('  inspect.status                 - Query coordinates, vitals & tick');
        appendLine('  inspect.milestones             - View recorded autonomous milestones');
        appendLine('  inspect.deaths                 - View death count and exact causes');
        appendLine('  milestone.save <name> [notes]  - Record new milestone to disk');
        appendLine('  inventory.find <keyword>       - Search inventory for item');
        appendLine('  task.navigate <x> <y> <z>      - 3D A* pathfinding to coordinates');
        appendLine('  clear                          - Clear terminal output');
        break;
      case 'inspect.status':
        appendLine('pos=(8.50, 64.00, 8.50) yaw=0.0 pitch=0.0 health=20.0 food=20 tick=1420 status=live');
        break;
      case 'inspect.milestones':
        appendLine('milestones total=3:');
        appendLine('  - [World Spawn] attempt #1 deaths=0 tick=40 pos=(8.5, 64.0, 8.5) notes="Spawning in Plains"');
        appendLine('  - [Crafted Wooden Pickaxe] attempt #1 deaths=0 tick=320 pos=(12.0, 64.0, 9.2) notes="3x3 recipe verified"');
        appendLine('  - [Iron Ingot Smelted] attempt #1 deaths=0 tick=840 pos=(18.4, 52.0, 4.0) notes="Furnace cycle complete"');
        break;
      case 'inspect.deaths':
        appendLine('death_records total=0 (attempt #1 active; zero lethal encounters)');
        break;
      case 'milestone.save':
      case 'milestone.record':
        const name = parts[1] || 'Custom Milestone';
        const notes = parts.slice(2).join(' ') || 'Recorded via portal terminal';
        appendLine(`milestone_recorded name="${name}" attempt=1 deaths=0 tick=1420 pos=(8.50, 64.00, 8.50) health=20.0 food=20 inventory=[oak_log:4, iron_pickaxe:1]`, '#4ade80');
        break;
      case 'inventory.find':
        const q = parts[1] || 'iron';
        appendLine(`{"ok": true, "match": {"slot": 36, "id": "minecraft:${q}", "count": 1}}`);
        break;
      case 'task.navigate':
        appendLine(`live 3D A* path active (14 waypoints) from (8, 64, 8) -> (${parts[1]||10}, ${parts[2]||64}, ${parts[3]||10})`, '#a855f7');
        break;
      case 'clear':
        body.innerHTML = '';
        break;
      default:
        appendLine(`unknown live command '${cmd}'. Type 'help' for command reference.`, '#ef4444');
    }
  }
}

// ==============================================================================
// 5. Interactive Benchmark Visualizer Tabs & Simulator
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
