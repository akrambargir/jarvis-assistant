/// BackgroundExecutionAdapter stubs.
///
/// Android: ForegroundService keeps the process alive while the assistant is active.
/// iOS:     BGTaskScheduler / Background Modes handle deferred and periodic work.
/// Desktop: Background execution is unrestricted; stubs are no-ops.

use crate::pal::types::Platform;

// ── Data types ────────────────────────────────────────────────────────────────

/// Configuration for an Android foreground service.
#[derive(Debug, Clone)]
pub struct ServiceConfig {
    pub notification_title: String,
    pub notification_body: String,
    pub channel_id: String,
}

/// A task to be scheduled for deferred background execution.
#[derive(Debug, Clone)]
pub struct BackgroundTask {
    pub task_id: String,
    pub earliest_begin_seconds: u64, // minimum delay before execution
    pub requires_network: bool,
    pub requires_charging: bool,
}

/// iOS background mode registration type.
#[derive(Debug, Clone, Copy)]
pub enum BackgroundMode {
    /// `audio` — keeps audio session alive in background.
    Audio,
    /// `fetch` — periodic background fetch.
    Fetch,
    /// `processing` — BGProcessingTask for heavy work.
    Processing,
}

// ── Adapter ───────────────────────────────────────────────────────────────────

pub struct BackgroundExecutionAdapter {
    platform: Platform,
}

impl BackgroundExecutionAdapter {
    pub fn new(platform: Platform) -> Self {
        Self { platform }
    }

    /// Start an Android foreground service (no-op on other platforms).
    pub fn start_foreground_service(&self, config: &ServiceConfig) -> anyhow::Result<()> {
        if self.platform == Platform::Android {
            // TODO: start Android ForegroundService via JNI
            log::debug!(
                "[Android] start_foreground_service: {}",
                config.notification_title
            );
        } else {
            log::debug!(
                "[{:?}] start_foreground_service is Android-only — no-op",
                self.platform
            );
        }
        Ok(())
    }

    /// Stop the Android foreground service (no-op on other platforms).
    pub fn stop_foreground_service(&self) -> anyhow::Result<()> {
        if self.platform == Platform::Android {
            // TODO: stop Android ForegroundService via JNI
            log::debug!("[Android] stop_foreground_service");
        }
        Ok(())
    }

    /// Register an iOS background mode (no-op on other platforms).
    pub fn register_background_mode(&self, mode: BackgroundMode) -> anyhow::Result<()> {
        if self.platform == Platform::Ios {
            // TODO: register background mode via iOS FFI
            log::debug!("[iOS] register_background_mode: {:?}", mode);
        }
        Ok(())
    }

    /// Schedule a deferred background task.
    /// Android: WorkManager / JobScheduler.
    /// iOS:     BGTaskScheduler.
    /// Desktop: OS task scheduler or simple thread sleep.
    pub fn schedule_background_task(&self, task: &BackgroundTask) -> anyhow::Result<()> {
        match self.platform {
            Platform::Android => {
                // TODO: schedule via Android WorkManager JNI
                log::debug!("[Android] schedule_background_task id={}", task.task_id);
            }
            Platform::Ios => {
                // TODO: schedule via BGTaskScheduler FFI
                log::debug!("[iOS] schedule_background_task id={}", task.task_id);
            }
            _ => {
                // Desktop: no OS-level scheduling needed; handled by async runtime
                log::debug!(
                    "[{:?}] schedule_background_task id={} (async runtime)",
                    self.platform,
                    task.task_id
                );
            }
        }
        Ok(())
    }

    /// Cancel a previously scheduled background task by ID.
    pub fn cancel_background_task(&self, task_id: &str) -> anyhow::Result<()> {
        log::debug!(
            "[{:?}] cancel_background_task id={}",
            self.platform,
            task_id
        );
        // TODO: platform-specific cancellation
        Ok(())
    }

    /// Returns whether background execution is currently allowed on this platform/device.
    pub fn is_background_execution_allowed(&self) -> bool {
        match self.platform {
            // Android: allowed when a foreground service is running or battery optimisation is disabled
            Platform::Android => true, // stub — real impl checks battery optimisation exemption
            // iOS: allowed only for registered background modes
            Platform::Ios => true, // stub — real impl checks BGTaskScheduler registration
            // Desktop: always allowed
            _ => true,
        }
    }
}
