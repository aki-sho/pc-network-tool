#![cfg_attr(all(windows, not(debug_assertions)), windows_subsystem = "windows")]

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fs;
use std::io::{self, Read, Write};
use std::net::IpAddr;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

mod powershell;

const APP_DIR_NAME: &str = "NexaLink Monitor";
const LEGACY_APP_DIR_NAME: &str = "PC Network Tool";
const PORTABLE_DATA_DIR_NAME: &str = "NexaLink-Monitor-PortableData";
const LEGACY_PORTABLE_DATA_DIR_NAME: &str = "PC-Network-Tool-PortableData";

#[derive(Default)]
struct NetworkRequestState(Arc<AtomicBool>);

struct NetworkRequestGuard(Arc<AtomicBool>);

impl NetworkRequestState {
    fn acquire(&self) -> Result<NetworkRequestGuard, String> {
        self.0
            .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
            .map_err(|_| "情報を取得中です。完了するまでお待ちください。".to_string())?;
        Ok(NetworkRequestGuard(Arc::clone(&self.0)))
    }
}

impl Drop for NetworkRequestGuard {
    fn drop(&mut self) {
        self.0.store(false, Ordering::Release);
    }
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct RuntimePaths {
    portable: bool,
    root: PathBuf,
    user_data: PathBuf,
    session_data: PathBuf,
    config: PathBuf,
    logs: PathBuf,
    cache: PathBuf,
    temp: PathBuf,
}

impl RuntimePaths {
    fn detect() -> io::Result<Self> {
        let executable = std::env::current_exe()?;
        let portable = is_portable_executable(&executable);

        if portable {
            let executable_dir = executable.parent().ok_or_else(|| {
                io::Error::new(
                    io::ErrorKind::NotFound,
                    "executable directory was not found",
                )
            })?;
            let root = data_root(
                executable_dir,
                PORTABLE_DATA_DIR_NAME,
                LEGACY_PORTABLE_DATA_DIR_NAME,
            );
            Ok(Self::from_roots(true, root.clone(), root))
        } else {
            let roaming = data_root(
                &required_environment_path("APPDATA")?,
                APP_DIR_NAME,
                LEGACY_APP_DIR_NAME,
            );
            let local = data_root(
                &required_environment_path("LOCALAPPDATA")?,
                APP_DIR_NAME,
                LEGACY_APP_DIR_NAME,
            );
            Ok(Self::from_roots(false, roaming, local))
        }
    }

    fn from_roots(portable: bool, roaming_root: PathBuf, local_root: PathBuf) -> Self {
        Self {
            portable,
            root: if portable {
                roaming_root.clone()
            } else {
                local_root.clone()
            },
            user_data: roaming_root.join("userData"),
            session_data: local_root.join("sessionData"),
            config: roaming_root.join("config"),
            logs: roaming_root.join("logs"),
            cache: local_root.join("cache"),
            temp: local_root.join("temp"),
        }
    }

    fn prepare(&self) -> io::Result<()> {
        for path in [
            &self.user_data,
            &self.session_data,
            &self.config,
            &self.logs,
            &self.cache,
            &self.temp,
        ] {
            fs::create_dir_all(path)?;
        }

        let settings_path = self.config.join("settings.json");
        match fs::File::create_new(&settings_path) {
            Ok(mut file) => writeln!(
                file,
                "{{\n  \"version\": \"{}\"\n}}",
                env!("CARGO_PKG_VERSION")
            )?,
            Err(error) if error.kind() == io::ErrorKind::AlreadyExists => {}
            Err(error) => return Err(error),
        }

        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        let mode = if self.portable {
            "portable"
        } else {
            "installed"
        };
        let log_line = format!(
            "timestamp={timestamp} mode={mode} root={} userData={} sessionData={} cache={} temp={}\n",
            self.root.display(),
            self.user_data.display(),
            self.session_data.display(),
            self.cache.display(),
            self.temp.display()
        );
        fs::write(self.logs.join("startup.log"), log_line)?;

        // These values are process-local and are set before Tauri/WebView2 is initialized.
        // Child processes inherit them, keeping their temporary files in the same mode.
        std::env::set_var("TEMP", &self.temp);
        std::env::set_var("TMP", &self.temp);
        if self.portable {
            std::env::set_var("APPDATA", &self.user_data);
            std::env::set_var("LOCALAPPDATA", &self.user_data);
            std::env::set_var("WEBVIEW2_USER_DATA_FOLDER", &self.session_data);
        }

        Ok(())
    }
}

fn data_root(parent: &Path, name: &str, legacy_name: &str) -> PathBuf {
    let current = parent.join(name);
    let legacy = parent.join(legacy_name);
    if !current.exists() && legacy.is_dir() {
        return legacy;
    }
    current
}

fn required_environment_path(name: &str) -> io::Result<PathBuf> {
    std::env::var_os(name).map(PathBuf::from).ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::NotFound,
            format!("{name} environment variable is not set"),
        )
    })
}

