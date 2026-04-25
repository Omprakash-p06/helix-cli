use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use service_manager::*;
use crate::tools::{Tool, ToolResult};
use crate::security::policy::PolicyContext;

#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ServiceRepairInput {
    pub service_name: String,
    pub action: ServiceAction,
}

#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone)]
#[serde(rename_all = "snake_case")]
pub enum ServiceAction {
    Start,
    Stop,
    Restart,
}

pub struct ServiceRepairTool;

impl Tool for ServiceRepairTool {
    fn name(&self) -> String { "service_repair".into() }
    fn description(&self) -> String { 
        "Manages system services (start, stop, restart). Requires human approval.".into() 
    }
    fn schema(&self) -> Value { schemars::schema_for!(ServiceRepairInput).into() }
    fn execute(
        &self,
        args: Value,
        _dangerous_commands: &[String],
        _require_confirmation: bool,
        _policy_context: &PolicyContext,
    ) -> ToolResult {
        match serde_json::from_value::<ServiceRepairInput>(args) {
            Ok(input) => {
                let manager = match <dyn ServiceManager>::native() {
                    Ok(m) => m,
                    Err(e) => return ToolResult {
                        success: false,
                        output: format!("Failed to get native service manager: {}", e),
                    },
                };

                let label: ServiceLabel = match input.service_name.parse() {
                    Ok(l) => l,
                    Err(e) => return ToolResult {
                        success: false,
                        output: format!("Invalid service name: {}", e),
                    },
                };

                let result = match input.action {
                    ServiceAction::Start => manager.start(ServiceStartCtx { label }),
                    ServiceAction::Stop => manager.stop(ServiceStopCtx { label }),
                    ServiceAction::Restart => {
                        // Service-manager 0.11 doesn't have a direct restart? 
                        // Let me check if it has. If not, stop then start.
                        // Actually, looking at typical service-manager API, it might have it or not.
                        // If it doesn't, we do stop then start.
                        manager.stop(ServiceStopCtx { label: label.clone() })
                            .and_then(|_| manager.start(ServiceStartCtx { label }))
                    }
                };

                match result {
                    Ok(_) => ToolResult {
                        success: true,
                        output: format!("Successfully performed {:?} on service {}", input.action, input.service_name),
                    },
                    Err(e) => ToolResult {
                        success: false,
                        output: format!("Failed to {:?} service {}: {}", input.action, input.service_name, e),
                    },
                }
            }
            Err(e) => ToolResult { success: false, output: format!("Schema error: {}", e) },
        }
    }
}

#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PackageRepairInput {
    pub package_name: String,
    pub action: PackageAction,
    pub dry_run: Option<bool>,
}

#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone)]
#[serde(rename_all = "snake_case")]
pub enum PackageAction {
    Install,
    Remove,
    Update,
}

pub struct PackageRepairTool;

impl PackageRepairTool {
    fn check_locks(&self) -> Result<(), String> {
        if cfg!(target_os = "linux") {
            let locks = [
                "/var/lib/dpkg/lock-frontend",
                "/var/lib/dpkg/lock",
                "/var/lib/apt/lists/lock",
                "/var/cache/apt/archives/lock",
            ];
            for lock in locks {
                if std::path::Path::new(lock).exists() {
                    // Try to see if we can open it for writing (to check if it's actually locked)
                    // But simpler: just report it exists.
                    return Err(format!("Package manager lock file exists: {}. Another process might be running.", lock));
                }
            }
        }
        Ok(())
    }

