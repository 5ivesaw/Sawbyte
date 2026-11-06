/**
 * sawbyte - Final Minecraft 26.3 Interactive Engine
 * ============================================================================
 * 1. Full-Screen 2.5D Shader-Lit Minecraft World Background (#bg-canvas):
 *    - Volumetric golden-hour sunbeams / shader lighting, drifting blocky clouds,
 *      swaying oak/cherry voxel trees, shimmering shader water river, grass hills,
 *      and underground cliffside ore veins (Diamond, Emerald, Gold, Redstone, Iron)
 *    - Animated Steve, Alex, and Technoblade (with real avatars from mc-heads.net /
 *      minotar.net + articulated 2.5D bodies, swinging arms, walking legs, crown,
 *      and live activity bubbles) walking, mining, fishing, and farming potatoes!
 *    - Every block & tree in the 2.5D background is breakable on click/tap with
 *      official destroy_stage_0..9.png cracks, debris particles, floating drops,
 *      and 2.0-second respawn!
 * 2. Slower, Natural Login Intro + Singleplayer Misclick + Technoblade Vortex:
 *    - Accidentally clicks Singleplayer -> opens Select World -> clicks Cancel ->
 *      clicks Multiplayer -> Add Server -> pastes https://5ivesaw.github.io/Sawbyte
 *      & names Sawbyte -> Done -> selects Sawbyte -> Join Server
 *    - During Loading terrain..., dozens of Minecraft blocks & items swirl in a
 *      3D vortex and assemble into Technoblade!
 * 3. 15-Second Rotating Weapon/Tool Cursor (Desktop + Mobile Touch-to-Swing):
 *    - Rotates every 15s, follows 1.9+ attack cooldowns, plays item-specific sounds
 * 4. 100% True Non-Hardcoded Stochastic disc_maker Studio:
 *    - Uses Bjorklund's Euclidean rhythm generator E(k, 16, rot), Markov melodic
 *      step-walk with Consonance Gravity, and random instrument voice allocation
 */

const SAWBYTE_REPO = '5ivesaw/Sawbyte';
const TARGET_MC_VERSION = '26.3';

const MC_ASSET_BRANCH_CANDIDATES = [
  '1.21.4',
  '1.21.1',
  '1.21',
  '1.20.4',
  '1.19.3'
];

let activeMcAssetBranch = '1.21.4';
let sfxEnabled = true;
let audioCtx = null;

const SOUND_SHA1_CACHE = {
  'minecraft/sounds/random/orb.ogg': 'e9833a1512b57bcf88ac4fdcc8df4e5a7e9d701d'
};

const DECODED_BUFFER_CACHE = {};
const WORLD_TEX_IMAGES = {};

const ALL_MINECRAFT_DISCS = [
  'item/music_disc_pigstep.png',
  'item/music_disc_otherside.png',
  'item/music_disc_relic.png',
  'item/music_disc_creator.png',
  'item/music_disc_creator_music_box.png',
  'item/music_disc_precipice.png',
  'item/music_disc_5.png',
  'item/music_disc_13.png',
  'item/music_disc_cat.png',
  'item/music_disc_blocks.png',
  'item/music_disc_chirp.png',
  'item/music_disc_far.png',
  'item/music_disc_mall.png',
  'item/music_disc_mellohi.png',
  'item/music_disc_stal.png',
  'item/music_disc_strad.png',
  'item/music_disc_ward.png',
  'item/music_disc_11.png',
  'item/music_disc_wait.png'
];

const CURSOR_ARSENAL = [
  { id: 'netherite_sword', name: 'Netherite Sword', tex: 'item/netherite_sword.png', cooldownMs: 625, baseDamage: 8, affinity: 'sword', sound: 'entity/player/attack/sweep1.ogg', fallbackFreq: 340 },
  { id: 'diamond_sword', name: 'Diamond Sword', tex: 'item/diamond_sword.png', cooldownMs: 625, baseDamage: 7, affinity: 'sword', sound: 'entity/player/attack/sweep2.ogg', fallbackFreq: 380 },
  { id: 'golden_sword', name: 'Golden Pigman Sword', tex: 'item/golden_sword.png', cooldownMs: 625, baseDamage: 6, affinity: 'sword', sound: 'entity/player/attack/sweep3.ogg', fallbackFreq: 420 },
  { id: 'netherite_pickaxe', name: 'Netherite Pickaxe', tex: 'item/netherite_pickaxe.png', cooldownMs: 833, baseDamage: 11, affinity: 'pickaxe', sound: 'dig/stone1.ogg', fallbackFreq: 260 },
  { id: 'diamond_pickaxe', name: 'Diamond Pickaxe', tex: 'item/diamond_pickaxe.png', cooldownMs: 833, baseDamage: 9, affinity: 'pickaxe', sound: 'dig/stone2.ogg', fallbackFreq: 280 },
  { id: 'iron_pickaxe', name: 'Iron Pickaxe', tex: 'item/iron_pickaxe.png', cooldownMs: 833, baseDamage: 7, affinity: 'pickaxe', sound: 'dig/stone3.ogg', fallbackFreq: 250 },
  { id: 'netherite_axe', name: 'Netherite Axe', tex: 'item/netherite_axe.png', cooldownMs: 1000, baseDamage: 12, affinity: 'axe', sound: 'entity/player/attack/crit1.ogg', fallbackFreq: 190 },
  { id: 'diamond_axe', name: 'Diamond Axe', tex: 'item/diamond_axe.png', cooldownMs: 1000, baseDamage: 10, affinity: 'axe', sound: 'entity/player/attack/strong1.ogg', fallbackFreq: 210 },
  { id: 'mace', name: 'Heavy Mace', tex: 'item/mace.png', fallbackTex: 'item/golden_axe.png', cooldownMs: 1600, baseDamage: 22, affinity: 'all', sound: 'random/anvil_land.ogg', fallbackFreq: 120 },
  { id: 'trident', name: 'Trident', tex: 'item/trident.png', cooldownMs: 909, baseDamage: 9, affinity: 'all', sound: 'entity/player/attack/crit2.ogg', fallbackFreq: 460 },
  { id: 'diamond_shovel', name: 'Diamond Shovel', tex: 'item/diamond_shovel.png', cooldownMs: 1000, baseDamage: 8, affinity: 'shovel', sound: 'dig/grass1.ogg', fallbackFreq: 230 },
  { id: 'diamond_hoe', name: 'Diamond Hoe', tex: 'item/diamond_hoe.png', cooldownMs: 250, baseDamage: 5, affinity: 'hoe', sound: 'dig/cloth1.ogg', fallbackFreq: 520 }
];

let currentTool = CURSOR_ARSENAL[3];
let lastAttackTs = 0;
let lastChargeRatio = 1.0;
let toolCountdownSecs = 15;
let totalMinedBlocks = 0;

const JUKEBOX_DISCS = [
  { id: 'pigstep', title: 'Lena Raine: Pigstep', shortName: 'Pigstep', soundPath: 'records/pigstep.ogg', altSoundPath: 'music/game/nether/pigstep.ogg', tex: 'item/music_disc_pigstep.png' },
  { id: 'otherside', title: 'Lena Raine: otherside', shortName: 'otherside', soundPath: 'records/otherside.ogg', tex: 'item/music_disc_otherside.png' },
  { id: 'relic', title: 'Aaron Cherof: Relic', shortName: 'Relic', soundPath: 'records/relic.ogg', tex: 'item/music_disc_relic.png' },
  { id: 'cat', title: 'C418: cat', shortName: 'cat', soundPath: 'records/cat.ogg', tex: 'item/music_disc_cat.png' },
  { id: 'stal', title: 'C418: stal', shortName: 'stal', soundPath: 'records/stal.ogg', tex: 'item/music_disc_stal.png' }
];

let currentDiscIndex = 0;
let bgMusicAudio = null;
let bgMusicPlaying = false;
let bgMusicVolume = 0.18;
let userPausedMusic = false;

document.addEventListener('DOMContentLoaded', () => {
  initOfficialMinecraftFont();
  initMcAssetRegistry();
  initOptionalScreenshots();
  initMobileNavDrawer();
  initSoundEngine();
  initPigstepJukebox();
  initMinecraftIntroSequence();
  initWeaponToolCursor();
  initShader25DWorldCanvas();
  initDiscMakerStudio();
  initFloatingTooltips();
  initScrollReveal();
  initReleaseDownloader();
  initTerminalConsole();
  initBenchmarkTabs();
  initRespectsAndTechnoSoundboard();
});

// ==============================================================================
// 1. Official Minecraft Font, Mobile Menu & Multi-CDN Asset Resolver
// ==============================================================================

function initOfficialMinecraftFont() {
  if (!('FontFace' in window)) return;
  const fontSources = [
    'url(https://cdn.jsdelivr.net/gh/IdreesInc/Minecraft-Font@main/Minecraft.otf)',
    'url(https://raw.githubusercontent.com/IdreesInc/Minecraft-Font/main/Minecraft.otf)',
    'url(https://cdn.jsdelivr.net/gh/Magisk-Modules-Alt-Repo/MinecraftFont@main/Minecraft.ttf)'
  ];
  const font = new FontFace('MinecraftOfficial', fontSources.join(', '));
  font.load().then((loadedFace) => {
    document.fonts.add(loadedFace);
  }).catch(() => {});
}

function initMobileNavDrawer() {
  const toggleBtn = document.getElementById('mobile-menu-toggle-btn');
  const drawer = document.getElementById('nav-links-drawer');
  const label = document.getElementById('mobile-menu-toggle-label');
  if (!toggleBtn || !drawer) return;

  toggleBtn.addEventListener('click', () => {
    const isOpen = drawer.classList.toggle('mobile-open');
    if (label) label.textContent = isOpen ? 'Close' : 'Menu';
    playMcClick(1.1);
  });

  drawer.querySelectorAll('a.nav-link').forEach((a) => {
    a.addEventListener('click', () => {
      drawer.classList.remove('mobile-open');
      if (label) label.textContent = 'Menu';
    });
  });
}

function getTextureUrls(texPath) {
  const clean = texPath.replace(/^\/+/, '');
  const urls = [];
  for (const br of [activeMcAssetBranch, ...MC_ASSET_BRANCH_CANDIDATES]) {
    const rawUrl = `https://raw.githubusercontent.com/InventivetalentDev/minecraft-assets/${br}/assets/minecraft/textures/${clean}`;
    const cdnUrl = `https://cdn.jsdelivr.net/gh/InventivetalentDev/minecraft-assets@${br}/assets/minecraft/textures/${clean}`;
    const cloudUrl = `https://assets.mcasset.cloud/${br}/assets/minecraft/textures/${clean}`;
    if (!urls.includes(rawUrl)) urls.push(rawUrl);
    if (!urls.includes(cdnUrl)) urls.push(cdnUrl);
    if (!urls.includes(cloudUrl)) urls.push(cloudUrl);
  }
  return urls;
}

function attachMultiSourceImage(imgEl, urls) {
  let idx = 0;
  imgEl.crossOrigin = 'anonymous';
  const tryNext = () => {
    if (idx < urls.length) {
      imgEl.src = urls[idx++];
    }
  };
  imgEl.onerror = tryNext;
  tryNext();
}

function loadCachedTexture(key, urls) {
  const img = new Image();
  attachMultiSourceImage(img, urls);
  WORLD_TEX_IMAGES[key] = img;
  return img;
}

function initMcAssetRegistry() {
  document.querySelectorAll('.mc-pano-face[data-pano-face]').forEach((faceEl) => {
    const idx = faceEl.getAttribute('data-pano-face');
    const urls = getTextureUrls(`gui/title/background/panorama_${idx}.png`);
    const probe = new Image();
    let i = 0;
    const tryLoad = () => {
      if (i < urls.length) {
        const u = urls[i++];
        probe.onload = () => {
          faceEl.style.backgroundImage = `url('${u}')`;
        };
        probe.onerror = tryLoad;
        probe.src = u;
      }
    };
    tryLoad();
  });

  hydrateAllMinecraftTextures();

  // Preload 2.5D World Block Textures, Crack Stages 0..9, Items & Character Avatars
  const blockAssets = {
    grass_side: 'block/grass_block_side.png',
    grass_top: 'block/grass_block_top.png',
    dirt: 'block/dirt.png',
    oak_log: 'block/oak_log.png',
    oak_leaves: 'block/azalea_flowering_leaves.png',
    cherry_leaves: 'block/cherry_leaves.png',
    stone: 'block/stone.png',
    deepslate: 'block/deepslate.png',
    cobblestone: 'block/cobblestone.png',
    coal_ore: 'block/coal_ore.png',
    iron_ore: 'block/iron_ore.png',
    gold_ore: 'block/gold_ore.png',
    redstone_ore: 'block/redstone_ore.png',
    emerald_ore: 'block/emerald_ore.png',
    diamond_ore: 'block/diamond_ore.png',
    deepslate_diamond_ore: 'block/deepslate_diamond_ore.png',
    crafting_table: 'block/crafting_table_front.png',
    poppy: 'block/poppy.png',
    crown: 'item/golden_helmet.png',
    pickaxe: 'item/diamond_pickaxe.png',
    sword: 'item/netherite_sword.png',
    potato: 'item/baked_potato.png',
    fishing_rod: 'item/fishing_rod.png',
    totem: 'item/totem_of_undying.png',
    diamond: 'item/diamond.png',
    pigstep_disc: 'item/music_disc_pigstep.png'
  };

  for (const [k, path] of Object.entries(blockAssets)) {
    loadCachedTexture(k, getTextureUrls(path));
  }

  for (let c = 0; c <= 9; c++) {
    loadCachedTexture(`crack_${c}`, getTextureUrls(`block/destroy_stage_${c}.png`));
  }

  // Load Steve, Alex, and Technoblade Avatars from Public Minecraft Skin CDNs
  loadCachedTexture('head_steve', [
    'https://mc-heads.net/avatar/MHF_Steve/64',
    'https://minotar.net/helm/MHF_Steve/64.png'
  ]);
  loadCachedTexture('head_alex', [
    'https://mc-heads.net/avatar/MHF_Alex/64',
    'https://minotar.net/helm/MHF_Alex/64.png'
  ]);
  loadCachedTexture('head_techno', [
    'https://mc-heads.net/avatar/Technoblade/64',
    'https://minotar.net/helm/Technoblade/64.png',
    `https://raw.githubusercontent.com/${SAWBYTE_REPO}/main/docs/assets/techno_pig.png`
  ]);
  loadCachedTexture('techno_full', [
    `https://raw.githubusercontent.com/${SAWBYTE_REPO}/main/docs/assets/techno_pig.png`,
    'https://mc-heads.net/body/Technoblade/right',
    'https://mc-heads.net/avatar/Technoblade/128'
  ]);

  // Fetch Minecraft Asset Index JSON from GitHub for SHA-1 Sound Resolution
  const indexUrls = [
    'https://raw.githubusercontent.com/catppuccin/minecraft/main/resource-packs/Catppuccin%20UI/lang/indexes/1.18.2.json',
    'https://cdn.jsdelivr.net/gh/catppuccin/minecraft@main/resource-packs/Catppuccin%20UI/lang/indexes/1.18.2.json'
  ];

  (async () => {
    for (const url of indexUrls) {
      try {
        const res = await fetch(url);
        if (!res.ok) continue;
        const data = await res.json();
        if (data && data.objects) {
          for (const [key, val] of Object.entries(data.objects)) {
            if (key.startsWith('minecraft/sounds/') && val && val.hash) {
              SOUND_SHA1_CACHE[key] = val.hash;
            }
          }
          if (bgMusicAudio && !bgMusicPlaying && !userPausedMusic) {
            loadDiscIntoPlayer(currentDiscIndex, true);
          }
          break;
        }
      } catch (_) {}
    }
  })();
}

function hydrateAllMinecraftTextures() {
  document.querySelectorAll('img[data-mc-tex]').forEach((img) => {
    const texPath = img.getAttribute('data-mc-tex');
    const fallbackTex = img.getAttribute('data-mc-fallback');
    const repoAsset = img.getAttribute('data-gh-repo-asset');
    const urls = [];
    if (repoAsset) {
      urls.push(`https://raw.githubusercontent.com/${SAWBYTE_REPO}/main/${repoAsset}`);
      urls.push(`https://raw.githubusercontent.com/${SAWBYTE_REPO}/main/docs/${repoAsset}`);
    }
    urls.push(...getTextureUrls(texPath));
    if (fallbackTex) {
      urls.push(...getTextureUrls(fallbackTex));
    }
    attachMultiSourceImage(img, urls);
  });

  document.querySelectorAll('img[data-fallback-skin]').forEach((img) => {
    const primary = img.getAttribute('src');
    const fallback = img.getAttribute('data-fallback-skin');
    const filename = primary.split('/').pop();
    const urls = [
      primary,
      `https://raw.githubusercontent.com/${SAWBYTE_REPO}/main/assets/${filename}`,
      `https://cdn.jsdelivr.net/gh/${SAWBYTE_REPO}@main/docs/assets/${filename}`,
      fallback
    ];
    attachMultiSourceImage(img, urls);
  });
}

