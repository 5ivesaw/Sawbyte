/**
 * Sawbyte TypeScript SDK — Workflow Builder, MCP JSON-RPC 2.0 Client, and Selector Types.
 */

export class SawbyteWorkflow {
  constructor() {
    this.statements = [];
  }

  createInstance(name, profile = "sawbyte-26.3", mode = "native") {
    this.statements.push(`launcher.create_instance ${name} ${profile} ${mode}`);
    return this;
  }

  connectLive(endpoint) {
    this.statements.push(`launcher.connect_live ${endpoint}`);
    return this;
  }

  launchInstance(instance, server, account, blocksJsonPath) {
    this.statements.push(`launcher.launch ${instance} ${server} ${account} ${blocksJsonPath}`);
    return this;
  }

  observeGym() {
    this.statements.push("gym.observe");
    return this;
  }

  gotoMenu(target) {
    this.statements.push(`menu.goto ${target}`);
    return this;
  }

  setSetting(key, value) {
    this.statements.push(`settings.set ${key} ${value}`);
    return this;
  }

  selectHotbar(slot) {
    this.statements.push(`player.select_hotbar ${slot}`);
    return this;
  }

  mineBlock(x, y, z) {
    this.statements.push(`interact.mine ${x} ${y} ${z}`);
    return this;
  }

  placeBlock(x, y, z, slot = 0) {
    this.statements.push(`interact.place ${x} ${y} ${z} ${slot}`);
    return this;
  }

  navigate(x, y, z) {
    this.statements.push(`task.navigate ${x} ${y} ${z}`);
    return this;
  }

  waitTicks(ticks) {
    this.statements.push(`wait.ticks ${ticks}`);
    return this;
  }

  waitUntil(condition, timeoutTicks = 100, filterArg) {
    if (filterArg) {
      this.statements.push(`wait.until ${condition} ${timeoutTicks} "${filterArg}"`);
    } else {
      this.statements.push(`wait.until ${condition} ${timeoutTicks}`);
    }
    return this;
  }

  guiClickSlot(slot, mode = "pickup") {
    this.statements.push(`gui.click_slot ${slot} ${mode}`);
    return this;
  }

  guiClose() {
    this.statements.push("gui.close");
    return this;
  }

  sendChat(message) {
    this.statements.push(`chat.send ${message}`);
    return this;
  }

  runCommand(command) {
    this.statements.push(`cmd.run ${command}`);
    return this;
  }

  spawnSwarm(count, mode = "shared") {
    this.statements.push(`swarm.spawn ${count} ${mode}`);
    return this;
  }

  observeSensory() {
    this.statements.push("inspect.sensory");
    return this;
  }

  observeSvpe(rx = 8, ry = 4, rz = 8) {
    this.statements.push(`inspect.svpe ${rx} ${ry} ${rz}`);
    return this;
  }

  observeXray() {
    this.statements.push("inspect.xray");
    return this;
  }

  observeViewport() {
    this.statements.push("inspect.viewport");
    return this;
  }

  saveMemory() {
    this.statements.push("memory.save");
    return this;
  }

  compileScript() {
    return this.statements.join("\n");
  }
}

export function createMcpToolCallRequest(id, toolName, script) {
  return JSON.stringify({
    jsonrpc: "2.0",
    id,
    method: "tools/call",
    params: {
      name: toolName,
      arguments: { script },
    },
  });
}


export function rleDecodeSvpe1Tokens(rleStr) {
  const out = [];
  let countAcc = 0;
  for (const ch of rleStr) {
    if (ch >= "0" && ch <= "9") {
      countAcc = countAcc * 10 + (ch.charCodeAt(0) - 48);
    } else {
      const n = countAcc > 0 ? countAcc : 1;
      for (let i = 0; i < n; i++) out.push(ch);
      countAcc = 0;
    }
  }
  return out;
}

export function decodeSvpe1Rle3d(encoded) {
  const parts = encoded.trim().split("|");
  if (parts.length < 6 || parts[0] !== "SVPE1:RLE3D") {
    throw new Error("Invalid SVPE1:RLE3D header");
  }
  const parseTriple = (s) =>
    s
      .split("=")[1]
      .replace(/[()]/g, "")
      .split(",")
      .map((v) => parseInt(v.trim(), 10));
  const origin = parseTriple(parts[1]);
  const dims = parseTriple(parts[2]);
  const [dx, dy, dz] = dims;
  const sliceLen = dx * dz;
  const rawLayers = parts.slice(5).join("|").split("/");
  const layersYzx = [];
  let prevFlat = [];
  for (const layerStr of rawLayers) {
    const flat = layerStr === "@dup" ? [...prevFlat] : rleDecodeSvpe1Tokens(layerStr);
    if (flat.length !== sliceLen) {
      throw new Error(`Decoded slice length ${flat.length} !== ${sliceLen}`);
    }
    prevFlat = flat;
    const rows = [];
    for (let z = 0; z < dz; z++) {
      rows.push(flat.slice(z * dx, (z + 1) * dx));
    }
    layersYzx.push(rows);
  }
  return { origin, dims, layersYzx };
}
