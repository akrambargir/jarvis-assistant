/// NotificationAdapter stubs for Android and iOS.
///
/// Desktop platforms (Windows/macOS/Linux) use OS-native toast/notification APIs
/// that are wired up through the Flutter layer; these stubs cover the mobile path.

use crate::pal::types::Platform;

// ── Data types ────────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct LocalNotification {
    pub id: String,
    pub title: String,
    pub body: String,
    pub channel_id: Option<String>, // Android notification channel
}

#[derive(Debug, Clone)]
pub struct PushNotification {
    pub token: String,
    pub title: String,
    pub body: String,
    pub data: std::collections::HashMap<String, String>,
}

/// Android-specific notification channel configuration.
#[derive(Debug, Clone)]
pub struct NotificationChannel {
    pub id: String,
    pub name: String,
    pub importance: ChannelImportance,
}

#[derive(Debug, Clone, Copy)]
pub enum ChannelImportance {
    Low,
    Default,
    High,
    Max,
}

// ── Adapter ───────────────────────────────────────────────────────────────────

pub struct NotificationAdapter {
    platform: Platform,
}

impl NotificationAdapter {
    pub fn new(platform: Platform) -> Self {
        Self { platform }
    }

    /// Send a local (on-device) notification.
    /// Android: uses NotificationCompat via JNI (stub).
    /// iOS: uses UNUserNotificationCenter via FFI (stub).
    pub fn send_local(&self, notification: &LocalNotification) -> anyhow::Result<()> {
        match self.platform {
            Platform::Android => {
                // TODO: call Android NotificationCompat via JNI
                log::debug!(
                    "[Android] send_local: id={} title={}",
                    notification.id,
                    notification.title
                );
                Ok(())
            }
            Platform::Ios => {
                // TODO: call UNUserNotificationCenter via iOS FFI
                log::debug!(
                    "[iOS] send_local: id={} title={}",
                    notification.id,
                    notification.title
                );
                Ok(())
            }
            _ => {
                // Desktop: delegated to Flutter layer
                log::debug!(
                    "[Desktop] send_local delegated to Flutter: {}",
                    notification.title
                );
                Ok(())
            }
        }
    }

    /// Send a push notification via FCM (Android) or APNs (iOS).
    pub fn send_push(&self, notification: &PushNotification) -> anyhow::Result<()> {
        match self.platform {
            Platform::Android => {
                // TODO: enqueue FCM push via HTTP API
                log::debug!("[Android] send_push to token={}", notification.token);
                Ok(())
            }
            Platform::Ios => {
                // TODO: enqueue APNs push via HTTP/2 API
                log::debug!("[iOS] send_push to token={}", notification.token);
                Ok(())
            }
            _ => {
                log::debug!("[Desktop] push notifications not applicable");
                Ok(())
            }
        }
    }

    /// Request notification permission from the OS.
    /// Returns `true` if permission was granted (stub always returns `true`).
    pub fn request_permission(&self) -> bool {
        match self.platform {
            Platform::Android | Platform::Ios => {
                // TODO: invoke platform permission dialog
                log::debug!("[{:?}] request_permission (stub → true)", self.platform);
                true
            }
            _ => true,
        }
    }

    /// Cancel a previously scheduled or displayed notification by ID.
    pub fn cancel_notification(&self, id: &str) -> anyhow::Result<()> {
        log::debug!("[{:?}] cancel_notification id={}", self.platform, id);
        // TODO: platform-specific cancellation
        Ok(())
    }

    /// Configure an Android notification channel (no-op on other platforms).
    pub fn set_notification_channel(&self, channel: &NotificationChannel) -> anyhow::Result<()> {
        if self.platform == Platform::Android {
            // TODO: create NotificationChannel via JNI
            log::debug!(
                "[Android] set_notification_channel id={} name={}",
                channel.id,
                channel.name
            );
        }
        Ok(())
    }
}