function initOptionalScreenshots() {
  document.querySelectorAll('img[data-optional-screenshot]').forEach((img) => {
    const path = img.getAttribute('data-optional-screenshot');
    if (!path) return;
    const probe = new Image();
    probe.onload = () => {
      img.src = path;
      img.style.display = 'block';
    };
    probe.onerror = () => {};
    probe.src = path;
  });
}

// ==============================================================================
// 2. Slower, Natural Login Intro + Singleplayer Misclick + Technoblade Vortex
// ==============================================================================

function initMinecraftIntroSequence() {
  const overlay = document.getElementById('mc-intro-overlay');
  const cursor = document.getElementById('mc-virtual-cursor');
  const replayBtn = document.getElementById('replay-intro-btn');
  const skipBtn = document.getElementById('mc-intro-skip-btn');
  if (!overlay) return;

  let panoramaActive = true;
  let introTimers = [];
  startPanoramaRenderer(() => panoramaActive);

  function clearAllIntroTimers() {
    introTimers.forEach((id) => clearTimeout(id));
    introTimers = [];
  }

  function scheduleIntroStep(fn, delayMs) {
    const id = setTimeout(fn, delayMs);
    introTimers.push(id);
  }

  const KEY_SHORT_COUNT = 'sawbyte_intro_short_visits';
  const KEY_LAST_TS = 'sawbyte_intro_last_ts';
  const LONG_WAIT_MS = 30 * 60 * 1000;

  let shortVisits = 0;
  let lastTs = 0;
  try {
    shortVisits = parseInt(localStorage.getItem(KEY_SHORT_COUNT) || '99', 10);
    lastTs = parseInt(localStorage.getItem(KEY_LAST_TS) || '0', 10);
  } catch (_) {
    shortVisits = 99;
  }

  const now = Date.now();
  const waitedLongTime = lastTs === 0 || (now - lastTs > LONG_WAIT_MS);
  const shouldPlayFullIntro = shortVisits >= 5 || waitedLongTime;

  try {
    localStorage.setItem(KEY_LAST_TS, String(now));
    if (shouldPlayFullIntro) {
      localStorage.setItem(KEY_SHORT_COUNT, '1');
    } else {
      localStorage.setItem(KEY_SHORT_COUNT, String(shortVisits + 1));
    }
  } catch (_) {}

  if (replayBtn) {
    replayBtn.addEventListener('click', () => {
      try {
        localStorage.setItem(KEY_SHORT_COUNT, '1');
      } catch (_) {}
      runIntroSequence(true);
    });
  }

  if (skipBtn) {
    skipBtn.addEventListener('click', (e) => {
      e.stopPropagation();
      clearAllIntroTimers();
      finishIntro();
    });
  }

  window.addEventListener('keydown', (e) => {
    if (e.key === 'Escape' && overlay.style.display !== 'none') {
      clearAllIntroTimers();
      finishIntro();
    }
  });

  runIntroSequence(shouldPlayFullIntro);

  function showIntroScreen(screenId) {
    document.querySelectorAll('.mc-intro-screen').forEach((s) => {
      s.classList.toggle('active', s.id === screenId);
    });
  }

  function moveCursorToElement(el, offsetX = 0, offsetY = 0) {
    if (!cursor || !el) return;
    const rect = el.getBoundingClientRect();
    cursor.style.left = `${rect.left + rect.width / 2 + offsetX}px`;
    cursor.style.top = `${rect.top + rect.height / 2 + offsetY}px`;
  }

  function clickButtonElement(btnEl, cb) {
    if (!btnEl) {
      if (cb) cb();
      return;
    }
    btnEl.classList.add('hovered');
    scheduleIntroStep(() => {
      if (cursor) cursor.classList.add('clicking');
      btnEl.classList.add('pressed');
      playMcClick(1.0);
      scheduleIntroStep(() => {
        if (cursor) cursor.classList.remove('clicking');
        btnEl.classList.remove('pressed');
        btnEl.classList.remove('hovered');
        if (cb) cb();
      }, 210);
    }, 340);
  }

  /**
   * Loading Terrain Phase + Swirling Minecraft Blocks & Items Forming Technoblade!
   */
  function runLoadingTerrainWithTechnoVortex(durationMs, onDone) {
    showIntroScreen('intro-screen-loading');
    if (cursor) cursor.style.display = 'none';

    const labelEl = document.getElementById('mc-loading-stage-text');
    const fillEl = document.getElementById('mc-terrain-progress-fill');
    const grayBox = document.getElementById('mc-chunk-gray-box');
    const vCanvas = document.getElementById('mc-techno-vortex-canvas');
    const vCtx = vCanvas ? vCanvas.getContext('2d') : null;

    if (labelEl) labelEl.textContent = 'Connecting to server...';
    if (fillEl) fillEl.style.width = '5%';
    if (grayBox) {
      grayBox.style.width = '24px';
      grayBox.style.height = '24px';
    }

    // Create 42 swirling Minecraft items/blocks that converge into Technoblade at the center!
    const itemKeys = ['crown', 'sword', 'potato', 'poppy', 'totem', 'diamond', 'pickaxe', 'gold_ore', 'diamond_ore', 'emerald_ore', 'pigstep_disc'];
    const particles = [];
    for (let i = 0; i < 42; i++) {
      const angle = (i / 42) * Math.PI * 2;
      const radius = 110 + (i % 3) * 35;
      particles.push({
        baseAngle: angle,
        startRadius: radius,
        texKey: itemKeys[i % itemKeys.length],
        targetX: ((i % 7) - 3) * 14,
        targetY: (Math.floor(i / 7) - 2.5) * 14
      });
    }

    const start = performance.now();
    function step(ts) {
      const elapsed = ts - start;
      const pct = Math.min(1, elapsed / durationMs);

      if (pct > 0.16 && pct < 0.72 && labelEl) {
        labelEl.textContent = 'Loading terrain...';
      } else if (pct >= 0.72 && labelEl) {
        labelEl.textContent = 'Assembling Technoblade tribute...';
      }

      if (fillEl) fillEl.style.width = `${Math.floor(pct * 100)}%`;
      if (grayBox) {
        const size = Math.floor(24 + pct * 54);
        grayBox.style.width = `${size}px`;
        grayBox.style.height = `${size}px`;
      }

      // Render Swirling Block/Item Vortex Forming Technoblade
      if (vCtx && vCanvas) {
        const cw = vCanvas.width;
        const ch = vCanvas.height;
        const cx = cw / 2;
        const cy = ch / 2 - 6;
        vCtx.clearRect(0, 0, cw, ch);
        vCtx.imageSmoothingEnabled = false;

        // Smooth ease-in-out convergence factor (0 = wide swirling ring, 1 = assembled Technoblade)
        const converge = Math.min(1, Math.max(0, (pct - 0.15) / 0.65));
        const easeConv = converge * converge * (3 - 2 * converge);

        for (const p of particles) {
          const spinAngle = p.baseAngle + elapsed * 0.0055 * (1.2 - easeConv * 0.9);
          const curRadius = p.startRadius * (1 - easeConv);
          const orbitX = Math.cos(spinAngle) * curRadius;
          const orbitY = Math.sin(spinAngle) * curRadius * 0.68;

          const px = cx + orbitX + p.targetX * easeConv;
          const py = cy + orbitY + p.targetY * easeConv;

          const img = WORLD_TEX_IMAGES[p.texKey];
          const sz = Math.floor(18 * (1 - easeConv * 0.35));
          if (img && img.complete && img.naturalWidth > 0) {
            vCtx.globalAlpha = 1 - easeConv * 0.55;
            vCtx.drawImage(img, Math.floor(px - sz / 2), Math.floor(py - sz / 2), sz, sz);
          } else {
            vCtx.fillStyle = '#FFAA00';
            vCtx.fillRect(Math.floor(px - 6), Math.floor(py - 6), 12, 12);
          }
        }
        vCtx.globalAlpha = 1;

        // As the blocks converge in the center, reveal Technoblade's Crowned Avatar & Golden Halo!
        if (easeConv > 0.25) {
          const revealAlpha = Math.min(1, (easeConv - 0.25) / 0.55);
          vCtx.save();
          vCtx.globalAlpha = revealAlpha;

          // Golden Crown Aura
          const grad = vCtx.createRadialGradient(cx, cy, 8, cx, cy, 74);
          grad.addColorStop(0, 'rgba(255, 170, 0, 0.55)');
          grad.addColorStop(1, 'rgba(255, 85, 85, 0)');
          vCtx.fillStyle = grad;
          vCtx.beginPath();
          vCtx.arc(cx, cy, 74, 0, Math.PI * 2);
          vCtx.fill();

          const technoImg = WORLD_TEX_IMAGES.head_techno || WORLD_TEX_IMAGES.techno_full;
          if (technoImg && technoImg.complete && technoImg.naturalWidth > 0) {
            const avatarSize = Math.floor(68 * (0.7 + 0.3 * revealAlpha));
            vCtx.drawImage(technoImg, Math.floor(cx - avatarSize / 2), Math.floor(cy - avatarSize / 2), avatarSize, avatarSize);
          }

          const crownImg = WORLD_TEX_IMAGES.crown;
          if (crownImg && crownImg.complete && crownImg.naturalWidth > 0) {
            vCtx.drawImage(crownImg, Math.floor(cx - 22), Math.floor(cy - 54), 44, 44);
          }

          vCtx.fillStyle = '#FFFF55';
          vCtx.font = '13px MinecraftOfficial, monospace';
          vCtx.textAlign = 'center';
          vCtx.fillText('TECHNOBLADE NEVER DIES [o7]', cx, ch - 10);
          vCtx.restore();
        }
      }

      if (pct < 1) {
        requestAnimationFrame(step);
      } else {
        playMcLevelUp();
        if (onDone) onDone();
      }
    }
    requestAnimationFrame(step);
  }

  function finishIntro() {
    overlay.classList.add('fade-out');
    document.body.classList.remove('intro-cursor-locked');
    setTimeout(() => {
      overlay.style.display = 'none';
      panoramaActive = false;
    }, 380);
  }

  function runIntroSequence(fullMode) {
    clearAllIntroTimers();
    panoramaActive = true;
    overlay.style.display = 'flex';
    overlay.classList.remove('fade-out');
    document.body.classList.add('intro-cursor-locked');

    const emptyMsg = document.getElementById('mc-mp-empty-msg');
    const serverRow = document.getElementById('mc-server-row-sawbyte');
    const joinBtn = document.getElementById('mc-btn-join-server');
    const editBtn = document.getElementById('mc-btn-edit-server');
    const deleteBtn = document.getElementById('mc-btn-delete-server');
    const nameInput = document.getElementById('mc-input-server-name');
    const addrInput = document.getElementById('mc-input-server-addr');

    if (emptyMsg) emptyMsg.style.display = 'block';
    if (serverRow) {
      serverRow.style.display = 'none';
      serverRow.classList.remove('selected');
    }
    if (joinBtn) joinBtn.classList.add('disabled');
    if (editBtn) editBtn.classList.add('disabled');
    if (deleteBtn) deleteBtn.classList.add('disabled');
    if (nameInput) {
      nameInput.textContent = 'Minecraft Server';
      nameInput.classList.remove('focused');
    }
    if (addrInput) {
      addrInput.textContent = '';
      addrInput.classList.remove('focused');
    }

    // Repeat visits 2..5: play the 2.3-second Loading Terrain + Technoblade Vortex
    if (!fullMode) {
      runLoadingTerrainWithTechnoVortex(2300, finishIntro);
      return;
    }

    // Full First-Visit Sequence (Comfortable, readable human pace!)
    showIntroScreen('intro-screen-title');
    if (cursor) {
      cursor.style.display = 'block';
      cursor.style.left = `${Math.floor(window.innerWidth * 0.75)}px`;
      cursor.style.top = `${Math.floor(window.innerHeight * 0.68)}px`;
    }

    const spBtn = document.getElementById('mc-btn-singleplayer');
    const spCancelBtn = document.getElementById('mc-btn-sp-cancel');
    const mpBtn = document.getElementById('mc-btn-multiplayer');
    const addServerBtn = document.getElementById('mc-btn-add-server');
    const addDoneBtn = document.getElementById('mc-btn-add-done');

    // Step 1: Move cursor to Singleplayer by mistake and click it!
    scheduleIntroStep(() => {
      if (spBtn) spBtn.classList.add('hovered');
      moveCursorToElement(spBtn);
    }, 650);

    scheduleIntroStep(() => {
      clickButtonElement(spBtn, () => {
        // Step 2: Inside Singleplayer Select World menu -> realize mistake -> move to Cancel
        showIntroScreen('intro-screen-singleplayer');

        scheduleIntroStep(() => {
          if (spCancelBtn) spCancelBtn.classList.add('hovered');
          moveCursorToElement(spCancelBtn);
        }, 750);

        scheduleIntroStep(() => {
          clickButtonElement(spCancelBtn, () => {
            // Step 3: Back on Title Screen -> move to Multiplayer and click!
            showIntroScreen('intro-screen-title');

            scheduleIntroStep(() => {
              if (mpBtn) mpBtn.classList.add('hovered');
              moveCursorToElement(mpBtn);
            }, 600);

            scheduleIntroStep(() => {
              clickButtonElement(mpBtn, () => {
                // Step 4: Open Play Multiplayer -> move to Add Server and click!
                showIntroScreen('intro-screen-multiplayer');

                scheduleIntroStep(() => {
                  if (addServerBtn) addServerBtn.classList.add('hovered');
                  moveCursorToElement(addServerBtn);
                }, 650);

                scheduleIntroStep(() => {
                  clickButtonElement(addServerBtn, () => {
                    // Step 5: Inside Edit Server Info -> paste URL into Server Address
                    showIntroScreen('intro-screen-add-server');

                    scheduleIntroStep(() => {
                      moveCursorToElement(addrInput);
                    }, 550);

                    scheduleIntroStep(() => {
                      if (cursor) cursor.classList.add('clicking');
                      playMcClick(1.15);
                      if (addrInput) {
                        addrInput.classList.add('focused');
                        addrInput.textContent = '_';
                      }
                      scheduleIntroStep(() => {
                        if (cursor) cursor.classList.remove('clicking');
                        if (addrInput) {
                          addrInput.textContent = 'https://5ivesaw.github.io/Sawbyte_';
                        }
                        playMcClick(1.3);
                      }, 280);
                    }, 1200);

                    // Step 6: Move up to Server Name and enter "Sawbyte"
                    scheduleIntroStep(() => {
                      moveCursorToElement(nameInput);
                    }, 1850);

                    scheduleIntroStep(() => {
                      if (cursor) cursor.classList.add('clicking');
                      playMcClick(1.15);
                      if (addrInput) {
                        addrInput.classList.remove('focused');
                        addrInput.textContent = 'https://5ivesaw.github.io/Sawbyte';
                      }
                      if (nameInput) {
                        nameInput.classList.add('focused');
                        nameInput.textContent = 'Sawbyte_';
                      }
                      scheduleIntroStep(() => {
                        if (cursor) cursor.classList.remove('clicking');
                      }, 180);
                    }, 2500);

                    // Step 7: Move down to Done and click
                    scheduleIntroStep(() => {
                      if (addDoneBtn) addDoneBtn.classList.add('hovered');
                      moveCursorToElement(addDoneBtn);
                    }, 3050);

                    scheduleIntroStep(() => {
                      clickButtonElement(addDoneBtn, () => {
                        // Step 8: Back on Play Multiplayer -> select Sawbyte -> click Join Server
                        if (emptyMsg) emptyMsg.style.display = 'none';
                        if (serverRow) serverRow.style.display = 'flex';
                        showIntroScreen('intro-screen-multiplayer');

                        scheduleIntroStep(() => {
                          moveCursorToElement(serverRow);
                        }, 550);

                        scheduleIntroStep(() => {
                          if (cursor) cursor.classList.add('clicking');
                          playMcClick(1.05);
                          if (serverRow) serverRow.classList.add('selected');
                          if (joinBtn) joinBtn.classList.remove('disabled');
                          if (editBtn) editBtn.classList.remove('disabled');
                          if (deleteBtn) deleteBtn.classList.remove('disabled');
                          scheduleIntroStep(() => {
                            if (cursor) cursor.classList.remove('clicking');
                          }, 180);
                        }, 1200);

                        scheduleIntroStep(() => {
                          if (joinBtn) joinBtn.classList.add('hovered');
                          moveCursorToElement(joinBtn);
                        }, 1750);

                        scheduleIntroStep(() => {
                          clickButtonElement(joinBtn, () => {
                            // Step 9: Loading terrain + Swirling Block Vortex Forming Technoblade!
                            runLoadingTerrainWithTechnoVortex(2850, finishIntro);
                          });
                        }, 2450);
                      });
                    }, 3750);
                  });
                }, 1350);
              });
            }, 1300);
          });
        }, 1500);
      });
    }, 1450);
  }
}

