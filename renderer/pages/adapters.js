import { createTable } from "../components/table.js";
import { formatAdaptersTable } from "../../services/network/adapters.js";

export function renderAdaptersPage(root, adapters) {
  const { headers, rows } = formatAdaptersTable(adapters);
  root.innerHTML = createTable(headers, rows);
}