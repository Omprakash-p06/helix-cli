use std::time::{Duration, Instant};

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