function startPanoramaRenderer(isActiveFn) {
  const canvas = document.getElementById('mc-panorama-canvas');
  if (!canvas) return;
  const ctx = canvas.getContext('2d');
  canvas.width = 480;
  canvas.height = 270;

  function draw(t) {
    if (!isActiveFn()) return;
    const w = canvas.width;
    const h = canvas.height;
    const pan = (t * 0.012) % 160;

    const skyGrad = ctx.createLinearGradient(0, 0, 0, h * 0.52);
    skyGrad.addColorStop(0, '#5478c4');
    skyGrad.addColorStop(1, '#9bb2db');
    ctx.fillStyle = skyGrad;
    ctx.fillRect(0, 0, w, h * 0.55);

    ctx.fillStyle = '#fff9d6';
    ctx.fillRect(w * 0.54 - pan * 0.15, 16, 36, 32);

    ctx.fillStyle = 'rgba(235, 240, 250, 0.78)';
    for (let i = -1; i < 5; i++) {
      const cx = ((i * 130 - pan * 0.4) % (w + 140)) - 40;
      ctx.fillRect(cx, 26 + (i % 2) * 18, 68, 14);
      ctx.fillRect(cx + 14, 18 + (i % 2) * 18, 42, 10);
    }

    ctx.fillStyle = '#485c3a';
    ctx.fillRect(0, h * 0.46, w, h * 0.18);
    ctx.fillStyle = '#526c7c';
    ctx.fillRect(w * 0.28 - pan * 0.2, h * 0.5, 95, 22);

    const groundGrad = ctx.createLinearGradient(0, h * 0.52, 0, h);
    groundGrad.addColorStop(0, '#783a16');
    groundGrad.addColorStop(1, '#54260c');
    ctx.fillStyle = groundGrad;
    ctx.fillRect(0, h * 0.55, w, h * 0.45);

    ctx.fillStyle = '#6b1d14';
    ctx.fillRect(28 - pan * 0.25, h * 0.28, 76, 52);
    ctx.fillStyle = '#b57b1e';
    ctx.fillRect(10 - pan * 0.3, h * 0.54, 130, 85);

    ctx.fillStyle = '#334d24';
    ctx.fillRect(w - 110 - pan * 0.2, h * 0.14, 115, 76);

    requestAnimationFrame(draw);
  }
  requestAnimationFrame(draw);
}

// ==============================================================================
// 3. 15-Second Rotating Weapon & Tool Cursor (Desktop Mouse + Mobile Touch Tap)
// ==============================================================================

function initWeaponToolCursor() {
  const cursorEl = document.getElementById('mc-weapon-cursor');
  const itemWrap = document.getElementById('mc-cursor-item-wrap');
  const itemImg = document.getElementById('mc-cursor-item-img');
  const cdWrap = document.getElementById('mc-cursor-cooldown-wrap');
  const cdFill = document.getElementById('mc-cursor-cooldown-fill');

  const hudImg = document.getElementById('hud-tool-img');
  const hudName = document.getElementById('hud-tool-name');
  const hudTimer = document.getElementById('hud-tool-timer');
  const hudCdText = document.getElementById('hud-tool-cd-text');
  const hudCdOverlay = document.getElementById('hud-tool-cd-overlay');
  const swapBtn = document.getElementById('tool-swap-btn');

  let hideTouchCursorTimer = null;

  function equipRandomTool(announce = false) {
    const candidates = CURSOR_ARSENAL.filter((t) => t.id !== currentTool.id);
    currentTool = candidates[Math.floor(Math.random() * candidates.length)] || CURSOR_ARSENAL[0];
    toolCountdownSecs = 15;

    const urls = getTextureUrls(currentTool.tex);
    if (currentTool.fallbackTex) {
      urls.push(...getTextureUrls(currentTool.fallbackTex));
    }
    if (itemImg) attachMultiSourceImage(itemImg, urls);
    if (hudImg) attachMultiSourceImage(hudImg, urls);
    if (hudName) hudName.textContent = currentTool.name;
    if (hudCdText) hudCdText.textContent = `${(currentTool.cooldownMs / 1000).toFixed(2)}s`;

    if (announce) {
      playMinecraftOgg('random/pop.ogg', 0.35, 1.15, () => synthClickFallback(1.3));
    }
  }

  equipRandomTool(false);

  setInterval(() => {
    if (document.body.classList.contains('intro-cursor-locked')) return;
    toolCountdownSecs--;
    if (toolCountdownSecs <= 0) {
      equipRandomTool(true);
    }
    if (hudTimer) hudTimer.textContent = `${toolCountdownSecs}s`;
  }, 1000);

  if (swapBtn) {
    swapBtn.addEventListener('click', (e) => {
      e.stopPropagation();
      equipRandomTool(true);
      showAdvancementToast('Equipped New Tool!', `${currentTool.name} (CD ${(currentTool.cooldownMs / 1000).toFixed(2)}s)`, currentTool.tex);
    });
  }

  window.swapCursorToolNow = () => equipRandomTool(true);

  window.addEventListener('pointermove', (e) => {
    if (!cursorEl || e.pointerType === 'touch') return;
    cursorEl.classList.add('cursor-visible');
    cursorEl.style.left = `${e.clientX}px`;
    cursorEl.style.top = `${e.clientY}px`;
  });

  window.addEventListener('pointerdown', (e) => {
    if (document.body.classList.contains('intro-cursor-locked')) return;

    // On mobile/touch, show the weapon cursor right where the user tapped!
    if (cursorEl) {
      cursorEl.classList.add('cursor-visible');
      cursorEl.style.left = `${e.clientX}px`;
      cursorEl.style.top = `${e.clientY}px`;
      if (e.pointerType === 'touch') {
        if (hideTouchCursorTimer) clearTimeout(hideTouchCursorTimer);
        hideTouchCursorTimer = setTimeout(() => {
          cursorEl.classList.remove('cursor-visible');
        }, 1200);
      }
    }

    const now = performance.now();
    const elapsed = now - lastAttackTs;
    lastChargeRatio = Math.min(1, elapsed / currentTool.cooldownMs);
    lastAttackTs = now;

    if (itemWrap) {
      itemWrap.classList.remove('swinging');
      void itemWrap.offsetWidth;
      itemWrap.classList.add('swinging');
    }

    if (lastChargeRatio >= 0.82) {
      playMinecraftOgg(currentTool.sound, 0.42, 0.95 + Math.random() * 0.1, () => {
        synthToolSoundFallback(currentTool.fallbackFreq, 1.0);
      });
      if (currentTool.id === 'golden_sword' && Math.random() < 0.35) {
        setTimeout(() => playMcPigSound(), 90);
      }
      if (['sword', 'axe', 'all'].includes(currentTool.affinity)) {
        spawnSweepSlash(e.clientX, e.clientY);
      }
    } else {
      playMinecraftOgg('entity/player/attack/weak1.ogg', 0.28, 1.2, () => {
        synthToolSoundFallback(currentTool.fallbackFreq * 0.7, 0.4);
      });
    }

    animateToolCooldown(currentTool.cooldownMs);

    // Check if the click/tap hit a 2.5D block or character in the background scene!
    const tag = e.target?.tagName?.toLowerCase() || '';
    if (!['a', 'button', 'input', 'select', 'textarea'].includes(tag) && !e.target?.closest('a, button, input, select, .seq-cell, .tribute-figure-card, .jukebox-dock, .tool-hud-dock')) {
      if (typeof window.hitShader25DWorldAt === 'function') {
        window.hitShader25DWorldAt(e.clientX, e.clientY);
      }
    }
  });

  let cdRafId = null;
  function animateToolCooldown(durationMs) {
    if (cdRafId) cancelAnimationFrame(cdRafId);
    if (cdWrap) cdWrap.classList.add('active');
    const start = performance.now();

    function step(ts) {
      const pct = Math.min(1, (ts - start) / durationMs);
      if (cdFill) cdFill.style.width = `${Math.floor(pct * 100)}%`;
      if (hudCdOverlay) hudCdOverlay.style.height = `${Math.floor((1 - pct) * 100)}%`;

      if (pct < 1) {
        cdRafId = requestAnimationFrame(step);
      } else {
        if (cdWrap) cdWrap.classList.remove('active');
        if (hudCdOverlay) hudCdOverlay.style.height = '0%';
      }
    }
    cdRafId = requestAnimationFrame(step);
  }
}

function spawnSweepSlash(x, y) {
  const slash = document.createElement('div');
  slash.className = 'mc-sweep-slash';
  slash.style.left = `${x}px`;
  slash.style.top = `${y}px`;
  document.body.appendChild(slash);
  setTimeout(() => {
    if (slash.parentNode) slash.parentNode.removeChild(slash);
  }, 250);
}

function synthToolSoundFallback(freq = 280, power = 1.0) {
  const ctx = ensureAudioCtx();
  if (!ctx) return;
  const now = ctx.currentTime;
  const osc = ctx.createOscillator();
  const gain = ctx.createGain();
  osc.type = power > 0.8 ? 'sawtooth' : 'triangle';
  osc.frequency.setValueAtTime(freq, now);
  osc.frequency.exponentialRampToValueAtTime(freq * 0.35, now + 0.09);
  gain.gain.setValueAtTime(0.08 * power, now);
  gain.gain.exponentialRampToValueAtTime(0.001, now + 0.095);
  osc.connect(gain);
  gain.connect(ctx.destination);
  osc.start(now);
  osc.stop(now + 0.1);
}

// ==============================================================================
// 4. Full-Screen Interactive 2.5D Shader-Lit Minecraft World Background
//    - Volumetric golden-hour sunbeams, shader water river, swaying trees,
//      breakable 2.5D blocks & ores (respawn after 2s), and animated
//      Steve, Alex, and Technoblade!
// ==============================================================================

const WORLD_BLOCK_DEFS = {
  grass: { name: 'Grass Block', tex: 'grass_side', topColor: '#48b535', hp: 14, cat: 'shovel', sound: 'dig/grass1.ogg' },
  dirt: { name: 'Dirt', tex: 'dirt', topColor: '#79553a', hp: 12, cat: 'shovel', sound: 'dig/grass2.ogg' },
  oak_log: { name: 'Oak Log', tex: 'oak_log', topColor: '#9c7f4e', hp: 18, cat: 'axe', sound: 'dig/wood1.ogg', dropTex: 'block/oak_log.png', dropLabel: '+1 Oak Log' },
  oak_leaves: { name: 'Oak Leaves', tex: 'oak_leaves', topColor: '#3a9628', hp: 6, cat: 'hoe', sound: 'dig/grass3.ogg' },
  cherry_leaves: { name: 'Cherry Leaves', tex: 'cherry_leaves', topColor: '#f29bc0', hp: 6, cat: 'hoe', sound: 'dig/grass3.ogg' },
  stone: { name: 'Stone', tex: 'stone', topColor: '#7a7a7a', hp: 20, cat: 'pickaxe', sound: 'dig/stone1.ogg' },
  deepslate: { name: 'Deepslate', tex: 'deepslate', topColor: '#4a4a52', hp: 24, cat: 'pickaxe', sound: 'dig/stone2.ogg' },
  gold_ore: { name: 'Gold Ore', tex: 'gold_ore', topColor: '#facc15', hp: 26, cat: 'pickaxe', sound: 'dig/stone3.ogg', dropTex: 'item/raw_gold.png', dropLabel: '+1 Raw Gold', glow: 'rgba(255, 204, 21, 0.35)' },
  redstone_ore: { name: 'Redstone Ore', tex: 'redstone_ore', topColor: '#ef4444', hp: 26, cat: 'pickaxe', sound: 'dig/stone4.ogg', dropTex: 'item/redstone.png', dropLabel: '+4 Redstone', glow: 'rgba(239, 68, 68, 0.38)' },
  emerald_ore: { name: 'Emerald Ore', tex: 'emerald_ore', topColor: '#22c55e', hp: 28, cat: 'pickaxe', sound: 'dig/stone1.ogg', dropTex: 'item/emerald.png', dropLabel: '+1 Emerald', glow: 'rgba(34, 197, 94, 0.38)' },
  diamond_ore: { name: 'Diamond Ore', tex: 'diamond_ore', topColor: '#38bdf8', hp: 30, cat: 'pickaxe', sound: 'dig/stone2.ogg', dropTex: 'item/diamond.png', dropLabel: '+1 Diamond', glow: 'rgba(56, 189, 248, 0.45)' },
  crafting_table: { name: 'Crafting Table', tex: 'crafting_table', topColor: '#a37246', hp: 18, cat: 'axe', sound: 'dig/wood2.ogg' }
};

