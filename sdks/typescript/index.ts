/**
 * Sawbyte TypeScript SDK — Workflow Builder, MCP JSON-RPC 2.0 Client, and Selector Types.
 */

export type PolicyMode = "humanised" | "smooth" | "direct";
export type RuntimeMode = "native" | "bridge";

export interface ItemSelector {
  id?: string;
  nameExact?: string;
  nameContains?: string;
  loreContains?: string;
  minCount?: number;
}

export class SawbyteWorkflow {
  private readonly statements: string[] = [];

  createInstance(name: string, profile = "sawbyte-26.3", mode: RuntimeMode = "native"): this {
    this.statements.push(`launcher.create_instance ${name} ${profile} ${mode}`);
    return this;
  }

  connectLive(endpoint: string): this {
    this.statements.push(`launcher.connect_live ${endpoint}`);
    return this;
  }

  launchInstance(instance: string, server: string, account: string, blocksJsonPath: string): this {
    this.statements.push(`launcher.launch ${instance} ${server} ${account} ${blocksJsonPath}`);
    return this;
  }

  observeGym(): this {
    this.statements.push("gym.observe");
    return this;
  }

  gotoMenu(target: string): this {
    this.statements.push(`menu.goto ${target}`);
    return this;
  }

  setSetting(key: string, value: string | number | boolean): this {
    this.statements.push(`settings.set ${key} ${value}`);
    return this;
  }

  selectHotbar(slot: number): this {
    this.statements.push(`player.select_hotbar ${slot}`);
    return this;
  }

  mineBlock(x: number, y: number, z: number): this {
    this.statements.push(`interact.mine ${x} ${y} ${z}`);
    return this;
  }

  placeBlock(x: number, y: number, z: number, slot = 0): this {
    this.statements.push(`interact.place ${x} ${y} ${z} ${slot}`);
    return this;
  }

  navigate(x: number, y: number, z: number): this {
    this.statements.push(`task.navigate ${x} ${y} ${z}`);
    return this;
  }

  waitTicks(ticks: number): this {
    this.statements.push(`wait.ticks ${ticks}`);
    return this;
  }

  waitUntil(condition: string, timeoutTicks = 100, filterArg?: string): this {
    if (filterArg) {
      this.statements.push(`wait.until ${condition} ${timeoutTicks} "${filterArg}"`);
    } else {
      this.statements.push(`wait.until ${condition} ${timeoutTicks}`);
    }
    return this;
  }

  guiClickSlot(slot: number, mode = "pickup"): this {
    this.statements.push(`gui.click_slot ${slot} ${mode}`);
    return this;
  }

  guiClose(): this {
    this.statements.push("gui.close");
    return this;
  }

  sendChat(message: string): this {
    this.statements.push(`chat.send ${message}`);
    return this;
  }

  runCommand(command: string): this {
    this.statements.push(`cmd.run ${command}`);
    return this;
  }

  spawnSwarm(count: number, mode: "shared" | "isolated" = "shared"): this {
    this.statements.push(`swarm.spawn ${count} ${mode}`);
    return this;
  }

  compileScript(): string {
    return this.statements.join("\n");
  }
}

export function createMcpToolCallRequest(
  id: number,
  toolName: string,
  script: string,
): string {
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
