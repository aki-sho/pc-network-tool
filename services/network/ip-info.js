import { adapterNameCell } from "./adapter-description.js";

export function formatIpDnsTable(adapters) {
  return {
    headers: [
      { text: "名前", description: "Windows上の接続名と用途" },
      { text: "IPv4", description: "この接続で使うIPv4アドレス" },
      { text: "IPv6", description: "この接続で使うIPv6アドレス" },
      { text: "ゲートウェイ", description: "ほかのネットワークへ送るときの既定の中継先（IPv4）" },
      { text: "DNS", description: "ドメイン名をIPアドレスに変換するサーバー" }
    ],
    rows: adapters.map((item) => [
      adapterNameCell(item),
      item.ipv4 || "-",
      item.ipv6 || "-",
      item.gateway || "-",
      item.dns || "-"
    ])
  };
}