function initShader25DWorldCanvas() {
  const canvas = document.getElementById('bg-canvas');
  if (!canvas) return;
  const ctx = canvas.getContext('2d');

  let width = (canvas.width = window.innerWidth);
  let height = (canvas.height = window.innerHeight);

  let worldBlocks = [];
  let characters = [];
  let cherryPetals = [];

  function buildSceneGrid() {
    width = canvas.width = window.innerWidth;
    height = canvas.height = window.innerHeight;
    worldBlocks = [];

    const bSize = width < 640 ? 36 : 44;
    const cols = Math.ceil(width / bSize) + 1;
    const baseGroundRowY = Math.floor(height * 0.72);

    // Terrain heightmap: left hill, center shader river, right hill + ore cliff
    const surfaceYByCol = [];

    for (let c = 0; c < cols; c++) {
      const normX = c / Math.max(1, cols - 1);
      const isRiver = normX > 0.38 && normX < 0.58;
      let hillOffset = 0;
      if (normX <= 0.38) {
        hillOffset = Math.round(Math.sin(normX * Math.PI * 2.2) * 2);
      } else if (normX >= 0.58) {
        hillOffset = Math.round(Math.sin((normX - 0.55) * Math.PI * 2.0) * 2) + 1;
      }

      const topY = isRiver ? baseGroundRowY + bSize : baseGroundRowY - hillOffset * bSize;
      surfaceYByCol[c] = topY;

      // Water blocks in river basin
      if (isRiver) {
        worldBlocks.push({
          col: c,
          x: c * bSize,
          y: baseGroundRowY,
          size: bSize,
          type: 'water',
          breakable: false
        });
      }

      // Surface Grass Block
      addBlock(c * bSize, topY, bSize, isRiver ? 'dirt' : 'grass');

      // Subsurface Dirt + Stone + Exposed Ores down to bottom of screen
      const oreChoices = ['stone', 'stone', 'deepslate', 'gold_ore', 'redstone_ore', 'emerald_ore', 'diamond_ore'];
      for (let depth = 1; depth <= 6; depth++) {
        const by = topY + depth * bSize;
        if (by > height + bSize) break;
        let bType = 'dirt';
        if (depth === 2) bType = 'stone';
        else if (depth >= 3) {
          bType = oreChoices[(c * 7 + depth * 3) % oreChoices.length];
        }
        addBlock(c * bSize, by, bSize, bType);
      }

      // Add Crafting Table on left hill
      if (c === Math.floor(cols * 0.22)) {
        addBlock(c * bSize, topY - bSize, bSize, 'crafting_table');
      }

      // Build 2.5D Voxel Trees on Left Hill & Right Hill
      if (c === Math.floor(cols * 0.12) || c === Math.floor(cols * 0.72) || c === Math.floor(cols * 0.88)) {
        const isCherry = c === Math.floor(cols * 0.72);
        const leafType = isCherry ? 'cherry_leaves' : 'oak_leaves';
        for (let th = 1; th <= 3; th++) {
          addBlock(c * bSize, topY - th * bSize, bSize, 'oak_log');
        }
        for (let lx = -2; lx <= 2; lx++) {
          for (let ly = 3; ly <= 5; ly++) {
            if (ly === 5 && Math.abs(lx) === 2) continue;
            addBlock((c + lx) * bSize, topY - ly * bSize, bSize, leafType);
          }
        }
      }
    }

    // Initialize Animated Steve, Alex, and Technoblade on the terrain
    characters = [
      {
        id: 'steve',
        name: 'Steve',
        headKey: 'head_steve',
        shirtColor: '#00aaaa',
        pantsColor: '#283593',
        heldItem: 'pickaxe',
        colFloat: cols * 0.18,
        minCol: cols * 0.06,
        maxCol: cols * 0.34,
        dir: 1,
        speed: 0.75,
        actionTimer: 0,
        bubbleText: 'Mining diamonds...',
        jumpVel: 0,
        jumpY: 0
      },
      {
        id: 'alex',
        name: 'Alex',
        headKey: 'head_alex',
        shirtColor: '#558b2f',
        pantsColor: '#4e342e',
        heldItem: 'fishing_rod',
        colFloat: cols * 0.36,
        minCol: cols * 0.25,
        maxCol: cols * 0.42,
        dir: 1,
        speed: 0.55,
        actionTimer: 0,
        bubbleText: 'Fishing by the river',
        jumpVel: 0,
        jumpY: 0
      },
      {
        id: 'techno',
        name: 'Technoblade [o7]',
        headKey: 'head_techno',
        hasCrown: true,
        hasCape: true,
        shirtColor: '#b71c1c',
        pantsColor: '#212121',
        heldItem: 'sword',
        colFloat: cols * 0.78,
        minCol: cols * 0.62,
        maxCol: cols * 0.92,
        dir: -1,
        speed: 0.85,
        actionTimer: 0,
        bubbleText: 'Technoblade Never Dies!',
        jumpVel: 0,
        jumpY: 0
      }
    ];

    characters.forEach((ch) => {
      ch.surfaceYByCol = surfaceYByCol;
      ch.bSize = bSize;
    });

    // Ambient falling cherry petals & shader fireflies
    cherryPetals = Array.from({ length: 45 }, () => ({
      x: Math.random() * width,
      y: Math.random() * height * 0.8,
      vx: (Math.random() - 0.3) * 0.45,
      vy: Math.random() * 0.35 + 0.15,
      size: Math.random() > 0.5 ? 4 : 3,
      isFirefly: Math.random() > 0.45
    }));
  }

  function addBlock(x, y, size, type) {
    const def = WORLD_BLOCK_DEFS[type] || WORLD_BLOCK_DEFS.stone;
    worldBlocks.push({
      x,
      y,
      size,
      type,
      def,
      hp: def.hp,
      maxHp: def.hp,
      broken: false,
      breakable: true,
      respawnScale: 1
    });
  }

  buildSceneGrid();
  window.addEventListener('resize', buildSceneGrid);

  // Hit-test background blocks or characters when user clicks/taps
  window.hitShader25DWorldAt = function (clientX, clientY) {
    // 1. Check if user clicked/tapped on Steve, Alex, or Technoblade!
    for (const ch of characters) {
      if (ch.screenX && ch.screenY) {
        if (Math.abs(clientX - ch.screenX) < 32 && Math.abs(clientY - (ch.screenY - 26)) < 42) {
          ch.jumpVel = -7.5;
          ch.actionTimer = 45;
          if (ch.id === 'techno') {
            const quotes = ['Technoblade Never Dies!', 'Blood for the Blood God!', '+500M Potatoes!', 'bruh [o7]'];
            ch.bubbleText = quotes[Math.floor(Math.random() * quotes.length)];
            playMcPigSound();
          } else if (ch.id === 'steve') {
            ch.bubbleText = 'Found Deepslate Diamonds!';
            playMcXpChime();
          } else {
            ch.bubbleText = 'Caught Enchanted Cod!';
            playMcXpChime();
          }
          return;
        }
      }
    }

    // 2. Check if user clicked/tapped on any breakable 2.5D block (search top-most first)
    for (let i = worldBlocks.length - 1; i >= 0; i--) {
      const b = worldBlocks[i];
      if (!b.breakable || b.broken) continue;
      if (clientX >= b.x && clientX <= b.x + b.size && clientY >= b.y - 10 && clientY <= b.y + b.size) {
        let dmg = (currentTool.baseDamage || 8) * (0.4 + 0.6 * lastChargeRatio);
        if (currentTool.affinity === b.def.cat || currentTool.affinity === 'all') {
          dmg *= 2.3;
        }
        b.hp -= dmg;
        b.hitShake = 6;

        const texUrl = getTextureUrls(b.def.tex ? `block/${b.def.tex}.png` : 'block/stone.png')[0];
        spawnBlockDebris(clientX, clientY, texUrl, 4);

        if (b.hp > 0) {
          playMinecraftOgg(b.def.sound, 0.32, 0.92 + Math.random() * 0.16);
        } else {
          // Block Broken!
          b.broken = true;
          totalMinedBlocks++;
          const minedHud = document.getElementById('hud-mined-count');
          if (minedHud) minedHud.textContent = totalMinedBlocks.toLocaleString();

          playMinecraftOgg(b.def.sound, 0.5, 0.82);
          spawnBlockDebris(clientX, clientY, texUrl, 9);

          if (b.def.dropTex) {
            playMcXpChime();
            spawnFloatingOreDrop(clientX, clientY, b.def.dropTex, b.def.dropLabel || `+1 ${b.def.name}`);
          }

          // Respawn block after 2.0 seconds (2000ms)!
          setTimeout(() => {
            b.broken = false;
            b.hp = b.maxHp;
            b.respawnScale = 0.25;
            playMinecraftOgg('random/pop.ogg', 0.2, 1.3);
          }, 2000);
        }
        break;
      }
    }
  };

  // Main 60 FPS 2.5D Shader World Render Loop
  function renderWorld(ts) {
    ctx.clearRect(0, 0, width, height);
    ctx.imageSmoothingEnabled = false;

    // 1. Complementary Shader Sky Gradient (Deep Twilight to Warm Golden Horizon)
    const skyGrad = ctx.createLinearGradient(0, 0, 0, height * 0.78);
    skyGrad.addColorStop(0, '#0a1124');
    skyGrad.addColorStop(0.45, '#1b2b52');
    skyGrad.addColorStop(0.75, '#4b365f');
    skyGrad.addColorStop(1, '#9c5243');
    ctx.fillStyle = skyGrad;
    ctx.fillRect(0, 0, width, height);

    // 2. Shader Sun & Volumetric God Rays
    const sunX = width * 0.76;
    const sunY = height * 0.2;
    const sunGlow = ctx.createRadialGradient(sunX, sunY, 10, sunX, sunY, 240);
    sunGlow.addColorStop(0, 'rgba(255, 235, 160, 0.55)');
    sunGlow.addColorStop(0.4, 'rgba(255, 165, 85, 0.18)');
    sunGlow.addColorStop(1, 'rgba(255, 120, 60, 0)');
    ctx.fillStyle = sunGlow;
    ctx.fillRect(0, 0, width, height);

    // Blocky Minecraft Shader Sun
    ctx.fillStyle = '#fff5c0';
    ctx.fillRect(Math.floor(sunX - 26), Math.floor(sunY - 26), 52, 52);
    ctx.fillStyle = '#ffffff';
    ctx.fillRect(Math.floor(sunX - 18), Math.floor(sunY - 18), 36, 36);

    // Diagonal Volumetric God Rays
    ctx.save();
    ctx.globalCompositeOperation = 'screen';
    for (let r = 0; r < 5; r++) {
      const rayAlpha = 0.035 + Math.sin(ts * 0.001 + r) * 0.015;
      ctx.fillStyle = `rgba(255, 210, 140, ${rayAlpha})`;
      ctx.beginPath();
      ctx.moveTo(sunX - 20 + r * 12, sunY);
      ctx.lineTo(sunX - 380 + r * 140, height);
      ctx.lineTo(sunX - 260 + r * 140, height);
      ctx.closePath();
      ctx.fill();
    }
    ctx.restore();

    // 3. Parallax Blocky Shader Clouds
    ctx.fillStyle = 'rgba(255, 230, 215, 0.22)';
    for (let c = 0; c < 6; c++) {
      const cx = ((c * 280 + ts * 0.015 * (c % 2 === 0 ? 1 : 0.6)) % (width + 240)) - 120;
      const cy = 58 + (c % 3) * 38;
      ctx.fillRect(Math.floor(cx), cy, 110, 18);
      ctx.fillRect(Math.floor(cx + 20), cy - 10, 70, 10);
    }

    // 4. Render 2.5D Voxel Blocks, Trees, Shader Water & Glowing Ores
    const depthBevel = 9;
    for (const b of worldBlocks) {
      if (b.broken) {
        // Draw dark cave cavity where block was mined
        ctx.fillStyle = 'rgba(5, 6, 10, 0.55)';
        ctx.fillRect(b.x, b.y, b.size, b.size);
        continue;
      }

      if (b.type === 'water') {
        // Animated Shader Water with Ripples & Sun Specular Reflection
        const waveY = Math.sin(ts * 0.004 + b.col * 0.8) * 3;
        const waterGrad = ctx.createLinearGradient(b.x, b.y + waveY, b.x, b.y + b.size);
        waterGrad.addColorStop(0, 'rgba(56, 189, 248, 0.72)');
        waterGrad.addColorStop(1, 'rgba(14, 116, 144, 0.88)');
        ctx.fillStyle = waterGrad;
        ctx.fillRect(b.x, b.y + waveY, b.size, b.size - waveY);

        // Shimmering water surface line
        ctx.fillStyle = 'rgba(255, 250, 210, 0.65)';
        ctx.fillRect(b.x + 4, b.y + waveY + 2, b.size - 8, 3);
        continue;
      }

      if (b.respawnScale < 1) {
        b.respawnScale = Math.min(1, b.respawnScale + 0.08);
      }
      const shakeX = b.hitShake ? (Math.random() - 0.5) * b.hitShake : 0;
      if (b.hitShake > 0) b.hitShake *= 0.75;

      // Gentle leaf sway in the wind for tree foliage
      const isLeaf = b.type === 'oak_leaves' || b.type === 'cherry_leaves';
      const windX = isLeaf ? Math.sin(ts * 0.0025 + b.x * 0.05 + b.y * 0.03) * 2.2 : 0;

      const drawX = Math.floor(b.x + shakeX + windX);
      const drawY = Math.floor(b.y);
      const sz = Math.floor(b.size * b.respawnScale);

      // Shader Ore Bloom
      if (b.def.glow) {
        ctx.fillStyle = b.def.glow;
        ctx.fillRect(drawX - 4, drawY - 4, sz + 8, sz + 8);
      }

      // 2.5D Isometric Top Face Bevel
      ctx.fillStyle = b.def.topColor || '#666';
      ctx.beginPath();
      ctx.moveTo(drawX, drawY);
      ctx.lineTo(drawX + depthBevel, drawY - depthBevel);
      ctx.lineTo(drawX + sz + depthBevel, drawY - depthBevel);
      ctx.lineTo(drawX + sz, drawY);
      ctx.closePath();
      ctx.fill();

      // 2.5D Front Block Texture
      const texImg = WORLD_TEX_IMAGES[b.def.tex];
      if (texImg && texImg.complete && texImg.naturalWidth > 0) {
        ctx.drawImage(texImg, drawX, drawY, sz, sz);
      } else {
        ctx.fillStyle = b.def.topColor || '#555';
        ctx.fillRect(drawX, drawY, sz, sz);
      }

      // Official Minecraft destroy_stage_0..9.png Crack Overlay if Damaged!
      if (b.hp < b.maxHp) {
        const dmgFrac = 1 - b.hp / b.maxHp;
        const stage = Math.min(9, Math.max(0, Math.floor(dmgFrac * 10)));
        const crackImg = WORLD_TEX_IMAGES[`crack_${stage}`];
        if (crackImg && crackImg.complete && crackImg.naturalWidth > 0) {
          ctx.drawImage(crackImg, drawX, drawY, sz, sz);
        }
      }

      // Subtle voxel shadow border
      ctx.strokeStyle = 'rgba(0, 0, 0, 0.35)';
      ctx.lineWidth = 1;
      ctx.strokeRect(drawX, drawY, sz, sz);
    }

    // 5. Update & Render Animated Steve, Alex, and Technoblade in the 2.5D Scene!
    for (const ch of characters) {
      ch.colFloat += ch.dir * ch.speed * 0.016;
      if (ch.colFloat >= ch.maxCol) {
        ch.colFloat = ch.maxCol;
        ch.dir = -1;
      } else if (ch.colFloat <= ch.minCol) {
        ch.colFloat = ch.minCol;
        ch.dir = 1;
      }

      // Jump physics
      if (ch.jumpVel !== 0 || ch.jumpY < 0) {
        ch.jumpY += ch.jumpVel;
        ch.jumpVel += 0.48;
        if (ch.jumpY >= 0) {
          ch.jumpY = 0;
          ch.jumpVel = 0;
        }
      } else if (Math.random() < 0.004) {
        // Spontaneous little hop
        ch.jumpVel = -5.2;
      }

      const colIdx = Math.max(0, Math.min(ch.surfaceYByCol.length - 1, Math.floor(ch.colFloat)));
      const groundY = ch.surfaceYByCol[colIdx] || height * 0.72;
      const cx = Math.floor(ch.colFloat * ch.bSize);
      const cy = Math.floor(groundY + ch.jumpY);
      ch.screenX = cx;
      ch.screenY = cy;

      const walkSwing = Math.sin(ts * 0.008 * ch.speed * 1.4) * 7;
      const armActionSwing = ch.actionTimer > 0
        ? Math.sin(ts * 0.035) * 18
        : Math.sin(ts * 0.007 + (ch.id === 'techno' ? 1 : 0)) * 9;
      if (ch.actionTimer > 0) ch.actionTimer--;

      // Character Shadow
      ctx.fillStyle = 'rgba(0, 0, 0, 0.42)';
      ctx.fillRect(cx - 12, groundY - 3, 24, 5);

      // Technoblade Royal Red Cape
      if (ch.hasCape) {
        ctx.fillStyle = '#991b1b';
        const capeX = ch.dir > 0 ? cx - 14 : cx + 6;
        ctx.fillRect(capeX, cy - 36, 8, 22);
      }

      // Animated Legs
      ctx.fillStyle = ch.pantsColor;
      ctx.fillRect(cx - 6 + walkSwing * 0.4, cy - 14, 6, 14);
      ctx.fillRect(cx + 1 - walkSwing * 0.4, cy - 14, 6, 14);

      // Torso
      ctx.fillStyle = ch.shirtColor;
      ctx.fillRect(cx - 7, cy - 32, 15, 18);

      // Animated Swinging Arm + Held Item (Pickaxe / Sword / Fishing Rod)
      ctx.save();
      ctx.translate(cx + (ch.dir > 0 ? 6 : -6), cy - 28);
      ctx.rotate(((armActionSwing + (ch.dir > 0 ? -25 : 25)) * Math.PI) / 180);
      ctx.fillStyle = ch.shirtColor;
      ctx.fillRect(-3, 0, 6, 14);

      const itemImg = WORLD_TEX_IMAGES[ch.heldItem];
      if (itemImg && itemImg.complete && itemImg.naturalWidth > 0) {
        ctx.drawImage(itemImg, -4, 6, 20, 20);
      }
      ctx.restore();

      // Official Minecraft Avatar Head (Steve, Alex, Technoblade)
      const headImg = WORLD_TEX_IMAGES[ch.headKey];
      if (headImg && headImg.complete && headImg.naturalWidth > 0) {
        ctx.drawImage(headImg, cx - 12, cy - 54, 24, 24);
      } else {
        ctx.fillStyle = ch.id === 'techno' ? '#f4a2a0' : '#d2a679';
        ctx.fillRect(cx - 12, cy - 54, 24, 24);
      }

      // Technoblade's Golden Crown on top of his head
      if (ch.hasCrown && WORLD_TEX_IMAGES.crown && WORLD_TEX_IMAGES.crown.complete) {
        ctx.drawImage(WORLD_TEX_IMAGES.crown, cx - 14, cy - 70, 28, 28);
      }

      // Floating Minecraft Nametag + Activity Status above each character
      ctx.font = '11px MinecraftOfficial, monospace';
      ctx.textAlign = 'center';
      const tagText = ch.actionTimer > 0 ? `${ch.name}: ${ch.bubbleText}` : ch.name;
      const textWidth = ctx.measureText(tagText).width + 10;
      ctx.fillStyle = 'rgba(0, 0, 0, 0.62)';
      ctx.fillRect(Math.floor(cx - textWidth / 2), cy - 84, Math.floor(textWidth), 16);
      ctx.fillStyle = ch.hasCrown ? '#FFAA00' : '#FFFFFF';
      ctx.fillText(tagText, cx, cy - 72);
    }

    // 6. Render Drifting Cherry Petals & Golden Shader Fireflies
    for (const p of cherryPetals) {
      p.x += p.vx;
      p.y += p.vy + (p.isFirefly ? Math.sin(ts * 0.003 + p.x) * 0.3 : 0);
      if (p.y > height * 0.85) {
        p.y = -10;
        p.x = Math.random() * width;
      }
      if (p.x < 0) p.x = width;
      if (p.x > width) p.x = 0;

      ctx.fillStyle = p.isFirefly
        ? `rgba(255, 230, 110, ${0.45 + Math.sin(ts * 0.006 + p.x) * 0.35})`
        : 'rgba(244, 165, 198, 0.68)';
      ctx.fillRect(Math.floor(p.x), Math.floor(p.y), p.size, p.size);
    }

    // Subtle dark vignette at very bottom so text cards stay crisp and readable
    const vigGrad = ctx.createLinearGradient(0, 0, 0, height);
    vigGrad.addColorStop(0, 'rgba(6, 9, 15, 0.28)');
    vigGrad.addColorStop(0.6, 'rgba(6, 9, 15, 0.32)');
    vigGrad.addColorStop(1, 'rgba(6, 9, 15, 0.62)');
    ctx.fillStyle = vigGrad;
    ctx.fillRect(0, 0, width, height);

    requestAnimationFrame(renderWorld);
  }

  requestAnimationFrame(renderWorld);
}

