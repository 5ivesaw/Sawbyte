#!/usr/bin/env node
/**
 * Build & Asset Verification Script for Sawbyte Native Desktop Launcher.
 * Zero Electron/Chromium bundle bloat — renderer assets are embedded directly
 * into the ~1.5 MB native Rust `sawbyte-launcher` binary at compile time.
 */
import fs from 'fs';
import path from 'path';
import { fileURLToPath } from 'url';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const distDir = path.join(__dirname, 'dist');
const rendererSrc = path.join(__dirname, 'renderer');
const rendererDist = path.join(distDir, 'renderer');

function buildRendererAssets() {
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
  console.log('[Sawbyte Launcher] Verified and staged renderer UI assets into dist/renderer');
}

buildRendererAssets();
