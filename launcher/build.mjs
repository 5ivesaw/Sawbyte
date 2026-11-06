#!/usr/bin/env node
/**
 * Build & Packaging Script for Sawbyte Desktop Launcher.
 * Zero XMCL dependencies — fast, clean cross-platform Electron packaging.
 */
import fs from 'fs';
import path from 'path';
import { fileURLToPath } from 'url';
import { execSync } from 'child_process';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const distDir = path.join(__dirname, 'dist');
const rendererSrc = path.join(__dirname, 'renderer');
const rendererDist = path.join(distDir, 'renderer');

const args = process.argv.slice(2);

function copyRenderer() {
  fs.mkdirSync(rendererDist, { recursive: true });
  for (const f of fs.readdirSync(rendererSrc)) {
    fs.copyFileSync(path.join(rendererSrc, f), path.join(rendererDist, f));
  }
  const rootIcon = path.resolve(__dirname, '..', 'assets', 'sawbyte.png');
  if (fs.existsSync(rootIcon)) {
    fs.mkdirSync(path.join(__dirname, 'assets'), { recursive: true });
    fs.copyFileSync(rootIcon, path.join(__dirname, 'assets', 'sawbyte.png'));
    fs.copyFileSync(rootIcon, path.join(rendererDist, 'sawbyte.png'));
  }
  console.log('[Sawbyte Launcher] Built renderer assets into dist/renderer');
}

function packageElectron() {
  copyRenderer();
  const builderConfig = {
    appId: 'dev.fivesaw.sawbyte.launcher',
    productName: 'Sawbyte Launcher',
    executableName: 'sawbyte-launcher',
    directories: {
      output: 'dist',
      app: '.'
    },
    files: [
      'main/**/*',
      'renderer/**/*',
      'assets/**/*',
      'package.json'
    ],
    linux: {
      executableName: 'sawbyte-launcher',
      category: 'Game'
    },
    win: {
      executableName: 'sawbyte-launcher'
    },
    mac: {
      category: 'public.app-category.games'
    }
  };
  const cfgPath = path.join(__dirname, 'electron-builder.json');
  fs.writeFileSync(cfgPath, JSON.stringify(builderConfig, null, 2));
  try {
    execSync('npx electron-builder --dir --config electron-builder.json', {
      cwd: __dirname,
      stdio: 'inherit'
    });
  } finally {
    if (fs.existsSync(cfgPath)) fs.unlinkSync(cfgPath);
  }

  // Mirror unpacked output to legacy xmcl-electron-app/dist path so any existing CI script finds it
  const legacyDist = path.join(__dirname, 'xmcl-electron-app', 'dist');
  fs.mkdirSync(legacyDist, { recursive: true });
  for (const sub of ['linux-unpacked', 'win-unpacked', 'mac', 'mac-arm64']) {
    const src = path.join(distDir, sub);
    if (fs.existsSync(src)) {
      const dstName = sub === 'mac-arm64' ? 'mac' : sub;
      fs.cpSync(src, path.join(legacyDist, dstName), { recursive: true });
    }
  }
  console.log('[Sawbyte Launcher] Electron unpacked build completed.');
}

if (args.includes('--renderer') || args.includes('--compile')) {
  copyRenderer();
} else if (args.includes('--package')) {
  packageElectron();
} else {
  copyRenderer();
}
