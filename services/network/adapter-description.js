const interfaceDescriptions = new Map([
  ["bluetooth device (personal area network)", "Bluetooth機器とネットワーク接続を共有します。"],
  ["microsoft kernel debug network adapter", "Windowsのカーネルを別のPCからデバッグするために使います。"],
  ["6to4 adapter", "IPv4ネットワーク上でIPv6通信を運ぶためのトンネルです。"],
  ["teredo tunneling pseudo-interface", "ルーターのNATを越えてIPv6通信を運ぶためのトンネルです。"],
  ["microsoft ip-https platform interface", "DirectAccessなどでIPv6通信をHTTPS経由で運びます。"],
  ["wan miniport (ikev2)", "IKEv2方式のVPN接続に使います。"],
  ["wan miniport (l2tp)", "L2TP方式のVPN接続に使います。"],
  ["wan miniport (sstp)", "HTTPSを利用するSSTP方式のVPN接続に使います。"],
  ["wan miniport (pptp)", "PPTP方式のVPN接続に使います。"],
  ["wan miniport (pppoe)", "PPPoE方式でインターネット回線に接続するために使います。"],
  ["wan miniport (ip)", "VPNなどのリモート接続でIPv4通信を扱います。"],
  ["wan miniport (ipv6)", "VPNなどのリモート接続でIPv6通信を扱います。"],
  ["wan miniport (network monitor)", "リモート接続の通信を監視・診断するために使います。"]
]);

export function adapterNameCell(adapter) {
  const description = adapter.interfaceDescription?.trim() ?? "";
  const known = interfaceDescriptions.get(description.toLowerCase())
    ?? interfaceDescriptions.get(adapter.name?.trim().toLowerCase());

  let purpose = known;
  if (!purpose) {
    switch (adapter.interfaceType) {
      case 6:
        if (!adapter.virtualAdapter) purpose = "LANケーブルを使うイーサネット接続です。";
        break;
      case 23:
        purpose = "PPP方式で接続先との間に通信経路を作ります。";
        break;
      case 24:
        purpose = "同じPCの中で通信するためのループバックです。";
        break;
      case 71:
        purpose = "Wi-Fiでネットワークに接続します。";
        break;
      case 131:
        purpose = "別のネットワーク経由で通信を運ぶトンネルです。";
        break;
    }
  }

  return {
    text: adapter.name,
    description: purpose ?? (adapter.virtualAdapter
      ? "ソフトウェアが作成した仮想ネットワーク接続です。"
      : "Windowsが認識しているネットワーク接続です。")
  };
}
