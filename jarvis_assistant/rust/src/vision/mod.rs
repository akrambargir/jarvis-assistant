/// Vision module — Camera and Screen Capture adapters + visual feature extraction.

use crate::pal::types::{CapabilityType, DegradedFallback, Platform};

// ── 14.1 CameraAdapter ───────────────────────────────────────────────────────

pub struct CameraAdapter {
    pub platform: Platform,
    pub available: bool,
}

impl CameraAdapter {
    pub fn for_platform(platform: Platform) -> Self {
        // All platforms support camera
        Self { platform, available: true }
    }

    /// Capture a frame. Stub: returns empty bytes.
    pub fn capture_frame(&self) -> anyhow::Result<Vec<u8>> {
        Ok(vec![])
    }
}

// ── 14.2 ScreenCaptureAdapter ────────────────────────────────────────────────

pub struct ScreenCaptureResult {
    pub available: bool,
    pub data: Option<Vec<u8>>,
    pub degraded_fallback: Option<DegradedFallback>,
}

pub struct ScreenCaptureAdapter {
    pub platform: Platform,
}

impl ScreenCaptureAdapter {
    pub fn for_platform(platform: Platform) -> Self {
        Self { platform }
    }

    /// Capture screen. iOS always returns available=false with DegradedFallback.
    /// Android returns degraded (limited). Windows/macOS/Linux return full support.
    pub fn capture(&self) -> ScreenCaptureResult {
        match self.platform {
            Platform::Ios => ScreenCaptureResult {
                available: false,
                data: None,
                degraded_fallback: Some(DegradedFallback {
                    description: "Screen capture not available on iOS due to sandbox restrictions."
                        .to_string(),
                    alternative_capability: Some(CapabilityType::Camera),
                    workaround: Some("Use camera input instead.".to_string()),
                }),
            },
            Platform::Android => ScreenCaptureResult {
                available: true,
                data: Some(vec![]),
                degraded_fallback: Some(DegradedFallback {
                    description:
                        "Screen capture on Android requires MediaProjection API permission."
                            .to_string(),
                    alternative_capability: None,
                    workaround: None,
                }),
            },
            _ => ScreenCaptureResult {
                available: true,
                data: Some(vec![]),
                degraded_fallback: None,
            },
        }
    }
}

// ── 14.3 VisualFeatureExtractor (LLaVA/CLIP stub) ───────────────────────────

pub struct VisualFeatures {
    pub embeddings: Vec<f32>,
    pub detected_objects: Vec<String>,
    pub scene_description: String,
}

pub struct VisualFeatureExtractor;

impl VisualFeatureExtractor {
    /// Extract visual features from image bytes. Stub: returns zero embeddings.
    pub fn extract(image: &[u8]) -> VisualFeatures {
        let _ = image;
        VisualFeatures {
            embeddings: vec![0.0; 512],
            detected_objects: vec![],
            scene_description: String::new(),
        }
    }
}

// ── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ios_screen_capture_returns_not_available() {
        let adapter = ScreenCaptureAdapter::for_platform(Platform::Ios);
        let result = adapter.capture();
        assert!(!result.available);
        assert!(result.degraded_fallback.is_some());
    }

    #[test]
    fn desktop_screen_capture_returns_available() {
        let adapter = ScreenCaptureAdapter::for_platform(Platform::Linux);
        let result = adapter.capture();
        assert!(result.available);
        assert!(result.degraded_fallback.is_none());
    }

    #[test]
    fn camera_adapter_available_on_all_platforms() {
        let platforms = [
            Platform::Android,
            Platform::Ios,
            Platform::Windows,
            Platform::Macos,
            Platform::Linux,
        ];
        for platform in platforms {
            let adapter = CameraAdapter::for_platform(platform);
            assert!(adapter.available, "Camera should be available on {platform:?}");
        }
    }

    // Property 20 variant: SCREEN_CAPTURE on iOS always returns available=false
    // with non-null degradedFallback.
    #[test]
    fn ios_screen_capture_always_unavailable_with_non_null_fallback() {
        let adapter = ScreenCaptureAdapter::for_platform(Platform::Ios);
        let result = adapter.capture();
        assert!(!result.available, "iOS screen capture must be unavailable");
        assert!(
            result.degraded_fallback.is_some(),
            "iOS screen capture must have a non-null DegradedFallback"
        );
        assert!(result.data.is_none(), "iOS screen capture must return no data");
    }

    // Complement: desktop screen capture is available with no fallback.
    #[test]
    fn desktop_screen_capture_available_no_fallback() {
        for platform in [Platform::Windows, Platform::Macos, Platform::Linux] {
            let adapter = ScreenCaptureAdapter::for_platform(platform);
            let result = adapter.capture();
            assert!(result.available, "{platform:?} screen capture must be available");
            assert!(result.degraded_fallback.is_none());
        }
    }
}
