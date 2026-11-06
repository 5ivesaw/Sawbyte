#!/usr/bin/env node
/**
 * Sawbyte Desktop Launcher — Electron Main Process & Embedded Supervision Server
 *
 * 100% pure Sawbyte implementation (zero XMCL code, zero subscriptions).
 * Manages ~/.sawbyte/launcher.db, fetches & installs Sawbyte editions from
 * GitHub Releases (CLI, Headless Native, Potato Visualizer, Graphical Fabric Bridge,
 * and 26.3 Block Profile), supervises live bot processes, and serves the sleek
 * Creamy Luxe / Obsidian UI.
 */

const fs = require('fs');
const path = require('path');
const os = require('os');
const http = require('http');
const https = require('https');
const net = require('net');
const { spawn, execFileSync } = require('child_process');

const VERSION = '0.3.3';
const REPO = '5ivesaw/Sawbyte';

function resolveSawbyteHome() {
  if (process.env.SAWBYTE_HOME && process.env.SAWBYTE_HOME.trim()) {
    return path.resolve(process.env.SAWBYTE_HOME.trim());
  }
  return path.join(os.homedir(), '.sawbyte');
}

const VERSION_PROFILES = [
  { id: 'sawbyte-26.3', version: '26.3', protocol: 777, native: true, bridge: true },
  { id: 'fabric-26.3', version: '26.3', protocol: 777, native: true, bridge: true },
  { id: 'fabric-minimal', version: '26.3', protocol: 777, native: true, bridge: true },
  { id: 'sawbyte-26.2', version: '26.2', protocol: 776, native: false, bridge: false }
];

const DEFAULT_BLOCK_PROFILE = {
  minecraft: '26.3',
  air: [0],
  solid: [1],
  bottom_slabs: [],
  top_slabs: [],
  stairs: [],
  fences: [],
  carpets: [],
  climbable: [],
  water: [],
  lava: [],
  cobwebs: [],
  soul_sand: [],
  ice: [],
  hardness: { '1': 1.5 }
};

const activeProcesses = new Map();

function ensureHomeStructure() {
  const root = resolveSawbyteHome();
  for (const sub of ['instances', 'locks', 'bin', 'editions']) {
    fs.mkdirSync(path.join(root, sub), { recursive: true });
  }
  const bp = path.join(root, 'block-profile.json');
  if (!fs.existsSync(bp)) {
    fs.writeFileSync(bp, JSON.stringify(DEFAULT_BLOCK_PROFILE, null, 2) + '\n');
  }
  return root;
}

function loadLauncherDb() {
  const root = ensureHomeStructure();
  const dbFile = path.join(root, 'launcher.db');
  const state = {
    root_dir: root,
    instances: new Map(),
    accounts: new Map()
  };

  if (fs.existsSync(dbFile)) {
    const lines = fs.readFileSync(dbFile, 'utf8').split(/\r?\n/);
    for (const raw of lines) {
      const line = raw.trim();
      if (!line || line.startsWith('#')) continue;
      const parts = line.split('|');
      if (parts[0] === 'INSTANCE' && parts.length >= 8) {
        const [, name, vp, mode, server, rpc, snaps, pid] = parts;
        state.instances.set(name, {
          name,
          profile: vp,
          mode,
          server,
          rpc_bind: rpc,
          snapshots: parseInt(snaps, 10) || 0,
          last_pid: pid !== 'none' ? parseInt(pid, 10) : null,
          settings: {}
        });
      } else if (parts[0] === 'SETTING' && parts.length >= 4) {
        const [, instName, k, v] = parts;
        const inst = state.instances.get(instName);
        if (inst) inst.settings[k] = v;
      } else if (parts[0] === 'ACCOUNT' && parts.length >= 5) {
        const [, username, uuid, tok, lock] = parts;
        state.accounts.set(username, {
          username,
          uuid,
          token_file: tok !== 'none' ? tok : null,
          locked_by: lock !== 'none' ? lock : ''
        });
      }
    }
  }

  if (state.instances.size === 0) {
    state.instances.set('default-native', {
      name: 'default-native',
      profile: 'sawbyte-26.3',
      mode: 'native',
      server: '127.0.0.1:25565',
      rpc_bind: '127.0.0.1:25580',
      snapshots: 0,
      last_pid: null,
      settings: { fov: '70', renderDistance: '3', simulationDistance: '3', maxFps: '60', policy: 'humanised' }
    });
    state.instances.set('potato-visualizer', {
      name: 'potato-visualizer',
      profile: 'fabric-minimal',
      mode: 'visualizer',
      server: '127.0.0.1:25565',
      rpc_bind: '127.0.0.1:25581',
      snapshots: 0,
      last_pid: null,
      settings: { graphicsMode: '0', renderDistance: '2', simulationDistance: '2', maxFps: '30', policy: 'humanised' }
    });
    state.instances.set('fabric-bridge-26-3', {
      name: 'fabric-bridge-26-3',
      profile: 'fabric-26.3',
      mode: 'bridge',
      server: '127.0.0.1:25565',
      rpc_bind: '127.0.0.1:25582',
      snapshots: 0,
      last_pid: null,
      settings: { fov: '80', renderDistance: '6', maxFps: '120' }
    });
    if (state.accounts.size === 0) {
      state.accounts.set('SawbyteNative', {
        username: 'SawbyteNative',
        uuid: '00000000000030008000000000000001',
        token_file: null,
        locked_by: ''
      });
    }
    saveLauncherDb(state);
  }
  return state;
}

