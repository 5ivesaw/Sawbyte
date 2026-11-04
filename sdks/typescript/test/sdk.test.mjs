import test from "node:test";
import assert from "node:assert/strict";
import { SawbyteWorkflow, createMcpToolCallRequest } from "../index.js";

test("SawbyteWorkflow compiles general navigation, wait, and gui primitives", () => {
  const wf = new SawbyteWorkflow()
    .createInstance("ts-bot", "sawbyte-26.3", "native")
    .gotoMenu("options")
    .setSetting("fov", 90)
    .connectLive("127.0.0.1:25580")
    .navigate(3, -60, 3)
    .waitUntil("screen_open", 100, "Chest")
    .guiClickSlot(0, "pickup")
    .waitTicks(20)
    .guiClose()
    .sendChat("done")
    .observeGym();

  const script = wf.compileScript();
  assert.match(script, /launcher\.create_instance ts-bot/);
  assert.match(script, /task\.navigate 3 -60 3/);
  assert.match(script, /wait\.until screen_open 100 "Chest"/);
  assert.match(script, /gui\.click_slot 0 pickup/);
  assert.match(script, /wait\.ticks 20/);
  assert.match(script, /gui\.close/);
  assert.match(script, /chat\.send done/);
  assert.match(script, /gym\.observe/);
});

test("createMcpToolCallRequest formats standard JSON-RPC 2.0 tool call", () => {
  const req = JSON.parse(createMcpToolCallRequest(42, "script_runner", "wait.ticks 5"));
  assert.equal(req.jsonrpc, "2.0");
  assert.equal(req.id, 42);
  assert.equal(req.method, "tools/call");
  assert.equal(req.params.name, "script_runner");
  assert.equal(req.params.arguments.script, "wait.ticks 5");
});
