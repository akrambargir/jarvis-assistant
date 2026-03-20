/// Connectivity Monitor
///
/// Probes DNS every 5 seconds and emits a `ConnectivityStatus`.
/// The monitor runs in a background thread; callers read the latest
/// status via `ConnectivityMonitor::status()` (lock-free atomic read).

use std::net::ToSocketAddrs;
use std::sync::atomic::{AtomicU8, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

// ── ConnectivityStatus ────────────────────────────────────────────────────────

/// Connectivity state emitted by `ConnectivityMonitor`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectivityStatus {
    /// DNS probe succeeded; cloud backends are reachable.
    Online,
    /// DNS probe failed; no network path available.
    Offline,
    /// DNS probe succeeded but a recent cloud call failed; partial connectivity.
    Degraded,
}

impl ConnectivityStatus {
    fn to_u8(self) -> u8 {
        match self {
            ConnectivityStatus::Online => 0,
            ConnectivityStatus::Offline => 1,
            ConnectivityStatus::Degraded => 2,
        }
    }

    fn from_u8(v: u8) -> Self {
        match v {
            0 => ConnectivityStatus::Online,
            1 => ConnectivityStatus::Offline,
            _ => ConnectivityStatus::Degraded,
        }
    }
}

// ── ConnectivityMonitor ───────────────────────────────────────────────────────

/// Probe interval used by the background thread.
const PROBE_INTERVAL: Duration = Duration::from_secs(5);

/// DNS host used to test connectivity (port 53 is the standard DNS port).
const PROBE_HOST: &str = "dns.google:53";

/// Monitors network connectivity by probing DNS every 5 seconds.
///
/// The latest `ConnectivityStatus` is stored in an `AtomicU8` so that
/// `status()` is always a cheap, lock-free read.
pub struct ConnectivityMonitor {
    status: Arc<AtomicU8>,
}

impl ConnectivityMonitor {
    /// Create a new monitor and immediately start the background probe thread.
    pub fn new() -> Self {
        let status = Arc::new(AtomicU8::new(ConnectivityStatus::Online.to_u8()));
        let status_clone = Arc::clone(&status);

        thread::Builder::new()
            .name("connectivity-monitor".into())
            .spawn(move || loop {
                let new_status = if probe_dns() {
                    ConnectivityStatus::Online
                } else {
                    ConnectivityStatus::Offline
                };
                status_clone.store(new_status.to_u8(), Ordering::Relaxed);
                thread::sleep(PROBE_INTERVAL);
            })
            .expect("failed to spawn connectivity monitor thread");

        Self { status }
    }

    /// Returns the most recently observed `ConnectivityStatus`.
    pub fn status(&self) -> ConnectivityStatus {
        ConnectivityStatus::from_u8(self.status.load(Ordering::Relaxed))
    }

    /// Manually override the status (used by `ModelRouter` to signal DEGRADED
    /// after a cloud-call failure without waiting for the next DNS probe).
    pub fn set_degraded(&self) {
        self.status
            .store(ConnectivityStatus::Degraded.to_u8(), Ordering::Relaxed);
    }

    /// Force an immediate DNS probe and update the stored status.
    /// Returns the new status.
    pub fn probe_now(&self) -> ConnectivityStatus {
        let new_status = if probe_dns() {
            ConnectivityStatus::Online
        } else {
            ConnectivityStatus::Offline
        };
        self.status.store(new_status.to_u8(), Ordering::Relaxed);
        new_status
    }
}

impl Default for ConnectivityMonitor {
    fn default() -> Self {
        Self::new()
    }
}

/// Test-only constructor that creates a monitor with a pre-set status
/// without spawning a background thread.
#[cfg(test)]
impl ConnectivityMonitor {
    pub(crate) fn with_status(status: ConnectivityStatus) -> Self {
        Self {
            status: Arc::new(AtomicU8::new(status.to_u8())),
        }
    }
}

// ── DNS probe ─────────────────────────────────────────────────────────────────

/// Returns `true` when the DNS probe host is resolvable / reachable.
fn probe_dns() -> bool {
    PROBE_HOST.to_socket_addrs().is_ok()
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn status_roundtrip() {
        for s in [
            ConnectivityStatus::Online,
            ConnectivityStatus::Offline,
            ConnectivityStatus::Degraded,
        ] {
            assert_eq!(ConnectivityStatus::from_u8(s.to_u8()), s);
        }
    }

    #[test]
    fn set_degraded_overrides_status() {
        let monitor = ConnectivityMonitor {
            status: Arc::new(AtomicU8::new(ConnectivityStatus::Online.to_u8())),
        };
        monitor.set_degraded();
        assert_eq!(monitor.status(), ConnectivityStatus::Degraded);
    }
}
