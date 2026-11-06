/**
 * sawbyte - Final Minecraft 26.3 Interactive Engine
 * ============================================================================
 * 1. Real Minecraft 26.3 Intro Screen:
 *    - Official 6-face 3D Panorama Skybox Cube (textures/gui/title/background/panorama_0..5.png)
 *    - Official Title Wordmark (textures/gui/title/minecraft.png & edition.png)
 *    - Official Widget Button Sprites & Tiled Dirt Multiplayer Background
 * 2. 15-Second Rotating Weapon & Tool Cursor:
 *    - Randomly chooses from 12 official Minecraft weapons/tools every 15 seconds
 *    - Authentic 1.9+ Attack Cooldown Indicator bar under the crosshair
 *    - Swing/chop animation + item-specific sounds on click
 * 3. Breakable Minecraft Block & Ore World (Respawns After 2.0 Seconds):
 *    - Built out of real Minecraft blocks & ores with official destroy_stage_0..9.png cracks
 *    - Tool affinity & cooldown scaling, debris particles, floating ore drops, 2s respawn
 * 4. disc_maker - Procedural Harmonic Minecraft Music Composer:
 *    - Disc icon continuously morphs through all 19 official Minecraft Music Discs
 *    - Music-theory constrained random generation (Scale quantization, 4-bar chords,
 *      Euclidean drum grooves, pocket swing, and call-and-response melody phrasing)
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

// Pre-seeded & dynamically populated SHA-1 Sound Map from GitHub Minecraft Asset Index
const SOUND_SHA1_CACHE = {
  'minecraft/sounds/random/orb.ogg': 'e9833a1512b57bcf88ac4fdcc8df4e5a7e9d701d'
};

// Decoded AudioBuffer Cache for low-latency Note Block & Tool SFX
const DECODED_BUFFER_CACHE = {};

// All 19 Official Minecraft Music Discs for disc_maker's continuously changing icon
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

// 12 Official Minecraft Weapons & Tools for the 15-Second Rotating Cursor
const CURSOR_ARSENAL = [
  {
    id: 'netherite_sword',
    name: 'Netherite Sword',
    tex: 'item/netherite_sword.png',
    cooldownMs: 625, // 1.6 attack speed
    baseDamage: 8,
    affinity: 'sword',
    sound: 'entity/player/attack/sweep1.ogg',
    fallbackFreq: 340
  },
  {
    id: 'diamond_sword',
    name: 'Diamond Sword',
    tex: 'item/diamond_sword.png',
    cooldownMs: 625,
    baseDamage: 7,
    affinity: 'sword',
    sound: 'entity/player/attack/sweep2.ogg',
    fallbackFreq: 380
  },
  {
    id: 'golden_sword',
    name: 'Golden Pigman Sword',
    tex: 'item/golden_sword.png',
    cooldownMs: 625,
    baseDamage: 6,
    affinity: 'sword',
    sound: 'entity/player/attack/sweep3.ogg',
    fallbackFreq: 420
  },
  {
    id: 'netherite_pickaxe',
    name: 'Netherite Pickaxe',
    tex: 'item/netherite_pickaxe.png',
    cooldownMs: 833, // 1.2 attack speed
    baseDamage: 10,
    affinity: 'pickaxe',
    sound: 'dig/stone1.ogg',
    fallbackFreq: 260
  },
  {
    id: 'diamond_pickaxe',
    name: 'Diamond Pickaxe',
    tex: 'item/diamond_pickaxe.png',
    cooldownMs: 833,
    baseDamage: 9,
    affinity: 'pickaxe',
    sound: 'dig/stone2.ogg',
    fallbackFreq: 280
  },
  {
    id: 'iron_pickaxe',
    name: 'Iron Pickaxe',
    tex: 'item/iron_pickaxe.png',
    cooldownMs: 833,
    baseDamage: 7,
    affinity: 'pickaxe',
    sound: 'dig/stone3.ogg',
    fallbackFreq: 250
  },
  {
    id: 'netherite_axe',
    name: 'Netherite Axe',
    tex: 'item/netherite_axe.png',
    cooldownMs: 1000, // 1.0 attack speed
    baseDamage: 11,
    affinity: 'axe',
    sound: 'entity/player/attack/crit1.ogg',
    fallbackFreq: 190
  },
  {
    id: 'diamond_axe',
    name: 'Diamond Axe',
    tex: 'item/diamond_axe.png',
    cooldownMs: 1000,
    baseDamage: 10,
    affinity: 'axe',
    sound: 'entity/player/attack/strong1.ogg',
    fallbackFreq: 210
  },
  {
    id: 'mace',
    name: 'Heavy Mace',
    tex: 'item/mace.png',
    fallbackTex: 'item/golden_axe.png',
    cooldownMs: 1600, // 0.6 attack speed (heavy smash!)
    baseDamage: 20,
    affinity: 'all',
    sound: 'random/anvil_land.ogg',
    fallbackFreq: 120
  },
  {
    id: 'trident',
    name: 'Trident',
    tex: 'item/trident.png',
    cooldownMs: 909, // 1.1 attack speed
    baseDamage: 9,
    affinity: 'all',
    sound: 'entity/player/attack/crit2.ogg',
    fallbackFreq: 460
  },
  {
    id: 'diamond_shovel',
    name: 'Diamond Shovel',
    tex: 'item/diamond_shovel.png',
    cooldownMs: 1000,
    baseDamage: 7,
    affinity: 'shovel',
    sound: 'dig/grass1.ogg',
    fallbackFreq: 230
  },
  {
    id: 'diamond_hoe',
    name: 'Diamond Hoe',
    tex: 'item/diamond_hoe.png',
    cooldownMs: 250, // 4.0 attack speed (rapid fire!)
    baseDamage: 4,
    affinity: 'hoe',
    sound: 'dig/cloth1.ogg',
    fallbackFreq: 520
  }
];

let currentTool = CURSOR_ARSENAL[3]; // Start with Netherite Pickaxe
let lastAttackTs = 0;
let toolCountdownSecs = 15;
let totalMinedBlocks = 0;

// Background Jukebox Playlist
const JUKEBOX_DISCS = [
  {
    id: 'pigstep',
    title: 'Lena Raine: Pigstep',
    shortName: 'Pigstep',
    soundPath: 'records/pigstep.ogg',
    altSoundPath: 'music/game/nether/pigstep.ogg',
    tex: 'item/music_disc_pigstep.png'
  },
  {
    id: 'otherside',
    title: 'Lena Raine: otherside',
    shortName: 'otherside',
    soundPath: 'records/otherside.ogg',
    tex: 'item/music_disc_otherside.png'
  },
  {
    id: 'relic',
    title: 'Aaron Cherof: Relic',
    shortName: 'Relic',
    soundPath: 'records/relic.ogg',
    tex: 'item/music_disc_relic.png'
  },
  {
    id: 'cat',
    title: 'C418: cat',
    shortName: 'cat',
    soundPath: 'records/cat.ogg',
    tex: 'item/music_disc_cat.png'
  },
  {
    id: 'stal',
    title: 'C418: stal',
    shortName: 'stal',
    soundPath: 'records/stal.ogg',
    tex: 'item/music_disc_stal.png'
  }
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
  initSoundEngine();
  initPigstepJukebox();
  initMinecraftIntroSequence();
  initWeaponToolCursor();
  initBreakableBlockWorld();
  initDiscMakerStudio();
  initFloatingTooltips();
  initScrollReveal();
  initParticleCanvas();
  initReleaseDownloader();
  initTerminalConsole();
  initBenchmarkTabs();
  initRespectsAndTechnoSoundboard();
});

// ==============================================================================
// 1. Official Minecraft Font & Multi-CDN Asset Resolver
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

function initMcAssetRegistry() {
  // Hydrate 3D Panorama Skybox Cube faces from GitHub / mcasset.cloud
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

  // Fetch Minecraft Asset Index JSON from GitHub to map all .ogg sounds to SHA-1 hashes
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
// 2. Real Minecraft 26.3 Intro Sequence (Full vs 2s Repeat Visit)
// ==============================================================================

function initMinecraftIntroSequence() {
  const overlay = document.getElementById('mc-intro-overlay');
  const cursor = document.getElementById('mc-virtual-cursor');
  const replayBtn = document.getElementById('replay-intro-btn');
  if (!overlay) return;

  let panoramaActive = true;
  startPanoramaRenderer(() => panoramaActive);

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
    setTimeout(() => {
      if (cursor) cursor.classList.add('clicking');
      btnEl.classList.add('pressed');
      playMcClick(1.0);
      setTimeout(() => {
        if (cursor) cursor.classList.remove('clicking');
        btnEl.classList.remove('pressed');
        btnEl.classList.remove('hovered');
        if (cb) cb();
      }, 130);
    }, 160);
  }

  function runLoadingTerrainPhase(durationMs, onDone) {
    showIntroScreen('intro-screen-loading');
    if (cursor) cursor.style.display = 'none';

    const labelEl = document.getElementById('mc-loading-stage-text');
    const fillEl = document.getElementById('mc-terrain-progress-fill');
    const grayBox = document.getElementById('mc-chunk-gray-box');

    if (labelEl) labelEl.textContent = 'Joining world...';
    if (fillEl) fillEl.style.width = '8%';
    if (grayBox) {
      grayBox.style.width = '26px';
      grayBox.style.height = '26px';
    }

    const start = performance.now();
    function step(ts) {
      const elapsed = ts - start;
      const pct = Math.min(1, elapsed / durationMs);

      if (pct > 0.18 && labelEl && labelEl.textContent !== 'Loading terrain...') {
        labelEl.textContent = 'Loading terrain...';
      }
      if (fillEl) fillEl.style.width = `${Math.floor(pct * 100)}%`;
      if (grayBox) {
        const size = Math.floor(26 + pct * 54);
        grayBox.style.width = `${size}px`;
        grayBox.style.height = `${size}px`;
      }

      if (pct < 1) {
        requestAnimationFrame(step);
      } else {
        playMcXpChime();
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
    }, 340);
  }

  function runIntroSequence(fullMode) {
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

    if (!fullMode) {
      runLoadingTerrainPhase(1850, finishIntro);
      return;
    }

    showIntroScreen('intro-screen-title');
    if (cursor) {
      cursor.style.display = 'block';
      cursor.style.left = `${Math.floor(window.innerWidth * 0.72)}px`;
      cursor.style.top = `${Math.floor(window.innerHeight * 0.64)}px`;
    }

    const mpBtn = document.getElementById('mc-btn-multiplayer');
    const addServerBtn = document.getElementById('mc-btn-add-server');
    const addDoneBtn = document.getElementById('mc-btn-add-done');

    setTimeout(() => {
      if (mpBtn) mpBtn.classList.add('hovered');
      moveCursorToElement(mpBtn);
    }, 320);

    setTimeout(() => {
      clickButtonElement(mpBtn, () => {
        showIntroScreen('intro-screen-multiplayer');
        setTimeout(() => {
          if (addServerBtn) addServerBtn.classList.add('hovered');
          moveCursorToElement(addServerBtn);
        }, 240);

        setTimeout(() => {
          clickButtonElement(addServerBtn, () => {
            showIntroScreen('intro-screen-add-server');

            setTimeout(() => {
              moveCursorToElement(addrInput);
            }, 200);

            setTimeout(() => {
              if (cursor) cursor.classList.add('clicking');
              playMcClick(1.15);
              if (addrInput) {
                addrInput.classList.add('focused');
                addrInput.textContent = '_';
              }
              setTimeout(() => {
                if (cursor) cursor.classList.remove('clicking');
                if (addrInput) {
                  addrInput.textContent = 'https://5ivesaw.github.io/Sawbyte_';
                }
                playMcClick(1.3);
              }, 150);
            }, 520);

            setTimeout(() => {
              moveCursorToElement(nameInput);
            }, 880);

            setTimeout(() => {
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
              setTimeout(() => {
                if (cursor) cursor.classList.remove('clicking');
              }, 110);
            }, 1220);

            setTimeout(() => {
              if (addDoneBtn) addDoneBtn.classList.add('hovered');
              moveCursorToElement(addDoneBtn);
            }, 1480);

            setTimeout(() => {
              clickButtonElement(addDoneBtn, () => {
                if (emptyMsg) emptyMsg.style.display = 'none';
                if (serverRow) serverRow.style.display = 'flex';
                showIntroScreen('intro-screen-multiplayer');

                setTimeout(() => {
                  moveCursorToElement(serverRow);
                }, 200);

                setTimeout(() => {
                  if (cursor) cursor.classList.add('clicking');
                  playMcClick(1.05);
                  if (serverRow) serverRow.classList.add('selected');
                  if (joinBtn) joinBtn.classList.remove('disabled');
                  if (editBtn) editBtn.classList.remove('disabled');
                  if (deleteBtn) deleteBtn.classList.remove('disabled');
                  setTimeout(() => {
                    if (cursor) cursor.classList.remove('clicking');
                  }, 100);
                }, 540);

                setTimeout(() => {
                  if (joinBtn) joinBtn.classList.add('hovered');
                  moveCursorToElement(joinBtn);
                }, 760);

                setTimeout(() => {
                  clickButtonElement(joinBtn, () => {
                    runLoadingTerrainPhase(1450, finishIntro);
                  });
                }, 1120);
              });
            }, 1820);
          });
        }, 680);
      });
    }, 780);
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
// 3. 15-Second Rotating Weapon & Tool Cursor + 1.9+ Cooldown Bar & Swing Arc
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

  // Rotate to a new random weapon/tool every 15 seconds
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
      showAdvancementToast('Equipped New Tool!', `${currentTool.name} (Cooldown ${(currentTool.cooldownMs / 1000).toFixed(2)}s)`, currentTool.tex);
    });
  }

  window.swapCursorToolNow = () => equipRandomTool(true);

  // Track pointer position smoothly
  window.addEventListener('mousemove', (e) => {
    if (!cursorEl) return;
    cursorEl.style.left = `${e.clientX}px`;
    cursorEl.style.top = `${e.clientY}px`;
  });

  // Trigger swing animation, item-specific sound, and 1.9+ cooldown bar on click
  window.addEventListener('mousedown', (e) => {
    if (document.body.classList.contains('intro-cursor-locked')) return;

    const now = performance.now();
    const elapsed = now - lastAttackTs;
    const chargeRatio = Math.min(1, elapsed / currentTool.cooldownMs);
    lastAttackTs = now;

    // 1. Swing animation
    if (itemWrap) {
      itemWrap.classList.remove('swinging');
      void itemWrap.offsetWidth;
      itemWrap.classList.add('swinging');
    }

    // 2. Play sound according to current cursor weapon/tool & cooldown charge
    if (chargeRatio >= 0.85) {
      playMinecraftOgg(currentTool.sound, 0.45, 0.95 + Math.random() * 0.1, () => {
        synthToolSoundFallback(currentTool.fallbackFreq, 1.0);
      });
      if (currentTool.id === 'golden_sword' && Math.random() < 0.35) {
        setTimeout(() => playMcPigSound(), 90);
      }
      // Spawn sweep slash particle on charged sword/axe/mace swing
      if (['sword', 'axe', 'all'].includes(currentTool.affinity)) {
        spawnSweepSlash(e.clientX, e.clientY);
      }
    } else {
      // Weak uncharged hit sound when spam-clicking during cooldown
      playMinecraftOgg('entity/player/attack/weak1.ogg', 0.3, 1.2, () => {
        synthToolSoundFallback(currentTool.fallbackFreq * 0.7, 0.4);
      });
    }

    // 3. Animate 1.9+ Crosshair Attack Cooldown Indicator Bar
    animateToolCooldown(currentTool.cooldownMs);
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
// 4. Breakable Minecraft Block & Ore World (destroy_stage_0..9 + 2s Respawn)
// ==============================================================================

const BLOCK_CATALOG = {
  grass_block: { name: 'Grass Block', tex: 'block/grass_block_side.png', hp: 12, cat: 'shovel', sound: 'dig/grass1.ogg' },
  dirt: { name: 'Dirt', tex: 'block/dirt.png', hp: 10, cat: 'shovel', sound: 'dig/grass2.ogg' },
  oak_log: { name: 'Oak Log', tex: 'block/oak_log.png', hp: 16, cat: 'axe', sound: 'dig/wood1.ogg', dropTex: 'block/oak_log.png' },
  oak_planks: { name: 'Oak Planks', tex: 'block/oak_planks.png', hp: 16, cat: 'axe', sound: 'dig/wood2.ogg' },
  crafting_table: { name: 'Crafting Table', tex: 'block/crafting_table_front.png', hp: 18, cat: 'axe', sound: 'dig/wood3.ogg' },
  bookshelf: { name: 'Bookshelf', tex: 'block/bookshelf.png', hp: 15, cat: 'axe', sound: 'dig/wood4.ogg', dropTex: 'item/book.png', dropLabel: '+3 Books' },
  stone: { name: 'Stone', tex: 'block/stone.png', hp: 18, cat: 'pickaxe', sound: 'dig/stone1.ogg' },
  cobblestone: { name: 'Cobblestone', tex: 'block/cobblestone.png', hp: 20, cat: 'pickaxe', sound: 'dig/stone2.ogg' },
  coal_ore: { name: 'Coal Ore', tex: 'block/coal_ore.png', hp: 22, cat: 'pickaxe', sound: 'dig/stone3.ogg', dropTex: 'item/coal.png', dropLabel: '+1 Coal' },
  iron_ore: { name: 'Iron Ore', tex: 'block/iron_ore.png', hp: 24, cat: 'pickaxe', sound: 'dig/stone4.ogg', dropTex: 'item/raw_iron.png', dropLabel: '+1 Raw Iron' },
  gold_ore: { name: 'Gold Ore', tex: 'block/gold_ore.png', hp: 26, cat: 'pickaxe', sound: 'dig/stone1.ogg', dropTex: 'item/raw_gold.png', dropLabel: '+1 Raw Gold' },
  redstone_ore: { name: 'Redstone Ore', tex: 'block/redstone_ore.png', hp: 26, cat: 'pickaxe', sound: 'dig/stone2.ogg', dropTex: 'item/redstone.png', dropLabel: '+4 Redstone' },
  emerald_ore: { name: 'Emerald Ore', tex: 'block/emerald_ore.png', hp: 28, cat: 'pickaxe', sound: 'dig/stone3.ogg', dropTex: 'item/emerald.png', dropLabel: '+1 Emerald' },
  diamond_ore: { name: 'Diamond Ore', tex: 'block/diamond_ore.png', hp: 30, cat: 'pickaxe', sound: 'dig/stone4.ogg', dropTex: 'item/diamond.png', dropLabel: '+1 Diamond' },
  deepslate: { name: 'Deepslate', tex: 'block/deepslate.png', hp: 24, cat: 'pickaxe', sound: 'dig/stone1.ogg' },
  deepslate_diamond_ore: { name: 'Deepslate Diamond Ore', tex: 'block/deepslate_diamond_ore.png', hp: 32, cat: 'pickaxe', sound: 'dig/stone2.ogg', dropTex: 'item/diamond.png', dropLabel: '+1 Diamond' },
  amethyst_block: { name: 'Amethyst Block', tex: 'block/amethyst_block.png', hp: 16, cat: 'pickaxe', sound: 'note/chime.ogg', dropTex: 'item/amethyst_shard.png', dropLabel: '+2 Shards' },
  obsidian: { name: 'Obsidian', tex: 'block/obsidian.png', hp: 40, cat: 'pickaxe', sound: 'dig/stone4.ogg' },
  gilded_blackstone: { name: 'Gilded Blackstone', tex: 'block/gilded_blackstone.png', hp: 28, cat: 'pickaxe', sound: 'dig/stone2.ogg', dropTex: 'item/gold_nugget.png', dropLabel: '+5 Gold Nuggets' },
  ancient_debris: { name: 'Ancient Debris', tex: 'block/ancient_debris_side.png', hp: 36, cat: 'pickaxe', sound: 'dig/stone3.ogg', dropTex: 'item/netherite_scrap.png', dropLabel: '+1 Netherite Scrap' }
};

const STRATUM_POOLS = {
  surface: ['grass_block', 'grass_block', 'dirt', 'oak_log', 'oak_planks', 'crafting_table', 'bookshelf', 'stone', 'coal_ore', 'iron_ore'],
  cavern: ['stone', 'cobblestone', 'coal_ore', 'iron_ore', 'gold_ore', 'redstone_ore', 'emerald_ore', 'diamond_ore'],
  deepslate: ['deepslate', 'deepslate', 'deepslate_diamond_ore', 'diamond_ore', 'emerald_ore', 'amethyst_block', 'redstone_ore', 'gold_ore'],
  nether: ['obsidian', 'gilded_blackstone', 'ancient_debris', 'gold_ore', 'deepslate_diamond_ore', 'obsidian', 'gilded_blackstone']
};

function initBreakableBlockWorld() {
  const grids = document.querySelectorAll('.mc-block-wall-grid[data-stratum]');
  const BLOCK_COUNT_PER_ROW = 26;

  grids.forEach((grid) => {
    const stratumKey = grid.getAttribute('data-stratum') || 'surface';
    const pool = STRATUM_POOLS[stratumKey] || STRATUM_POOLS.surface;

    for (let i = 0; i < BLOCK_COUNT_PER_ROW; i++) {
      const blockTypeKey = pool[i % pool.length];
      const def = BLOCK_CATALOG[blockTypeKey] || BLOCK_CATALOG.stone;

      const blockEl = document.createElement('div');
      blockEl.className = 'mc-world-block';
      blockEl.title = `${def.name} (Click to mine)`;

      const texUrl = `https://raw.githubusercontent.com/InventivetalentDev/minecraft-assets/1.21.4/assets/minecraft/textures/${def.tex}`;
      blockEl.style.backgroundImage = `url('${texUrl}')`;

      const crackEl = document.createElement('div');
      crackEl.className = 'mc-block-crack-layer';
      blockEl.appendChild(crackEl);

      let currentHp = def.hp;
      let isBroken = false;

      blockEl.addEventListener('mousedown', (e) => {
        if (isBroken) return;

        // Calculate damage from current cursor weapon/tool + affinity + cooldown
        let dmg = currentTool.baseDamage || 8;
        if (currentTool.affinity === def.cat || currentTool.affinity === 'all') {
          dmg *= 2.2;
        }
        currentHp -= dmg;

        blockEl.classList.remove('hitting');
        void blockEl.offsetWidth;
        blockEl.classList.add('hitting');

        // Spawn 3 chip particles on hit
        spawnBlockDebris(e.clientX, e.clientY, texUrl, 3);

        if (currentHp > 0) {
          // Calculate crack stage 0..9
          const damageFraction = 1 - currentHp / def.hp;
          const stage = Math.min(9, Math.max(0, Math.floor(damageFraction * 10)));
          const crackUrl = `https://raw.githubusercontent.com/InventivetalentDev/minecraft-assets/1.21.4/assets/minecraft/textures/block/destroy_stage_${stage}.png`;
          crackEl.style.backgroundImage = `url('${crackUrl}')`;
          crackEl.style.opacity = '0.88';
          playMinecraftOgg(def.sound, 0.35, 0.9 + Math.random() * 0.2);
        } else {
          // Block Broken!
          isBroken = true;
          crackEl.style.opacity = '0';
          blockEl.classList.add('broken-block');
          totalMinedBlocks++;

          const minedHud = document.getElementById('hud-mined-count');
          if (minedHud) minedHud.textContent = totalMinedBlocks.toLocaleString();

          playMinecraftOgg(def.sound, 0.55, 0.82);
          spawnBlockDebris(e.clientX, e.clientY, texUrl, 9);

          if (def.dropTex) {
            playMcXpChime();
            spawnFloatingOreDrop(e.clientX, e.clientY, def.dropTex, def.dropLabel || `+1 ${def.name}`);
          }

          // Respawn block after exactly 2.0 seconds (2000ms)!
          setTimeout(() => {
            isBroken = false;
            currentHp = def.hp;
            blockEl.classList.remove('broken-block');
            blockEl.classList.remove('respawning-pop');
            void blockEl.offsetWidth;
            blockEl.classList.add('respawning-pop');
            playMinecraftOgg('random/pop.ogg', 0.22, 1.3);
          }, 2000);
        }
      });

      grid.appendChild(blockEl);
    }
  });
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

    const vx = (Math.random() - 0.5) * 90;
    const vy = -(Math.random() * 55 + 15);
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
// 5. disc_maker - Procedural Harmonic Minecraft Music Composer
//    - Cycling random disc icon across all 19 official Minecraft Music Discs
//    - Music-theory constrained generator (Scale quantization, 4-bar chords,
//      Euclidean groove drums, pocket swing, and call-and-response melody)
// ==============================================================================

const DM_SCALES = {
  phrygian: [0, 1, 4, 5, 7, 8, 10],    // Pigstep Nether Phrygian Dominant
  dorian: [0, 2, 3, 5, 7, 9, 10],      // Aria Math Dorian Groove
  pentatonic: [0, 3, 5, 7, 10],        // Otherside Minor Pentatonic Funk
  lydian: [0, 2, 4, 6, 7, 9, 11],      // C418 Nostalgic Lydian Dream
  hirajoshi: [0, 2, 3, 7, 8]           // Cherry Grove Hirajoshi
};

const DM_PROGRESSIONS = [
  [0, 5, 2, 6], // i - VI - III - VII (Anthem)
  [0, 1, 0, 6], // i - bII - i - vii (Bastion Pigstep)
  [0, 3, 6, 2], // i - iv - VII - III (Funk Groove)
  [0, 4, 5, 3]  // I - V - vi - IV (Bright)
];

const DM_TRACKS = [
  { id: 'lead', name: 'Lead Pling', tex: 'block/note_block.png', sound: 'note/pling.ogg', synthType: 'square', baseMidi: 66, css: 'track-lead' },
  { id: 'arp', name: 'Chord Harp', tex: 'item/amethyst_shard.png', sound: 'note/harp.ogg', synthType: 'triangle', baseMidi: 60, css: 'track-arp' },
  { id: 'bell', name: 'Gold Bell', tex: 'item/bell.png', sound: 'note/bell.ogg', synthType: 'sine', baseMidi: 72, css: 'track-bell' },
  { id: 'bass', name: 'Sub Bass', tex: 'block/soul_sand.png', sound: 'note/bass.ogg', synthType: 'sawtooth', baseMidi: 42, css: 'track-bass' },
  { id: 'drums', name: 'Groove Kit', tex: 'block/tnt_side.png', sound: 'note/bd.ogg', synthType: 'drum', baseMidi: 36, css: 'track-drums' },
  { id: 'fx', name: 'MC Foley', tex: 'item/experience_bottle.png', sound: 'random/orb.ogg', synthType: 'fx', baseMidi: 66, css: 'track-fx' }
];

let dmPlaying = false;
let dmAutoMutate = true;
let dmStep = 0;
let dmBar = 0;
let dmTimerId = null;

// 6 tracks x 16 steps pattern state (each cell holds null or { degree, octave, drumType })
let dmPattern = Array.from({ length: 6 }, () => Array(16).fill(null));

function initDiscMakerStudio() {
  // 1. Continuously cycle all .disc-maker-cycling-icon elements through all 19 Minecraft Music Discs!
  let discCycleCounter = 0;
  setInterval(() => {
    discCycleCounter++;
    const randomDiscTex = ALL_MINECRAFT_DISCS[Math.floor(Math.random() * ALL_MINECRAFT_DISCS.length)];
    document.querySelectorAll('.disc-maker-cycling-icon').forEach((img) => {
      attachMultiSourceImage(img, getTextureUrls(randomDiscTex));
    });
    const discNameClean = randomDiscTex.replace('item/', '').replace('.png', '');
    const scaleSelect = document.getElementById('dm-scale-select');
    const scaleLabel = scaleSelect ? scaleSelect.options[scaleSelect.selectedIndex]?.text : 'Phrygian Groove';
    const readout = document.getElementById('dm-active-disc-label');
    if (readout) {
      readout.textContent = `Disc Sprite: ${discNameClean} • Vibe: ${scaleLabel}`;
    }
  }, 1600);

  // Preload Note Block .ogg buffers into Web Audio API for sample-accurate pitch shifting
  preloadNoteBlockSamples();

  // Build 16-Step x 6-Track Sequencer Grid UI
  const gridContainer = document.getElementById('dm-sequencer-grid');
  if (gridContainer) {
    gridContainer.innerHTML = '';
    DM_TRACKS.forEach((track, tIdx) => {
      const row = document.createElement('div');
      row.className = 'sequencer-row';

      const label = document.createElement('div');
      label.className = 'seq-track-label';
      const icon = document.createElement('img');
      icon.className = 'mc-png mc-png-sm';
      attachMultiSourceImage(icon, getTextureUrls(track.tex));
      const text = document.createElement('span');
      text.className = 'mc-f';
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

  // Generate initial harmonically locked pattern
  generateMusicalPattern();

  // Bind Studio Controls
  const playBtn = document.getElementById('dm-play-btn');
  const turntableBtn = document.getElementById('dm-turntable-btn');
  const randBtn = document.getElementById('dm-randomize-btn');
  const mutateBtn = document.getElementById('dm-mutate-btn');
  const bpmSlider = document.getElementById('dm-bpm-slider');
  const swingSlider = document.getElementById('dm-swing-slider');
  const densitySlider = document.getElementById('dm-density-slider');
  const scaleSelect = document.getElementById('dm-scale-select');
  const rootSelect = document.getElementById('dm-root-select');
  const progSelect = document.getElementById('dm-prog-select');

  const toggleStudioPlay = () => {
    if (dmPlaying) {
      stopDiscMaker();
    } else {
      startDiscMaker();
    }
  };

  if (playBtn) playBtn.addEventListener('click', toggleStudioPlay);
  if (turntableBtn) turntableBtn.addEventListener('click', toggleStudioPlay);

  if (randBtn) {
    randBtn.addEventListener('click', () => {
      randomizeAllStudioParameters();
      generateMusicalPattern();
      if (!dmPlaying) startDiscMaker();
      showAdvancementToast('disc_maker Randomized!', 'Generated new scale-locked Minecraft beat', 'item/music_disc_relic.png');
    });
  }

  if (mutateBtn) {
    mutateBtn.addEventListener('click', () => {
      dmAutoMutate = !dmAutoMutate;
      const lbl = document.getElementById('dm-mutate-label');
      if (lbl) lbl.textContent = `Auto-Evolve: ${dmAutoMutate ? 'ON' : 'OFF'}`;
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
  if (densitySlider) {
    densitySlider.addEventListener('input', (e) => {
      const el = document.getElementById('dm-density-val');
      if (el) el.textContent = `${e.target.value}%`;
      generateMusicalPattern();
    });
  }
  if (scaleSelect) scaleSelect.addEventListener('change', () => generateMusicalPattern());
  if (rootSelect) rootSelect.addEventListener('change', () => generateMusicalPattern());
  if (progSelect) progSelect.addEventListener('change', () => generateMusicalPattern());

  window.startDiscMakerFromConsole = () => {
    randomizeAllStudioParameters();
    generateMusicalPattern();
    startDiscMaker();
  };
}

function randomizeAllStudioParameters() {
  const scaleSelect = document.getElementById('dm-scale-select');
  const rootSelect = document.getElementById('dm-root-select');
  const progSelect = document.getElementById('dm-prog-select');
  const bpmSlider = document.getElementById('dm-bpm-slider');
  const swingSlider = document.getElementById('dm-swing-slider');
  const densitySlider = document.getElementById('dm-density-slider');

  if (scaleSelect) scaleSelect.selectedIndex = Math.floor(Math.random() * scaleSelect.options.length);
  if (rootSelect) rootSelect.selectedIndex = Math.floor(Math.random() * rootSelect.options.length);
  if (progSelect) progSelect.selectedIndex = Math.floor(Math.random() * progSelect.options.length);

  const randomBpm = Math.floor(96 + Math.random() * 38);
  const randomSwing = Math.floor(10 + Math.random() * 24);
  const randomDensity = Math.floor(52 + Math.random() * 32);

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
}

/**
 * Generates a 16-step pattern across all 6 tracks using real music theory:
 * - Track 0 (Lead Pling): Call-and-response motif in bars 1 & 2, resolving on chord tones
 * - Track 1 (Chord Harp): Syncopated arpeggio locked to chord triad (degrees 0, 2, 4)
 * - Track 2 (Gold Bell): Sparse high-octave harmonic sparkles on off-beats
 * - Track 3 (Sub Bass): Root & fifth groove locked to kick drum
 * - Track 4 (Groove Kit): Kick on 0, 6, 8, 10; Snare locked to backbeat 4 & 12; groovy Hi-Hats
 * - Track 5 (MC Foley): Pitch-quantized XP Orb / Pig accent at bar turnarounds (step 0 or 14)
 */