function spawnBlockDebris(x, y, texUrl, count = 6) {
  for (let i = 0; i < count; i++) {
    const p = document.createElement('div');
    p.className = 'mc-block-debris';
    p.style.left = `${x}px`;
    p.style.top = `${y}px`;
    p.style.backgroundImage = `url('${texUrl}')`;
    p.style.backgroundPosition = `${Math.floor(Math.random() * 20)}px ${Math.floor(Math.random() * 20)}px`;
    document.body.appendChild(p);

    const vx = (Math.random() - 0.5) * 95;
    const vy = -(Math.random() * 60 + 15);
    const start = performance.now();

    function step(ts) {
      const dt = (ts - start) / 1000;
      if (dt > 0.48) {
        if (p.parentNode) p.parentNode.removeChild(p);
        return;
      }
      const nx = x + vx * dt;
      const ny = y + vy * dt + 260 * dt * dt;
      p.style.left = `${nx}px`;
      p.style.top = `${ny}px`;
      p.style.opacity = String(1 - dt / 0.48);
      requestAnimationFrame(step);
    }
    requestAnimationFrame(step);
  }
}

function spawnFloatingOreDrop(x, y, dropTex, label) {
  const el = document.createElement('div');
  el.className = 'mc-floating-drop';
  el.style.left = `${x}px`;
  el.style.top = `${y - 10}px`;

  const img = document.createElement('img');
  img.className = 'mc-png mc-png-sm';
  attachMultiSourceImage(img, getTextureUrls(dropTex));

  const span = document.createElement('span');
  span.textContent = label;

  el.appendChild(img);
  el.appendChild(span);
  document.body.appendChild(el);

  setTimeout(() => {
    if (el.parentNode) el.parentNode.removeChild(el);
  }, 1150);
}

// ==============================================================================
// 5. 100% True Non-Hardcoded Stochastic disc_maker Composer
//    - Uses Bjorklund's Euclidean Algorithm E(k, 16, rotation) for rhythms
//    - Uses Markov Melodic Step-Walk weighted by Consonance Gravity for pitches
//    - Randomly allocates instrument voices from 16 official Minecraft sounds
// ==============================================================================

const DM_SCALES = {
  phrygian: [0, 1, 4, 5, 7, 8, 10],    // Nether Phrygian Dominant
  dorian: [0, 2, 3, 5, 7, 9, 10],      // Aria Dorian Groove
  pentatonic: [0, 3, 5, 7, 10],        // Minor Pentatonic Funk
  lydian: [0, 2, 4, 6, 7, 9, 11],      // C418 Lydian Dream
  hirajoshi: [0, 2, 3, 7, 8],          // Cherry Blossom Hirajoshi
  mixolydian: [0, 2, 4, 5, 7, 9, 10]   // Upbeat Jam
};

// Pool of Official Minecraft Melodic & Harmonic Instruments to Randomly Assign!
const MELODIC_VOICE_POOL = [
  { name: 'Note Pling', sound: 'note/pling.ogg', synth: 'square', tex: 'block/glowstone.png' },
  { name: 'Chiptune Bit', sound: 'note/bit.ogg', synth: 'square', tex: 'block/emerald_block.png' },
  { name: 'Bamboo Flute', sound: 'note/flute.ogg', synth: 'sine', tex: 'item/bamboo.png' },
  { name: 'Iron Xylophone', sound: 'note/iron_xylophone.ogg', synth: 'triangle', tex: 'item/iron_ingot.png' },
  { name: 'Bone Xylophone', sound: 'note/xylophone.ogg', synth: 'sine', tex: 'item/bone.png' },
  { name: 'Hay Banjo', sound: 'note/banjo.ogg', synth: 'sawtooth', tex: 'item/wheat.png' }
];

const HARMONIC_VOICE_POOL = [
  { name: 'Note Harp', sound: 'note/harp.ogg', synth: 'triangle', tex: 'block/note_block.png' },
  { name: 'Wool Guitar', sound: 'note/guitar.ogg', synth: 'sawtooth', tex: 'block/white_wool.png' },
  { name: 'Ice Chime', sound: 'note/chime.ogg', synth: 'sine', tex: 'item/amethyst_shard.png' },
  { name: 'Gold Bell', sound: 'note/bell.ogg', synth: 'sine', tex: 'item/bell.png' }
];

const BASS_VOICE_POOL = [
  { name: 'Wood Bass', sound: 'note/bass.ogg', synth: 'sawtooth', tex: 'block/oak_log.png' },
  { name: 'Didgeridoo', sound: 'note/didgeridoo.ogg', synth: 'sawtooth', tex: 'block/pumpkin_side.png' }
];

let dmTracks = [
  { id: 'lead', name: 'Note Pling', tex: 'block/glowstone.png', sound: 'note/pling.ogg', synthType: 'square', css: 'track-lead' },
  { id: 'arp', name: 'Note Harp', tex: 'block/note_block.png', sound: 'note/harp.ogg', synthType: 'triangle', css: 'track-arp' },
  { id: 'bell', name: 'Gold Bell', tex: 'item/bell.png', sound: 'note/bell.ogg', synthType: 'sine', css: 'track-bell' },
  { id: 'bass', name: 'Wood Bass', tex: 'block/oak_log.png', sound: 'note/bass.ogg', synthType: 'sawtooth', css: 'track-bass' },
  { id: 'drums', name: 'Euclidean Kit', tex: 'block/tnt_side.png', sound: 'note/bd.ogg', synthType: 'drum', css: 'track-drums' },
  { id: 'fx', name: 'MC Foley', tex: 'item/experience_bottle.png', sound: 'random/orb.ogg', synthType: 'fx', css: 'track-fx' }
];

let dmPlaying = false;
let dmAutoMutate = true;
let dmStep = 0;
let dmBar = 0;
let dmTimerId = null;
let dmCurrentChords = [0, 5, 2, 6];
let dmPattern = Array.from({ length: 6 }, () => Array(16).fill(null));

/**
 * Bjorklund's Euclidean Rhythm Generator E(pulses, steps, rotation)
 * Distributes `pulses` hits across `steps` slots as evenly as mathematically possible,
 * producing natural syncopated musical rhythms without hardcoding step indices.
 */
function generateEuclideanRhythm(pulses, steps, rotation = 0) {
  const k = Math.max(0, Math.min(steps, Math.round(pulses)));
  const pattern = new Array(steps).fill(false);
  if (k === 0) return pattern;
  for (let i = 0; i < steps; i++) {
    if (((i * k) % steps) < k) {
      pattern[(i + rotation) % steps] = true;
    }
  }
  return pattern;
}

/**
 * Markov + Consonance-Gravity Pitch Picker
 * Chooses the next scale degree randomly, weighted by:
 * 1. Consonance Gravity (chord triad tones 0, 2, 4 get boosted weight on strong beats)
 * 2. Stepwise Proximity to the previous note (prevents jarring random leaps)
 */
function pickStochasticScaleDegree(prevDegree, isStrongBeat, consonanceWeight) {
  const candidates = [0, 1, 2, 3, 4, 5, 6];
  const weights = candidates.map((deg) => {
    const isChordTone = deg === 0 || deg === 2 || deg === 4;
    const dist = Math.abs(deg - prevDegree);
    const proximityScore = 1 / (1 + dist * 0.75);
    const harmonicScore = isChordTone
      ? (isStrongBeat ? consonanceWeight * 4.5 : consonanceWeight * 2.2)
      : (1 - consonanceWeight * 0.7) * 1.8;
    return proximityScore * harmonicScore;
  });

  const totalWeight = weights.reduce((a, b) => a + b, 0);
  let r = Math.random() * totalWeight;
  for (let i = 0; i < candidates.length; i++) {
    r -= weights[i];
    if (r <= 0) return candidates[i];
  }
  return 0;
}

function initDiscMakerStudio() {
  // Continuously cycle all .disc-maker-cycling-icon elements through all 19 Minecraft Music Discs!
  setInterval(() => {
    const randomDiscTex = ALL_MINECRAFT_DISCS[Math.floor(Math.random() * ALL_MINECRAFT_DISCS.length)];
    document.querySelectorAll('.disc-maker-cycling-icon').forEach((img) => {
      attachMultiSourceImage(img, getTextureUrls(randomDiscTex));
    });
    const discNameClean = randomDiscTex.replace('item/', '').replace('.png', '');
    const scaleSelect = document.getElementById('dm-scale-select');
    const scaleLabel = scaleSelect ? scaleSelect.options[scaleSelect.selectedIndex]?.text : 'Phrygian';
    const readout = document.getElementById('dm-active-disc-label');
    if (readout) {
      readout.textContent = `Disc: ${discNameClean} • Scale: ${scaleLabel}`;
    }
  }, 1500);

  preloadNoteBlockSamples();
  buildSequencerDOM();
  generateStochasticMusicalPattern(true);

  const playBtn = document.getElementById('dm-play-btn');
  const turntableBtn = document.getElementById('dm-turntable-btn');
  const randBtn = document.getElementById('dm-randomize-btn');
  const mutateBtn = document.getElementById('dm-mutate-btn');
  const bpmSlider = document.getElementById('dm-bpm-slider');
  const swingSlider = document.getElementById('dm-swing-slider');
  const densitySlider = document.getElementById('dm-density-slider');
  const consonanceSlider = document.getElementById('dm-consonance-slider');
  const scaleSelect = document.getElementById('dm-scale-select');
  const rootSelect = document.getElementById('dm-root-select');

  const toggleStudioPlay = () => {
    if (dmPlaying) stopDiscMaker();
    else startDiscMaker();
  };

  if (playBtn) playBtn.addEventListener('click', toggleStudioPlay);
  if (turntableBtn) turntableBtn.addEventListener('click', toggleStudioPlay);

  if (randBtn) {
    randBtn.addEventListener('click', () => {
      randomizeAllStudioParameters();
      generateStochasticMusicalPattern(true);
      if (!dmPlaying) startDiscMaker();
      showAdvancementToast('disc_maker Randomized!', `Lead: ${dmTracks[0].name} • Arp: ${dmTracks[1].name}`, 'item/music_disc_creator.png');
    });
  }

  if (mutateBtn) {
    mutateBtn.addEventListener('click', () => {
      dmAutoMutate = !dmAutoMutate;
      const lbl = document.getElementById('dm-mutate-label');
      if (lbl) lbl.textContent = `Live Mutate: ${dmAutoMutate ? 'ON' : 'OFF'}`;
      playMcClick(1.1);
    });
  }

  if (bpmSlider) {
    bpmSlider.addEventListener('input', (e) => {
      const el = document.getElementById('dm-bpm-val');
      if (el) el.textContent = `${e.target.value} BPM`;
    });
  }
  if (swingSlider) {
    swingSlider.addEventListener('input', (e) => {
      const el = document.getElementById('dm-swing-val');
      if (el) el.textContent = `${e.target.value}%`;
    });
  }
  if (consonanceSlider) {
    consonanceSlider.addEventListener('input', (e) => {
      const el = document.getElementById('dm-consonance-val');
      if (el) el.textContent = `${e.target.value}%`;
      generateStochasticMusicalPattern(false);
    });
  }
  if (densitySlider) {
    densitySlider.addEventListener('input', (e) => {
      const el = document.getElementById('dm-density-val');
      if (el) el.textContent = `${e.target.value}%`;
      generateStochasticMusicalPattern(false);
    });
  }
  if (scaleSelect) scaleSelect.addEventListener('change', () => generateStochasticMusicalPattern(false));
  if (rootSelect) rootSelect.addEventListener('change', () => generateStochasticMusicalPattern(false));

  window.startDiscMakerFromConsole = () => {
    randomizeAllStudioParameters();
    generateStochasticMusicalPattern(true);
    startDiscMaker();
  };
}

function buildSequencerDOM() {
  const gridContainer = document.getElementById('dm-sequencer-grid');
  if (!gridContainer) return;
  gridContainer.innerHTML = '';

  dmTracks.forEach((track, tIdx) => {
    const row = document.createElement('div');
    row.className = 'sequencer-row';

    const label = document.createElement('div');
    label.className = 'seq-track-label';
    const icon = document.createElement('img');
    icon.className = 'mc-png mc-png-sm';
    icon.id = `seq-track-icon-${tIdx}`;
    attachMultiSourceImage(icon, getTextureUrls(track.tex));
    const text = document.createElement('span');
    text.className = 'mc-f';
    text.id = `seq-track-name-${tIdx}`;
    text.textContent = track.name;
    label.appendChild(icon);
    label.appendChild(text);
    row.appendChild(label);

    for (let s = 0; s < 16; s++) {
      const cell = document.createElement('div');
      cell.className = `seq-cell ${s % 4 === 0 ? 'beat-marker' : ''} ${track.css}`;
      cell.dataset.track = String(tIdx);
      cell.dataset.step = String(s);

      cell.addEventListener('click', () => {
        toggleSequencerCell(tIdx, s);
        playMcClick(1.2);
      });

      row.appendChild(cell);
    }
    gridContainer.appendChild(row);
  });
}

function randomizeAllStudioParameters() {
  const scaleSelect = document.getElementById('dm-scale-select');
  const rootSelect = document.getElementById('dm-root-select');
  const bpmSlider = document.getElementById('dm-bpm-slider');
  const swingSlider = document.getElementById('dm-swing-slider');
  const densitySlider = document.getElementById('dm-density-slider');
  const consonanceSlider = document.getElementById('dm-consonance-slider');

  if (scaleSelect) scaleSelect.selectedIndex = Math.floor(Math.random() * scaleSelect.options.length);
  if (rootSelect) rootSelect.selectedIndex = Math.floor(Math.random() * rootSelect.options.length);

  const randomBpm = Math.floor(94 + Math.random() * 44);
  const randomSwing = Math.floor(8 + Math.random() * 26);
  const randomDensity = Math.floor(48 + Math.random() * 36);
  const randomConsonance = Math.floor(72 + Math.random() * 24);

  if (bpmSlider) {
    bpmSlider.value = String(randomBpm);
    const el = document.getElementById('dm-bpm-val');
    if (el) el.textContent = `${randomBpm} BPM`;
  }
  if (swingSlider) {
    swingSlider.value = String(randomSwing);
    const el = document.getElementById('dm-swing-val');
    if (el) el.textContent = `${randomSwing}%`;
  }
  if (densitySlider) {
    densitySlider.value = String(randomDensity);
    const el = document.getElementById('dm-density-val');
    if (el) el.textContent = `${randomDensity}%`;
  }
  if (consonanceSlider) {
    consonanceSlider.value = String(randomConsonance);
    const el = document.getElementById('dm-consonance-val');
    if (el) el.textContent = `${randomConsonance}%`;
  }
}

/**
 * 100% Stochastic Pattern Generator (Zero hardcoded step patterns!)
 */
