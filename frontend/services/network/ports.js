export function formatPortsTable(connections) {
  return {
    headers: ["プロトコル", "ローカル IP", "ローカルポート", "リモート IP", "リモートポート", "状態", "PID", "プロセス"],
    rows: connections.map((item) => [
      item.protocol,
      item.localAddress,
      item.localPort,
      item.remoteAddress,
      item.remotePort,
      item.state,
      item.pid,
      item.process
    ])
  };
}
