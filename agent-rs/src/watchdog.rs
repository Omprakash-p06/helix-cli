use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessOwnership {
    pub pid: u32,
    pub port: u16,
    pub started_by: String,  // e.g., "helix-agent"
    pub start_time_unix: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WatchdogState {
    Healthy,
    Degraded,
    Recovering,
    Cooldown,
    Unhealthy,
}

pub struct Watchdog {
    state: WatchdogState,
    max_restarts: u32,
    restart_count: u32,
    cooldown_duration: Duration,
    last_failure: Option<Instant>,
    last_restart: Option<Instant>,
}

impl Watchdog {
    pub fn new(max_restarts: u32, cooldown_secs: u64) -> Self {
        Self {
            state: WatchdogState::Healthy,
            max_restarts,
            restart_count: 0,
            cooldown_duration: Duration::from_secs(cooldown_secs),
            last_failure: None,
            last_restart: None,
        }
    }

    pub fn state(&self) -> WatchdogState {
        self.state
    }

    pub fn on_success(&mut self) {
        self.state = WatchdogState::Healthy;
        self.restart_count = 0;
        self.last_failure = None;
    }

    pub fn on_failure(&mut self) -> (WatchdogState, String) {
        let now = Instant::now();
        self.last_failure = Some(now);

        match self.state {
            WatchdogState::Healthy => {
                self.state = WatchdogState::Degraded;
                (self.state, "Health probe failed. Entering Degraded state.".to_string())
            }
            WatchdogState::Degraded | WatchdogState::Recovering => {
                if self.restart_count >= self.max_restarts {
                    self.state = WatchdogState::Cooldown;
                    (self.state, format!("Restart budget (max: {}) exceeded. Entering Cooldown.", self.max_restarts))
                } else {
                    self.state = WatchdogState::Recovering;
                    self.restart_count += 1;
                    self.last_restart = Some(now);
                    (self.state, format!("Initiating recovery attempt {}/{}", self.restart_count, self.max_restarts))
                }
            }
            WatchdogState::Cooldown => {
                if let Some(last_fail) = self.last_failure {
                    if now.duration_since(last_fail) >= self.cooldown_duration {
                        self.state = WatchdogState::Healthy;
                        self.restart_count = 0;
                        (self.state, "Cooldown period ended. Resetting to Healthy for retry.".to_string())
                    } else {
                        (self.state, format!("Still in Cooldown. {}s remaining.", 
                            self.cooldown_duration.as_secs().saturating_sub(now.duration_since(last_fail).as_secs())))
                    }
                } else {
                    self.state = WatchdogState::Healthy;
                    (self.state, "Recovered from unknown cooldown state.".to_string())
                }
            }
            WatchdogState::Unhealthy => {
                (self.state, "System marked as Unhealthy. Manual intervention may be required.".to_string())
            }
        }
    }

    pub fn can_restart(&self) -> bool {
        match self.state {
            WatchdogState::Healthy | WatchdogState::Degraded | WatchdogState::Recovering => {
                self.restart_count < self.max_restarts
            }
            _ => false,
        }
    }

    pub fn next_backoff(&self) -> Duration {
        match self.restart_count {
            0 => Duration::from_secs(0),
            1 => Duration::from_secs(2),
            2 => Duration::from_secs(10),
            3 => Duration::from_secs(30),
            _ => Duration::from_secs(60),
        }
    }
}

/// Errors that can occur during process management operations.
#[derive(Debug)]
pub enum ProcessError {
    Io(std::io::Error),
    Parse(String),
    Ownership(String),
    Serde(serde_json::Error),
}

impl std::fmt::Display for ProcessError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProcessError::Io(e) => write!(f, "IO error: {}", e),
            ProcessError::Parse(e) => write!(f, "Parse error: {}", e),
            ProcessError::Ownership(e) => write!(f, "Ownership error: {}", e),
            ProcessError::Serde(e) => write!(f, "Serde error: {}", e),
        }
    }
}

