export function formatIpDnsTable(adapters) {
  return {
    headers: ["アダプター", "IPv4", "IPv6", "ゲートウェイ", "DNS"],
    rows: adapters.map((item) => [
      item.name,
      item.ipv4 || "-",
      item.ipv6 || "-",
      item.gateway || "-",
      item.dns || "-"
    ])
  };
}