function generateStochasticMusicalPattern(randomizeVoices = false) {
  if (randomizeVoices) {
    const leadVoice = MELODIC_VOICE_POOL[Math.floor(Math.random() * MELODIC_VOICE_POOL.length)];
    const arpVoice = HARMONIC_VOICE_POOL[Math.floor(Math.random() * HARMONIC_VOICE_POOL.length)];
    const bellVoice = HARMONIC_VOICE_POOL[Math.floor(Math.random() * HARMONIC_VOICE_POOL.length)];
    const bassVoice = BASS_VOICE_POOL[Math.floor(Math.random() * BASS_VOICE_POOL.length)];

    dmTracks[0] = { ...dmTracks[0], name: leadVoice.name, sound: leadVoice.sound, synthType: leadVoice.synth, tex: leadVoice.tex };
    dmTracks[1] = { ...dmTracks[1], name: arpVoice.name, sound: arpVoice.sound, synthType: arpVoice.synth, tex: arpVoice.tex };
    dmTracks[2] = { ...dmTracks[2], name: bellVoice.name, sound: bellVoice.sound, synthType: bellVoice.synth, tex: bellVoice.tex };
    dmTracks[3] = { ...dmTracks[3], name: bassVoice.name, sound: bassVoice.sound, synthType: bassVoice.synth, tex: bassVoice.tex };

    for (let i = 0; i < 4; i++) {
      const iconEl = document.getElementById(`seq-track-icon-${i}`);
      const nameEl = document.getElementById(`seq-track-name-${i}`);
      if (iconEl) attachMultiSourceImage(iconEl, getTextureUrls(dmTracks[i].tex));
      if (nameEl) nameEl.textContent = dmTracks[i].name;
    }
  }

  // Generate a random 4-bar chord progression starting on root (0) and resolving cleanly
  const chordPool = [1, 2, 3, 4, 5, 6];
  dmCurrentChords = [
    0,
    chordPool[Math.floor(Math.random() * chordPool.length)],
    chordPool[Math.floor(Math.random() * chordPool.length)],
    chordPool[Math.floor(Math.random() * chordPool.length)]
  ];

  const density = (parseInt(document.getElementById('dm-density-slider')?.value || '64', 10) || 64) / 100;
  const consonance = (parseInt(document.getElementById('dm-consonance-slider')?.value || '82', 10) || 82) / 100;

  // Generate Euclidean rhythms E(k, 16, rot) for each track with random pulse counts & rotations!
  const leadPulses = Math.max(4, Math.min(12, Math.round(density * 11 + (Math.random() * 3 - 1.5))));
  const arpPulses = Math.max(5, Math.min(13, Math.round(density * 12 + (Math.random() * 3 - 1.5))));
  const bellPulses = Math.max(2, Math.min(6, Math.round(density * 5 + Math.random() * 2)));
  const bassPulses = Math.max(4, Math.min(9, Math.round(density * 8 + Math.random() * 2)));
  const kickPulses = Math.max(3, Math.min(6, Math.round(3 + density * 3)));
  const hatPulses = Math.max(5, Math.min(14, Math.round(density * 13)));

  const leadEuc = generateEuclideanRhythm(leadPulses, 16, Math.floor(Math.random() * 4));
  const arpEuc = generateEuclideanRhythm(arpPulses, 16, Math.floor(Math.random() * 4));
  const bellEuc = generateEuclideanRhythm(bellPulses, 16, Math.floor(Math.random() * 8));
  const bassEuc = generateEuclideanRhythm(bassPulses, 16, 0);
  const kickEuc = generateEuclideanRhythm(kickPulses, 16, 0);
  const hatEuc = generateEuclideanRhythm(hatPulses, 16, Math.floor(Math.random() * 2));

  const newPat = Array.from({ length: 6 }, () => Array(16).fill(null));
  let prevLeadDeg = 0;
  let prevArpDeg = 2;

  for (let s = 0; s < 16; s++) {
    const isStrongBeat = s % 4 === 0;
    const isBackbeat = s % 8 === 4;

    // Track 0: Stochastic Markov Lead Melody
    if (leadEuc[s]) {
      const deg = pickStochasticScaleDegree(prevLeadDeg, isStrongBeat, consonance);
      prevLeadDeg = deg;
      const oct = Math.random() < 0.22 ? 1 : 0;
      newPat[0][s] = { degree: deg, octave: oct, label: String(deg + 1) };
    }

    // Track 1: Stochastic Harmonic Arpeggio
    if (arpEuc[s]) {
      const deg = pickStochasticScaleDegree(prevArpDeg, true, Math.min(0.98, consonance + 0.1));
      prevArpDeg = deg;
      newPat[1][s] = { degree: deg, octave: 0, label: String(deg + 1) };
    }

    // Track 2: High Bell Sparkles
    if (bellEuc[s]) {
      const deg = pickStochasticScaleDegree(4, true, 0.92);
      newPat[2][s] = { degree: deg, octave: 1, label: '★' };
    }

    // Track 3: Euclidean Bassline (Favors Root 0 and Fifth 4)
    if (bassEuc[s]) {
      const deg = isStrongBeat || Math.random() < consonance ? 0 : 4;
      newPat[3][s] = { degree: deg, octave: -1, label: deg === 0 ? 'R' : '5' };
    }

    // Track 4: Euclidean Drum Groove + Backbeat Anchor
    if (isBackbeat) {
      newPat[4][s] = { drumType: 'snare', label: 'S' };
    } else if (kickEuc[s]) {
      newPat[4][s] = { drumType: 'kick', label: 'K' };
    } else if (hatEuc[s]) {
      newPat[4][s] = { drumType: 'hat', label: 'H' };
    }

    // Track 5: Stochastic Foley Accent (1 to 2 hits per bar on phrase edges)
    if (Math.random() < 0.09 && isStrongBeat) {
      const fxType = Math.random() < 0.6 ? 'orb' : 'pig';
      newPat[5][s] = { fxType, label: fxType === 'orb' ? 'XP' : 'PIG' };
    }
  }

  dmPattern = newPat;
  renderSequencerGridUI();
}

function toggleSequencerCell(tIdx, sIdx) {
  if (dmPattern[tIdx][sIdx]) {
    dmPattern[tIdx][sIdx] = null;
  } else {
    if (tIdx === 4) {
      dmPattern[tIdx][sIdx] = { drumType: sIdx % 4 === 0 ? 'kick' : 'snare', label: sIdx % 4 === 0 ? 'K' : 'S' };
    } else if (tIdx === 5) {
      dmPattern[tIdx][sIdx] = { fxType: 'orb', label: 'XP' };
    } else {
      const deg = pickStochasticScaleDegree(0, sIdx % 4 === 0, 0.85);
      dmPattern[tIdx][sIdx] = { degree: deg, octave: tIdx === 3 ? -1 : 0, label: String(deg + 1) };
    }
  }
  renderSequencerGridUI();
}

function renderSequencerGridUI() {
  document.querySelectorAll('#dm-sequencer-grid .seq-cell').forEach((cell) => {
    const t = parseInt(cell.dataset.track, 10);
    const s = parseInt(cell.dataset.step, 10);
    const data = dmPattern[t]?.[s];
    cell.classList.toggle('active-step', Boolean(data));
    cell.textContent = data ? (data.label || '●') : '';
  });
}

function startDiscMaker() {
  if (bgMusicAudio && bgMusicPlaying) {
    bgMusicAudio.pause();
    bgMusicPlaying = false;
    updateJukeboxUI();
  }

  ensureAudioCtx();
  dmPlaying = true;
  dmStep = 0;
  dmBar = 0;

  const playLabel = document.getElementById('dm-play-label');
  const badge = document.getElementById('dm-status-badge');
  const mainDisc = document.getElementById('dm-main-disc-icon');
  if (playLabel) playLabel.textContent = 'STOP COMPOSER';
  if (badge) badge.textContent = 'COMPOSING LIVE';
  if (mainDisc) mainDisc.classList.add('disc-spinning');

  scheduleNextDiscMakerStep();
}

function stopDiscMaker() {
  dmPlaying = false;
  if (dmTimerId) {
    clearTimeout(dmTimerId);
    dmTimerId = null;
  }
  const playLabel = document.getElementById('dm-play-label');
  const badge = document.getElementById('dm-status-badge');
  const mainDisc = document.getElementById('dm-main-disc-icon');
  if (playLabel) playLabel.textContent = 'START COMPOSER';
  if (badge) badge.textContent = 'PAUSED';
  if (mainDisc) mainDisc.classList.remove('disc-spinning');

  document.querySelectorAll('#dm-sequencer-grid .seq-cell.playhead-now').forEach((c) => {
    c.classList.remove('playhead-now');
  });
}

function scheduleNextDiscMakerStep() {
  if (!dmPlaying) return;

  const bpm = parseInt(document.getElementById('dm-bpm-slider')?.value || '114', 10) || 114;
  const swingPct = (parseInt(document.getElementById('dm-swing-slider')?.value || '20', 10) || 20) / 100;

  const sixteenthMs = (60000 / bpm) / 4;
  const currentStepDuration = (dmStep % 2 === 0)
    ? sixteenthMs * (1 + swingPct * 0.5)
    : sixteenthMs * (1 - swingPct * 0.5);

  playDiscMakerStep(dmStep, dmBar);

  dmStep++;
  if (dmStep >= 16) {
    dmStep = 0;
    dmBar = (dmBar + 1) % 4;
    if (dmBar === 0 && dmAutoMutate) {
      generateStochasticMusicalPattern(Math.random() < 0.35);
    }
  }

  dmTimerId = setTimeout(scheduleNextDiscMakerStep, currentStepDuration);
}

function playDiscMakerStep(stepIdx, barIdx) {
  document.querySelectorAll('#dm-sequencer-grid .seq-cell').forEach((cell) => {
    const s = parseInt(cell.dataset.step, 10);
    cell.classList.toggle('playhead-now', s === stepIdx);
  });

  const scaleKey = document.getElementById('dm-scale-select')?.value || 'phrygian';
  const scale = DM_SCALES[scaleKey] || DM_SCALES.phrygian;
  const rootOffset = parseInt(document.getElementById('dm-root-select')?.value || '0', 10) || 0;
  const activeChordDegree = dmCurrentChords[barIdx % dmCurrentChords.length] || 0;

  const readout = document.getElementById('dm-chord-readout');
  if (readout) {
    readout.textContent = `Bar ${barIdx + 1}/4 • Scale Degree Root: +${activeChordDegree} • Step ${stepIdx + 1}/16`;
  }

  for (let t = 0; t < dmTracks.length; t++) {
    const cellData = dmPattern[t][stepIdx];
    if (!cellData) continue;
    const track = dmTracks[t];

    if (track.id === 'drums') {
      playComposerDrum(cellData.drumType);
    } else if (track.id === 'fx') {
      playComposerFoley(cellData.fxType, rootOffset);
    } else {
      const totalDegree = activeChordDegree + (cellData.degree || 0);
      const scaleLen = scale.length;
      const octaveFromDegree = Math.floor(totalDegree / scaleLen);
      const scaleSemitone = scale[((totalDegree % scaleLen) + scaleLen) % scaleLen];
      const semitoneOffset = rootOffset + scaleSemitone + (octaveFromDegree + (cellData.octave || 0)) * 12;

      playQuantizedMinecraftNote(track, semitoneOffset);
    }
  }
}

function preloadNoteBlockSamples() {
  const paths = [
    'note/pling.ogg',
    'note/harp.ogg',
    'note/bell.ogg',
    'note/bass.ogg',
    'note/bit.ogg',
    'note/flute.ogg',
    'note/guitar.ogg',
    'note/chime.ogg',
    'note/xylophone.ogg',
    'note/iron_xylophone.ogg',
    'note/banjo.ogg',
    'note/didgeridoo.ogg',
    'note/bd.ogg',
    'note/snare.ogg',
    'note/hat.ogg',
    'random/orb.ogg',
    'mob/pig/say1.ogg'
  ];

  paths.forEach((p) => {
    const urls = getSoundUrls(p);
    (async () => {
      for (const u of urls) {
        try {
          const res = await fetch(u);
          if (!res.ok) continue;
          const arr = await res.arrayBuffer();
          const ctx = ensureAudioCtx();
          if (ctx) {
            const buf = await ctx.decodeAudioData(arr);
            DECODED_BUFFER_CACHE[p] = buf;
            break;
          }
        } catch (_) {}
      }
    })();
  });
}

function playQuantizedMinecraftNote(track, semitoneOffset) {
  if (!sfxEnabled) return;
  const ctx = ensureAudioCtx();
  if (!ctx) return;

  const rate = Math.pow(2, semitoneOffset / 12);
  const clampedRate = Math.max(0.35, Math.min(2.8, rate));

  const buf = DECODED_BUFFER_CACHE[track.sound];
  if (buf) {
    const src = ctx.createBufferSource();
    const gain = ctx.createGain();
    src.buffer = buf;
    src.playbackRate.setValueAtTime(clampedRate, ctx.currentTime);
    gain.gain.setValueAtTime(track.id === 'bass' ? 0.36 : 0.24, ctx.currentTime);
    src.connect(gain);
    gain.connect(ctx.destination);
    src.start(ctx.currentTime);
    return;
  }

  const now = ctx.currentTime;
  const osc = ctx.createOscillator();
  const gain = ctx.createGain();
  const filter = ctx.createBiquadFilter();

  const baseHz = track.id === 'bass' ? 92.5 : (track.id === 'bell' ? 739.99 : 369.99);
  const freq = baseHz * clampedRate;

  osc.type = track.synthType || 'triangle';
  osc.frequency.setValueAtTime(freq, now);

  filter.type = 'lowpass';
  filter.frequency.setValueAtTime(track.id === 'bass' ? 420 : 2400, now);

  const peakGain = track.id === 'bass' ? 0.14 : 0.08;
  const decay = track.id === 'bell' ? 0.38 : 0.22;

  gain.gain.setValueAtTime(peakGain, now);
  gain.gain.exponentialRampToValueAtTime(0.001, now + decay);

  osc.connect(filter);
  filter.connect(gain);
  gain.connect(ctx.destination);

  osc.start(now);
  osc.stop(now + decay + 0.02);
}

function playComposerDrum(drumType) {
  if (!sfxEnabled) return;
  const ctx = ensureAudioCtx();
  if (!ctx) return;
  const now = ctx.currentTime;

  if (drumType === 'kick') {
    const buf = DECODED_BUFFER_CACHE['note/bd.ogg'];
    if (buf) {
      const src = ctx.createBufferSource();
      const g = ctx.createGain();
      src.buffer = buf;
      g.gain.setValueAtTime(0.5, now);
      src.connect(g);
      g.connect(ctx.destination);
      src.start(now);
    }
    const osc = ctx.createOscillator();
    const gain = ctx.createGain();
    osc.type = 'sine';
    osc.frequency.setValueAtTime(135, now);
    osc.frequency.exponentialRampToValueAtTime(38, now + 0.12);
    gain.gain.setValueAtTime(0.22, now);
    gain.gain.exponentialRampToValueAtTime(0.001, now + 0.14);
    osc.connect(gain);
    gain.connect(ctx.destination);
    osc.start(now);
    osc.stop(now + 0.15);
  } else if (drumType === 'snare') {
    const buf = DECODED_BUFFER_CACHE['note/snare.ogg'];
    if (buf) {
      const src = ctx.createBufferSource();
      const g = ctx.createGain();
      src.buffer = buf;
      g.gain.setValueAtTime(0.42, now);
      src.connect(g);
      g.connect(ctx.destination);
      src.start(now);
      return;
    }
    const osc = ctx.createOscillator();
    const gain = ctx.createGain();
    osc.type = 'triangle';
    osc.frequency.setValueAtTime(220, now);
    osc.frequency.exponentialRampToValueAtTime(95, now + 0.09);
    gain.gain.setValueAtTime(0.14, now);
    gain.gain.exponentialRampToValueAtTime(0.001, now + 0.11);
    osc.connect(gain);
    gain.connect(ctx.destination);
    osc.start(now);
    osc.stop(now + 0.12);
  } else {
    const buf = DECODED_BUFFER_CACHE['note/hat.ogg'];
    if (buf) {
      const src = ctx.createBufferSource();
      const g = ctx.createGain();
      src.buffer = buf;
      g.gain.setValueAtTime(0.26, now);
      src.connect(g);
      g.connect(ctx.destination);
      src.start(now);
      return;
    }
    const osc = ctx.createOscillator();
    const gain = ctx.createGain();
    osc.type = 'square';
    osc.frequency.setValueAtTime(3200, now);
    gain.gain.setValueAtTime(0.025, now);
    gain.gain.exponentialRampToValueAtTime(0.001, now + 0.035);
    osc.connect(gain);
    gain.connect(ctx.destination);
    osc.start(now);
    osc.stop(now + 0.04);
  }
}

function playComposerFoley(fxType, rootOffset) {
  const rate = Math.pow(2, rootOffset / 12);
  if (fxType === 'pig') {
    playMinecraftOgg('mob/pig/say1.ogg', 0.25, Math.max(0.8, Math.min(1.25, rate)));
  } else {
    playMinecraftOgg('random/orb.ogg', 0.22, Math.max(0.8, Math.min(1.3, rate)));
  }
}

// ==============================================================================
// 6. Official Minecraft Audio Engine & Pigstep Jukebox
// ==============================================================================

