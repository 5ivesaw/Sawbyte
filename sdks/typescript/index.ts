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

  runShopPurchase(itemId: string, amount = 64): this {
    this.statements.push(`gui.run_shop_workflow ${itemId} ${amount}`);
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
    params: { name: toolName, script },
  });
}