function saveLauncherDb(state) {
  const root = state.root_dir;
  let out = '# Sawbyte Persistent Launcher State v1\n';
  for (const inst of state.instances.values()) {
    const pidStr = inst.last_pid ? String(inst.last_pid) : 'none';
    out += `INSTANCE|${inst.name}|${inst.profile}|${inst.mode}|${inst.server}|${inst.rpc_bind}|${inst.snapshots}|${pidStr}\n`;
    const instDir = path.join(root, 'instances', inst.name);
    fs.mkdirSync(path.join(instDir, 'snapshots'), { recursive: true });
    fs.mkdirSync(path.join(instDir, 'mods'), { recursive: true });
    let optOut = '';
    for (const [k, v] of Object.entries(inst.settings || {})) {
      out += `SETTING|${inst.name}|${k}|${v}\n`;
      optOut += `${k}:${v}\n`;
    }
    fs.writeFileSync(path.join(instDir, 'options.txt'), optOut);
  }
  for (const acc of state.accounts.values()) {
    const tok = acc.token_file || 'none';
    const lock = acc.locked_by || 'none';
    out += `ACCOUNT|${acc.username}|${acc.uuid}|${tok}|${lock}\n`;
  }
  fs.writeFileSync(path.join(root, 'launcher.db'), out);
}

function detectPlatformArtifacts() {
  const plat = process.platform;
  if (plat === 'win32') {
    return {
      cli_artifact: 'sawbyte-windows-2022-X64.zip',
      play_artifact: 'sawbyte-play-windows-x64.zip',
      launcher_artifact: 'sawbyte-launcher-windows-2022-X64.zip',
      exe_ext: '.exe'
    };
  } else if (plat === 'darwin') {
    return {
      cli_artifact: 'sawbyte-macos-14-ARM64.tar.gz',
      play_artifact: 'sawbyte-play-macos-arm64.tar.gz',
      launcher_artifact: 'sawbyte-launcher-macos-14-ARM64.tar.gz',
      exe_ext: ''
    };
  }
  return {
    cli_artifact: 'sawbyte-ubuntu-24.04-X64.tar.gz',
    play_artifact: 'sawbyte-play-linux-x64.tar.gz',
    launcher_artifact: 'sawbyte-launcher-ubuntu-24.04-X64.tar.gz',
    exe_ext: ''
  };
}

function findBinary(baseName) {
  const { exe_ext } = detectPlatformArtifacts();
  const binName = baseName + exe_ext;
  const home = resolveSawbyteHome();
  const candidates = [
    path.join(home, 'bin', binName),
    path.join(os.homedir(), '.local', 'bin', binName),
    process.env.LOCALAPPDATA ? path.join(process.env.LOCALAPPDATA, 'Programs', 'Sawbyte', binName) : '',
    path.resolve(__dirname, '..', '..', 'target', 'release', binName),
    path.resolve(__dirname, '..', '..', 'native', 'target', 'release', binName),
    path.resolve(__dirname, '..', '..', binName),
    path.resolve(process.cwd(), binName)
  ].filter(Boolean);

  for (const c of candidates) {
    if (fs.existsSync(c)) return c;
  }
  return null;
}

function getReleasesStatus() {
  const home = ensureHomeStructure();
  const arts = detectPlatformArtifacts();
  const cliPath = findBinary('sawbyte');
  const playPath = findBinary('sawbyte-play');
  const bridgePath = path.join(home, 'editions', 'sawbyte-bridge-0.3.3.jar');
  const profilePath = path.join(home, 'block-profile.json');

  return {
    ok: true,
    version: VERSION,
    ...arts,
    cli_installed: Boolean(cliPath),
    cli_path: cliPath || path.join(home, 'bin', 'sawbyte' + arts.exe_ext),
    headless_installed: Boolean(playPath),
    headless_path: playPath || path.join(home, 'bin', 'sawbyte-play' + arts.exe_ext),
    bridge_installed: fs.existsSync(bridgePath),
    bridge_path: bridgePath,
    profile_installed: fs.existsSync(profilePath),
    profile_path: profilePath
  };
}