impl std::error::Error for ProcessError {}

/// Reads the server PID file and returns the ProcessOwnership if it exists.
///
/// The PID file is expected at `<workspace>/.helix/server.pid` as JSON.
pub fn read_server_pid(workspace: &std::path::Path) -> Result<ProcessOwnership, ProcessError> {
    let pid_path = workspace.join(".helix").join("server.pid");
    let data = std::fs::read_to_string(&pid_path).map_err(|e| {
        if e.kind() == std::io::ErrorKind::NotFound {
            ProcessError::Ownership(format!(
                "PID file not found at {}. No server is registered as owned by this agent.",
                pid_path.display()
            ))
        } else {
            ProcessError::Io(e)
        }
    })?;
    serde_json::from_str(&data).map_err(ProcessError::Serde)
}

/// Writes a ProcessOwnership record to `<workspace>/.helix/server.pid`.
pub fn write_server_pid(
    workspace: &std::path::Path,
    ownership: &ProcessOwnership,
) -> Result<(), ProcessError> {
    let helix_dir = workspace.join(".helix");
    std::fs::create_dir_all(&helix_dir).map_err(ProcessError::Io)?;
    let pid_path = helix_dir.join("server.pid");
    let data = serde_json::to_string_pretty(ownership).map_err(ProcessError::Serde)?;
    std::fs::write(&pid_path, data).map_err(ProcessError::Io)?;
    Ok(())
}

/// Determines whether a process identified by `ownership` should be terminated.
///
/// Only returns `true` if:
/// 1. `ownership.started_by` is `"helix-agent"` (or the configured agent name)
/// 2. The process is not a foreign/unowned process
pub fn should_kill(ownership: &ProcessOwnership, agent_name: &str) -> bool {
    ownership.started_by == agent_name
}

/// Sends SIGTERM on Unix via the `kill` command.
fn send_terminate_signal(pid: u32) -> Result<(), ProcessError> {
    let status = std::process::Command::new("kill")
        .arg("-TERM")
        .arg(pid.to_string())
        .status()
        .map_err(ProcessError::Io)?;
    if status.success() {
        Ok(())
    } else {
        Err(ProcessError::Ownership(format!(
            "kill command returned non-zero for PID {}",
            pid
        )))
    }
}

/// Safely terminates a process identified by the given PID, but only after
/// verifying ownership via the PID file.
///
/// Returns `Ok(true)` if the process was terminated, `Ok(false)` if the
/// process was not running or didn't need termination, or `Err` if
/// ownership verification failed or the terminate signal failed.
///
/// This function follows the safety contract from P1-4:
/// - Reads `.helix/server.pid` to get the ProcessOwnership record
/// - Verifies `started_by == "helix-agent"` before sending any signal
/// - Returns `Err(ProcessError::Ownership(...))` for foreign processes
pub fn terminate(
    pid: u32,
    workspace: &std::path::Path,
    agent_name: &str,
) -> Result<bool, ProcessError> {
    let ownership = read_server_pid(workspace)?;

    if !should_kill(&ownership, agent_name) {
        return Err(ProcessError::Ownership(format!(
            "Refusing to terminate PID {}: process was started by '{}', not '{}'. \
             This is a foreign process — the watchdog must not kill it.",
            pid, ownership.started_by, agent_name
        )));
    }

    if ownership.pid != pid {
        return Err(ProcessError::Ownership(format!(
            "PID mismatch: server.pid records PID {} but terminate was called for PID {}. \
             Refusing to terminate — the ownership record does not match.",
            ownership.pid, pid
        )));
    }

    send_terminate_signal(pid)?;
    Ok(true)
}

/// Checks whether a TCP port is available by attempting to bind to it.
/// Used after recovery to verify the old server has released the port.
pub fn is_port_available(port: u16) -> bool {
    std::net::TcpListener::bind(std::net::SocketAddrV4::new(
        std::net::Ipv4Addr::LOCALHOST,
        port,
    ))
    .is_ok()
}
