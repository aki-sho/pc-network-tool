use serde::Serialize;
use serde_json::Value;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

const APP_DIR_NAME: &str = "PC Network Tool";
const PORTABLE_DATA_DIR_NAME: &str = "PC-Network-Tool-PortableData";

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
            let root = executable_dir.join(PORTABLE_DATA_DIR_NAME);
            Ok(Self::from_roots(true, root.clone(), root))
        } else {
            let roaming = required_environment_path("APPDATA")?.join(APP_DIR_NAME);
            let local = required_environment_path("LOCALAPPDATA")?.join(APP_DIR_NAME);
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
        if !settings_path.exists() {
            fs::write(&settings_path, "{\n  \"version\": \"1.0.1\"\n}\n")?;
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
            lowercase.starts_with("pc-network-tool-portable-") && lowercase.ends_with(".exe")
        })
        .unwrap_or(false)
}

#[derive(Serialize)]
struct Summary {
    public_ip: String,
    default_adapter: String,
    adapter_count: usize,
    connection_count: usize,
}

#[derive(Serialize, Clone)]
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

#[derive(Serialize)]
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

#[derive(Serialize)]
struct NetworkInfo {
    summary: Summary,
    adapters: Vec<Adapter>,
    connections: Vec<Connection>,
}

fn run_powershell(script: &str) -> Result<String, String> {
    let output = Command::new("powershell")
        .args([
            "-NoProfile",
            "-ExecutionPolicy",
            "Bypass",
            "-Command",
            script,
        ])
        .output()
        .map_err(|e| e.to_string())?;

    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).to_string());
    }

    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

fn get_public_ip() -> String {
    let response = reqwest::blocking::get("https://api.ipify.org?format=json");
    match response {
        Ok(res) => match res.json::<Value>() {
            Ok(json) => json["ip"].as_str().unwrap_or("-").to_string(),
            Err(_) => "-".to_string(),
        },
        Err(_) => "-".to_string(),
    }
}

fn get_adapters() -> Result<Vec<Adapter>, String> {
    let script = r#"
$adapters = Get-NetAdapter -IncludeHidden | Sort-Object InterfaceIndex
$configs = Get-NetIPConfiguration
$dhcps = Get-NetIPInterface -AddressFamily IPv4

$result = foreach ($adapter in $adapters) {
  $cfg = $configs | Where-Object { $_.InterfaceIndex -eq $adapter.InterfaceIndex } | Select-Object -First 1
  $dhcp = $dhcps | Where-Object { $_.InterfaceIndex -eq $adapter.InterfaceIndex } | Select-Object -First 1

  [PSCustomObject]@{
    name = $adapter.Name
    status = [string]$adapter.Status
    ipv4 = (($cfg.IPv4Address | ForEach-Object { $_.IPAddress }) -join ', ')
    ipv6 = (($cfg.IPv6Address | ForEach-Object { $_.IPAddress }) -join ', ')
    gateway = [string]$cfg.IPv4DefaultGateway.NextHop
    dns = (($cfg.DNSServer.ServerAddresses) -join ', ')
    mac = [string]$adapter.MacAddress
    dhcp = if ($dhcp) { [string]$dhcp.Dhcp } else { 'Unknown' }
    isDefault = [bool]($cfg.IPv4DefaultGateway -ne $null -and $adapter.Status -eq 'Up')
  }
}

$result | ConvertTo-Json -Depth 4
"#;

    let raw = run_powershell(script)?;
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

fn get_connections() -> Result<Vec<Connection>, String> {
    let script = r#"
$conns = Get-NetTCPConnection |
  Where-Object { $_.State -in @('Listen','Established') } |
  Select-Object -First 100

$result = foreach ($c in $conns) {
  $p = Get-Process -Id $c.OwningProcess -ErrorAction SilentlyContinue
  [PSCustomObject]@{
    protocol = 'tcp'
    localAddress = [string]$c.LocalAddress
    localPort = [string]$c.LocalPort
    remoteAddress = [string]$c.RemoteAddress
    remotePort = [string]$c.RemotePort
    state = [string]$c.State
    pid = [string]$c.OwningProcess
    process = if ($p) { [string]$p.ProcessName } else { '-' }
  }
}

$result | ConvertTo-Json -Depth 4
"#;

    let raw = run_powershell(script)?;
    if raw.is_empty() {
        return Ok(vec![]);
    }

    let value: Value = serde_json::from_str(&raw).map_err(|e| e.to_string())?;
    let list = match value {
        Value::Array(items) => items,
        one => vec![one],
    };

    let mut connections = Vec::new();

    for item in list {
        connections.push(Connection {
            protocol: item["protocol"].as_str().unwrap_or("-").to_string(),
            local_address: item["localAddress"].as_str().unwrap_or("-").to_string(),
            local_port: item["localPort"].as_str().unwrap_or("-").to_string(),
            remote_address: item["remoteAddress"].as_str().unwrap_or("-").to_string(),
            remote_port: item["remotePort"].as_str().unwrap_or("-").to_string(),
            state: item["state"].as_str().unwrap_or("-").to_string(),
            pid: item["pid"].as_str().unwrap_or("-").to_string(),
            process: item["process"].as_str().unwrap_or("-").to_string(),
        });
    }

    Ok(connections)
}

#[tauri::command]
fn get_network_info() -> Result<NetworkInfo, String> {
    let adapters = get_adapters()?;
    let connections = get_connections()?;
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
        connection_count: connections.len(),
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
    fn portable_release_name_is_detected_case_insensitively() {
        assert!(is_portable_executable(Path::new(
            r"C:\Tools\PC-Network-Tool-Portable-1.0.1.exe"
        )));
        assert!(is_portable_executable(Path::new(
            r"C:\Tools\pc-network-tool-portable-test.EXE"
        )));
    }

    #[test]
    fn installed_and_development_names_are_not_portable() {
        assert!(!is_portable_executable(Path::new(
            r"C:\Program Files\PC Network Tool\PC Network Tool.exe"
        )));
        assert!(!is_portable_executable(Path::new(
            r"C:\repo\target\release\pc-network-tool.exe"
        )));
    }

    #[test]
    fn portable_paths_stay_under_one_root() {
        let root = PathBuf::from(r"D:\Portable\PC-Network-Tool-PortableData");
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
