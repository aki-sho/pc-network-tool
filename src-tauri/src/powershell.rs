use std::io::Read;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

const EXECUTION_TIMEOUT: Duration = Duration::from_secs(30);
const OUTPUT_LIMIT: usize = 4 * 1024 * 1024;

#[cfg(windows)]
fn system_directory() -> Result<PathBuf, String> {
    use std::ffi::OsString;
    use std::os::windows::ffi::OsStringExt;
    use windows_sys::Win32::System::SystemInformation::GetSystemDirectoryW;

    let mut buffer = vec![0u16; 32768];
    // SAFETY: buffer is writable for the supplied number of UTF-16 code units.
    let length = unsafe { GetSystemDirectoryW(buffer.as_mut_ptr(), buffer.len() as u32) } as usize;
    if length == 0 || length >= buffer.len() {
        return Err("Windowsのシステムディレクトリを取得できませんでした。".into());
    }
    Ok(PathBuf::from(OsString::from_wide(&buffer[..length])))
}

pub fn executable() -> Result<PathBuf, String> {
    #[cfg(windows)]
    {
        // Resolve through the OS API, never PATH, the working directory or SystemRoot.
        let path = system_directory()?.join(r"WindowsPowerShell\v1.0\powershell.exe");
        if !path.is_absolute() || !path.is_file() {
            return Err("Windows標準のPowerShellが見つかりません。".into());
        }
        Ok(path)
    }
    #[cfg(not(windows))]
    Err("このアプリはWindows専用です。".into())
}

#[cfg(windows)]
fn assign_cleanup_job(
    child: &std::process::Child,
) -> Result<std::os::windows::io::OwnedHandle, String> {
    use std::mem::{size_of, zeroed};
    use std::os::windows::io::{AsRawHandle, FromRawHandle, OwnedHandle};
    use windows_sys::Win32::System::JobObjects::{
        AssignProcessToJobObject, CreateJobObjectW, JobObjectExtendedLimitInformation,
        SetInformationJobObject, JOBOBJECT_EXTENDED_LIMIT_INFORMATION,
        JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE,
    };

    // SAFETY: null security attributes and name request a private, unnamed job.
    let handle = unsafe { CreateJobObjectW(std::ptr::null(), std::ptr::null()) };
    if handle.is_null() {
        return Err(std::io::Error::last_os_error().to_string());
    }
    // SAFETY: CreateJobObjectW returned a new owned handle.
    let job = unsafe { OwnedHandle::from_raw_handle(handle) };
    // SAFETY: all-zero is a valid initial state for this Windows POD structure.
    let mut limits: JOBOBJECT_EXTENDED_LIMIT_INFORMATION = unsafe { zeroed() };
    limits.BasicLimitInformation.LimitFlags = JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE;
    // SAFETY: the handles are live and the information buffer has the stated size/type.
    let configured = unsafe {
        SetInformationJobObject(
            job.as_raw_handle(),
            JobObjectExtendedLimitInformation,
            (&limits as *const JOBOBJECT_EXTENDED_LIMIT_INFORMATION).cast(),
            size_of::<JOBOBJECT_EXTENDED_LIMIT_INFORMATION>() as u32,
        ) != 0
            && AssignProcessToJobObject(job.as_raw_handle(), child.as_raw_handle()) != 0
    };
    if !configured {
        return Err(std::io::Error::last_os_error().to_string());
    }
    Ok(job)
}

fn read_limited(reader: impl Read, limit: usize) -> Result<Vec<u8>, String> {
    let mut bytes = Vec::new();
    reader
        .take(limit as u64 + 1)
        .read_to_end(&mut bytes)
        .map_err(|error| error.to_string())?;
    if bytes.len() > limit {
        return Err("PowerShellの出力が上限を超えました。".into());
    }
    Ok(bytes)
}

pub fn run(script: &str) -> Result<String, String> {
    run_with_limits(script, EXECUTION_TIMEOUT, OUTPUT_LIMIT)
}

