export function formatAdaptersTable(adapters) {
  return {
    headers: ["名前", "状態", "IPv4", "IPv6", "MAC", "DHCP"],
    rows: adapters.map((item) => [
      item.isDefault ? `${item.name}（既定）` : item.name,
      item.status,
      item.ipv4 || "-",
      item.ipv6 || "-",
      item.mac || "-",
      item.dhcp || "-"
    ])
  };
}
