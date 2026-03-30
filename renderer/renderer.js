const summaryGrid = document.getElementById('summaryGrid');
const adapterTableBody = document.getElementById('adapterTableBody');
const connectionTableBody = document.getElementById('connectionTableBody');
const reloadButton = document.getElementById('reloadButton');

function escapeHtml(value) {
  return String(value ?? '-')
    .replaceAll('&', '&amp;')
    .replaceAll('<', '&lt;')
    .replaceAll('>', '&gt;')
    .replaceAll('"', '&quot;')
    .replaceAll("'", '&#039;');
}

function renderSummary(summary) {
  const items = [
    ['パブリックIP', summary.publicIp],
    ['既定アダプタ', summary.defaultInterface],
    ['アダプタ数', summary.adapterCount],
    ['有効アダプタ数', summary.activeAdapterCount],
    ['接続中セッション数', summary.activeConnectionCount],
    ['待受ポート数', summary.listeningPortCount]
  ];

  summaryGrid.innerHTML = items
    .map(([label, value]) => {
      return `
        <div class="summary-item">
          <div class="summary-label">${escapeHtml(label)}</div>
          <div class="summary-value">${escapeHtml(value)}</div>
        </div>
      `;
    })
    .join('');
}

function renderAdapters(adapters) {
  adapterTableBody.innerHTML = adapters
    .map((item) => {
      return `
        <tr>
          <td>${item.default ? 'Yes' : '-'}</td>
          <td title="${escapeHtml(item.displayName)}">${escapeHtml(item.name)}</td>
          <td>${escapeHtml(item.operstate)}</td>
          <td>${escapeHtml(item.type)}</td>
          <td>${escapeHtml(item.ipv4)}</td>
          <td>${escapeHtml(item.ipv6)}</td>
          <td>${escapeHtml(item.gateway)}</td>
          <td>${escapeHtml(item.dns)}</td>
          <td>${escapeHtml(item.mac)}</td>
          <td>${escapeHtml(item.dhcp)}</td>
          <td>${escapeHtml(item.virtual)}</td>
        </tr>
      `;
    })
    .join('');
}

function renderConnections(connections) {
  connectionTableBody.innerHTML = connections
    .map((item) => {
      return `
        <tr>
          <td>${escapeHtml(item.protocol)}</td>
          <td>${escapeHtml(item.localAddress)}</td>
          <td>${escapeHtml(item.localPort)}</td>
          <td>${escapeHtml(item.peerAddress)}</td>
          <td>${escapeHtml(item.peerPort)}</td>
          <td>${escapeHtml(item.state)}</td>
          <td>${escapeHtml(item.pid)}</td>
          <td>${escapeHtml(item.process)}</td>
        </tr>
      `;
    })
    .join('');
}

async function loadData() {
  summaryGrid.innerHTML = '<div class="loading">読み込み中...</div>';
  adapterTableBody.innerHTML = '';
  connectionTableBody.innerHTML = '';

  try {
    const data = await window.networkApi.getAll();
    renderSummary(data.summary);
    renderAdapters(data.adapters);
    renderConnections(data.connections);
  } catch (error) {
    summaryGrid.innerHTML = `<div class="error">取得に失敗しました: ${escapeHtml(error.message)}</div>`;
  }
}

reloadButton.addEventListener('click', loadData);

loadData();