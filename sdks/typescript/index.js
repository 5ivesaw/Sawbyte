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
