import { renderSummaryPage } from "./pages/summary.js";
import { renderAdaptersPage } from "./pages/adapters.js";
import { renderIpDnsPage } from "./pages/ip-dns.js";
import { renderPortsPage } from "./pages/ports.js";
import { renderAboutPage } from "./pages/about.js";

const { invoke } = window.__TAURI__.core;

const summaryRoot = document.getElementById("summaryRoot");
const adaptersRoot = document.getElementById("adaptersRoot");
const ipDnsRoot = document.getElementById("ipDnsRoot");
const portsRoot = document.getElementById("portsRoot");
const aboutRoot = document.getElementById("aboutRoot");
const reloadButton = document.getElementById("reloadButton");

function setLoading() {
  summaryRoot.innerHTML = `<p>読み込み中...</p>`;
  adaptersRoot.innerHTML = "";
  ipDnsRoot.innerHTML = "";
  portsRoot.innerHTML = "";
  aboutRoot.innerHTML = "";
}

function setError(message) {
  summaryRoot.innerHTML = `<p class="error">${message}</p>`;
}

async function loadAll() {
  try {
    setLoading();

    const data = await invoke("get_network_info");

    renderSummaryPage(summaryRoot, data.summary);
    renderAdaptersPage(adaptersRoot, data.adapters);
    renderIpDnsPage(ipDnsRoot, data.adapters);
    renderPortsPage(portsRoot, data.connections);
    renderAboutPage(aboutRoot);
  } catch (error) {
    console.error(error);
    setError("取得に失敗しました。");
  }
}

reloadButton.addEventListener("click", loadAll);

loadAll();