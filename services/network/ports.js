export function formatPortsTable(connections) {
  return {
    headers: ["Protocol", "Local Address", "Local Port", "Remote Address", "Remote Port", "State", "PID", "Process"],
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