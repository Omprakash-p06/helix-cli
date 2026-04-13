#[path = "../src/runtime_profile.rs"]
mod runtime_profile;
#[path = "../src/watchdog.rs"]
mod watchdog;

use runtime_profile::{RuntimeProfile, select_runtime_profile};
use watchdog::{Watchdog, WatchdogState};
use std::time::Duration;

#[test]
fn runtime_profile_string_ids_are_stable() {
    assert_eq!(RuntimeProfile::LatencyCpu.as_str(), "latency_cpu");
    assert_eq!(RuntimeProfile::BalancedCpu.as_str(), "balanced_cpu");
    assert_eq!(RuntimeProfile::SafeRecovery.as_str(), "safe_recovery");
}

#[test]
fn safe_recovery_profile_is_conservative() {
    let settings = RuntimeProfile::SafeRecovery.settings(16, 8192);
    assert_eq!(settings.ttft_target_ms, 8000);
    assert_eq!(settings.batch_size, 128);
    assert_eq!(settings.ubatch_size, 128);
    assert_eq!(settings.cpu_threads, 2);
    assert_eq!(settings.context_size, 2048);
}

#[test]
fn cpu_only_low_core_hosts_choose_latency_profile() {
    assert_eq!(select_runtime_profile(true, 2), RuntimeProfile::LatencyCpu);
    assert_eq!(select_runtime_profile(true, 4), RuntimeProfile::LatencyCpu);
}

#[test]
fn watchdog_backoff_caps_after_many_failures() {
    let mut wd = Watchdog::new(10, 60);
    for _ in 0..6 {
        let _ = wd.on_failure();
    }
    assert_eq!(wd.state(), WatchdogState::Recovering);
    assert_eq!(wd.next_backoff(), Duration::from_secs(60));
}

#[test]
fn watchdog_success_from_cooldown_resets_state() {
    let mut wd = Watchdog::new(1, 60);
    let _ = wd.on_failure();
    let _ = wd.on_failure();
    let _ = wd.on_failure();
    assert_eq!(wd.state(), WatchdogState::Cooldown);

    wd.on_success();
    assert_eq!(wd.state(), WatchdogState::Healthy);
    assert!(wd.can_restart());
}
