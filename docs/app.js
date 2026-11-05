/**
 * sawbyte - Minecraft 26.3 Interactive Portal
 * ============================================================================
 * 1. Minecraft 26.3 Java Edition Joining Server Intro Sequence:
 *    - Hides real mouse cursor and drives a virtual cursor through:
 *      Title Screen -> Multiplayer -> Add Server -> Paste https://5ivesaw.github.io/Sawbyte
 *      & Name "Sawbyte" -> Done -> Select Sawbyte -> Join Server -> Loading terrain...
 *    - Smart Repeat-Visit Logic:
 *      First visit (or after 5 short visits, or after 30+ minutes away): plays full sequence.
 *      Visits 2..5: shows only the 2-second "Loading terrain..." screen so it never gets annoying.
 * 2. Official Minecraft PNGs & OGG Sounds Streamed from Public GitHub
 * 3. Low-Volume Background Pigstep Jukebox + Technoblade Soundboard
 * 4. 100% Auto-Fetched GitHub Releases & Screenshot Placeholder Loader
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
  initFloatingTooltips();
  initScrollReveal();
  initParticleCanvas();
  initReleaseDownloader();
  initTerminalConsole();
  initBenchmarkTabs();
  initRespectsAndTechnoSoundboard();
});

// ==============================================================================
// 1. Minecraft 26.3 Joining Server Intro Sequence (Full vs 2s Repeat Visit)
// ==============================================================================

function initMinecraftIntroSequence() {
  const overlay = document.getElementById('mc-intro-overlay');
  const cursor = document.getElementById('mc-virtual-cursor');
  const replayBtn = document.getElementById('replay-intro-btn');
  if (!overlay) return;

  // Start rendering the blurred autumn Minecraft panorama background immediately
  let panoramaActive = true;
  startPanoramaRenderer(() => panoramaActive);

  const KEY_SHORT_COUNT = 'sawbyte_intro_short_visits';
  const KEY_LAST_TS = 'sawbyte_intro_last_ts';
  const LONG_WAIT_MS = 30 * 60 * 1000; // 30 minutes

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
    const targetX = rect.left + rect.width / 2 + offsetX;
    const targetY = rect.top + rect.height / 2 + offsetY;
    cursor.style.left = `${targetX}px`;
    cursor.style.top = `${targetY}px`;
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

      if (fillEl) {
        fillEl.style.width = `${Math.floor(pct * 100)}%`;
      }

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

    // Reset UI state inside intro screens
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

    // Short 2-second loading screen for repeat visits 2..5
    if (!fullMode) {
      runLoadingTerrainPhase(1850, finishIntro);
      return;
    }

    // Full first-time sequence: Title -> Multiplayer -> Add Server -> Paste URL & Name -> Join Server -> Loading Terrain
    showIntroScreen('intro-screen-title');
    if (cursor) {
      cursor.style.display = 'block';
      cursor.style.left = `${Math.floor(window.innerWidth * 0.72)}px`;
      cursor.style.top = `${Math.floor(window.innerHeight * 0.64)}px`;
    }

    const mpBtn = document.getElementById('mc-btn-multiplayer');
    const addServerBtn = document.getElementById('mc-btn-add-server');
    const addDoneBtn = document.getElementById('mc-btn-add-done');

    // Step 1: Move cursor to Multiplayer and click
    setTimeout(() => {
      if (mpBtn) mpBtn.classList.add('hovered');
      moveCursorToElement(mpBtn);
    }, 320);

    setTimeout(() => {
      clickButtonElement(mpBtn, () => {
        // Step 2: Open Play Multiplayer screen and move to Add Server
        showIntroScreen('intro-screen-multiplayer');
        setTimeout(() => {
          if (addServerBtn) addServerBtn.classList.add('hovered');
          moveCursorToElement(addServerBtn);
        }, 240);

        setTimeout(() => {
          clickButtonElement(addServerBtn, () => {
            // Step 3: Open Edit Server Info, click Server Address, paste URL
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

            // Step 4: Move up to Server Name and set "Sawbyte"
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

            // Step 5: Move down to Done and click
            setTimeout(() => {
              if (addDoneBtn) addDoneBtn.classList.add('hovered');
              moveCursorToElement(addDoneBtn);
            }, 1480);

            setTimeout(() => {
              clickButtonElement(addDoneBtn, () => {
                // Step 6: Return to Play Multiplayer with Sawbyte added
                if (emptyMsg) emptyMsg.style.display = 'none';
                if (serverRow) serverRow.style.display = 'flex';
                showIntroScreen('intro-screen-multiplayer');

                setTimeout(() => {
                  moveCursorToElement(serverRow);
                }, 200);

                // Select Sawbyte server row
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

                // Move to Join Server and click
                setTimeout(() => {
                  if (joinBtn) joinBtn.classList.add('hovered');
                  moveCursorToElement(joinBtn);
                }, 760);

                setTimeout(() => {
                  clickButtonElement(joinBtn, () => {
                    // Step 7: Show Loading terrain... screen and reveal page
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

  function resize() {
    canvas.width = 480;
    canvas.height = 270;
  }
  resize();

  function draw(t) {
    if (!isActiveFn()) return;
    const w = canvas.width;
    const h = canvas.height;
    const pan = (t * 0.012) % 160;

    // Sky (Matches image_80219e.png & image_80303f.png bright blue sky)
    const skyGrad = ctx.createLinearGradient(0, 0, 0, h * 0.52);
    skyGrad.addColorStop(0, '#5478c4');
    skyGrad.addColorStop(1, '#9bb2db');
    ctx.fillStyle = skyGrad;
    ctx.fillRect(0, 0, w, h * 0.55);

    // Blocky Sun
    ctx.fillStyle = '#fff9d6';
    ctx.fillRect(w * 0.54 - pan * 0.15, 16, 36, 32);

    // Blocky Clouds
    ctx.fillStyle = 'rgba(235, 240, 250, 0.78)';
    for (let i = -1; i < 5; i++) {
      const cx = ((i * 130 - pan * 0.4) % (w + 140)) - 40;
      ctx.fillRect(cx, 26 + (i % 2) * 18, 68, 14);
      ctx.fillRect(cx + 14, 18 + (i % 2) * 18, 42, 10);
    }

    // Distant Hills & River Valley
    ctx.fillStyle = '#485c3a';
    ctx.fillRect(0, h * 0.46, w, h * 0.18);
    ctx.fillStyle = '#526c7c';
    ctx.fillRect(w * 0.28 - pan * 0.2, h * 0.5, 95, 22);

    // Foreground Autumn Podzol / Red-Orange Terrain (Matches uploaded screenshots)
    const groundGrad = ctx.createLinearGradient(0, h * 0.52, 0, h);
    groundGrad.addColorStop(0, '#783a16');
    groundGrad.addColorStop(1, '#54260c');
    ctx.fillStyle = groundGrad;
    ctx.fillRect(0, h * 0.55, w, h * 0.45);

    // Golden & Crimson Autumn Voxel Trees (Left & Right)
    ctx.fillStyle = '#6b1d14';
    ctx.fillRect(28 - pan * 0.25, h * 0.28, 76, 52);
    ctx.fillStyle = '#b57b1e';
    ctx.fillRect(10 - pan * 0.3, h * 0.54, 130, 85);

    ctx.fillStyle = '#334d24';
    ctx.fillRect(w - 110 - pan * 0.2, h * 0.14, 115, 76);
    ctx.fillStyle = '#3d2817';
    ctx.fillRect(w - 42 - pan * 0.2, h * 0.42, 24, 95);

    requestAnimationFrame(draw);
  }
  requestAnimationFrame(draw);
}

// ==============================================================================
// 2. Official Minecraft Font & GitHub Texture / Sound Asset Resolver
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
    if (!urls.includes(rawUrl)) urls.push(rawUrl);
    if (!urls.includes(cdnUrl)) urls.push(cdnUrl);
  }
  if (clean.includes('compass_16.png')) {
    urls.push('https://raw.githubusercontent.com/InventivetalentDev/minecraft-assets/1.21.4/assets/minecraft/textures/item/recovery_compass_16.png');
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
  fetch('https://api.github.com/repos/InventivetalentDev/minecraft-assets/branches?per_page=100')
    .then((r) => (r.ok ? r.json() : null))
    .then((branches) => {
      if (Array.isArray(branches) && branches.length > 0) {
        const names = branches.map((b) => b.name);
        for (const candidate of ['26.3', '1.21.10', '1.21.5', '1.21.4', '1.21.1', '1.21', '1.20.4']) {
          if (names.includes(candidate)) {
            activeMcAssetBranch = candidate;
            break;
          }
        }
      }
    })
    .catch(() => {})
    .finally(() => {
      hydrateAllMinecraftTextures();
    });

  hydrateAllMinecraftTextures();

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
// 3. Official Minecraft Audio Engine (GitHub OGG Streamer + WebAudio Fallback)
// ==============================================================================

function getSoundUrls(soundRelativePath) {
  const clean = soundRelativePath.replace(/^\/+/, '');
  const fullKey = `minecraft/sounds/${clean}`;
  const urls = [];

  for (const br of [activeMcAssetBranch, ...MC_ASSET_BRANCH_CANDIDATES]) {
    urls.push(`https://raw.githubusercontent.com/InventivetalentDev/minecraft-assets/${br}/assets/minecraft/sounds/${clean}`);
    urls.push(`https://cdn.jsdelivr.net/gh/InventivetalentDev/minecraft-assets@${br}/assets/minecraft/sounds/${clean}`);
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

// ==============================================================================
// 4. Persistent Pigstep Background Music Jukebox (Starts at Low 18% Volume)
// ==============================================================================

function initPigstepJukebox() {
  bgMusicAudio = new Audio();
  bgMusicAudio.crossOrigin = 'anonymous';
  bgMusicAudio.loop = true;
  bgMusicAudio.volume = bgMusicVolume;

  const playBtn = document.getElementById('jukebox-play-btn');
  const toggleBtn = document.getElementById('jukebox-toggle-btn');
  const nextBtn = document.getElementById('jukebox-next-btn');
  const volSlider = document.getElementById('jukebox-volume');
  const heroPigstepBtn = document.getElementById('hero-pigstep-btn');

  loadDiscIntoPlayer(0, true);

  const unlockAutoplay = () => {
    if (!bgMusicPlaying && !userPausedMusic) {
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

  if (heroPigstepBtn) {
    heroPigstepBtn.addEventListener('click', () => {
      currentDiscIndex = 0;
      userPausedMusic = false;
      loadDiscIntoPlayer(0, true);
      showAdvancementToast(
        'Now Playing: Pigstep',
        'Lena Raine • Background Jukebox Active',
        'item/music_disc_pigstep.png'
      );
    });
  }

  if (nextBtn) {
    nextBtn.addEventListener('click', () => {
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
      if (autoPlay && !userPausedMusic) {
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
  const heroLabel = document.getElementById('hero-pigstep-label');

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
  if (heroLabel) {
    heroLabel.textContent = bgMusicPlaying && currentDiscIndex === 0
      ? `Pigstep Playing (${volPct}%)`
      : 'Play Pigstep';
  }
}

// ==============================================================================
// 5. Advancement Toasts, Lore Tooltips & Particle Canvas
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
      const x = Math.min(window.innerWidth - 320, e.clientX + 16);
      const y = Math.min(window.innerHeight - 110, e.clientY + 16);
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
  const COUNT = Math.min(85, Math.floor((width * height) / 15000));
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

// ==============================================================================
// 6. Auto-Fetched GitHub Releases & Repo Metadata
// ==============================================================================

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

// ==============================================================================
// 7. Interactive Console & §0-§f Color Code Parser
// ==============================================================================

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
        appendFormattedLine('  §binspect.milestones             §7: Show saved survival milestones');
        appendFormattedLine('  §bsound.pigstep                  §7: Play Lena Raine Pigstep disc');
        appendFormattedLine('  §bsound.pig                      §7: Play Minecraft pig sound');
        appendFormattedLine('  §btechnoblade                    §7: Pay respects [o7]');
        appendFormattedLine('  §binventory.find <item>          §7: Search 45 inventory slots');
        appendFormattedLine('  §btask.navigate <x> <y> <z>      §7: Walk to XYZ coordinates');
        appendFormattedLine('  §bcolors                         §7: Show all §0 to §f color codes');
        appendFormattedLine('  §bclear                          §7: Clear console');
        break;
      case 'colors':
        appendFormattedLine('§0§0Black §1§1DarkBlue §2§2DarkGreen §3§3DarkAqua §4§4DarkRed §5§5Purple §6§6Gold §7§7Gray');
        appendFormattedLine('§8§8DarkGray §9§9Blue §a§aGreen §b§bAqua §c§cRed §d§dLightPurple §e§eYellow §f§fWhite');
        break;
      case 'inspect.status':
        appendFormattedLine(`§apos=(8.50, 64.00, 8.50) §emc=${TARGET_MC_VERSION} §chealth=20.0 §6food=20 §btick=2630 tps=20.00`);
        break;
      case 'inspect.milestones':
        appendFormattedLine('§6Saved milestones (4):');
        appendFormattedLine('  §a[OK] [Spawned in World] §7tick=40 pos=(8.5, 64.0, 8.5)');
        appendFormattedLine('  §a[OK] [Crafted Wooden Pickaxe] §7tick=320 pos=(12.0, 64.0, 9.2)');
        appendFormattedLine('  §a[OK] [Smelted Iron Ingot] §7tick=840 pos=(18.4, 52.0, 4.0)');
        appendFormattedLine('  §b[OK] [Mined Diamond] §7tick=1420 pos=(22.0, -54.0, 11.0)');
        break;
      case 'sound.pigstep':
        currentDiscIndex = 0;
        userPausedMusic = false;
        loadDiscIntoPlayer(0, true);
        appendFormattedLine('§d[Jukebox] §fPlaying §6Lena Raine: Pigstep');
        break;
      case 'sound.pig':
        playMcPigSound();
        appendFormattedLine('§d[Audio] §fPlayed §emob/pig/say.ogg');
        break;
      case 'technoblade':
        playMcTotemSound();
        appendFormattedLine('§6[Crown] §e“Technoblade Never Dies!” §c[o7]');
        break;
      case 'inventory.find':
        const q = parts[1] || 'diamond_pickaxe';
        appendFormattedLine(`§b{"ok": true, "slot": 36, "id": "minecraft:${q}", "count": 1}`);
        break;
      case 'task.navigate':
        appendFormattedLine(`§dWalking to (${parts[1] || 120}, ${parts[2] || 64}, ${parts[3] || -30}) via 3D A* path...`);
        break;
      case 'clear':
        body.innerHTML = '';
        break;
      default:
        appendFormattedLine(`§cUnknown command '${cmd}'. Type §ehelp §cfor a list of commands.`);
    }
  }
}

// ==============================================================================
// 8. Benchmark Tabs & Technoblade Soundboard + Tribute Button
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
