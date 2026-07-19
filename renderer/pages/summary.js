import { createCardGrid } from "../components/card.js";

export function renderSummaryPage(root, summary) {
  const items = [
    ["パブリックIP", summary.publicIp],
    ["既定アダプタ", summary.defaultAdapter],
    ["アダプタ数", summary.adapterCount],
    ["接続中セッション数", summary.connectionCount]
  ];

  root.innerHTML = createCardGrid(items);
}