function generateMusicalPattern() {
  const densitySlider = document.getElementById('dm-density-slider');
  const density = (parseInt(densitySlider?.value || '68', 10) || 68) / 100;

  const newPat = Array.from({ length: 6 }, () => Array(16).fill(null));

  // Create a 4-step melodic motif for call-and-response coherence
  const motifDegrees = [0, 2, 4, 3, 2, 0];

  for (let s = 0; s < 16; s++) {
    const isDownbeat = s % 4 === 0;
    const isBackbeat = s === 4 || s === 12;

    // Track 4: Groove Drums (Always tight and musical!)
    if (s === 0 || s === 8 || (s === 6 && Math.random() < 0.75) || (s === 10 && Math.random() < 0.6)) {
      newPat[4][s] = { drumType: 'kick', label: 'K' };
    } else if (isBackbeat) {
      newPat[4][s] = { drumType: 'snare', label: 'S' };
    } else if (s % 2 === 1 || Math.random() < density * 0.65) {
      newPat[4][s] = { drumType: 'hat', label: 'H' };
    }

    // Track 3: Sub Bass (Locked to root on downbeats, fifth/octave on syncopations)
    if (s === 0 || s === 8) {
      newPat[3][s] = { degree: 0, octave: -1, label: 'R' };
    } else if (s === 3 || s === 6 || s === 10 || s === 14) {
      if (Math.random() < density * 0.85) {
        const deg = Math.random() < 0.6 ? 0 : 4;
        newPat[3][s] = { degree: deg, octave: deg === 0 ? 0 : -1, label: deg === 0 ? '8' : '5' };
      }
    }

    // Track 1: Chord Harp Arpeggio (Triad degrees 0, 2, 4)
    const arpTriad = [0, 2, 4, 2];
    if (s % 2 === 0 || Math.random() < density * 0.6) {
      const deg = arpTriad[s % 4];
      newPat[1][s] = { degree: deg, octave: 0, label: String(deg + 1) };
    }

    // Track 0: Lead Pling (Call in steps 0..6, breath rest on 7, Response in 8..14, resolve on 12/14)
    if (s !== 7 && s !== 15) {
      if ((isDownbeat && Math.random() < 0.85) || Math.random() < density * 0.65) {
        const deg = s === 12 || s === 14
          ? 0 // Resolve to root at end of phrase
          : motifDegrees[(s + Math.floor(Math.random() * 3)) % motifDegrees.length];
        const oct = Math.random() < 0.25 ? 1 : 0;
        newPat[0][s] = { degree: deg, octave: oct, label: String(deg + 1) };
      }
    }

    // Track 2: Gold Bell Counter-Melody (High sparkling accents on 2, 6, 10, 14)
    if ((s === 2 || s === 6 || s === 10 || s === 14) && Math.random() < density * 0.65) {
      const bellDeg = [4, 2, 0][Math.floor(Math.random() * 3)];
      newPat[2][s] = { degree: bellDeg, octave: 1, label: '★' };
    }

    // Track 5: Minecraft Foley Accents (Only on phrase turnarounds so it stays musical!)
    if ((s === 0 && Math.random() < 0.5) || (s === 15 && Math.random() < 0.65)) {
      const fxType = s === 0 ? 'orb' : 'pig';
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
      dmPattern[tIdx][sIdx] = { degree: (sIdx % 3) * 2, octave: tIdx === 3 ? -1 : 0, label: '♪' };
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
  // Pause background Pigstep jukebox so they don't play over each other
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

  const bpm = parseInt(document.getElementById('dm-bpm-slider')?.value || '116', 10) || 116;
  const swingPct = (parseInt(document.getElementById('dm-swing-slider')?.value || '22', 10) || 22) / 100;

  const sixteenthMs = (60000 / bpm) / 4;
  // Apply pocket swing to odd 16th steps
  const currentStepDuration = (dmStep % 2 === 0)
    ? sixteenthMs * (1 + swingPct * 0.5)
    : sixteenthMs * (1 - swingPct * 0.5);

  playDiscMakerStep(dmStep, dmBar);

  dmStep++;
  if (dmStep >= 16) {
    dmStep = 0;
    dmBar = (dmBar + 1) % 4;
    // Gently evolve melody every 4 bars if Auto-Evolve is enabled
    if (dmBar === 0 && dmAutoMutate) {
      mutateMelodyPhraseSlightly();
    }
  }

  dmTimerId = setTimeout(scheduleNextDiscMakerStep, currentStepDuration);
}

function mutateMelodyPhraseSlightly() {
  for (let s = 0; s < 16; s++) {
    if (Math.random() < 0.25 && s !== 12 && s !== 14) {
      if (dmPattern[0][s]) {
        const nextDeg = [0, 2, 3, 4][Math.floor(Math.random() * 4)];
        dmPattern[0][s].degree = nextDeg;
        dmPattern[0][s].label = String(nextDeg + 1);
      }
    }
  }
  renderSequencerGridUI();
}

function playDiscMakerStep(stepIdx, barIdx) {
  // Highlight current step column
  document.querySelectorAll('#dm-sequencer-grid .seq-cell').forEach((cell) => {
    const s = parseInt(cell.dataset.step, 10);
    cell.classList.toggle('playhead-now', s === stepIdx);
  });

  const scaleKey = document.getElementById('dm-scale-select')?.value || 'phrygian';
  const scale = DM_SCALES[scaleKey] || DM_SCALES.phrygian;
  const rootOffset = parseInt(document.getElementById('dm-root-select')?.value || '0', 10) || 0;
  const progIdx = parseInt(document.getElementById('dm-prog-select')?.value || '0', 10) || 0;
  const progression = DM_PROGRESSIONS[progIdx] || DM_PROGRESSIONS[0];
  const activeChordDegree = progression[barIdx % progression.length];

  const chordNames = ['i (Root)', 'bII', 'III', 'iv', 'v', 'VI', 'VII'];
  const readout = document.getElementById('dm-chord-readout');
  if (readout) {
    readout.textContent = `Bar ${barIdx + 1}/4 • Chord: ${chordNames[activeChordDegree % chordNames.length]} • Step ${stepIdx + 1}/16`;
  }

  // Trigger each active track on this step
  for (let t = 0; t < DM_TRACKS.length; t++) {
    const cellData = dmPattern[t][stepIdx];
    if (!cellData) continue;
    const track = DM_TRACKS[t];

    if (track.id === 'drums') {
      playComposerDrum(cellData.drumType);
    } else if (track.id === 'fx') {
      playComposerFoley(cellData.fxType, rootOffset);
    } else {
      // Calculate exact semitone offset in the chosen scale + active chord root!
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

  // 1. If official Minecraft Note Block AudioBuffer is decoded, play it with exact WebAudio pitch rate!
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

  // 2. Clean WebAudio Harmonic Synth + HTML5 Audio stream if buffer still loading
  const now = ctx.currentTime;
  const osc = ctx.createOscillator();
  const gain = ctx.createGain();
  const filter = ctx.createBiquadFilter();

  // F#4 = 369.99 Hz (Minecraft's native note block center frequency)
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
    // Punchy sub kick reinforcement
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
  const COUNT = Math.min(75, Math.floor((width * height) / 16000));
  const palette = [
    'rgba(85, 255, 255, ',
    'rgba(85, 255, 85, ',
    'rgba(255, 170, 0, ',
    'rgba(255, 85, 85, ',
    'rgba(255, 85, 255, '
  ];

  for (let i = 0; i < COUNT; i++) {
    particles.push({
      x: Math.random() * width,
      y: Math.random() * height,
      size: Math.random() > 0.6 ? 3 : 2,
      vy: -(Math.random() * 0.32 + 0.08),
      vx: (Math.random() - 0.5) * 0.16,
      alpha: Math.random() * 0.5 + 0.15,
      pulse: Math.random() * 0.015 + 0.005,
      color: palette[Math.floor(Math.random() * palette.length)]
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
        appendFormattedLine('  §bdisc_maker                     §7: Generate & play a random harmonic track');
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
        appendFormattedLine('§d[disc_maker] §fRandomized scale-locked Minecraft beat & started studio playback!');
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