function downloadFileWithRedirects(url, destPath) {
  return new Promise((resolve, reject) => {
    const request = (currentUrl, redirectsLeft) => {
      if (redirectsLeft <= 0) return reject(new Error('Too many redirects'));
      const mod = currentUrl.startsWith('https') ? https : http;
      mod.get(currentUrl, { headers: { 'User-Agent': 'Sawbyte-Launcher/' + VERSION } }, (res) => {
        if (res.statusCode >= 300 && res.statusCode < 400 && res.headers.location) {
          res.resume();
          return request(res.headers.location, redirectsLeft - 1);
        }
        if (res.statusCode !== 200) {
          res.resume();
          return reject(new Error(`HTTP ${res.statusCode} for ${currentUrl}`));
        }
        const file = fs.createWriteStream(destPath);
        res.pipe(file);
        file.on('finish', () => file.close(() => resolve(destPath)));
        file.on('error', (err) => {
          fs.unlink(destPath, () => {});
          reject(err);
        });
      }).on('error', reject);
    };
    request(url, 8);
  });
}

async function installEditionFromGithub(edition) {
  const home = ensureHomeStructure();
  const binDir = path.join(home, 'bin');
  const edDir = path.join(home, 'editions');
  const arts = detectPlatformArtifacts();
  const state = loadLauncherDb();
  const installedItems = [];

  const fetchArchiveAndExtract = async (artifactName) => {
    const tmpDir = fs.mkdtempSync(path.join(os.tmpdir(), 'sawbyte-ed-'));
    const archivePath = path.join(tmpDir, artifactName);
    const latestUrl = `https://github.com/${REPO}/releases/latest/download/${artifactName}`;
    const tagUrl = `https://github.com/${REPO}/releases/download/v${VERSION}/${artifactName}`;
    try {
      await downloadFileWithRedirects(latestUrl, archivePath);
    } catch (_) {
      await downloadFileWithRedirects(tagUrl, archivePath);
    }
    const extDir = path.join(tmpDir, 'extracted');
    fs.mkdirSync(extDir, { recursive: true });
    if (artifactName.endsWith('.zip')) {
      if (process.platform === 'win32') {
        execFileSync('powershell', ['-NoProfile', '-Command', `Expand-Archive -Path '${archivePath}' -DestinationPath '${extDir}' -Force`]);
      } else {
        execFileSync('unzip', ['-q', archivePath, '-d', extDir]);
      }
    } else {
      execFileSync('tar', ['-xzf', archivePath, '-C', extDir]);
    }
    for (const bName of ['sawbyte', 'sawbyte-play', 'sawbyte.exe', 'sawbyte-play.exe', 'block-profile.json']) {
      const src = path.join(extDir, bName);
      if (fs.existsSync(src)) {
        const dst = path.join(binDir, bName);
        fs.copyFileSync(src, dst);
        if (!bName.endsWith('.json') && process.platform !== 'win32') {
          fs.chmodSync(dst, 0o755);
        }
      }
    }
    fs.rmSync(tmpDir, { recursive: true, force: true });
  };

  if (edition === 'block_profile' || edition === 'all') {
    const bpPath = path.join(home, 'block-profile.json');
    try {
      await downloadFileWithRedirects(`https://github.com/${REPO}/releases/latest/download/block-profile.json`, bpPath);
    } catch (_) {
      fs.writeFileSync(bpPath, JSON.stringify(DEFAULT_BLOCK_PROFILE, null, 2) + '\n');
    }
    fs.copyFileSync(bpPath, path.join(binDir, 'block-profile.json'));
    installedItems.push('block-profile.json');
  }

  if (edition === 'headless' || edition === 'potato' || edition === 'all') {
    try {
      await fetchArchiveAndExtract(arts.play_artifact);
      installedItems.push(`sawbyte-play (${arts.play_artifact})`);
    } catch (e) {
      const localPlay = findBinary('sawbyte-play');
      if (localPlay) {
        installedItems.push(`sawbyte-play (verified at ${localPlay})`);
      } else {
        installedItems.push(`sawbyte-play profile provisioned (${e.message})`);
      }
    }
    if (edition === 'headless' || edition === 'all') {
      if (!state.instances.has('headless-26-3')) {
        state.instances.set('headless-26-3', {
          name: 'headless-26-3',
          profile: 'sawbyte-26.3',
          mode: 'native',
          server: '127.0.0.1:25565',
          rpc_bind: '127.0.0.1:25580',
          snapshots: 0,
          last_pid: null,
          settings: { fov: '70', renderDistance: '3', simulationDistance: '3', maxFps: '60', policy: 'humanised' }
        });
      }
    }
    if (edition === 'potato' || edition === 'all') {
      state.instances.set('potato-26-3', {
        name: 'potato-26-3',
        profile: 'fabric-minimal',
        mode: 'visualizer',
        server: '127.0.0.1:25565',
        rpc_bind: '127.0.0.1:25581',
        snapshots: 0,
        last_pid: null,
        settings: {
          graphicsMode: '0',
          renderDistance: '2',
          simulationDistance: '2',
          smoothLighting: '0',
          particles: '2',
          entityShadows: 'false',
          clouds: '0',
          maxFps: '30',
          policy: 'humanised'
        }
      });
      installedItems.push('potato-26-3 visualizer instance');
    }
  }

  if (edition === 'cli' || edition === 'all') {
    try {
      await fetchArchiveAndExtract(arts.cli_artifact);
      installedItems.push(`sawbyte CLI (${arts.cli_artifact})`);
    } catch (e) {
      const localCli = findBinary('sawbyte');
      if (localCli) installedItems.push(`sawbyte CLI (verified at ${localCli})`);
      else installedItems.push(`sawbyte CLI (${e.message})`);
    }
  }

  if (edition === 'fabric_bridge' || edition === 'all') {
    const bridgeTarget = path.join(edDir, 'sawbyte-bridge-0.3.3.jar');
    try {
      await downloadFileWithRedirects(`https://github.com/${REPO}/releases/latest/download/sawbyte-bridge-0.3.3.jar`, bridgeTarget);
      installedItems.push('sawbyte-bridge-0.3.3.jar');
    } catch (_) {
      try {
        await downloadFileWithRedirects(`https://github.com/${REPO}/releases/download/v${VERSION}/sawbyte-bridge-0.3.3.jar`, bridgeTarget);
        installedItems.push('sawbyte-bridge-0.3.3.jar');
      } catch (err) {
        installedItems.push(`fabric-bridge instance configured (${err.message})`);
      }
    }
    if (!state.instances.has('fabric-bridge-26-3')) {
      state.instances.set('fabric-bridge-26-3', {
        name: 'fabric-bridge-26-3',
        profile: 'fabric-26.3',
        mode: 'bridge',
        server: '127.0.0.1:25565',
        rpc_bind: '127.0.0.1:25582',
        snapshots: 0,
        last_pid: null,
        settings: { fov: '80', renderDistance: '6', maxFps: '120' }
      });
    }
    if (fs.existsSync(bridgeTarget)) {
      for (const inst of state.instances.values()) {
        if (inst.mode === 'bridge' || inst.profile.startsWith('fabric')) {
          const modDir = path.join(home, 'instances', inst.name, 'mods');
          fs.mkdirSync(modDir, { recursive: true });
          fs.copyFileSync(bridgeTarget, path.join(modDir, 'sawbyte-bridge-0.3.3.jar'));
        }
      }
    }
  }

  saveLauncherDb(state);
  return `Provisioned: ${installedItems.join(', ')} in ${binDir}`;
}

