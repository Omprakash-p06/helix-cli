#[path = "../src/runtime_profile.rs"]
mod runtime_profile;
#[path = "../src/watchdog.rs"]
mod watchdog;

use runtime_profile::{RuntimeProfile, select_runtime_profile};
use watchdog::{Watchdog, WatchdogState};
use std::time::Duration;

#[test]
fn test_runtime_profile_selection() {
    assert_eq!(select_runtime_profile(true, 4), RuntimeProfile::LatencyCpu);
    assert_eq!(select_runtime_profile(true, 8), RuntimeProfile::BalancedCpu);
    assert_eq!(select_runtime_profile(false, 16), RuntimeProfile::BalancedCpu);
}

#[test]
fn test_profile_settings() {
    let settings = RuntimeProfile::LatencyCpu.settings(8, 8192);
    assert_eq!(settings.ttft_target_ms, 1500);
    assert_eq!(settings.cpu_threads, 8);
    assert_eq!(settings.context_size, 4096);
}

#[test]
fn test_watchdog_restarts() {
    let mut wd = Watchdog::new(2, 60);
    assert_eq!(wd.state(), WatchdogState::Healthy);
    assert!(wd.can_restart());

    // 1st failure: Degraded
    wd.on_failure();
    assert_eq!(wd.state(), WatchdogState::Degraded);
    assert!(wd.can_restart());

    // 2nd failure: Recovering (Attempt 1)
    wd.on_failure();
    assert_eq!(wd.state(), WatchdogState::Recovering);
    assert!(wd.can_restart());

    // 3rd failure: Recovering (Attempt 2)
    wd.on_failure();
    assert_eq!(wd.state(), WatchdogState::Recovering);
    assert!(!wd.can_restart());

    // 4th failure: Cooldown (Exceeded budget)
    wd.on_failure();
    assert_eq!(wd.state(), WatchdogState::Cooldown);
    assert!(!wd.can_restart());
}

#[test]
fn test_watchdog_success_reset() {
    let mut wd = Watchdog::new(2, 60);
    wd.on_failure();
    wd.on_failure();
    wd.on_success();
    assert_eq!(wd.state(), WatchdogState::Healthy);
    assert!(wd.can_restart());
}

#[test]
fn test_watchdog_cooldown_lockout() {
    let mut wd = Watchdog::new(1, 60);
    wd.on_failure(); // Degraded
    wd.on_failure(); // Recovering (Attempt 1)
    wd.on_failure(); // Cooldown
    assert_eq!(wd.state(), WatchdogState::Cooldown);
    assert!(!wd.can_restart());

    // Even if we fail again, we stay in cooldown and cannot restart
    let (state, _) = wd.on_failure();
    assert_eq!(state, WatchdogState::Cooldown);
    assert!(!wd.can_restart());
}

#[test]
fn test_watchdog_backoff() {
    let mut wd = Watchdog::new(5, 60);
    assert_eq!(wd.next_backoff(), Duration::from_secs(0));
    wd.on_failure(); // Degraded
    wd.on_failure(); // Attempt 1
    assert_eq!(wd.next_backoff(), Duration::from_secs(2));
    wd.on_failure(); // Attempt 2
    assert_eq!(wd.next_backoff(), Duration::from_secs(10));
}
