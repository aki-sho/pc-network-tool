import assert from "node:assert/strict";
import test from "node:test";

test("network reload suppresses duplicates and recovers safely after failure", async (t) => {
  class Element {
    innerHTML = "";
    textContent = "";
    disabled = false;
    children = [];
    listeners = {};
    addEventListener(name, listener) { this.listeners[name] = listener; }
    replaceChildren(...children) { this.children = children; }
  }

  const elements = new Map(
    ["summaryRoot", "adaptersRoot", "ipDnsRoot", "portsRoot", "aboutRoot", "reloadButton"]
      .map((id) => [id, new Element()])
  );
  const requests = [];
  globalThis.document = {
    getElementById: (id) => elements.get(id),
    createElement: () => new Element()
  };
  globalThis.window = {
    __TAURI__: { core: { invoke(command) {
      assert.equal(command, "get_network_info");
      return new Promise((resolve, reject) => requests.push({ resolve, reject }));
    } } }
  };
  t.mock.method(console, "error", () => {});
  t.after(() => {
    delete globalThis.document;
    delete globalThis.window;
  });
  await import("../renderer/renderer.js");
  const button = elements.get("reloadButton");
  assert.equal(requests.length, 1);
  assert.equal(button.disabled, true);
  await button.listeners.click();
  await button.listeners.click();
  assert.equal(requests.length, 1);

  const message = '<img src=x onerror="alert(1)"> timeout';
  requests[0].reject(message);
  await new Promise(setImmediate);
  assert.equal(elements.get("summaryRoot").children[0].textContent, message);
  assert.equal(elements.get("summaryRoot").children[0].className, "error");
  assert.equal(button.disabled, false);
  assert.equal(button.textContent, "再取得");

  const retry = button.listeners.click();
  assert.equal(requests.length, 2);
  assert.equal(button.disabled, true);
  requests[1].resolve({
    summary: { publicIp: "203.0.113.1", defaultAdapter: "Ethernet", adapterCount: 0, connectionCount: 0 },
    adapters: [], connections: []
  });
  await retry;
  assert.equal(button.disabled, false);
  assert.match(elements.get("summaryRoot").innerHTML, /203\.0\.113\.1/);
});
