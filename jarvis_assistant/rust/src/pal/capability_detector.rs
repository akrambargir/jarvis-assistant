/// CapabilityDetector — detects the current platform and maps capabilities.
///
/// The capability matrix (from design doc):
///
/// | Capability          | Android  | iOS      | Windows | macOS   | Linux   |
/// |---------------------|----------|----------|---------|---------|---------|
/// | AUDIO_CAPTURE       | full     | full     | full    | full    | full    |
/// | CAMERA              | full     | full     | degraded| degraded| degraded|
/// | SCREEN_CAPTURE      | degraded | none     | full    | full    | full    |
/// | OS_CONTROL_FULL     | degraded | degraded | full    | full    | full    |
/// | SHELL_COMMANDS      | degraded | none     | full    | full    | full    |
/// | GLOBAL_HOTKEY       | none     | none     | full    | full    | full    |
/// | SYSTEM_TRAY         | none     | none     | full    | full    | full    |
/// | FOREGROUND_SERVICE  | full     | none     | none    | none    | none    |
/// | BACKGROUND_AUDIO    | full     | full     | full    | full    | full    |
/// | LORA_TRAINING       | degraded | none     | full    | full    | full    |
/// | BROWSER_AUTOMATION  | none     | none     | full    | full    | full    |
/// | IOT_HUB             | degraded | degraded | full    | full    | full    |
/// | LOCAL_LLM           | full     | full     | full    | full    | full    |

use crate::pal::types::{
    CapabilityResult, CapabilityType, DegradedFallback, DeviceClass, DeviceProfile, Platform,
};

pub struct CapabilityDetector {
    profile: DeviceProfile,
}

impl CapabilityDetector {
    pub fn new(profile: DeviceProfile) -> Self {
        Self { profile }
    }

    /// Returns a reference to the underlying device profile.
    pub fn profile(&self) -> &DeviceProfile {
        &self.profile
    }

    /// Returns the detected platform.
    pub fn detect_platform(&self) -> Platform {
        self.profile.platform
    }

    /// Returns all capabilities that are fully available on this platform.
    pub fn get_available_capabilities(&self) -> Vec<CapabilityType> {
        use CapabilityType::*;
        let all = [
            AudioCapture,
            Camera,
            ScreenCapture,
            OsControlFull,
            ShellCommands,
            GlobalHotkey,
            SystemTray,
            ForegroundService,
            BackgroundAudio,
            LoraTraining,
            BrowserAutomation,
            IotHub,
            LocalLlm,
        ];
        all.iter()
            .filter(|&&c| self.get_capability(c).available)
            .copied()
            .collect()
    }

    /// Returns a `DegradedFallback` when the capability is unavailable or degraded.
    pub fn get_degraded_fallback(&self, capability: CapabilityType) -> Option<DegradedFallback> {
        use CapabilityType::*;
        use Platform::*;

        match (self.profile.platform, capability) {
            // ── iOS unavailable ──────────────────────────────────────────────
            (Ios, ScreenCapture) => Some(DegradedFallback {
                description: "Screen capture is not available on iOS due to sandbox restrictions."
                    .into(),
                alternative_capability: None,
                workaround: None,
            }),
            (Ios, ShellCommands) => Some(DegradedFallback {
                description: "Shell commands are not available on iOS.".into(),
                alternative_capability: None,
                workaround: None,
            }),
            (Ios, GlobalHotkey) | (Android, GlobalHotkey) => Some(DegradedFallback {
                description: "Global hotkeys are not supported on mobile platforms.".into(),
                alternative_capability: None,
                workaround: Some("Use the in-app wake word or notification shortcut.".into()),
            }),
            (Ios, SystemTray) | (Android, SystemTray) => Some(DegradedFallback {
                description: "System tray is not available on mobile platforms.".into(),
                alternative_capability: None,
                workaround: Some("Use persistent notification as a tray substitute.".into()),
            }),
            (Ios, ForegroundService) => Some(DegradedFallback {
                description: "Android ForegroundService is not available on iOS.".into(),
                alternative_capability: None,
                workaround: Some("Use iOS Background Modes (BGTaskScheduler) instead.".into()),
            }),
            (Ios, LoraTraining) => Some(DegradedFallback {
                description: "LoRA training is not supported on iOS.".into(),
                alternative_capability: None,
                workaround: None,
            }),
            (Ios, BrowserAutomation) => Some(DegradedFallback {
                description: "Browser automation is not available on iOS.".into(),
                alternative_capability: None,
                workaround: None,
            }),

            // ── iOS degraded ─────────────────────────────────────────────────
            (Ios, OsControlFull) => Some(DegradedFallback {
                description: "Full OS control is not available on iOS due to sandboxing.".into(),
                alternative_capability: None,
                workaround: Some("Use Shortcuts/Siri integration as a workaround.".into()),
            }),
            (Ios, IotHub) => Some(DegradedFallback {
                description: "IoT hub control is limited on iOS.".into(),
                alternative_capability: None,
                workaround: Some("Use HomeKit or third-party app integrations.".into()),
            }),

            // ── Android unavailable ──────────────────────────────────────────
            (Android, BrowserAutomation) => Some(DegradedFallback {
                description: "Browser automation via Playwright is not available on Android."
                    .into(),
                alternative_capability: None,
                workaround: None,
            }),
            (Android, ForegroundService) => None, // full support — no fallback needed

            // ── Android degraded ─────────────────────────────────────────────
            (Android, ScreenCapture) => Some(DegradedFallback {
                description: "Screen capture on Android requires MediaProjection permission."
                    .into(),
                alternative_capability: None,
                workaround: Some(
                    "Request MediaProjection permission and use the Accessibility API.".into(),
                ),
            }),
            (Android, OsControlFull) => Some(DegradedFallback {
                description: "Full OS control is limited on Android.".into(),
                alternative_capability: None,
                workaround: Some("Use ADB or the Accessibility API for supported actions.".into()),
            }),
            (Android, ShellCommands) => Some(DegradedFallback {
                description: "Shell commands on Android are restricted to ADB-accessible commands."
                    .into(),
                alternative_capability: None,
                workaround: Some("Use ADB shell or root access if available.".into()),
            }),
            (Android, LoraTraining) => Some(DegradedFallback {
                description: "LoRA training on Android is limited by RAM and compute.".into(),
                alternative_capability: None,
                workaround: Some("Use a smaller base model and reduced batch size.".into()),
            }),
            (Android, IotHub) => Some(DegradedFallback {
                description: "IoT hub control on Android may be limited by background restrictions."
                    .into(),
                alternative_capability: None,
                workaround: Some("Use a foreground service to maintain IoT connections.".into()),
            }),

            // ── Desktop platforms: Windows/macOS/Linux — no fallback needed for most ──
            _ => None,
        }
    }

