import assert from "node:assert/strict";
import test from "node:test";
import { renderAdaptersPage } from "../renderer/pages/adapters.js";
import { renderIpDnsPage } from "../renderer/pages/ip-dns.js";
import { renderSummaryPage } from "../renderer/pages/summary.js";

test("adapter tables explain each connection using Windows metadata", () => {
  const adapters = [
    { name: "ローカル エリア接続* 73", interfaceDescription: "WAN Miniport (IKEv2)", interfaceType: 131, virtualAdapter: true, status: "Not Present", dhcp: "Unknown" },
    { name: "Teredo Tunneling Pseudo-Interface", interfaceType: 131, virtualAdapter: true },
    { name: "<custom adapter>", interfaceType: 6, virtualAdapter: true }
  ];

  for (const render of [renderAdaptersPage, renderIpDnsPage]) {
    const root = { innerHTML: "" };
    render(root, adapters);
    assert.match(root.innerHTML, /ローカル エリア接続\* 73<span[^>]+>IKEv2方式のVPN接続/);
    assert.match(root.innerHTML, /Teredo Tunneling Pseudo-Interface<span[^>]+>ルーターのNAT/);
    assert.match(root.innerHTML, /&lt;custom adapter&gt;<span[^>]+>ソフトウェアが作成した仮想/);
    assert.doesNotMatch(root.innerHTML, /<custom adapter>/);
  }
});

test("summary explains the displayed counts and adapter selection", () => {
  const root = { innerHTML: "" };
  renderSummaryPage(root, { publicIp: "203.0.113.1", defaultAdapter: "Ethernet", adapterCount: 14, connectionCount: 100 });
  assert.equal((root.innerHTML.match(/info-card-description/g) ?? []).length, 4);
  assert.match(root.innerHTML, /IPv4の既定ゲートウェイがある接続のうち、一覧で最初/);
  assert.match(root.innerHTML, /TCPの接続済みと待受の合計。最大100件/);
});

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
