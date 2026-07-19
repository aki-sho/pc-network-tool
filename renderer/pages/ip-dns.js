import { createTable } from "../components/table.js";
import { formatIpDnsTable } from "../../services/network/ip-info.js";

export function renderIpDnsPage(root, adapters) {
  const { headers, rows } = formatIpDnsTable(adapters);
  root.innerHTML = createTable(headers, rows);
}