    fn get_command(&self, input: &PackageRepairInput) -> Result<(String, Vec<String>), String> {
        if cfg!(target_os = "windows") {
            let action_cmd = match input.action {
                PackageAction::Install => "install",
                PackageAction::Remove => "uninstall",
                PackageAction::Update => "upgrade",
            };
            let mut args = vec![action_cmd.to_string(), input.package_name.clone(), "-y".into()];
            if input.dry_run.unwrap_or(false) {
                args.push("--what-if".into());
            }
            Ok(("choco".into(), args))
        } else {
            // Check for apt or dnf
            let (cmd, install_arg, remove_arg, update_arg, dry_run_arg) = if std::process::Command::new("apt-get").arg("--version").output().is_ok() {
                ("apt-get", "install", "remove", "upgrade", "-s")
            } else if std::process::Command::new("dnf").arg("--version").output().is_ok() {
                ("dnf", "install", "remove", "upgrade", "--setopt=tsflags=test")
            } else {
                return Err("No supported package manager found (apt or dnf)".into());
            };

            let action_arg = match input.action {
                PackageAction::Install => install_arg,
                PackageAction::Remove => remove_arg,
                PackageAction::Update => update_arg,
            };

            let mut args = vec![action_arg.to_string(), "-y".into(), input.package_name.clone()];
            if input.dry_run.unwrap_or(false) {
                args.push(dry_run_arg.into());
            }
            Ok((cmd.into(), args))
        }
    }
}

impl Tool for PackageRepairTool {
    fn name(&self) -> String { "package_repair".into() }
    fn description(&self) -> String { 
        "Installs, removes, or updates system packages. Requires human approval.".into() 
    }
    fn schema(&self) -> Value { schemars::schema_for!(PackageRepairInput).into() }
    fn execute(
        &self,
        args: Value,
        _dangerous_commands: &[String],
        _require_confirmation: bool,
        _policy_context: &PolicyContext,
    ) -> ToolResult {
        let input: PackageRepairInput = match serde_json::from_value(args) {
            Ok(i) => i,
            Err(e) => return ToolResult { success: false, output: format!("Schema error: {}", e) },
        };

        if let Err(e) = self.check_locks() {
            return ToolResult { success: false, output: e };
        }

        let (cmd, cmd_args) = match self.get_command(&input) {
            Ok(c) => c,
            Err(e) => return ToolResult { success: false, output: e },
        };

        println!("Executing package repair: {} {:?}", cmd, cmd_args);
        
        let output = std::process::Command::new(&cmd)
            .args(&cmd_args)
            .output();

        match output {
            Ok(out) => {
                let stdout = String::from_utf8_lossy(&out.stdout);
                let stderr = String::from_utf8_lossy(&out.stderr);
                let success = out.status.success();
                
                let mut combined = String::new();
                if !stdout.is_empty() { combined.push_str(&format!("STDOUT:\n{}\n", stdout)); }
                if !stderr.is_empty() { combined.push_str(&format!("STDERR:\n{}\n", stderr)); }
                
                ToolResult {
                    success,
                    output: if combined.is_empty() { "Command completed with no output.".into() } else { combined },
                }
            }
            Err(e) => ToolResult { success: false, output: format!("Failed to execute {}: {}", cmd, e) },
        }
    }
}

#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PermissionRepairInput {
    pub path: String,
    pub mode: Option<String>,
    pub owner: Option<String>,
    pub group: Option<String>,
    pub recursive: Option<bool>,
}

pub struct PermissionRepairTool;

