import { createTable } from "../components/table.js";
import { formatAdaptersTable } from "../../services/network/adapters.js";

export function renderAdaptersPage(root, adapters) {
  const ordered = [...adapters].sort((a, b) => Number(b.isDefault) - Number(a.isDefault));
  const { headers, rows } = formatAdaptersTable(ordered);
  root.innerHTML = createTable(headers, rows, {
    label: "ネットワークアダプター",
    emptyMessage: "アダプターが見つかりませんでした。",
    rowClass(index) {
      const adapter = ordered[index];
      return [adapter.isDefault && "is-default", adapter.status === "Up" && "is-up"]
        .filter(Boolean)
        .join(" ");
    }
  });
}
