/// PAL types: enums and structs for the Platform Abstraction Layer.

// ── Enums ────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Platform {
    Android,
    Ios,
    Windows,
    Macos,
    Linux,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DeviceClass {
    Mobile,
    Desktop,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CapabilityType {
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
}

// ── Structs ──────────────────────────────────────────────────────────────────

/// Human-readable degradation hint returned when a capability is unavailable.
#[derive(Debug, Clone)]
pub struct DegradedFallback {
    pub description: String,
    pub alternative_capability: Option<CapabilityType>,
    pub workaround: Option<String>,
}

/// Result of a capability query — never panics, always returns a value.
#[derive(Debug, Clone)]
pub struct CapabilityResult {
    pub capability: CapabilityType,
    pub available: bool,
    pub degraded_fallback: Option<DegradedFallback>,
}

/// Hardware and OS descriptor for the current device.
#[derive(Debug, Clone)]
pub struct DeviceProfile {
    pub platform: Platform,
    pub device_class: DeviceClass,
    pub ram_gb: f32,
    pub has_gpu: bool,
    pub gpu_vram_gb: Option<f32>,
    pub cpu_cores: u32,
    /// `None` on plugged-in desktops; `Some(0.0..=1.0)` on mobile / laptops.
    pub battery_level: Option<f32>,
    pub is_charging: Option<bool>,
    pub storage_available_gb: f32,
    pub os_version: String,
}

/// Battery level below which the PAL activates battery-aware degradation.
pub const BATTERY_LOW_THRESHOLD: f32 = 0.2;
