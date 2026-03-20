/// Platform Abstraction Layer (PAL)
///
/// Provides a unified interface over all platform-specific capabilities so the
/// shared Rust core never calls platform APIs directly.  Each capability has a
/// concrete implementation per platform (Android, iOS, Windows, macOS, Linux).
///
/// Key guarantee: `PlatformAbstractionLayer::get_capability` NEVER panics —
/// it always returns a `CapabilityResult`.

pub mod adapters;
pub mod capability_detector;
pub mod types;

use adapters::{
    background_execution::BackgroundExecutionAdapter, filesystem::FileSystemAdapter,
    notification::NotificationAdapter,
};
use capability_detector::CapabilityDetector;
use types::{CapabilityResult, CapabilityType, DeviceProfile, Platform};

/// Top-level PAL entry point used by the Core Engine.
pub struct PlatformAbstractionLayer {
    detector: CapabilityDetector,
    pub filesystem: FileSystemAdapter,
    pub notifications: NotificationAdapter,
    pub background_execution: BackgroundExecutionAdapter,
}

impl PlatformAbstractionLayer {
    /// Construct the PAL from a `DeviceProfile` (provided by the Flutter layer at startup).
    pub fn new(profile: DeviceProfile) -> Self {
        let platform = profile.platform;
        Self {
            detector: CapabilityDetector::new(profile),
            filesystem: FileSystemAdapter::new(platform),
            notifications: NotificationAdapter::new(platform),
            background_execution: BackgroundExecutionAdapter::new(platform),
        }
    }

    /// Query a capability — NEVER panics, always returns a `CapabilityResult`.
    pub fn get_capability(&self, capability: CapabilityType) -> CapabilityResult {
        self.detector.get_capability(capability)
    }

    /// Returns `true` if the capability is fully or partially available.
    pub fn is_available(&self, capability: CapabilityType) -> bool {
        self.detector.get_capability(capability).available
    }

    /// Returns the device profile used to initialise this PAL instance.
    pub fn get_device_profile(&self) -> &DeviceProfile {
        self.detector.profile()
    }

    /// Returns the current platform.
    pub fn get_platform(&self) -> Platform {
        self.detector.detect_platform()
    }
}