function getSoundUrls(soundRelativePath) {
  const clean = soundRelativePath.replace(/^\/+/, '');
  const fullKey = `minecraft/sounds/${clean}`;
  const urls = [];

  for (const br of [activeMcAssetBranch, ...MC_ASSET_BRANCH_CANDIDATES]) {
    urls.push(`https://raw.githubusercontent.com/InventivetalentDev/minecraft-assets/${br}/assets/minecraft/sounds/${clean}`);
    urls.push(`https://cdn.jsdelivr.net/gh/InventivetalentDev/minecraft-assets@${br}/assets/minecraft/sounds/${clean}`);
    urls.push(`https://assets.mcasset.cloud/${br}/assets/minecraft/sounds/${clean}`);
  }

  const sha1 = SOUND_SHA1_CACHE[fullKey];
  if (sha1) {
    urls.push(`https://resources.download.minecraft.net/${sha1.slice(0, 2)}/${sha1}`);
  }

  return urls;
}

function playMinecraftOgg(soundRelativePath, volume = 0.45, playbackRate = 1.0, fallbackFn = null) {
  if (!sfxEnabled) return;
  const candidateUrls = getSoundUrls(soundRelativePath);
  let idx = 0;
  const audio = new Audio();
  audio.crossOrigin = 'anonymous';
  audio.volume = Math.max(0, Math.min(1, volume));
  audio.playbackRate = playbackRate;

  const tryNextUrl = () => {
    if (idx < candidateUrls.length) {
      audio.src = candidateUrls[idx++];
      audio.play().catch(() => {
        tryNextUrl();
      });
    } else if (typeof fallbackFn === 'function') {
      fallbackFn();
    }
  };

  audio.onerror = tryNextUrl;
  tryNextUrl();
}

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

function synthClickFallback(pitch = 1.0) {
  const ctx = ensureAudioCtx();
  if (!ctx) return;
  const now = ctx.currentTime;
  const osc = ctx.createOscillator();
  const gain = ctx.createGain();
  osc.type = 'triangle';
  osc.frequency.setValueAtTime(190 * pitch, now);
  osc.frequency.exponentialRampToValueAtTime(65 * pitch, now + 0.045);
  gain.gain.setValueAtTime(0.08, now);
  gain.gain.exponentialRampToValueAtTime(0.001, now + 0.045);
  osc.connect(gain);
  gain.connect(ctx.destination);
  osc.start(now);
  osc.stop(now + 0.05);
}

