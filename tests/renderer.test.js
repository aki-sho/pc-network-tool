import assert from "node:assert/strict";
import test from "node:test";

test("network reload suppresses duplicates and recovers safely after failure", async (t) => {
  class Element {
    innerHTML = "";
    textContent = "";
    disabled = false;
    dataset = {};
    hidden = true;
    listeners = {};
    addEventListener(name, listener) { this.listeners[name] = listener; }
    querySelector() { return null; }
  }

  const elements = new Map(
    [
      "summaryRoot", "adaptersRoot", "ipDnsRoot", "portsRoot", "connectionSummaryRoot",
      "connectionListCount", "reloadButton", "refreshStatus", "refreshError", "appVersion"
    ]
      .map((id) => [id, new Element()])
  );
  const requests = [];
  globalThis.document = {
    getElementById: (id) => elements.get(id),
    querySelectorAll: () => []
  };
  globalThis.window = {
    addEventListener() {},
    __TAURI__: {
      app: { async getVersion() { return "1.1.0"; } },
      core: { invoke(command) {
        assert.equal(command, "get_network_info");
        return new Promise((resolve, reject) => requests.push({ resolve, reject }));
      } }
    }
  };
  t.mock.method(console, "error", () => {});
  t.after(() => {
    delete globalThis.document;
    delete globalThis.window;
  });
  await import("../frontend/renderer/renderer.js");
  const button = elements.get("reloadButton");
  assert.equal(requests.length, 1);
  assert.equal(button.disabled, true);
  await button.listeners.click();
  await button.listeners.click();
  assert.equal(requests.length, 1);

  const message = '<img src=x onerror="alert(1)"> timeout';
  requests[0].reject(message);
  await new Promise(setImmediate);
  const error = elements.get("refreshError");
  assert.equal(error.textContent, message);
  assert.equal(error.innerHTML, "");
  assert.equal(error.hidden, false);
  assert.equal(elements.get("refreshStatus").dataset.state, "error");
  assert.equal(button.disabled, false);
  assert.equal(elements.get("appVersion").textContent, "v1.1.0");

  const retry = button.listeners.click();
  assert.equal(requests.length, 2);
  assert.equal(button.disabled, true);
  requests[1].resolve({
    summary: {
      publicIp: "203.0.113.1", defaultAdapter: "<Ethernet>", adapterCount: 1, connectionCount: 123,
      establishedCount: 103, listeningCount: 20
    },
    adapters: [{ name: "<Ethernet>", isDefault: true, status: "Up", ipv4: "192.0.2.10" }],
    connections: Array.from({ length: 100 }, () => ({ protocol: "tcp", state: "Established" }))
  });
  await retry;
  assert.equal(button.disabled, false);
  assert.equal(error.hidden, true);
  assert.equal(elements.get("refreshStatus").dataset.state, "ready");
  const snapshotMarkup = elements.get("summaryRoot").innerHTML;
  assert.match(snapshotMarkup, /203\.0\.113\.1/);
  assert.match(elements.get("adaptersRoot").innerHTML, /&lt;Ethernet&gt;（既定）/);
  assert.match(elements.get("ipDnsRoot").innerHTML, /192\.0\.2\.10/);
  assert.equal(elements.get("connectionListCount").textContent, "123 件中 100 件を表示");

  const refresh = button.listeners.click();
  assert.equal(elements.get("summaryRoot").innerHTML, snapshotMarkup);
  requests[2].reject(message);
  await refresh;
  assert.equal(elements.get("summaryRoot").innerHTML, snapshotMarkup);
  assert.match(error.textContent, /前回の結果を表示しています/);
});
