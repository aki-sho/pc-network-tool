import { adapterNameCell } from "./adapter-description.js";

const statusDescriptions = new Map([
  ["Up", "アダプタは稼働中"],
  ["Disconnected", "リンク未接続"],
  ["Not Present", "現在使える実体がない"],
  ["Disabled", "無効に設定されている"],
  ["Down", "アダプタは停止中"],
  ["Dormant", "接続成立を待機中"],
  ["Unknown", "状態を判定できない"]
]);

const dhcpDescriptions = new Map([
  ["Enabled", "自動取得が有効"],
  ["Disabled", "自動取得が無効"],
  ["Unknown", "設定を取得できない"]
]);

export function formatAdaptersTable(adapters) {
  return {
    headers: [
      { text: "既定", description: "稼働中でIPv4の既定ゲートウェイがあると Yes" },
      { text: "名前", description: "Windows上の接続名と用途" },
      { text: "状態", description: "接続や稼働の状況" },
      { text: "IPv4", description: "割り当てられたIPv4アドレス" },
      { text: "IPv6", description: "割り当てられたIPv6アドレス" },
      { text: "MAC", description: "同じネットワークで接続口を識別するアドレス" },
      { text: "DHCP", description: "IPv4アドレスの自動取得設定" }
    ],
    rows: adapters.map((item) => [
      item.isDefault ? "Yes" : "-",
      adapterNameCell(item),
      { text: item.status, description: statusDescriptions.get(item.status) },
      item.ipv4 || "-",
      item.ipv6 || "-",
      item.mac || "-",
      { text: item.dhcp || "-", description: dhcpDescriptions.get(item.dhcp) }
    ])
  };
}
