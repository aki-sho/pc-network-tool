import { createTable } from "../components/table.js";
import { createDetailList } from "../components/markup.js";
import { formatIpDnsTable } from "../../services/network/ip-info.js";

export function renderIpDnsPage(root, adapters) {
  const adapter = adapters.find((item) => item.isDefault);
  const overview = adapter ? createDetailList([
    ["IPv4", adapter.ipv4],
    ["IPv6", adapter.ipv6],
    ["ゲートウェイ", adapter.gateway],
    ["DNS", adapter.dns]
  ]) : '<p class="empty-state">IPv4 の既定アダプターを取得できませんでした。</p>';

  const detailsOpen = root.querySelector("details")?.open;
  const { headers, rows } = formatIpDnsTable(adapters);
  const details = rows.length ? `
    <details class="adapter-details"${detailsOpen ? " open" : ""}>
      <summary>すべてのアダプターの IP / DNS</summary>
      ${createTable(headers, rows, { label: "アダプター別の IP と DNS" })}
    </details>
  ` : "";
  root.innerHTML = overview + details;
}
