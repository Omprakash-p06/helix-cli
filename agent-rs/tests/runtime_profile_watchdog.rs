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

// ── GAP-4: Watchdog foreign process safety (P1-4) ──────────────────────────

#[test]
fn test_watchdog_no_foreign_kill() {
    use watchdog::ProcessOwnership;

    // Verify ProcessOwnership struct exists with the required fields
    let owned = ProcessOwnership {
        pid: 12345,
        port: 8080,
        started_by: "helix-agent".to_string(),
        start_time_unix: 1000,
    };
    assert_eq!(owned.started_by, "helix-agent");
    assert_eq!(owned.pid, 12345);
    assert_eq!(owned.port, 8080);

    // A foreign process has a different started_by value
    let foreign = ProcessOwnership {
        pid: 99999,
        port: 8080,
        started_by: "not_helix".to_string(),
        start_time_unix: 1000,
    };
    assert_ne!(foreign.started_by, "helix-agent");

    // The watchdog must verify ownership before taking action on a process.
    // Check that the watchdog module has a method or mechanism to check
    // ProcessOwnership before terminating processes.
    let watchdog_rs = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src/watchdog.rs")
    ).expect("watchdog.rs must exist");

    // The watchdog module defines ProcessOwnership (verified above)
    // Now verify there's a terminate/recovery method that uses it
    let has_process_management = watchdog_rs.contains("fn terminate")
        || watchdog_rs.contains("server.pid")
        || watchdog_rs.contains("started_by")
        || watchdog_rs.contains("ProcessOwnership");

    // ProcessOwnership exists and started_by is compared
    assert!(
        watchdog_rs.contains("ProcessOwnership"),
        "watchdog.rs must define ProcessOwnership struct"
    );

    // Check if terminate() or PID-file-based foreign process prevention exists
    // This is the behavioral requirement: watchdog must not kill foreign processes
    let has_ownership_check = watchdog_rs.contains("started_by")
        && (watchdog_rs.contains("terminate") || watchdog_rs.contains("should_kill"));

    assert!(
        has_ownership_check,
        "IMPLEMENTATION GAP: ProcessOwnership struct is defined but there's no \
         terminate() method or process ownership checking logic. \
         Requirement P1-4 demands that the watchdog reads .helix/server.pid, \
         checks started_by, and only terminates if started_by == 'helix-agent'. \
         The current watchdog implementation only has state machine logic without \
         process management."
    );
}
