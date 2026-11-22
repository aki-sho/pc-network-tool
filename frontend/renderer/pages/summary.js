import { createCardGrid } from "../components/card.js";

export function renderSummaryPage(root, summary = {}) {
  const items = [
    { label: "パブリック IP", value: summary.publicIp, icon: "globe", note: "インターネットから見える IP" },
    { label: "既定アダプター", value: summary.defaultAdapter, icon: "plug", note: "IPv4 の既定ルート" },
    { label: "アダプター数", value: summary.adapterCount, icon: "layers", note: "仮想・未接続を含む" },
    { label: "TCP セッション数", value: summary.connectionCount, icon: "connections", note: "確立済みと待受中の合計" }
  ];

  root.innerHTML = createCardGrid(items);
}
