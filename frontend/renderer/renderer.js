import { renderSummaryPage } from "./pages/summary.js";
import { renderAdaptersPage } from "./pages/adapters.js";
import { renderIpDnsPage } from "./pages/ip-dns.js";
import { renderConnectionSummary, renderPortsPage } from "./pages/ports.js";
import { initNavigation } from "./navigation.js";

const { core, app } = window.__TAURI__ ?? {};

const summaryRoot = document.getElementById("summaryRoot");
const adaptersRoot = document.getElementById("adaptersRoot");
const ipDnsRoot = document.getElementById("ipDnsRoot");
const portsRoot = document.getElementById("portsRoot");
const connectionSummaryRoot = document.getElementById("connectionSummaryRoot");
const connectionListCount = document.getElementById("connectionListCount");
const reloadButton = document.getElementById("reloadButton");
const refreshStatus = document.getElementById("refreshStatus");
const refreshError = document.getElementById("refreshError");
const appVersion = document.getElementById("appVersion");
let loading = false;
let hasSnapshot = false;

function setStatus(state, message) {
  refreshStatus.dataset.state = state;
  refreshStatus.textContent = message;
}

function setError(message) {
  setStatus("error", "取得失敗");
  refreshError.textContent = hasSnapshot
    ? `再取得に失敗しました。前回の結果を表示しています。 ${message}`
    : message;
  refreshError.hidden = false;
}

async function loadAll() {
  if (loading) return;
  loading = true;
  reloadButton.disabled = true;
  refreshError.hidden = true;
  setStatus("loading", "取得中…");
  try {
    const { summary, adapters, connections } = await core.invoke("get_network_info");
    renderSummaryPage(summaryRoot, summary);
    renderAdaptersPage(adaptersRoot, adapters);
    renderIpDnsPage(ipDnsRoot, adapters);
    renderConnectionSummary(connectionSummaryRoot, summary);
    renderPortsPage(portsRoot, connections);
    connectionListCount.textContent = connections.length < summary.connectionCount
      ? `${summary.connectionCount} 件中 ${connections.length} 件を表示`
      : `${connections.length} 件`;
    hasSnapshot = true;
    setStatus("ready", "取得完了");
  } catch (error) {
    console.error(error);
    setError(typeof error === "string" ? error : "取得に失敗しました。再取得してください。");
  } finally {
    loading = false;
    reloadButton.disabled = false;
  }
}

initNavigation();
renderSummaryPage(summaryRoot);
app?.getVersion().then((version) => {
  appVersion.textContent = `v${version}`;
}).catch((error) => console.error(error));

if (core) {
  reloadButton.addEventListener("click", loadAll);
  loadAll();
} else {
  setError("ネットワーク情報の取得には Windows 版の NexaLink Monitor を起動してください。");
}