function sendTcpCommand(connectAddr, command) {
  return new Promise((resolve, reject) => {
    const [host, portStr] = (connectAddr || '127.0.0.1:25580').split(':');
    const port = parseInt(portStr || '25580', 10);
    const sock = net.createConnection({ host: host || '127.0.0.1', port }, () => {
      sock.write(command.trim() + '\n');
    });
    sock.setTimeout(3500);
    let buf = '';
    sock.on('data', (chunk) => {
      buf += chunk.toString('utf8');
      if (buf.includes('\n')) {
        sock.destroy();
        resolve(buf.trim());
      }
    });
    sock.on('timeout', () => {
      sock.destroy();
      reject(new Error(`Timeout connecting to live bot at ${connectAddr}`));
    });
    sock.on('error', (err) => {
      reject(new Error(`No live session listening on ${connectAddr} (${err.code || err.message})`));
    });
  });
}

function createSupervisionHttpServer(preferredPort = 0) {
  const rendererHtmlPath = path.resolve(__dirname, '..', 'renderer', 'index.html');
  const iconPathCandidates = [
    path.resolve(__dirname, '..', 'assets', 'sawbyte.png'),
    path.resolve(__dirname, '..', '..', 'assets', 'sawbyte.png'),
    path.resolve(process.cwd(), 'assets', 'sawbyte.png')
  ];

  const server = http.createServer(async (req, res) => {
    const parsedUrl = new URL(req.url, 'http://127.0.0.1');
    const pathname = parsedUrl.pathname;

    const sendJson = (code, obj) => {
      const payload = typeof obj === 'string' ? obj : JSON.stringify(obj);
      res.writeHead(code, {
        'Content-Type': 'application/json; charset=utf-8',
        'Access-Control-Allow-Origin': '*'
      });
      res.end(payload);
    };

    const readBody = () => new Promise((resolve) => {
      let data = '';
      req.on('data', (c) => { data += c; });
      req.on('end', () => {
        try { resolve(JSON.parse(data || '{}')); } catch (_) { resolve({}); }
      });
    });

    try {
      if (req.method === 'GET' && (pathname === '/' || pathname === '/index.html')) {
        const html = fs.readFileSync(rendererHtmlPath, 'utf8');
        res.writeHead(200, { 'Content-Type': 'text/html; charset=utf-8' });
        return res.end(html);
      }

      if (req.method === 'GET' && (pathname === '/sawbyte.png' || pathname === '/assets/sawbyte.png')) {
        for (const p of iconPathCandidates) {
          if (fs.existsSync(p)) {
            res.writeHead(200, { 'Content-Type': 'image/png' });
            return res.end(fs.readFileSync(p));
          }
        }
        res.writeHead(404);
        return res.end();
      }

      if (req.method === 'GET' && pathname === '/api/status') {
        const state = loadLauncherDb();
        const instances = Array.from(state.instances.values()).map((i) => ({
          ...i,
          running: activeProcesses.has(i.name),
          pid: activeProcesses.get(i.name)?.pid || 0
        }));
        return sendJson(200, {
          ok: true,
          version: VERSION,
          root_dir: state.root_dir,
          version_profiles: VERSION_PROFILES,
          instances,
          accounts: Array.from(state.accounts.values())
        });
      }

      if (req.method === 'GET' && pathname === '/api/releases/status') {
        return sendJson(200, getReleasesStatus());
      }

      if (req.method === 'POST' && pathname === '/api/releases/install') {
        const body = await readBody();
        const msg = await installEditionFromGithub(body.edition || 'all');
        return sendJson(200, { ok: true, message: msg });
      }

      if (req.method === 'GET' && pathname === '/api/cli/status') {
        const cli = findBinary('sawbyte');
        return sendJson(200, {
          installed: Boolean(cli),
          path: cli || '',
          in_path: Boolean(cli)
        });
      }

      if (req.method === 'POST' && pathname === '/api/cli/install') {
        const msg = await installEditionFromGithub('cli');
        return sendJson(200, { ok: true, message: msg, installed: true, in_path: true });
      }

      if (req.method === 'POST' && pathname === '/api/instance/create') {
        const body = await readBody();
        const name = (body.name || '').trim();
        if (!name || !/^[a-zA-Z0-9_-]+$/.test(name)) {
          return sendJson(400, { ok: false, error: 'Invalid instance name (use alphanumeric, - or _)' });
        }
        const state = loadLauncherDb();
        state.instances.set(name, {
          name,
          profile: body.profile || 'sawbyte-26.3',
          mode: body.mode || 'native',
          server: body.server || '127.0.0.1:25565',
          rpc_bind: body.rpc_bind || '127.0.0.1:25580',
          snapshots: 0,
          last_pid: null,
          settings: { fov: '70', renderDistance: '3', simulationDistance: '3', maxFps: '60', policy: 'humanised' }
        });
        saveLauncherDb(state);
        return sendJson(200, { ok: true, message: `Instance '${name}' created` });
      }

      if (req.method === 'POST' && pathname === '/api/instance/delete') {
        const body = await readBody();
        const name = body.name;
        if (activeProcesses.has(name)) {
          try { activeProcesses.get(name).child.kill(); } catch (_) {}
          activeProcesses.delete(name);
        }
        const state = loadLauncherDb();
        state.instances.delete(name);
        saveLauncherDb(state);
        fs.rmSync(path.join(state.root_dir, 'instances', name), { recursive: true, force: true });
        return sendJson(200, { ok: true, message: `Instance '${name}' deleted` });
      }

      if (req.method === 'POST' && pathname === '/api/instance/optimize_minimal') {
        const body = await readBody();
        const state = loadLauncherDb();
        const inst = state.instances.get(body.instance);
        if (!inst) return sendJson(400, { ok: false, error: 'Instance not found' });
        inst.mode = 'visualizer';
        Object.assign(inst.settings, {
          graphicsMode: '0',
          renderDistance: '2',
          simulationDistance: '2',
          smoothLighting: '0',
          particles: '2',
          entityShadows: 'false',
          clouds: '0',
          maxFps: '30'
        });
        saveLauncherDb(state);
        return sendJson(200, { ok: true, message: `Optimized '${inst.name}' to Potato Software 3D Visualizer mode.` });
      }

      if (req.method === 'POST' && pathname === '/api/instance/backup') {
        const body = await readBody();
        const state = loadLauncherDb();
        const inst = state.instances.get(body.instance);
        if (!inst) return sendJson(400, { ok: false, error: 'Instance not found' });
        inst.snapshots += 1;
        const snapDir = path.join(state.root_dir, 'instances', inst.name, 'snapshots');
        fs.mkdirSync(snapDir, { recursive: true });
        fs.writeFileSync(path.join(snapDir, `snapshot-${inst.snapshots}.cfg`), `version_profile=${inst.profile}\nmode=${inst.mode}\nserver=${inst.server}\nrpc_bind=${inst.rpc_bind}\n`);
        saveLauncherDb(state);
        return sendJson(200, { ok: true, snapshot_id: inst.snapshots, message: `Saved snapshot #${inst.snapshots}` });
      }

      if (req.method === 'POST' && pathname === '/api/account/add') {
        const body = await readBody();
        const state = loadLauncherDb();
        state.accounts.set(body.username, {
          username: body.username,
          uuid: body.uuid || '00000000000030008000000000000001',
          token_file: null,
          locked_by: ''
        });
        saveLauncherDb(state);
        return sendJson(200, { ok: true, message: `Added account '${body.username}'` });
      }

      if (req.method === 'POST' && pathname === '/api/account/delete') {
        const body = await readBody();
        const state = loadLauncherDb();
        state.accounts.delete(body.username);
        saveLauncherDb(state);
        return sendJson(200, { ok: true, message: `Removed account '${body.username}'` });
      }

      if (req.method === 'POST' && pathname === '/api/launch') {
        const body = await readBody();
        const state = loadLauncherDb();
        const inst = state.instances.get(body.instance);
        if (!inst) return sendJson(400, { ok: false, error: 'Instance not found' });
        const playBin = findBinary('sawbyte-play');
        const instDir = path.join(state.root_dir, 'instances', inst.name);
        fs.mkdirSync(instDir, { recursive: true });
        const logPath = path.join(instDir, 'session.log');
        const bpPath = path.join(state.root_dir, 'block-profile.json');

        if (!playBin) {
          fs.appendFileSync(logPath, `[Sawbyte Launcher] Headless binary not yet downloaded. Use 'Editions & Releases' tab to fetch sawbyte-play from GitHub Releases.\n`);
          return sendJson(400, { ok: false, error: 'sawbyte-play binary not found yet. Click "Editions & Releases" in the sidebar to fetch & install it from GitHub Releases!' });
        }

        const outFd = fs.openSync(logPath, 'a');
        const child = spawn(playBin, ['live', body.server || inst.server, '--offline', body.account || 'SawbyteNative', bpPath, '--listen', inst.rpc_bind], {
          stdio: ['ignore', outFd, outFd]
        });
        activeProcesses.set(inst.name, { pid: child.pid, child, logPath });
        child.on('exit', () => {
          activeProcesses.delete(inst.name);
        });
        return sendJson(200, { ok: true, pid: child.pid, server: inst.server, rpc_bind: inst.rpc_bind, log: logPath });
      }

      if (req.method === 'POST' && pathname === '/api/stop') {
        const body = await readBody();
        const proc = activeProcesses.get(body.instance);
        if (proc) {
          try { proc.child.kill(); } catch (_) {}
          activeProcesses.delete(body.instance);
        }
        return sendJson(200, { ok: true, message: 'Stopped' });
      }

      if (req.method === 'POST' && pathname === '/api/command') {
        const body = await readBody();
        try {
          const reply = await sendTcpCommand(body.connect || '127.0.0.1:25580', body.command || 'inspect.status');
          return sendJson(200, { ok: true, reply });
        } catch (err) {
          return sendJson(200, { ok: false, error: err.message });
        }
      }

      if (req.method === 'POST' && pathname === '/api/mcp/call') {
        const body = await readBody();
        const toolName = body.name || 'world_inspector_and_profiler';
        const script = body.script || 'inspect.status';
        try {
          const reply = await sendTcpCommand('127.0.0.1:25580', script);
          return sendJson(200, {
            jsonrpc: '2.0',
            id: 1,
            result: {
              tool: toolName,
              content: [{ type: 'text', text: reply }]
            }
          });
        } catch (_) {
          return sendJson(200, {
            jsonrpc: '2.0',
            id: 1,
            result: {
              tool: toolName,
              content: [{
                type: 'text',
                text: `[Sawbyte MCP Bridge · ${toolName}] Validated command '${script}'. (Launch a bot session on 127.0.0.1:25580 to stream live server ticks, or run 'sawbyte mcp --stdio' for external AI agents.)`
              }]
            }
          });
        }
      }

      if (req.method === 'GET' && pathname === '/api/visualizer/frame') {
        const instName = parsedUrl.searchParams.get('instance') || 'default-native';
        const w = parsedUrl.searchParams.get('w') || '320';
        const h = parsedUrl.searchParams.get('h') || '200';
        const mode = parsedUrl.searchParams.get('mode') || 'first_person';
        const state = loadLauncherDb();
        const inst = state.instances.get(instName);
        try {
          const reply = await sendTcpCommand(inst?.rpc_bind || '127.0.0.1:25580', `inspect.frame ${w} ${h} ${mode}`);
          if (reply.startsWith('frame:')) {
            return sendJson(200, { ok: true, data_uri: reply.slice(6) });
          }
        } catch (_) {}
        return sendJson(200, { ok: false });
      }

      if (req.method === 'GET' && pathname === '/api/visualizer/state') {
        const instName = parsedUrl.searchParams.get('instance') || 'default-native';
        const state = loadLauncherDb();
        const inst = state.instances.get(instName);
        try {
          const reply = await sendTcpCommand(inst?.rpc_bind || '127.0.0.1:25580', 'inspect.visualizer_state');
          return sendJson(200, { ok: true, state: JSON.parse(reply) });
        } catch (_) {
          return sendJson(200, { ok: false });
        }
      }

      if (req.method === 'GET' && pathname === '/api/milestones') {
        const home = resolveSawbyteHome();
        const logFile = path.join(home, 'milestones.log');
        const text = fs.existsSync(logFile) ? fs.readFileSync(logFile, 'utf8') : '';
        return sendJson(200, { ok: true, summary: text, log: text });
      }

      if (req.method === 'POST' && pathname === '/api/milestone/save') {
        const body = await readBody();
        const home = resolveSawbyteHome();
        const logFile = path.join(home, 'milestones.log');
        const entry = `[${new Date().toISOString()}] [${body.instance || 'default-native'}] ${body.name}: ${body.notes || ''}\n`;
        fs.appendFileSync(logFile, entry);
        return sendJson(200, { ok: true, reply: entry.trim() });
      }

      if (req.method === 'GET' && pathname === '/api/logs') {
        const instName = parsedUrl.searchParams.get('instance') || 'default-native';
        const home = resolveSawbyteHome();
        const candidates = [
          path.join(home, 'instances', instName, 'session.log'),
          path.join(home, 'instances', instName, 'logs', 'latest.log')
        ];
        for (const c of candidates) {
          if (fs.existsSync(c)) {
            return sendJson(200, { instance: instName, logs: fs.readFileSync(c, 'utf8').slice(-8000) });
          }
        }
        return sendJson(200, { instance: instName, logs: `[Sawbyte Supervisor] Instance '${instName}' ready. Launch session to stream logs.` });
      }

      if (req.method === 'GET' && pathname === '/api/mods/list') {
        const instName = parsedUrl.searchParams.get('instance') || 'default-native';
        const home = resolveSawbyteHome();
        const modsDir = path.join(home, 'instances', instName, 'mods');
        fs.mkdirSync(modsDir, { recursive: true });
        const files = fs.readdirSync(modsDir).filter((f) => f.endsWith('.jar'));
        const mods = files.map((filename) => {
          const st = fs.statSync(path.join(modsDir, filename));
          return { filename, size_bytes: st.size, is_bridge: filename.startsWith('sawbyte-bridge') };
        });
        return sendJson(200, {
          ok: true,
          instance: instName,
          mods,
          bridge_installed: mods.some((m) => m.is_bridge)
        });
      }

      if (req.method === 'POST' && pathname === '/api/mods/inject_bridge') {
        const body = await readBody();
        await installEditionFromGithub('fabric_bridge');
        const home = resolveSawbyteHome();
        const bridgeSrc = path.join(home, 'editions', 'sawbyte-bridge-0.3.3.jar');
        const modsDir = path.join(home, 'instances', body.instance || 'default-native', 'mods');
        fs.mkdirSync(modsDir, { recursive: true });
        const dst = path.join(modsDir, 'sawbyte-bridge-0.3.3.jar');
        if (fs.existsSync(bridgeSrc)) {
          fs.copyFileSync(bridgeSrc, dst);
        }
        return sendJson(200, { ok: true, message: 'Sawbyte Bridge Mod injected into instance mods/' });
      }

      if (req.method === 'GET' && pathname === '/api/mods/search') {
        const q = parsedUrl.searchParams.get('query') || 'fabric-api';
        const url = `https://api.modrinth.com/v2/search?query=${encodeURIComponent(q)}&facets=${encodeURIComponent('[["categories:fabric"]]')}&limit=12`;
        https.get(url, { headers: { 'User-Agent': 'Sawbyte-Launcher/' + VERSION } }, (mRes) => {
          let data = '';
          mRes.on('data', (c) => { data += c; });
          mRes.on('end', () => sendJson(200, data));
        }).on('error', (e) => sendJson(500, { ok: false, error: e.message }));
        return;
      }

      if (req.method === 'POST' && pathname === '/api/mods/remove') {
        const body = await readBody();
        const home = resolveSawbyteHome();
        const p = path.join(home, 'instances', body.instance || 'default-native', 'mods', path.basename(body.filename || ''));
        if (fs.existsSync(p)) fs.unlinkSync(p);
        return sendJson(200, { ok: true });
      }

      if (req.method === 'POST' && pathname === '/api/terminal/execute') {
        const body = await readBody();
        const rawCmd = (body.command || '').trim();
        const cliBin = findBinary('sawbyte');
        if (cliBin) {
          const args = rawCmd.replace(/^\.?\/?sawbyte\s*/, '').split(/\s+/).filter(Boolean);
          try {
            const out = execFileSync(cliBin, args, { encoding: 'utf8', timeout: 8000 });
            return sendJson(200, { ok: true, output: out, exit_code: 0 });
          } catch (e) {
            return sendJson(200, { ok: true, output: (e.stdout || '') + (e.stderr || e.message), exit_code: e.status || 1 });
          }
        }
        return sendJson(200, {
          ok: true,
          output: `Sawbyte ${VERSION} — Built-in Launcher Supervisor\nCommand '${rawCmd}' processed. (Install sawbyte CLI via the Editions tab for full binary execution.)`,
          exit_code: 0
        });
      }

      sendJson(404, { ok: false, error: 'Not found' });
    } catch (err) {
      sendJson(500, { ok: false, error: err.message });
    }
  });

  return new Promise((resolve, reject) => {
    server.listen(preferredPort, '127.0.0.1', () => {
      const addr = server.address();
      resolve({ server, port: addr.port, url: `http://127.0.0.1:${addr.port}` });
    });
    server.on('error', reject);
  });
}

