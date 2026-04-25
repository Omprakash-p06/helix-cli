use sysinfo::System;
use serde::{Serialize, Deserialize};
use network_interface::{NetworkInterface, NetworkInterfaceConfig};

#[derive(Debug, Serialize, Deserialize)]
pub struct ProcessInfo {
    pub pid: u32,
    pub name: String,
    pub cpu_usage: f32,
    pub memory_usage: u64,
    pub status: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SystemStats {
    pub total_memory: u64,
    pub used_memory: u64,
    pub cpu_count: usize,
    pub global_cpu_usage: f32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NetworkInterfaceInfo {
    pub name: String,
    pub addr: Vec<String>,
}

pub struct SystemProvider {
    sys: System,
}

impl Default for SystemProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl SystemProvider {
    pub fn new() -> Self {
        Self {
            sys: System::new_all(),
        }
    }

    pub fn list_processes(&mut self) -> Vec<ProcessInfo> {
        self.sys.refresh_all();
        self.sys.processes().iter().map(|(pid, process)| {
            ProcessInfo {
                pid: pid.as_u32(),
                name: process.name().to_string_lossy().to_string(),
                cpu_usage: process.cpu_usage(),
                memory_usage: process.memory(),
                status: format!("{:?}", process.status()),
            }
        }).collect()
    }

    pub fn get_system_stats(&mut self) -> SystemStats {
        self.sys.refresh_all();
        SystemStats {
            total_memory: self.sys.total_memory(),
            used_memory: self.sys.used_memory(),
            cpu_count: self.sys.cpus().len(),
            global_cpu_usage: self.sys.global_cpu_usage(),
        }
    }

    pub fn list_network_interfaces() -> Vec<NetworkInterfaceInfo> {
        match NetworkInterface::show() {
            Ok(interfaces) => interfaces.iter().map(|iface| {
                NetworkInterfaceInfo {
                    name: iface.name.clone(),
                    addr: iface.addr.iter().map(|a| a.ip().to_string()).collect(),
                }
            }).collect(),
            Err(_) => Vec::new(),
        }
    }
    
    // Service introspection
    pub fn get_service_status(service_name: &str) -> String {
        #[cfg(target_os = "linux")]
        {
            use std::process::Command;
            let output = Command::new("systemctl")
                .arg("status")
                .arg(service_name)
                .output();
            
            match output {
                Ok(out) => {
                    let stdout = String::from_utf8_lossy(&out.stdout).to_string();
                    let stderr = String::from_utf8_lossy(&out.stderr).to_string();
                    if stdout.is_empty() && !stderr.is_empty() {
                        stderr
                    } else {
                        stdout
                    }
                },
                Err(e) => format!("Failed to execute systemctl: {}", e),
            }
        }

        #[cfg(target_os = "windows")]
        {
            use windows_service::service_control_manager::{ServiceControlManager, ServiceControlManagerAccess};
            use windows_service::service::{ServiceAccess};
            
            let scm = match ServiceControlManager::local() {
                Ok(scm) => match scm.open(ServiceControlManagerAccess::CONNECT) {
                    Ok(scm) => scm,
                    Err(e) => return format!("Failed to open SCM: {}", e),
                },
                Err(e) => return format!("Failed to connect to SCM: {}", e),
            };

            let service = match scm.open_service(service_name, ServiceAccess::QUERY_STATUS) {
                Ok(s) => s,
                Err(e) => return format!("Service not found or access denied: {}", e),
            };

            match service.query_status() {
                Ok(status) => format!("State: {:?}", status.current_state),
                Err(e) => format!("Failed to query service status: {}", e),
            }
        }

        #[cfg(not(any(target_os = "linux", target_os = "windows")))]
        {
            let _ = service_name;
            format!("Service introspection not supported on this OS")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_list_processes() {
        let mut provider = SystemProvider::new();
        let processes = provider.list_processes();
        assert!(!processes.is_empty());
    }

    #[test]
    fn test_get_system_stats() {
        let mut provider = SystemProvider::new();
        let stats = provider.get_system_stats();
        assert!(stats.total_memory > 0);
        assert!(stats.cpu_count > 0);
    }

    #[test]
    fn test_list_network_interfaces() {
        let interfaces = SystemProvider::list_network_interfaces();
        // Might be empty in some CI environments, but usually has at least loopback
        assert!(!interfaces.is_empty());
    }

    #[test]
    #[cfg(target_os = "linux")]
    fn test_get_service_status_linux() {
        // systemd-journald is almost always present on systemd systems
        let status = SystemProvider::get_service_status("systemd-journald");
        assert!(status.contains("active") || status.contains("systemd-journald"));
    }
}
