const { app, BrowserWindow, ipcMain } = require('electron');
const path = require('path');
const si = require('systeminformation');
const https = require('https');
const { exec } = require('child_process');

function createWindow() {
  const win = new BrowserWindow({
    width: 1280,
    height: 900,
    minWidth: 1000,
    minHeight: 700,
    webPreferences: {
      preload: path.join(__dirname, 'preload.js'),
      contextIsolation: true,
      nodeIntegration: false
    }
  });

  win.loadFile('index.html');
}

app.whenReady().then(() => {
  createWindow();

  ipcMain.handle('network:get-all', async () => {
    return await getNetworkInfo();
  });

  app.on('activate', () => {
    if (BrowserWindow.getAllWindows().length === 0) createWindow();
  });
});

app.on('window-all-closed', () => {
  if (process.platform !== 'darwin') app.quit();
});

function getPublicIp() {
  return new Promise((resolve) => {
    const req = https.get('https://api.ipify.org?format=json', (res) => {
      let data = '';

      res.on('data', (chunk) => {
        data += chunk;
      });

      res.on('end', () => {
        try {
          const parsed = JSON.parse(data);
          resolve(parsed.ip || '取得失敗');
        } catch {
          resolve('取得失敗');
        }
      });
    });

    req.on('error', () => resolve('取得失敗'));
    req.setTimeout(5000, () => {
      req.destroy();
      resolve('取得失敗');
    });
  });
}

function runPowerShell(command) {
  return new Promise((resolve) => {
    exec(
      `powershell -NoProfile -ExecutionPolicy Bypass -Command "${command}"`,
      { windowsHide: true, maxBuffer: 1024 * 1024 * 10 },
      (error, stdout) => {
        if (error) {
          resolve(null);
          return;
        }

        try {
          resolve(JSON.parse(stdout));
        } catch {
          resolve(null);
        }
      }
    );
  });
}

async function getDhcpInfo() {
  const command = `
    $items = Get-NetIPInterface -AddressFamily IPv4 |
      Select-Object InterfaceAlias, InterfaceIndex, Dhcp;
    $items | ConvertTo-Json -Depth 3
  `;
  return await runPowerShell(command);
}

async function getGatewayDnsInfo() {
  const command = `
    $items = Get-NetIPConfiguration |
      Select-Object InterfaceAlias, InterfaceIndex,
        @{Name='IPv4DefaultGateway';Expression={($_.IPv4DefaultGateway.NextHop)}},
        @{Name='DNSServer';Expression={($_.DNSServer.ServerAddresses -join ', ')}};
    $items | ConvertTo-Json -Depth 4
  `;
  return await runPowerShell(command);
}

function normalizeArray(value) {
  if (!value) return [];
  return Array.isArray(value) ? value : [value];
}

async function getNetworkInfo() {
  const [
    interfaces,
    defaultInterface,
    connections,
    publicIp,
    dhcpInfoRaw,
    gatewayDnsRaw
  ] = await Promise.all([
    si.networkInterfaces(),
    si.networkInterfaceDefault(),
    si.networkConnections(),
    getPublicIp(),
    getDhcpInfo(),
    getGatewayDnsInfo()
  ]);

  const dhcpInfo = normalizeArray(dhcpInfoRaw);
  const gatewayDnsInfo = normalizeArray(gatewayDnsRaw);

  const activeConnections = connections.filter((c) => {
    return c.state === 'ESTABLISHED' || c.state === 'LISTEN';
  });

  const adapters = interfaces.map((item) => {
    const dhcp = dhcpInfo.find((d) =>
      d.InterfaceAlias === item.iface || d.InterfaceIndex === item.iface
    );

    const gatewayDns = gatewayDnsInfo.find((g) =>
      g.InterfaceAlias === item.iface || g.InterfaceIndex === item.iface
    );

    return {
      name: item.iface || '-',
      displayName: item.ifaceName || item.iface || '-',
      type: item.type || '-',
      operstate: item.operstate || '-',
      internal: item.internal ? 'Yes' : 'No',
      virtual: item.virtual ? 'Yes' : 'No',
      default: defaultInterface === item.iface,
      mac: item.mac || '-',
      ipv4: item.ip4 || '-',
      ipv6: item.ip6 || '-',
      gateway: gatewayDns?.IPv4DefaultGateway || item.gateway || '-',
      dns: gatewayDns?.DNSServer || '-',
      dhcp: dhcp?.Dhcp || 'Unknown',
      netmaskV4: item.ip4subnet || '-',
      netmaskV6: item.ip6subnet || '-',
      speed: item.speed ? `${item.speed} Mbps` : '-'
    };
  });

  const summary = {
    publicIp,
    defaultInterface: defaultInterface || '-',
    adapterCount: adapters.length,
    activeAdapterCount: adapters.filter((a) => a.operstate === 'up').length,
    activeConnectionCount: activeConnections.filter((c) => c.state === 'ESTABLISHED').length,
    listeningPortCount: activeConnections.filter((c) => c.state === 'LISTEN').length
  };

  return {
    summary,
    adapters,
    connections: activeConnections.slice(0, 200).map((c) => ({
      protocol: c.protocol || '-',
      localAddress: c.localAddress || '-',
      localPort: c.localPort || '-',
      peerAddress: c.peerAddress || '-',
      peerPort: c.peerPort || '-',
      state: c.state || '-',
      process: c.process || '-',
      pid: c.pid || '-'
    }))
  };
}