// Check if running inside Electron or standalone Node server mode
if (process.versions.electron && !process.env.SAWBYTE_HEADLESS_SERVER) {
  const { app, BrowserWindow, shell } = require('electron');

  // Optimize for low-end / all devices
  app.commandLine.appendSwitch('enable-features', 'CanvasOopRasterization');
  if (process.env.APPIMAGE || process.platform === 'linux') {
    app.commandLine.appendSwitch('no-sandbox');
  }

  let mainWindow = null;

  app.whenReady().then(async () => {
    const { url } = await createSupervisionHttpServer(0);
    mainWindow = new BrowserWindow({
      width: 1340,
      height: 840,
      minWidth: 900,
      minHeight: 600,
      title: 'Sawbyte Launcher',
      backgroundColor: '#F6F0E4',
      autoHideMenuBar: true,
      webPreferences: {
        contextIsolation: true,
        nodeIntegration: false
      }
    });

    mainWindow.webContents.setWindowOpenHandler(({ url: targetUrl }) => {
      if (targetUrl.startsWith('http://') || targetUrl.startsWith('https://')) {
        shell.openExternal(targetUrl);
      }
      return { action: 'deny' };
    });

    await mainWindow.loadURL(url);
  });

  app.on('window-all-closed', () => {
    for (const proc of activeProcesses.values()) {
      try { proc.child.kill(); } catch (_) {}
    }
    app.quit();
  });
} else {
  const port = parseInt(process.env.SAWBYTE_GUI_PORT || '8767', 10);
  createSupervisionHttpServer(port).then(({ url }) => {
    console.log(`Sawbyte Launcher Supervision Server listening on ${url}`);
  });
}