fn is_portable_executable(path: &Path) -> bool {
    path.file_name()
        .and_then(|name| name.to_str())
        .map(|name| {
            let lowercase = name.to_ascii_lowercase();
            lowercase.ends_with(".exe")
                && (lowercase.starts_with("nexalink-monitor-portable-")
                    || lowercase.starts_with("pc-network-tool-portable-"))
        })
        .unwrap_or(false)
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Summary {
    public_ip: String,
    default_adapter: String,
    adapter_count: usize,
    connection_count: usize,
    established_count: usize,
    listening_count: usize,
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
struct Adapter {
    name: String,
    status: String,
    ipv4: String,
    ipv6: String,
    gateway: String,
    dns: String,
    mac: String,
    dhcp: String,
    is_default: bool,
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct Connection {
    protocol: String,
    local_address: String,
    local_port: String,
    remote_address: String,
    remote_port: String,
    state: String,
    pid: String,
    process: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct ConnectionSnapshot {
    established_count: usize,
    listening_count: usize,
    connections: Vec<Connection>,
}

#[derive(Serialize)]
struct NetworkInfo {
    summary: Summary,
    adapters: Vec<Adapter>,
    connections: Vec<Connection>,
}

fn parse_public_ip(bytes: &[u8]) -> Option<String> {
    let value: Value = serde_json::from_slice(bytes).ok()?;
    value["ip"]
        .as_str()?
        .parse::<IpAddr>()
        .ok()
        .map(|ip| ip.to_string())
}

fn get_public_ip() -> String {
    let fetch = || -> Option<String> {
        let client = reqwest::blocking::Client::builder()
            .connect_timeout(Duration::from_secs(3))
            .timeout(Duration::from_secs(5))
            .redirect(reqwest::redirect::Policy::none())
            .https_only(true)
            .build()
            .ok()?;
        let response = client
            .get("https://api.ipify.org?format=json")
            .send()
            .ok()?
            .error_for_status()
            .ok()?;
        if !response.status().is_success() {
            return None;
        }
        let mut bytes = Vec::new();
        response.take(1025).read_to_end(&mut bytes).ok()?;
        if bytes.len() > 1024 {
            return None;
        }
        parse_public_ip(&bytes)
    };
    fetch().unwrap_or_else(|| "-".to_string())
}

fn get_adapters() -> Result<Vec<Adapter>, String> {
    let script = r#"
$adapters = Get-NetAdapter -IncludeHidden | Sort-Object InterfaceIndex
$configs = Get-NetIPConfiguration -All
$dhcps = Get-NetIPInterface -AddressFamily IPv4
$activeIndexes = @($adapters | Where-Object Status -eq 'Up' | ForEach-Object InterfaceIndex)
$interfaceMetrics = @{}
foreach ($interface in $dhcps) {
  $interfaceMetrics[$interface.InterfaceIndex] = $interface.InterfaceMetric
}
$defaultRoute = Get-NetRoute -AddressFamily IPv4 -DestinationPrefix '0.0.0.0/0' -PolicyStore ActiveStore -ErrorAction SilentlyContinue |
  Where-Object { $activeIndexes -contains $_.InterfaceIndex } |
  Sort-Object @{ Expression = { [long]$_.RouteMetric + [long]$interfaceMetrics[$_.InterfaceIndex] } }, InterfaceIndex |
  Select-Object -First 1

$result = foreach ($adapter in $adapters) {
  $cfg = $configs | Where-Object { $_.InterfaceIndex -eq $adapter.InterfaceIndex } | Select-Object -First 1
  $dhcp = $dhcps | Where-Object { $_.InterfaceIndex -eq $adapter.InterfaceIndex } | Select-Object -First 1

  [PSCustomObject]@{
    name = $adapter.Name
    status = [string]$adapter.Status
    ipv4 = (($cfg.IPv4Address | ForEach-Object { $_.IPAddress }) -join ', ')
    ipv6 = (($cfg.IPv6Address | ForEach-Object { $_.IPAddress }) -join ', ')
    gateway = (($cfg.IPv4DefaultGateway | ForEach-Object { $_.NextHop }) -join ', ')
    dns = (($cfg.DNSServer.ServerAddresses) -join ', ')
    mac = [string]$adapter.MacAddress
    dhcp = if ($dhcp) { [string]$dhcp.Dhcp } else { 'Unknown' }
    isDefault = [bool]($adapter.InterfaceIndex -eq $defaultRoute.InterfaceIndex)
  }
}

$result | ConvertTo-Json -Depth 4
"#;

    let raw = powershell::run(script)?;
    if raw.is_empty() {
        return Ok(vec![]);
    }

    let value: Value = serde_json::from_str(&raw).map_err(|e| e.to_string())?;
    let list = match value {
        Value::Array(items) => items,
        one => vec![one],
    };

    let mut adapters = Vec::new();

    for item in list {
        adapters.push(Adapter {
            name: item["name"].as_str().unwrap_or("-").to_string(),
            status: item["status"].as_str().unwrap_or("-").to_string(),
            ipv4: item["ipv4"].as_str().unwrap_or("-").to_string(),
            ipv6: item["ipv6"].as_str().unwrap_or("-").to_string(),
            gateway: item["gateway"].as_str().unwrap_or("-").to_string(),
            dns: item["dns"].as_str().unwrap_or("-").to_string(),
            mac: item["mac"].as_str().unwrap_or("-").to_string(),
            dhcp: item["dhcp"].as_str().unwrap_or("-").to_string(),
            is_default: item["isDefault"].as_bool().unwrap_or(false),
        });
    }

    Ok(adapters)
}

fn get_connections() -> Result<ConnectionSnapshot, String> {
    let script = r#"
$conns = @(Get-NetTCPConnection | Where-Object { $_.State -in @('Listen','Established') })
$processNames = @{}
$result = @(
  foreach ($c in ($conns | Select-Object -First 100)) {
    if (-not $processNames.ContainsKey($c.OwningProcess)) {
      $process = Get-Process -Id $c.OwningProcess -ErrorAction SilentlyContinue
      $processNames[$c.OwningProcess] = if ($process) { [string]$process.ProcessName } else { '-' }
    }
    [PSCustomObject]@{
      protocol = 'tcp'
      localAddress = [string]$c.LocalAddress
      localPort = [string]$c.LocalPort
      remoteAddress = [string]$c.RemoteAddress
      remotePort = [string]$c.RemotePort
      state = [string]$c.State
      pid = [string]$c.OwningProcess
      process = $processNames[$c.OwningProcess]
    }
  }
)

[PSCustomObject]@{
  establishedCount = @($conns | Where-Object State -eq 'Established').Count
  listeningCount = @($conns | Where-Object State -eq 'Listen').Count
  connections = $result
} | ConvertTo-Json -Depth 4 -Compress
"#;

    let raw = powershell::run(script)?;
    serde_json::from_str(&raw).map_err(|error| error.to_string())
}

#[tauri::command]
async fn get_network_info(
    requests: tauri::State<'_, NetworkRequestState>,
) -> Result<NetworkInfo, String> {
    let guard = requests.acquire()?;
    tauri::async_runtime::spawn_blocking(move || {
        // Keep the gate until the worker finishes, even if the IPC future is dropped.
        let _guard = guard;
        collect_network_info()
    })
    .await
    .map_err(|_| "情報取得処理を完了できませんでした。再取得してください。".to_string())?
}

fn collect_network_info() -> Result<NetworkInfo, String> {
    let adapters = get_adapters()?;
    let ConnectionSnapshot {
        established_count,
        listening_count,
        connections,
    } = get_connections()?;
    let public_ip = get_public_ip();

    let default_adapter = adapters
        .iter()
        .find(|x| x.is_default)
        .map(|x| x.name.clone())
        .unwrap_or_else(|| "-".to_string());

    let summary = Summary {
        public_ip,
        default_adapter,
        adapter_count: adapters.len(),
        connection_count: established_count + listening_count,
        established_count,
        listening_count,
    };

    Ok(NetworkInfo {
        summary,
        adapters,
        connections,
    })
}

#[tauri::command]
fn get_storage_info(paths: tauri::State<'_, RuntimePaths>) -> RuntimePaths {
    paths.inner().clone()
}

fn main() {
    let runtime_paths =
        RuntimePaths::detect().expect("failed to determine application storage directories");
    runtime_paths
        .prepare()
        .expect("failed to prepare application storage directories");

    let setup_paths = runtime_paths.clone();
    tauri::Builder::default()
        .manage(runtime_paths)
        .manage(NetworkRequestState::default())
        .setup(move |app| {
            let window_config = app
                .config()
                .app
                .windows
                .first()
                .ok_or("main window configuration is missing")?;
            tauri::WebviewWindowBuilder::from_config(app.handle(), window_config)?
                .data_directory(setup_paths.session_data.clone())
                .build()?;
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![get_network_info, get_storage_info])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_duplicate_requests_and_releases_after_error() {
        let state = NetworkRequestState::default();
        let guard = state.acquire().unwrap();
        assert!(state.acquire().is_err());
        drop(guard);
        assert!(state.acquire().is_ok());
    }

    #[test]
    fn public_ip_accepts_only_ip_addresses() {
        assert_eq!(
            parse_public_ip(br#"{"ip":"203.0.113.1"}"#).as_deref(),
            Some("203.0.113.1")
        );
        assert_eq!(
            parse_public_ip(br#"{"ip":"2001:db8::1"}"#).as_deref(),
            Some("2001:db8::1")
        );
        assert!(parse_public_ip(br#"{"ip":"<script>bad()</script>"}"#).is_none());
        assert!(parse_public_ip(br#"{"ip":null}"#).is_none());
        assert!(parse_public_ip(b"not json").is_none());
    }

    #[test]
    fn portable_release_name_is_detected_case_insensitively() {
        assert!(is_portable_executable(Path::new(
            r"C:\Tools\NexaLink-Monitor-Portable-1.1.0.exe"
        )));
        assert!(is_portable_executable(Path::new(
            r"C:\Tools\NEXALINK-MONITOR-PORTABLE-TEST.EXE"
        )));
        assert!(is_portable_executable(Path::new(
            r"C:\Tools\pc-network-tool-portable-test.EXE"
        )));
    }

    #[test]
    fn installed_and_development_names_are_not_portable() {
        assert!(!is_portable_executable(Path::new(
            r"C:\Program Files\NexaLink Monitor\NexaLink Monitor.exe"
        )));
        assert!(!is_portable_executable(Path::new(
            r"C:\repo\target\release\nexalink-monitor.exe"
        )));
    }

    #[test]
    fn portable_paths_stay_under_one_root() {
        let root = PathBuf::from(r"D:\Portable\NexaLink-Monitor-PortableData");
        let paths = RuntimePaths::from_roots(true, root.clone(), root.clone());
        for path in [
            paths.user_data,
            paths.session_data,
            paths.config,
            paths.logs,
            paths.cache,
            paths.temp,
        ] {
            assert!(path.starts_with(&root));
        }
    }
}
