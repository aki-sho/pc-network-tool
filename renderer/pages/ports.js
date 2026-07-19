import { createTable } from "../components/table.js";
import { formatPortsTable } from "../../services/network/ports.js";

export function renderPortsPage(root, connections) {
  const { headers, rows } = formatPortsTable(connections);
  root.innerHTML = createTable(headers, rows);
}