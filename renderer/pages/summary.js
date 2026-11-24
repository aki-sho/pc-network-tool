import { createCardGrid } from "../components/card.js";

export function renderSummaryPage(root, summary) {
  const items = [
    ["パブリックIP", summary.publicIp, "外部のIP確認サービスから見えた、このPCの通信元IPアドレス。"],
    ["既定アダプタ", summary.defaultAdapter, "稼働中でIPv4の既定ゲートウェイがある接続のうち、一覧で最初のもの。"],
    ["アダプタ数", summary.adapterCount, "Windowsから取得した接続口の数。仮想・非表示・未接続のものも含みます。"],
    ["接続中セッション数", summary.connectionCount, "一覧に表示したTCPの接続済みと待受の合計。最大100件です。"]
  ];

  root.innerHTML = createCardGrid(items);
}