fn run_with_limits(script: &str, timeout: Duration, output_limit: usize) -> Result<String, String> {
    let executable = executable()?;
    let modules = executable
        .parent()
        .ok_or("PowerShellの配置先が不正です。")?
        .join("Modules");
    let utf8_script = format!(
        r#"
$env:PSModulePath = $PSHOME + '\Modules'
$ErrorActionPreference = 'Stop'
[Console]::OutputEncoding = [System.Text.UTF8Encoding]::new($false)
$OutputEncoding = [Console]::OutputEncoding
{script}
"#
    );
    let mut command = Command::new(&executable);
    command
        .args(["-NoLogo", "-NoProfile", "-NonInteractive", "-Command"])
        .arg(utf8_script)
        // Only the built-in Windows modules are needed by the fixed scripts.
        .env("PSModulePath", modules)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        use windows_sys::Win32::System::Threading::CREATE_NO_WINDOW;
        command.creation_flags(CREATE_NO_WINDOW);
    }
    let mut child = command.spawn().map_err(|error| error.to_string())?;
    #[cfg(windows)]
    let _job = match assign_cleanup_job(&child) {
        Ok(job) => job,
        Err(error) => {
            let _ = child.kill();
            let _ = child.wait();
            return Err(format!(
                "PowerShellの終了管理を設定できませんでした: {error}"
            ));
        }
    };

    let (sender, receiver) = mpsc::channel();
    let stdout = child
        .stdout
        .take()
        .ok_or("標準出力を取得できませんでした。")?;
    let stderr = child
        .stderr
        .take()
        .ok_or("標準エラーを取得できませんでした。")?;
    let stdout_sender = sender.clone();
    // Drain both pipes concurrently so a full stderr pipe cannot deadlock stdout.
    thread::spawn(move || {
        let _ = stdout_sender.send(read_limited(stdout, output_limit).map(|bytes| (true, bytes)));
    });
    thread::spawn(move || {
        let _ = sender.send(read_limited(stderr, output_limit).map(|bytes| (false, bytes)));
    });

    let started = Instant::now();
    let result = (|| {
        let mut stdout: Option<Vec<u8>> = None;
        let mut stderr: Option<Vec<u8>> = None;
        loop {
            let status = child.try_wait().map_err(|error| error.to_string())?;
            if let (Some(status), Some(stdout), Some(stderr)) = (status, &stdout, &stderr) {
                if !status.success() {
                    return Err(format!(
                        "PowerShellによる取得に失敗しました: {}",
                        String::from_utf8_lossy(stderr).trim()
                    ));
                }
                return String::from_utf8(stdout.clone())
                    .map(|text| text.trim().to_string())
                    .map_err(|_| "PowerShellの出力がUTF-8ではありません。".into());
            }
            let remaining = timeout.checked_sub(started.elapsed()).ok_or_else(|| {
                "情報の取得がタイムアウトしました。再取得してください。".to_string()
            })?;
            match receiver.recv_timeout(remaining.min(Duration::from_millis(20))) {
                Ok(output) => {
                    let (is_stdout, bytes) = output?;
                    if is_stdout {
                        stdout = Some(bytes);
                    } else {
                        stderr = Some(bytes);
                    }
                }
                Err(mpsc::RecvTimeoutError::Timeout) => {}
                Err(mpsc::RecvTimeoutError::Disconnected) => {
                    if stdout.is_none() || stderr.is_none() {
                        return Err("PowerShellの出力を取得できませんでした。".into());
                    }
                    thread::sleep(remaining.min(Duration::from_millis(20)));
                }
            }
        }
    })();
    if result.is_err() {
        // The job also terminates descendants when its handle goes out of scope.
        let _ = child.kill();
        let _ = child.wait();
    }
    result
}

#[cfg(all(test, windows))]
mod tests {
    use super::*;

    #[test]
    fn uses_system_powershell_and_only_builtin_modules() {
        let expected = executable().unwrap();
        assert!(expected.is_absolute());
        let output = run(
            "[PSCustomObject]@{ home = $PSHOME; modules = $env:PSModulePath } | ConvertTo-Json",
        )
        .unwrap();
        let value: serde_json::Value = serde_json::from_str(&output).unwrap();
        assert_eq!(
            PathBuf::from(value["home"].as_str().unwrap())
                .canonicalize()
                .unwrap(),
            expected.parent().unwrap().canonicalize().unwrap()
        );
        let module_paths = std::env::split_paths(value["modules"].as_str().unwrap())
            .map(|path| path.canonicalize().unwrap())
            .collect::<Vec<_>>();
        assert_eq!(
            module_paths,
            vec![expected
                .parent()
                .unwrap()
                .join("Modules")
                .canonicalize()
                .unwrap()]
        );
    }

    #[test]
    fn times_out_and_cleans_up_process() {
        let started = Instant::now();
        let error = run_with_limits("Start-Sleep -Seconds 30", Duration::from_millis(300), 1024)
            .unwrap_err();
        assert!(error.contains("タイムアウト"), "{error}");
        assert!(started.elapsed() < Duration::from_secs(10));
    }

    #[test]
    fn caps_output_without_waiting_for_the_process_to_finish() {
        let started = Instant::now();
        let error = run_with_limits(
            "[Console]::Out.Write(('x' * 8192)); Start-Sleep -Seconds 30",
            Duration::from_secs(10),
            1024,
        )
        .unwrap_err();
        assert!(error.contains("上限"), "{error}");
        assert!(started.elapsed() < Duration::from_secs(10));
    }

    #[test]
    fn returns_nonzero_exit_as_error() {
        let error = run("[Console]::Error.Write('test failure'); exit 7").unwrap_err();
        assert!(error.contains("test failure"));
    }
}
