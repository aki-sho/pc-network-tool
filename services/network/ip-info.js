export function formatIpDnsTable(adapters) {
  return {
    headers: ["名前", "IPv4", "IPv6", "ゲートウェイ", "DNS"],
    rows: adapters.map((item) => [
      item.name,
      item.ipv4 || "-",
      item.ipv6 || "-",
      item.gateway || "-",
      item.dns || "-"
    ])
  };
}