function synthXpFallback() {
  const ctx = ensureAudioCtx();
  if (!ctx) return;
  const now = ctx.currentTime;
  [587.33, 783.99, 987.77, 1174.66].forEach((f, idx) => {
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

function synthPigFallback() {
  const ctx = ensureAudioCtx();
  if (!ctx) return;
  const now = ctx.currentTime;
  const osc = ctx.createOscillator();
  const gain = ctx.createGain();
  osc.type = 'sawtooth';
  osc.frequency.setValueAtTime(145, now);
  osc.frequency.linearRampToValueAtTime(210, now + 0.09);
  osc.frequency.exponentialRampToValueAtTime(95, now + 0.24);
  gain.gain.setValueAtTime(0.08, now);
  gain.gain.exponentialRampToValueAtTime(0.001, now + 0.25);
  osc.connect(gain);
  gain.connect(ctx.destination);
  osc.start(now);
  osc.stop(now + 0.26);
}

function playMcClick(pitch = 1.0) {
  playMinecraftOgg('random/click.ogg', 0.35, pitch, () => synthClickFallback(pitch));
}

function playMcXpChime() {
  playMinecraftOgg('random/orb.ogg', 0.4, 1.0 + (Math.random() * 0.2 - 0.1), () => synthXpFallback());
}

function playMcLevelUp() {
  playMinecraftOgg('random/levelup.ogg', 0.45, 1.0, () => synthXpFallback());
}

function playMcPigSound() {
  const variant = Math.floor(Math.random() * 3) + 1;
  playMinecraftOgg(`mob/pig/say${variant}.ogg`, 0.55, 0.95 + Math.random() * 0.12, () => synthPigFallback());
}

function playMcTotemSound() {
  playMinecraftOgg('item/totem/use_totem.ogg', 0.5, 1.0, () => synthXpFallback());
}

function playMcBellSound() {
  playMinecraftOgg('note/bell.ogg', 0.55, 1.0, () => synthXpFallback());
}

function playMcEatSound() {
  const variant = Math.floor(Math.random() * 3) + 1;
  playMinecraftOgg(`random/eat${variant}.ogg`, 0.45, 1.0, () => synthClickFallback(1.2));
}

function playMcFireworkSound() {
  playMinecraftOgg('fireworks/launch1.ogg', 0.45, 1.0, () => synthXpFallback());
  setTimeout(() => {
    playMinecraftOgg('fireworks/twinkle1.ogg', 0.4, 1.0);
  }, 550);
}

function initSoundEngine() {
  const toggleBtn = document.getElementById('sfx-toggle-btn');
  const toggleLabel = document.getElementById('sfx-toggle-label');
  if (toggleBtn) {
    toggleBtn.addEventListener('click', () => {
      sfxEnabled = !sfxEnabled;
      toggleBtn.classList.toggle('muted', !sfxEnabled);
      if (toggleLabel) toggleLabel.textContent = sfxEnabled ? 'SFX: ON' : 'SFX: OFF';
      if (sfxEnabled) playMcClick(1.1);
    });
  }

  document.querySelectorAll('.sfx-trigger').forEach((el) => {
    el.addEventListener('click', () => playMcClick(1.0));
  });

  document.querySelectorAll('.sfx-xp').forEach((el) => {
    el.addEventListener('click', () => playMcXpChime());
  });
}

function initPigstepJukebox() {
  bgMusicAudio = new Audio();
  bgMusicAudio.crossOrigin = 'anonymous';
  bgMusicAudio.loop = true;
  bgMusicAudio.volume = bgMusicVolume;

  const playBtn = document.getElementById('jukebox-play-btn');
  const toggleBtn = document.getElementById('jukebox-toggle-btn');
  const nextBtn = document.getElementById('jukebox-next-btn');
  const volSlider = document.getElementById('jukebox-volume');

  loadDiscIntoPlayer(0, true);

  const unlockAutoplay = () => {
    if (!bgMusicPlaying && !userPausedMusic && !dmPlaying) {
      bgMusicAudio.volume = bgMusicVolume;
      bgMusicAudio.play().then(() => {
        bgMusicPlaying = true;
        updateJukeboxUI();
      }).catch(() => {});
    }
  };

  window.addEventListener('pointerdown', unlockAutoplay, { once: true });
  window.addEventListener('keydown', unlockAutoplay, { once: true });

  const toggleMusic = () => {
    if (!bgMusicAudio) return;
    if (dmPlaying) stopDiscMaker();
    if (bgMusicPlaying) {
      bgMusicAudio.pause();
      bgMusicPlaying = false;
      userPausedMusic = true;
    } else {
      userPausedMusic = false;
      bgMusicAudio.volume = bgMusicVolume;
      bgMusicAudio.play().then(() => {
        bgMusicPlaying = true;
        updateJukeboxUI();
      }).catch(() => {
        loadDiscIntoPlayer(currentDiscIndex, true);
      });
    }
    updateJukeboxUI();
  };

  if (playBtn) playBtn.addEventListener('click', toggleMusic);
  if (toggleBtn) toggleBtn.addEventListener('click', toggleMusic);

  if (nextBtn) {
    nextBtn.addEventListener('click', () => {
      if (dmPlaying) stopDiscMaker();
      currentDiscIndex = (currentDiscIndex + 1) % JUKEBOX_DISCS.length;
      userPausedMusic = false;
      loadDiscIntoPlayer(currentDiscIndex, true);
      const disc = JUKEBOX_DISCS[currentDiscIndex];
      showAdvancementToast('Disc Changed', disc.title, disc.tex);
      playMcClick(1.15);
    });
  }

  if (volSlider) {
    volSlider.addEventListener('input', (e) => {
      const pct = parseInt(e.target.value, 10) || 0;
      bgMusicVolume = pct / 100;
      if (bgMusicAudio) bgMusicAudio.volume = bgMusicVolume;
      updateJukeboxUI();
    });
  }
}

function loadDiscIntoPlayer(discIdx, autoPlay = true) {
  const disc = JUKEBOX_DISCS[discIdx] || JUKEBOX_DISCS[0];
  const candidateUrls = [
    ...getSoundUrls(disc.soundPath),
    ...(disc.altSoundPath ? getSoundUrls(disc.altSoundPath) : [])
  ];

  const discImg = document.getElementById('jukebox-disc-img');
  if (discImg) {
    attachMultiSourceImage(discImg, getTextureUrls(disc.tex));
  }

  let urlIndex = 0;
  const tryNextStream = () => {
    if (!bgMusicAudio) return;
    if (urlIndex < candidateUrls.length) {
      bgMusicAudio.src = candidateUrls[urlIndex++];
      bgMusicAudio.volume = bgMusicVolume;
      if (autoPlay && !userPausedMusic && !dmPlaying) {
        bgMusicAudio.play().then(() => {
          bgMusicPlaying = true;
          updateJukeboxUI();
        }).catch((err) => {
          if (err && err.name === 'NotAllowedError') {
            bgMusicPlaying = false;
            updateJukeboxUI();
          } else {
            tryNextStream();
          }
        });
      }
    }
  };

  bgMusicAudio.onerror = tryNextStream;
  tryNextStream();
  updateJukeboxUI();
}

function updateJukeboxUI() {
  const disc = JUKEBOX_DISCS[currentDiscIndex] || JUKEBOX_DISCS[0];
  const nowPlayingEl = document.getElementById('jukebox-now-playing');
  const statusSubEl = document.getElementById('jukebox-status-sub');
  const toggleBtn = document.getElementById('jukebox-toggle-btn');
  const discImg = document.getElementById('jukebox-disc-img');

  const volPct = Math.round(bgMusicVolume * 100);
  if (nowPlayingEl) {
    nowPlayingEl.textContent = `${bgMusicPlaying ? 'Playing' : 'Paused'}: ${disc.shortName}`;
  }
  if (statusSubEl) {
    statusSubEl.textContent = `${disc.title} • Vol ${volPct}%`;
  }
  if (toggleBtn) {
    toggleBtn.textContent = bgMusicPlaying ? 'PAUSE' : 'PLAY';
  }
  if (discImg) {
    discImg.classList.toggle('disc-paused', !bgMusicPlaying);
  }
}

// ==============================================================================
// 7. Advancement Toasts, Tooltips, Releases, Console & Tribute
// ==============================================================================

function showAdvancementToast(title, subtitle, texPath = 'item/diamond.png') {
  const stack = document.getElementById('advancement-toast-stack');
  if (!stack) return;

  const toast = document.createElement('div');
  toast.className = 'mc-advancement-toast';

  const slot = document.createElement('div');
  slot.className = 'toast-icon-slot';
  const img = document.createElement('img');
  img.className = 'mc-png mc-png-lg';
  img.alt = title;
  attachMultiSourceImage(img, getTextureUrls(texPath));
  slot.appendChild(img);

  const textWrap = document.createElement('div');
  textWrap.innerHTML = `
    <div class="mc-e" style="font-size: 12.5px;">${title}</div>
    <div class="mc-f" style="font-size: 12px;">${subtitle}</div>
  `;

  toast.appendChild(slot);
  toast.appendChild(textWrap);
  stack.appendChild(toast);

  setTimeout(() => {
    toast.classList.add('hiding');
    setTimeout(() => {
      if (toast.parentNode) toast.parentNode.removeChild(toast);
    }, 340);
  }, 3600);
}

function initFloatingTooltips() {
  const tooltip = document.getElementById('mc-floating-tooltip');
  if (!tooltip) return;

  document.querySelectorAll('[data-tooltip-title]').forEach((el) => {
    el.addEventListener('mouseenter', () => {
      const title = el.getAttribute('data-tooltip-title') || '';
      const lore = (el.getAttribute('data-tooltip-lore') || '').split('|');
      tooltip.innerHTML = '';
      tooltip.appendChild(parseMinecraftColorText(title, 'mc-f'));
      lore.forEach((line) => {
        if (line.trim()) tooltip.appendChild(parseMinecraftColorText(line, 'mc-7'));
      });
      tooltip.style.display = 'block';
    });

    el.addEventListener('mousemove', (e) => {
      const x = Math.min(window.innerWidth - 320, e.clientX + 20);
      const y = Math.min(window.innerHeight - 110, e.clientY + 20);
      tooltip.style.left = `${Math.max(8, x)}px`;
      tooltip.style.top = `${Math.max(8, y)}px`;
    });

    el.addEventListener('mouseleave', () => {
      tooltip.style.display = 'none';
    });
  });
}

function initScrollReveal() {
  const items = document.querySelectorAll('.reveal-on-scroll');
  if (!('IntersectionObserver' in window)) {
    items.forEach((el) => el.classList.add('revealed'));
    return;
  }
  const observer = new IntersectionObserver(
    (entries) => {
      entries.forEach((entry) => {
        if (entry.isIntersecting) {
          entry.target.classList.add('revealed');
        }
      });
    },
    { threshold: 0.08 }
  );
  items.forEach((el) => observer.observe(el));
}

function formatBytes(bytes) {
  if (!bytes || isNaN(bytes)) return '';
  if (bytes >= 1048576) return `${(bytes / 1048576).toFixed(2)} MB`;
  if (bytes >= 1024) return `${(bytes / 1024).toFixed(1)} KB`;
  return `${bytes} B`;
}

async function initReleaseDownloader() {
  const syncStatusEl = document.getElementById('release-sync-status');
  const pubDateEl = document.getElementById('release-pub-date');
  const refreshBtn = document.getElementById('refresh-releases-btn');
  const copyBtn = document.getElementById('copy-install-btn');

  if (refreshBtn) {
    refreshBtn.addEventListener('click', () => {
      fetchAndBindReleases(true);
    });
  }

  if (copyBtn) {
    copyBtn.addEventListener('click', () => {
      const cmdText = document.getElementById('install-text')?.textContent || '';
      copyCommandToClipboard(cmdText);
    });
  }

  await fetchAndBindReleases(false);

  async function fetchAndBindReleases(showToastOnComplete) {
    if (syncStatusEl) {
      syncStatusEl.textContent = `Checking api.github.com/repos/${SAWBYTE_REPO}/releases ...`;
    }

    let defaultBranch = 'main';
    try {
      const repoRes = await fetch(`https://api.github.com/repos/${SAWBYTE_REPO}`);
      if (repoRes.ok) {
        const repoData = await repoRes.json();
        defaultBranch = repoData.default_branch || 'main';
        const starEl = document.getElementById('nav-star-count');
        if (starEl && typeof repoData.stargazers_count === 'number') {
          starEl.textContent = `★ ${repoData.stargazers_count}`;
        }
        document.querySelectorAll('[data-repo-link]').forEach((a) => {
          a.href = repoData.html_url || `https://github.com/${SAWBYTE_REPO}`;
        });
        document.querySelectorAll('[data-repo-releases-link]').forEach((a) => {
          a.href = `${repoData.html_url || `https://github.com/${SAWBYTE_REPO}`}/releases`;
        });
        document.querySelectorAll('[data-repo-actions-link]').forEach((a) => {
          a.href = `${repoData.html_url || `https://github.com/${SAWBYTE_REPO}`}/actions`;
        });
      }
    } catch (_) {}

    const installCmd = `curl -sSfL https://raw.githubusercontent.com/${SAWBYTE_REPO}/${defaultBranch}/install.sh | bash`;
    const installEl = document.getElementById('install-text');
    if (installEl) installEl.textContent = installCmd;

    const skillDocLink = document.getElementById('skill-doc-link');
    const skillRawLink = document.getElementById('skill-raw-link');
    const footerSkillLink = document.getElementById('footer-skill-link');
    const footerLicenseLink = document.getElementById('footer-license-link');
    if (skillDocLink) skillDocLink.href = `https://github.com/${SAWBYTE_REPO}/blob/${defaultBranch}/skills/sawbyte/SKILL.md`;
    if (skillRawLink) skillRawLink.href = `https://raw.githubusercontent.com/${SAWBYTE_REPO}/${defaultBranch}/skills/sawbyte/SKILL.md`;
    if (footerSkillLink) footerSkillLink.href = `https://github.com/${SAWBYTE_REPO}/blob/${defaultBranch}/skills/sawbyte/SKILL.md`;
    if (footerLicenseLink) footerLicenseLink.href = `https://github.com/${SAWBYTE_REPO}/blob/${defaultBranch}/LICENSE`;

    let release = null;
    try {
      const latestRes = await fetch(`https://api.github.com/repos/${SAWBYTE_REPO}/releases/latest`);
      if (latestRes.ok) {
        release = await latestRes.json();
      } else {
        const listRes = await fetch(`https://api.github.com/repos/${SAWBYTE_REPO}/releases?per_page=5`);
        if (listRes.ok) {
          const list = await listRes.json();
          if (Array.isArray(list) && list.length > 0) release = list[0];
        }
      }
    } catch (_) {}

    if (release && Array.isArray(release.assets)) {
      const tag = release.tag_name || 'latest';
      document.querySelectorAll('.release-tag-badge').forEach((el) => {
        el.textContent = tag;
      });

      if (pubDateEl && release.published_at) {
        const d = new Date(release.published_at);
        pubDateEl.textContent = d.toISOString().split('T')[0];
      }
      if (syncStatusEl) {
        syncStatusEl.innerHTML = `Synced with <span class="mc-f">${SAWBYTE_REPO} (${tag})</span> • ${release.assets.length} files`;
      }

      bindReleaseAssetsToButtons(release.assets, tag);
      renderLiveAssetsTable(release.assets, release.html_url);

      if (showToastOnComplete) {
        showAdvancementToast('Releases Synced', `Loaded ${release.assets.length} files from ${tag}`, 'item/emerald.png');
      }
    } else {
      const dynamicLatestBase = `https://github.com/${SAWBYTE_REPO}/releases/latest/download`;
      document.querySelectorAll('.release-tag-badge').forEach((el) => {
        el.textContent = 'Latest';
      });
      if (syncStatusEl) {
        syncStatusEl.textContent = `Connected to GitHub latest release redirect (${SAWBYTE_REPO})`;
      }
      if (pubDateEl) pubDateEl.textContent = 'Auto Redirect';

      const fallbackAssets = [
        { name: 'sawbyte-launcher-ubuntu-24.04-X64.tar.gz', browser_download_url: `${dynamicLatestBase}/sawbyte-launcher-ubuntu-24.04-X64.tar.gz` },
        { name: 'sawbyte-ubuntu-24.04-X64.tar.gz', browser_download_url: `${dynamicLatestBase}/sawbyte-ubuntu-24.04-X64.tar.gz` },
        { name: 'sawbyte-play-linux-x64.tar.gz', browser_download_url: `${dynamicLatestBase}/sawbyte-play-linux-x64.tar.gz` },
        { name: 'sawbyte-launcher-windows-2022-X64.zip', browser_download_url: `${dynamicLatestBase}/sawbyte-launcher-windows-2022-X64.zip` },
        { name: 'sawbyte-windows-2022-X64.zip', browser_download_url: `${dynamicLatestBase}/sawbyte-windows-2022-X64.zip` },
        { name: 'sawbyte-play-windows-x64.zip', browser_download_url: `${dynamicLatestBase}/sawbyte-play-windows-x64.zip` },
        { name: 'sawbyte-launcher-macos-14-ARM64.tar.gz', browser_download_url: `${dynamicLatestBase}/sawbyte-launcher-macos-14-ARM64.tar.gz` },
        { name: 'sawbyte-macos-14-ARM64.tar.gz', browser_download_url: `${dynamicLatestBase}/sawbyte-macos-14-ARM64.tar.gz` },
        { name: 'sawbyte-play-macos-arm64.tar.gz', browser_download_url: `${dynamicLatestBase}/sawbyte-play-macos-arm64.tar.gz` }
      ];
      bindReleaseAssetsToButtons(fallbackAssets, 'Latest');
      renderLiveAssetsTable(fallbackAssets, `https://github.com/${SAWBYTE_REPO}/releases/latest`);
    }
  }
}

function bindReleaseAssetsToButtons(assets, tag) {
  const matchers = {
    'linux-launcher': (n) => n.includes('launcher') && (n.includes('ubuntu') || n.includes('linux')),
    'linux-engine': (n) => !n.includes('launcher') && !n.includes('play') && (n.includes('ubuntu') || n.includes('linux')),
    'linux-play': (n) => n.includes('play') && (n.includes('linux') || n.includes('ubuntu')),
    'win-launcher': (n) => n.includes('launcher') && n.includes('win'),
    'win-engine': (n) => !n.includes('launcher') && !n.includes('play') && n.includes('win'),
    'win-play': (n) => n.includes('play') && n.includes('win'),
    'mac-launcher': (n) => n.includes('launcher') && (n.includes('mac') || n.includes('darwin')),
    'mac-engine': (n) => !n.includes('launcher') && !n.includes('play') && (n.includes('mac') || n.includes('darwin')),
    'mac-play': (n) => n.includes('play') && (n.includes('mac') || n.includes('darwin')),
    'bridge-jar': (n) => n.includes('bridge') && n.endsWith('.jar')
  };

  for (const [key, matchFn] of Object.entries(matchers)) {
    const btn = document.querySelector(`[data-dl-key="${key}"]`);
    if (!btn) continue;
    const found = assets.find((a) => matchFn((a.name || '').toLowerCase()));
    if (found) {
      btn.href = found.browser_download_url;
      const badge = btn.querySelector('.dl-meta-badge');
      const ext = found.name.endsWith('.tar.gz') ? '.tar.gz' : `.${found.name.split('.').pop()}`;
      const sizeText = formatBytes(found.size);
      if (badge) {
        badge.textContent = sizeText ? `${sizeText} (${ext})` : ext;
      }
      if (key === 'bridge-jar') {
        const bridgeLabel = document.getElementById('bridge-jar-btn-label');
        if (bridgeLabel) {
          bridgeLabel.textContent = `Download ${found.name}${sizeText ? ` (${sizeText})` : ''}`;
        }
      }
    } else {
      btn.href = `https://github.com/${SAWBYTE_REPO}/releases`;
      const badge = btn.querySelector('.dl-meta-badge');
      if (badge) badge.textContent = tag;
      if (key === 'bridge-jar') {
        const bridgeLabel = document.getElementById('bridge-jar-btn-label');
        if (bridgeLabel) bridgeLabel.textContent = `Open Releases (${tag})`;
      }
    }
  }
}

function renderLiveAssetsTable(assets, fallbackUrl) {
  const tbody = document.getElementById('live-assets-tbody');
  if (!tbody) return;
  tbody.innerHTML = '';

  assets.forEach((asset) => {
    const tr = document.createElement('tr');
    const name = asset.name || 'binary';
    const lower = name.toLowerCase();
    let platform = 'Cross-Platform';
    if (lower.includes('ubuntu') || lower.includes('linux')) platform = 'Linux x86_64';
    else if (lower.includes('win')) platform = 'Windows x64';
    else if (lower.includes('mac')) platform = 'macOS ARM64';
    else if (lower.includes('bridge') || lower.endsWith('.jar')) platform = `Fabric ${TARGET_MC_VERSION} Mod`;

    const sizeStr = formatBytes(asset.size) || 'Direct Link';
    const dlCount = typeof asset.download_count === 'number' ? asset.download_count.toLocaleString() : 'Live';
    const url = asset.browser_download_url || fallbackUrl;

    tr.innerHTML = `
      <td class="mc-b">${name}</td>
      <td class="mc-7">${platform}</td>
      <td class="mc-e">${sizeStr}</td>
      <td class="mc-a">${dlCount}</td>
      <td><a href="${url}" target="_blank" rel="noopener" class="status-tag" style="text-decoration: none;">Download -&gt;</a></td>
    `;
    tbody.appendChild(tr);
  });
}

function copyCommandToClipboard(text) {
  const label = document.getElementById('copy-install-label');
  const finishCopy = () => {
    playMcXpChime();
    if (label) {
      label.textContent = 'COPIED!';
      setTimeout(() => {
        label.textContent = 'COPY';
      }, 2000);
    }
    showAdvancementToast('Command Copied!', 'Paste it into your terminal to install', 'block/command_block_front.png');
  };

  if (navigator.clipboard && navigator.clipboard.writeText) {
    navigator.clipboard.writeText(text).then(finishCopy).catch(() => fallbackCopy(text, finishCopy));
  } else {
    fallbackCopy(text, finishCopy);
  }
}

function fallbackCopy(text, cb) {
  const ta = document.createElement('textarea');
  ta.value = text;
  document.body.appendChild(ta);
  ta.select();
  try {
    document.execCommand('copy');
  } catch (_) {}
  document.body.removeChild(ta);
  if (cb) cb();
}

const MC_CODE_MAP = {
  '0': 'mc-0', '1': 'mc-1', '2': 'mc-2', '3': 'mc-3',
  '4': 'mc-4', '5': 'mc-5', '6': 'mc-6', '7': 'mc-7',
  '8': 'mc-8', '9': 'mc-9', 'a': 'mc-a', 'b': 'mc-b',
  'c': 'mc-c', 'd': 'mc-d', 'e': 'mc-e', 'f': 'mc-f'
};

function parseMinecraftColorText(rawText, defaultClass = 'mc-7') {
  const container = document.createElement('div');
  container.style.marginBottom = '4px';

  const parts = String(rawText).split(/(§[0-9a-frA-FR])/g);
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

window.runTerminalQuickCommand = function (cmd) {
  const input = document.getElementById('term-input');
  if (!input) return;
  input.value = cmd;
  const ev = new KeyboardEvent('keydown', { key: 'Enter' });
  input.dispatchEvent(ev);
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
      appendFormattedLine(`§8[Preview] §r${cmd}`);
      return;
    }

    const parts = cmd.split(' ');
    const root = parts[0].toLowerCase();

    switch (root) {
      case 'help':
        appendFormattedLine('§6Sawbyte Console Commands:');
        appendFormattedLine('  §binspect.status                 §7: Show coordinates, health, and tick rate');
        appendFormattedLine('  §bdisc_maker                     §7: Generate & play a random stochastic track');
        appendFormattedLine('  §btool.swap                      §7: Roll a new random cursor weapon/tool');
        appendFormattedLine('  §bsound.pigstep                  §7: Play Lena Raine Pigstep disc');
        appendFormattedLine('  §btechnoblade                    §7: Pay respects [o7]');
        appendFormattedLine('  §bcolors                         §7: Show all §0 to §f color codes');
        appendFormattedLine('  §bclear                          §7: Clear console');
        break;
      case 'colors':
        appendFormattedLine('§0§0Black §1§1DarkBlue §2§2DarkGreen §3§3DarkAqua §4§4DarkRed §5§5Purple §6§6Gold §7§7Gray');
        appendFormattedLine('§8§8DarkGray §9§9Blue §a§aGreen §b§bAqua §c§cRed §d§dLightPurple §e§eYellow §f§fWhite');
        break;
      case 'inspect.status':
        appendFormattedLine(`§apos=(8.50, 64.00, 8.50) §emc=${TARGET_MC_VERSION} §btool="${currentTool.name}" §amined=${totalMinedBlocks} §btps=20.00`);
        break;
      case 'disc_maker':
        if (typeof window.startDiscMakerFromConsole === 'function') {
          window.startDiscMakerFromConsole();
        }
        appendFormattedLine('§d[disc_maker] §fRandomized Euclidean + Markov Minecraft beat & started studio!');
        break;
      case 'tool.swap':
        if (typeof window.swapCursorToolNow === 'function') {
          window.swapCursorToolNow();
        }
        appendFormattedLine(`§b[Cursor Tool] §fEquipped §e${currentTool.name} §7(cooldown ${(currentTool.cooldownMs / 1000).toFixed(2)}s)`);
        break;
      case 'sound.pigstep':
        if (dmPlaying) stopDiscMaker();
        currentDiscIndex = 0;
        userPausedMusic = false;
        loadDiscIntoPlayer(0, true);
        appendFormattedLine('§d[Jukebox] §fPlaying §6Lena Raine: Pigstep');
        break;
      case 'technoblade':
        playMcTotemSound();
        appendFormattedLine('§6[Crown] §e“Technoblade Never Dies!” §c[o7]');
        break;
      case 'clear':
        body.innerHTML = '';
        break;
      default:
        appendFormattedLine(`§cUnknown command '${cmd}'. Type §ehelp §cfor a list of commands.`);
    }
  }
}

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

let potatoWarTotal = 500000000;

window.triggerTechnoSound = function (type) {
  switch (type) {
    case 'pig':
      playMcPigSound();
      showAdvancementToast('Oink! [o7]', 'Played Minecraft Pig sound (mob/pig/say.ogg)', 'item/porkchop.png');
      break;
    case 'totem':
      playMcTotemSound();
      showAdvancementToast('Technoblade Never Dies!', 'Activated Totem of Undying', 'item/totem_of_undying.png');
      break;
    case 'potato':
      potatoWarTotal += Math.floor(Math.random() * 500) + 100;
      const formatted = potatoWarTotal.toLocaleString('en-US');
      const p1 = document.getElementById('potato-war-count');
      const p2 = document.getElementById('potato-badge-count');
      if (p1) p1.textContent = formatted;
      if (p2) p2.textContent = formatted;
      playMcEatSound();
      setTimeout(() => playMcXpChime(), 180);
      showAdvancementToast('Skyblock Potato War #1', `Farmed ${formatted} Baked Potatoes!`, 'item/baked_potato.png');
      break;
    case 'bell':
      playMcBellSound();
      showAdvancementToast('Channel Member Bell!', '“One of Us!”', 'item/bell.png');
      break;
    case 'pigstep':
      if (dmPlaying) stopDiscMaker();
      currentDiscIndex = 0;
      userPausedMusic = false;
      loadDiscIntoPlayer(0, true);
      showAdvancementToast('Playing: Pigstep', 'Lena Raine • Nether Update Disc', 'item/music_disc_pigstep.png');
      break;
    case 'firework':
      playMcFireworkSound();
      showAdvancementToast('Firework Salute', 'Launched tribute firework [o7]', 'item/firework_rocket.png');
      break;
  }
};

function initRespectsAndTechnoSoundboard() {
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
  const COOLDOWN_MS = 3500;

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

  let tributeCount = parseInt(safeStorageGet(STORAGE_KEY_COUNT, '0'), 10) || 0;
  let lastTributeTime = parseInt(safeStorageGet(STORAGE_KEY_LAST_TIME, '0'), 10) || 0;
  let cooldownTimer = null;
  let isCooldown = false;

  function updateDisplay() {
    if (countEl) countEl.textContent = Number(tributeCount).toLocaleString('en-US');
    if (tributeCount > 0) {
      label.innerHTML = `<span class="mc-e">Respects Paid (o7 × ${tributeCount}) • Technoblade Never Dies</span>`;
    } else {
      label.textContent = 'Leave a Tribute Poppy [o7]';
    }
  }

  function spawnFloatingPoppy() {
    if (!wrap) return;
    const poppy = document.createElement('div');
    poppy.className = 'tribute-floating-poppy';

    const pImg = document.createElement('img');
    pImg.className = 'mc-png mc-png-sm';
    pImg.alt = 'Poppy';
    attachMultiSourceImage(pImg, getTextureUrls('block/poppy.png'));

    const textSpan = document.createElement('span');
    textSpan.textContent = '+1 Tribute Poppy [o7]';

    poppy.appendChild(pImg);
    poppy.appendChild(textSpan);

    const offset = (Math.random() - 0.5) * 90;
    poppy.style.transform = `translateX(${offset}px)`;
    poppy.style.top = '8px';

    wrap.appendChild(poppy);
    setTimeout(() => {
      if (poppy.parentNode) poppy.parentNode.removeChild(poppy);
    }, 1700);
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
      label.innerHTML = `<span class="mc-8">[Wait ${secsLeft}s]</span> <span class="mc-7">• Technoblade Never Dies</span>`;

      if (left <= 0) {
        clearInterval(cooldownTimer);
        cooldownTimer = null;
        isCooldown = false;
        btn.classList.remove('tribute-btn-disabled');
        btn.removeAttribute('aria-disabled');
        if (cooldownBar) cooldownBar.style.display = 'none';
        if (cooldownFill) cooldownFill.style.width = '0%';
        if (statusHint) statusHint.textContent = 'Plays Minecraft Pig & Level-Up sounds • Saved in browser storage';
        updateDisplay();
      }
    }, 80);
  }

  updateDisplay();
  const elapsedSinceLast = Date.now() - lastTributeTime;
  if (lastTributeTime > 0 && elapsedSinceLast < COOLDOWN_MS) {
    startCooldown(COOLDOWN_MS - elapsedSinceLast);
  }

  btn.addEventListener('click', (e) => {
    e.preventDefault();
    const now = Date.now();
    const elapsed = now - lastTributeTime;

    if (isCooldown || (lastTributeTime > 0 && elapsed < COOLDOWN_MS)) {
      playMcClick(0.65);
      return;
    }

    tributeCount++;
    lastTributeTime = now;
    safeStorageSet(STORAGE_KEY_COUNT, tributeCount);
    safeStorageSet(STORAGE_KEY_LAST_TIME, lastTributeTime);

    playMcLevelUp();
    setTimeout(() => playMcPigSound(), 280);

    spawnFloatingPoppy();
    updateDisplay();
    showAdvancementToast('Respects Paid [o7]', `Tribute #${tributeCount} • Technoblade Never Dies`, 'block/poppy.png');
    startCooldown(COOLDOWN_MS);
  });
}