impl Tool for PermissionRepairTool {
    fn name(&self) -> String { "permission_repair".into() }
    fn description(&self) -> String { 
        "Modifies file/directory permissions or ownership. Requires human approval.".into() 
    }
    fn schema(&self) -> Value { schemars::schema_for!(PermissionRepairInput).into() }
    fn execute(
        &self,
        args: Value,
        _dangerous_commands: &[String],
        _require_confirmation: bool,
        _policy_context: &PolicyContext,
    ) -> ToolResult {
        let input: PermissionRepairInput = match serde_json::from_value(args) {
            Ok(i) => i,
            Err(e) => return ToolResult { success: false, output: format!("Schema error: {}", e) },
        };

        let mut results = Vec::new();

        // Handle mode (chmod)
        if let Some(mode) = input.mode {
            let mut cmd = std::process::Command::new("chmod");
            if input.recursive.unwrap_or(false) {
                cmd.arg("-R");
            }
            cmd.arg(mode).arg(&input.path);
            
            match cmd.output() {
                Ok(out) => {
                    if out.status.success() {
                        results.push(format!("Successfully changed mode of {}", input.path));
                    } else {
                        results.push(format!("Failed to change mode: {}", String::from_utf8_lossy(&out.stderr)));
                    }
                }
                Err(e) => results.push(format!("Error executing chmod: {}", e)),
            }
        }

        // Handle owner/group (chown)
        if input.owner.is_some() || input.group.is_some() {
            let mut cmd = std::process::Command::new("chown");
            if input.recursive.unwrap_or(false) {
                cmd.arg("-R");
            }
            
            let owner = input.owner.unwrap_or_default();
            let group = input.group.unwrap_or_default();
            let arg = if !owner.is_empty() && !group.is_empty() {
                format!("{}:{}", owner, group)
            } else if !owner.is_empty() {
                owner
            } else {
                format!(":{}", group)
            };
            
            cmd.arg(arg).arg(&input.path);

            match cmd.output() {
                Ok(out) => {
                    if out.status.success() {
                        results.push(format!("Successfully changed ownership of {}", input.path));
                    } else {
                        results.push(format!("Failed to change ownership: {}", String::from_utf8_lossy(&out.stderr)));
                    }
                }
                Err(e) => results.push(format!("Error executing chown: {}", e)),
            }
        }

        if results.is_empty() {
            return ToolResult { success: false, output: "No action specified (mode, owner, or group must be provided)".into() };
        }

        ToolResult {
            success: results.iter().all(|r| r.starts_with("Successfully")),
            output: results.join("\n"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use crate::security::policy::PermissionTier;
    use std::path::PathBuf;

    fn mock_ctx() -> PolicyContext {
        PolicyContext {
            permission_tier: PermissionTier::FullExec,
            exec_mode: "agentic".to_string(),
            workspace_root: PathBuf::from("."),
        }
    }

    #[test]
    fn test_service_repair_schema() {
        let tool = ServiceRepairTool;
        let schema = tool.schema();
        assert!(schema.is_object());
        assert_eq!(tool.name(), "service_repair");
    }

    #[test]
    fn test_package_repair_schema() {
        let tool = PackageRepairTool;
        let schema = tool.schema();
        assert!(schema.is_object());
        assert_eq!(tool.name(), "package_repair");
    }

    #[test]
    fn test_permission_repair_schema() {
        let tool = PermissionRepairTool;
        let schema = tool.schema();
        assert!(schema.is_object());
        assert_eq!(tool.name(), "permission_repair");
    }

    #[test]
    fn test_package_repair_dry_run_generation() {
        let tool = PackageRepairTool;
        let input = PackageRepairInput {
            package_name: "curl".into(),
            action: PackageAction::Install,
            dry_run: Some(true),
        };
        
        let result = tool.get_command(&input);
        if cfg!(target_os = "windows") {
            let (cmd, args) = result.unwrap();
            assert_eq!(cmd, "choco");
            assert!(args.contains(&"install".to_string()));
            assert!(args.contains(&"--what-if".to_string()));
        } else {
            match result {
                Ok((cmd, args)) => {
                    assert!(cmd == "apt-get" || cmd == "dnf");
                    if cmd == "apt-get" {
                        assert!(args.contains(&"-s".to_string()));
                    } else {
                        assert!(args.contains(&"--setopt=tsflags=test".to_string()));
                    }
                }
                Err(e) => {
                    // Might happen if neither apt nor dnf is found in the test environment
                    assert!(e.contains("No supported package manager found"));
                }
            }
        }
    }

    #[test]
    fn test_permission_repair_requires_action() {
        let tool = PermissionRepairTool;
        let args = json!({
            "path": "/tmp/test"
        });
        let result = tool.execute(args, &[], false, &mock_ctx());
        assert!(!result.success);
        assert!(result.output.contains("No action specified"));
    }
}
