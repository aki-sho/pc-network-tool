import { createTable } from "../components/table.js";
import { createDetailList } from "../components/markup.js";
import { formatPortsTable } from "../../services/network/ports.js";

export function renderPortsPage(root, connections) {
  const { headers, rows } = formatPortsTable(connections);
  root.innerHTML = createTable(headers, rows, {
    label: "TCP の確立済み・待受中セッション",
    emptyMessage: "確立済みまたは待受中の TCP セッションはありません。"
  });
}

export function renderConnectionSummary(root, summary) {
  root.innerHTML = createDetailList([
    ["確立済み", `${summary.establishedCount} 件`],
    ["待受中", `${summary.listeningCount} 件`]
  ]);
}