    /// Returns a `CapabilityResult` for the given capability.
    /// This method NEVER panics — it always returns a value.
    pub fn get_capability(&self, capability: CapabilityType) -> CapabilityResult {
        use CapabilityType::*;
        use Platform::*;

        let available = match (self.profile.platform, capability) {
            // Fully unavailable combinations
            (Ios, ScreenCapture)
            | (Ios, ShellCommands)
            | (Ios, GlobalHotkey)
            | (Ios, SystemTray)
            | (Ios, ForegroundService)
            | (Ios, LoraTraining)
            | (Ios, BrowserAutomation)
            | (Android, GlobalHotkey)
            | (Android, SystemTray)
            | (Android, BrowserAutomation)
            | (Windows, ForegroundService)
            | (Macos, ForegroundService)
            | (Linux, ForegroundService) => false,

            // Everything else is available (full or degraded — both count as available)
            _ => true,
        };

        let degraded_fallback = if available {
            // For degraded-but-available capabilities, still provide the fallback hint
            self.get_degraded_fallback(capability)
        } else {
            self.get_degraded_fallback(capability)
        };

        CapabilityResult {
            capability,
            available,
            degraded_fallback,
        }
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pal::types::{DeviceClass, BATTERY_LOW_THRESHOLD};

    fn all_capabilities() -> [CapabilityType; 13] {
        use CapabilityType::*;
        [
            AudioCapture, Camera, ScreenCapture, OsControlFull, ShellCommands,
            GlobalHotkey, SystemTray, ForegroundService, BackgroundAudio,
            LoraTraining, BrowserAutomation, IotHub, LocalLlm,
        ]
    }

    fn all_platforms() -> [(Platform, DeviceClass); 5] {
        [
            (Platform::Android, DeviceClass::Mobile),
            (Platform::Ios, DeviceClass::Mobile),
            (Platform::Windows, DeviceClass::Desktop),
            (Platform::Macos, DeviceClass::Desktop),
            (Platform::Linux, DeviceClass::Desktop),
        ]
    }

    fn make_profile(platform: Platform, device_class: DeviceClass) -> DeviceProfile {
        DeviceProfile {
            platform,
            device_class,
            ram_gb: 8.0,
            has_gpu: false,
            gpu_vram_gb: None,
            cpu_cores: 4,
            battery_level: Some(0.8),
            is_charging: None,
            storage_available_gb: 32.0,
            os_version: "test".to_string(),
        }
    }

    // Property 20: get_capability never panics; always returns CapabilityResult
    #[test]
    fn get_capability_never_panics_for_all_platform_capability_combinations() {
        for (platform, device_class) in all_platforms() {
            let detector = CapabilityDetector::new(make_profile(platform, device_class));
            for cap in all_capabilities() {
                let result = detector.get_capability(cap);
                // capability field must round-trip
                assert_eq!(result.capability, cap);
            }
        }
    }

    // Property 20: detect_platform always returns a value in the Platform enum
    #[test]
    fn detect_platform_always_returns_valid_platform() {
        let valid = [
            Platform::Android, Platform::Ios, Platform::Windows,
            Platform::Macos, Platform::Linux,
        ];
        for (platform, device_class) in all_platforms() {
            let detector = CapabilityDetector::new(make_profile(platform, device_class));
            let p = detector.detect_platform();
            assert!(valid.contains(&p), "detect_platform returned unexpected value");
        }
    }

    // iOS screen capture must be unavailable with a non-null degraded fallback
    #[test]
    fn ios_screen_capture_unavailable_with_fallback() {
        let detector = CapabilityDetector::new(make_profile(Platform::Ios, DeviceClass::Mobile));
        let result = detector.get_capability(CapabilityType::ScreenCapture);
        assert!(!result.available);
        assert!(result.degraded_fallback.is_some());
    }

    // LocalLlm must be available on every platform
    #[test]
    fn local_llm_available_on_all_platforms() {
        for (platform, device_class) in all_platforms() {
            let detector = CapabilityDetector::new(make_profile(platform, device_class));
            let result = detector.get_capability(CapabilityType::LocalLlm);
            assert!(result.available, "LocalLlm must be available on {platform:?}");
        }
    }
}
