use crate::security::SandboxConfig;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::time::Duration;
use thiserror::Error;

pub const SCRIPT_TIMEOUT_SECS: u64 = 60;

#[derive(Debug, Error)]
pub enum ScriptError {
    #[error("Script not found: {0}")]
    NotFound(PathBuf),

    #[error("Script execution failed: {0}")]
    ExecutionFailed(String),

    #[error("Script timed out after {0} seconds")]
    Timeout(u64),

    #[error("Failed to spawn sandbox process: {0}")]
    SpawnFailed(String),

    #[error("Script exited with code {0}")]
    NonZeroExit(i32),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

pub struct ScriptResult {
    pub success: bool,
    pub exit_code: Option<i32>,
    pub stdout: String,
    pub stderr: String,
}

pub fn run_script_sandboxed(
    script: &Path,
    module_dir: &Path,
    config: &SandboxConfig,
    timeout: Duration,
) -> Result<ScriptResult, ScriptError> {
    if !script.exists() {
        return Err(ScriptError::NotFound(script.to_path_buf()));
    }

    let exe = std::env::current_exe().map_err(|e| ScriptError::SpawnFailed(e.to_string()))?;

    let config_json =
        serde_json::to_string(config).map_err(|e| ScriptError::SpawnFailed(e.to_string()))?;

    let mut child = Command::new(&exe)
        .arg("internal-sandbox-exec")
        .arg("--script")
        .arg(script)
        .arg("--module-dir")
        .arg(module_dir)
        .env("WAYBAR_SANDBOX_CONFIG", config_json)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| ScriptError::SpawnFailed(e.to_string()))?;

    let result = wait_with_timeout(&mut child, timeout)?;

    Ok(result)
}

fn wait_with_timeout(child: &mut Child, timeout: Duration) -> Result<ScriptResult, ScriptError> {
    let start = std::time::Instant::now();
    let poll_interval = Duration::from_millis(100);

    loop {
        match child.try_wait() {
            Ok(Some(status)) => {
                let stdout = child
                    .stdout
                    .take()
                    .map(|mut s| {
                        let mut buf = String::new();
                        std::io::Read::read_to_string(&mut s, &mut buf).ok();
                        buf
                    })
                    .unwrap_or_default();

                let stderr = child
                    .stderr
                    .take()
                    .map(|mut s| {
                        let mut buf = String::new();
                        std::io::Read::read_to_string(&mut s, &mut buf).ok();
                        buf
                    })
                    .unwrap_or_default();

                return Ok(ScriptResult {
                    success: status.success(),
                    exit_code: status.code(),
                    stdout,
                    stderr,
                });
            }
            Ok(None) => {
                if start.elapsed() > timeout {
                    let _ = child.kill();
                    let _ = child.wait();
                    return Err(ScriptError::Timeout(timeout.as_secs()));
                }
                std::thread::sleep(poll_interval);
            }
            Err(e) => return Err(ScriptError::IoError(e)),
        }
    }
}

pub fn run_script_unsandboxed(
    script: &Path,
    module_dir: &Path,
    timeout: Duration,
) -> Result<ScriptResult, ScriptError> {
    if !script.exists() {
        return Err(ScriptError::NotFound(script.to_path_buf()));
    }

    let mut child = Command::new("bash")
        .arg(script)
        .current_dir(module_dir)
        .env("MODULE_DIR", module_dir)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| ScriptError::SpawnFailed(e.to_string()))?;

    wait_with_timeout(&mut child, timeout)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn setup_test_script(content: &str) -> (TempDir, PathBuf) {
        let dir = TempDir::new().unwrap();
        let script_path = dir.path().join("test.sh");
        fs::write(&script_path, content).unwrap();

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(&script_path, fs::Permissions::from_mode(0o755)).unwrap();
        }

        (dir, script_path)
    }

    #[test]
    fn script_not_found_error() {
        let result = run_script_unsandboxed(
            Path::new("/nonexistent/script.sh"),
            Path::new("/tmp"),
            Duration::from_secs(5),
        );
        assert!(matches!(result, Err(ScriptError::NotFound(_))));
    }

    #[test]
    fn successful_script_execution() {
        let (dir, script) = setup_test_script("#!/bin/bash\necho 'hello'");
        let result = run_script_unsandboxed(&script, dir.path(), Duration::from_secs(5)).unwrap();
        assert!(result.success);
        assert!(result.stdout.contains("hello"));
    }

    #[test]
    fn script_with_nonzero_exit() {
        let (dir, script) = setup_test_script("#!/bin/bash\nexit 42");
        let result = run_script_unsandboxed(&script, dir.path(), Duration::from_secs(5)).unwrap();
        assert!(!result.success);
        assert_eq!(result.exit_code, Some(42));
    }

    #[test]
    fn script_timeout() {
        let (dir, script) = setup_test_script("#!/bin/bash\nsleep 10");
        let result = run_script_unsandboxed(&script, dir.path(), Duration::from_millis(500));
        assert!(matches!(result, Err(ScriptError::Timeout(_))));
    }

    #[test]
    fn script_captures_stderr() {
        let (dir, script) = setup_test_script("#!/bin/bash\necho 'error' >&2");
        let result = run_script_unsandboxed(&script, dir.path(), Duration::from_secs(5)).unwrap();
        assert!(result.stderr.contains("error"));
    }
}
