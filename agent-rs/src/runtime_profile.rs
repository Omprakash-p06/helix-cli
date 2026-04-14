use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RuntimeProfile {
    LatencyCpu,
    BalancedCpu,
    SafeRecovery,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileSettings {
    pub ttft_target_ms: u64,
    pub batch_size: usize,
    pub ubatch_size: usize,
    pub cpu_threads: usize,
    pub context_size: usize,
}

impl RuntimeProfile {
    pub fn settings(&self, current_threads: usize, current_context: usize) -> ProfileSettings {
        match self {
            RuntimeProfile::LatencyCpu => ProfileSettings {
                ttft_target_ms: 1500,
                batch_size: 256,
                ubatch_size: 256,
                cpu_threads: current_threads.max(4),
                context_size: current_context.min(4096),
            },
            RuntimeProfile::BalancedCpu => ProfileSettings {
                ttft_target_ms: 3000,
                batch_size: 512,
                ubatch_size: 512,
                cpu_threads: current_threads,
                context_size: current_context,
            },
            RuntimeProfile::SafeRecovery => ProfileSettings {
                ttft_target_ms: 8000,
                batch_size: 128,
                ubatch_size: 128,
                cpu_threads: 2,
                context_size: 2048,
            },
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            RuntimeProfile::LatencyCpu => "latency_cpu",
            RuntimeProfile::BalancedCpu => "balanced_cpu",
            RuntimeProfile::SafeRecovery => "safe_recovery",
        }
    }
}

pub fn select_runtime_profile(is_cpu_only: bool, cpu_cores: usize) -> RuntimeProfile {
    if is_cpu_only {
        if cpu_cores <= 4 {
            RuntimeProfile::LatencyCpu
        } else {
            RuntimeProfile::BalancedCpu
        }
    } else {
        // GPU assisted systems default to balanced unless specified
        RuntimeProfile::BalancedCpu
    }
}
