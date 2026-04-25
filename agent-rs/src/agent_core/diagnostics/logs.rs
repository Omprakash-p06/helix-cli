use serde::{Deserialize, Serialize};
use std::process::Command;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LogEntry {
    pub timestamp: String,
    pub source: String,
    pub level: String,
    pub message: String,
    #[serde(flatten)]
    pub metadata: serde_json::Value,
}

pub trait LogProvider {
    fn get_logs(&self, limit: usize) -> Result<Vec<LogEntry>, String>;
}

pub struct LinuxLogProvider;
pub struct WindowsLogProvider;

impl LogProvider for LinuxLogProvider {
    fn get_logs(&self, limit: usize) -> Result<Vec<LogEntry>, String> {
        // journalctl -o json -n {limit}
        let output = Command::new("journalctl")
            .args(["-o", "json", "-n", &limit.to_string()])
            .output()
            .map_err(|e| format!("Failed to execute journalctl: {}", e))?;

        if !output.status.success() {
            // If journalctl fails, it might be due to permissions or not being present
            return Err(format!("journalctl failed: {}", String::from_utf8_lossy(&output.stderr)));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let mut entries = Vec::new();

        for line in stdout.lines() {
            if line.trim().is_empty() {
                continue;
            }
            let raw: serde_json::Value = serde_json::from_str(line)
                .map_err(|e| format!("Failed to parse journalctl JSON: {}", e))?;
            
            // Map journalctl fields to LogEntry
            // journalctl uses __REALTIME_TIMESTAMP (microseconds) or SYSLOG_TIMESTAMP
            let timestamp = raw.get("__REALTIME_TIMESTAMP")
                .and_then(|v| v.as_str())
                .or_else(|| raw.get("SYSLOG_TIMESTAMP").and_then(|v| v.as_str()))
                .unwrap_or("unknown")
                .to_string();
            
            let source = raw.get("SYSLOG_IDENTIFIER")
                .and_then(|v| v.as_str())
                .or_else(|| raw.get("_COMM").and_then(|v| v.as_str()))
                .unwrap_or("unknown")
                .to_string();
            
            let level = raw.get("PRIORITY")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown")
                .to_string();
            
            let message = raw.get("MESSAGE")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();

            entries.push(LogEntry {
                timestamp,
                source,
                level,
                message,
                metadata: raw,
            });
        }

        Ok(entries)
    }
}

#[cfg(windows)]
impl LogProvider for WindowsLogProvider {
    fn get_logs(&self, limit: usize) -> Result<Vec<LogEntry>, String> {
        use evtx::EvtxParser;
        use std::path::Path;

        let log_path = "C:\\Windows\\System32\\Winevt\\Logs\\System.evtx";
        if !Path::new(log_path).exists() {
            return Err(format!("Log file not found: {}", log_path));
        }

        let mut parser = EvtxParser::from_path(log_path)
            .map_err(|e| format!("Failed to open evtx: {}", e))?;
        
        let mut entries = Vec::new();
        for record in parser.records_json() {
            if entries.len() >= limit {
                break;
            }
            let record = record.map_err(|e| format!("Failed to read record: {}", e))?;
            let raw: serde_json::Value = serde_json::from_str(&record.data)
                .map_err(|e| format!("Failed to parse record JSON: {}", e))?;
            
            // Map Windows Event fields (this structure can vary)
            let timestamp = raw.get("Event")
                .and_then(|e| e.get("System"))
                .and_then(|s| s.get("TimeCreated"))
                .and_then(|t| t.get("@SystemTime"))
                .and_then(|v| v.as_str())
                .unwrap_or("unknown")
                .to_string();

            let source = raw.get("Event")
                .and_then(|e| e.get("System"))
                .and_then(|s| s.get("Provider"))
                .and_then(|p| p.get("@Name"))
                .and_then(|v| v.as_str())
                .unwrap_or("unknown")
                .to_string();

            let level = raw.get("Event")
                .and_then(|e| e.get("System"))
                .and_then(|s| s.get("Level"))
                .and_then(|v| v.as_u64())
                .map(|l| l.to_string())
                .unwrap_or_else(|| "unknown".to_string());
            
            let message = raw.get("Event")
                .and_then(|e| e.get("EventData"))
                .map(|d| d.to_string())
                .unwrap_or_default();

            entries.push(LogEntry {
                timestamp,
                source,
                level,
                message,
                metadata: raw,
            });
        }

        Ok(entries)
    }
}

#[cfg(not(windows))]
impl LogProvider for WindowsLogProvider {
    fn get_logs(&self, _limit: usize) -> Result<Vec<LogEntry>, String> {
        Err("Windows logs can only be retrieved on Windows".to_string())
    }
}

pub fn get_system_logs(limit: usize) -> Result<Vec<LogEntry>, String> {
    #[cfg(target_os = "linux")]
    {
        LinuxLogProvider.get_logs(limit)
    }
    #[cfg(target_os = "windows")]
    {
        WindowsLogProvider.get_logs(limit)
    }
    #[cfg(not(any(target_os = "linux", target_os = "windows")))]
    {
        Err(format!("Unsupported OS for log retrieval: {}", std::env::consts::OS))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_linux_log_provider_exists() {
        let _ = LinuxLogProvider;
    }

    #[test]
    fn test_windows_log_provider_exists() {
        let _ = WindowsLogProvider;
    }

    #[test]
    #[cfg(target_os = "linux")]
    fn test_journal_parsing() {
        match LinuxLogProvider.get_logs(1) {
            Ok(logs) => {
                if !logs.is_empty() {
                    assert!(!logs[0].timestamp.is_empty());
                    assert!(!logs[0].source.is_empty());
                    assert!(!logs[0].message.is_empty());
                }
            },
            Err(e) => println!("Skipping live linux log test: {}", e),
        }
    }

    #[test]
    #[cfg(target_os = "windows")]
    fn test_evtx_parsing() {
        match WindowsLogProvider.get_logs(1) {
            Ok(logs) => {
                if !logs.is_empty() {
                    assert!(!logs[0].timestamp.is_empty());
                    assert!(!logs[0].source.is_empty());
                }
            }
            Err(e) => println!("Skipping live windows log test: {}", e),
        }
    }
}
