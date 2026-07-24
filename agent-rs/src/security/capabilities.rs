use std::collections::BTreeSet;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Capability {
    ReadFile,
    ListDirectory,
    SearchCodebase,
    SystemDiagnostic,
    WriteFile,
    AppendFile,
    EditFile,
    ExecuteSandboxed,   // Docker sandbox only
    ExecuteNative,      // Direct host execution (dangerous)
    NetworkAccess,
    ServiceControl,
    CapabilityElevation, // Can grant temporary elevation
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CapabilitySet(BTreeSet<Capability>);

impl CapabilitySet {
    /// ReadOnly: safe inspection only, no writes, no execution
    pub fn read_only() -> Self {
        let caps = [
            Capability::ReadFile,
            Capability::ListDirectory,
            Capability::SearchCodebase,
            Capability::SystemDiagnostic,
        ];
        Self(caps.into_iter().collect())
    }

    /// GuidedRepair: workspace writes + sandboxed execution only
    pub fn guided_repair() -> Self {
        let mut set = Self::read_only().0;
        set.insert(Capability::WriteFile);
        set.insert(Capability::AppendFile);
        set.insert(Capability::EditFile);
        set.insert(Capability::ExecuteSandboxed);
        Self(set)
    }

    /// Autonomous: full capabilities minus direct network and CapabilityElevation
    pub fn autonomous() -> Self {
        let mut set = Self::guided_repair().0;
        set.insert(Capability::ExecuteNative);
        set.insert(Capability::ServiceControl);
        Self(set)
    }

    pub fn has(&self, cap: Capability) -> bool {
        self.0.contains(&cap)
    }

    pub fn grant(&mut self, cap: Capability) {
        self.0.insert(cap);
    }

    pub fn revoke(&mut self, cap: Capability) {
        self.0.remove(&cap);
    }
}

/// Map a tool name to the capabilities it requires
pub fn required_capabilities(tool_name: &str) -> Vec<Capability> {
    match tool_name {
        "read_file" => vec![Capability::ReadFile],
        "list_directory" => vec![Capability::ListDirectory],
        "search_codebase" => vec![Capability::SearchCodebase],
        "get_system_stats" | "list_processes" | "get_service_status"
        | "search_system_files" | "get_system_logs" => vec![Capability::SystemDiagnostic],
        "write_file" => vec![Capability::WriteFile],
        "append_file" => vec![Capability::AppendFile],
        "edit_file" => vec![Capability::EditFile],
        "run_terminal_command" => vec![Capability::ExecuteSandboxed],
        "service_repair" => vec![Capability::ExecuteSandboxed, Capability::ServiceControl],
        "package_repair" => vec![Capability::ExecuteSandboxed],
        "permission_repair" => vec![Capability::ExecuteSandboxed, Capability::ServiceControl],
        _ => vec![Capability::ExecuteSandboxed],
    }
}
