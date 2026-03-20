/// Model Router
///
/// Routes LLM / STT / TTS / Vision workloads to either a local backend
/// (Ollama / llama.cpp) or a cloud backend, based on:
///   1. `BackendPreference` — ALWAYS_LOCAL bypasses cloud entirely.
///   2. `ConnectivityStatus` — OFFLINE forces local.
///   3. Cloud-call failure — falls back to local and marks result `degraded`.

use crate::infra::connectivity::{ConnectivityMonitor, ConnectivityStatus};

// ── Enums ─────────────────────────────────────────────────────────────────────

/// Which backend the router should prefer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BackendPreference {
    /// Use cloud when online; fall back to local on failure or offline.
    PreferCloud,
    /// Use local when possible; only use cloud if local is unavailable.
    PreferLocal,
    /// Always use local — never attempt a cloud call.
    AlwaysLocal,
}

/// The pipeline stage being routed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PipelineStage {
    Llm,
    Stt,
    Tts,
    Vision,
}

// ── Result types ──────────────────────────────────────────────────────────────

/// Response from an LLM call.
#[derive(Debug, Clone)]
pub struct LlmResponse {
    pub content: String,
    /// `true` when the result came from the local fallback after a cloud failure.
    pub degraded: bool,
}

/// Generic routed response for STT / TTS / Vision stages.
#[derive(Debug, Clone)]
pub struct RoutedResponse {
    pub payload: Vec<u8>,
    pub degraded: bool,
}

/// Which backend was actually used for a given call.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BackendUsed {
    Cloud,
    Local,
}

// ── Backend stubs ─────────────────────────────────────────────────────────────

/// Stub: call the cloud LLM backend.
/// Replace with real HTTP/gRPC call in production.
fn call_cloud_llm(prompt: &str) -> Result<String, anyhow::Error> {
    // Stub — always succeeds in tests; real impl would call cloud API.
    let _ = prompt;
    Err(anyhow::anyhow!("cloud LLM stub: not implemented"))
}

/// Stub: call the local LLM backend (Ollama / llama.cpp).
fn call_local_llm(prompt: &str) -> Result<String, anyhow::Error> {
    let _ = prompt;
    Ok(format!("[local] response to: {}", prompt))
}

/// Stub: call the cloud STT backend.
fn call_cloud_stt(audio: &[u8]) -> Result<Vec<u8>, anyhow::Error> {
    let _ = audio;
    Err(anyhow::anyhow!("cloud STT stub: not implemented"))
}

/// Stub: call the local STT backend.
fn call_local_stt(audio: &[u8]) -> Result<Vec<u8>, anyhow::Error> {
    Ok(audio.to_vec())
}

/// Stub: call the cloud TTS backend.
fn call_cloud_tts(text: &[u8]) -> Result<Vec<u8>, anyhow::Error> {
    let _ = text;
    Err(anyhow::anyhow!("cloud TTS stub: not implemented"))
}

/// Stub: call the local TTS backend.
fn call_local_tts(text: &[u8]) -> Result<Vec<u8>, anyhow::Error> {
    Ok(text.to_vec())
}

/// Stub: call the cloud Vision backend.
fn call_cloud_vision(image: &[u8]) -> Result<Vec<u8>, anyhow::Error> {
    let _ = image;
    Err(anyhow::anyhow!("cloud Vision stub: not implemented"))
}

/// Stub: call the local Vision backend.
fn call_local_vision(image: &[u8]) -> Result<Vec<u8>, anyhow::Error> {
    Ok(image.to_vec())
}

// ── ModelRouter ───────────────────────────────────────────────────────────────

/// Routes workloads to local or cloud backends.
pub struct ModelRouter {
    monitor: ConnectivityMonitor,
    preference: BackendPreference,
}

impl ModelRouter {
    /// Create a router with the given connectivity monitor and backend preference.
    pub fn new(monitor: ConnectivityMonitor, preference: BackendPreference) -> Self {
        Self { monitor, preference }
    }

    /// Create a router that always uses the local backend.
    pub fn always_local() -> Self {
        Self::new(ConnectivityMonitor::new(), BackendPreference::AlwaysLocal)
    }

    /// Returns `true` when the router should skip cloud and go straight to local.
    fn use_local_only(&self) -> bool {
        match self.preference {
            BackendPreference::AlwaysLocal => true,
            BackendPreference::PreferLocal => true,
            BackendPreference::PreferCloud => {
                self.monitor.status() == ConnectivityStatus::Offline
            }
        }
    }

    // ── Public routing methods ────────────────────────────────────────────────

    /// Route an LLM inference request.
    pub fn route_llm(&self, prompt: &str) -> LlmResponse {
        if self.use_local_only() {
            let content = call_local_llm(prompt).unwrap_or_default();
            return LlmResponse { content, degraded: false };
        }

        // Try cloud first.
        match call_cloud_llm(prompt) {
            Ok(content) => LlmResponse { content, degraded: false },
            Err(_) => {
                // Cloud failed → fall back to local and mark degraded.
                self.monitor.set_degraded();
                let content = call_local_llm(prompt).unwrap_or_default();
                LlmResponse { content, degraded: true }
            }
        }
    }

    /// Route a Speech-to-Text request.
    pub fn route_stt(&self, audio: &[u8]) -> RoutedResponse {
        self.route_bytes(
            audio,
            call_cloud_stt,
            call_local_stt,
        )
    }

    /// Route a Text-to-Speech request.
    pub fn route_tts(&self, text: &[u8]) -> RoutedResponse {
        self.route_bytes(
            text,
            call_cloud_tts,
            call_local_tts,
        )
    }

    /// Route a Vision inference request.
    pub fn route_vision(&self, image: &[u8]) -> RoutedResponse {
        self.route_bytes(
            image,
            call_cloud_vision,
            call_local_vision,
        )
    }

    /// Returns the current connectivity status as seen by this router.
    pub fn connectivity_status(&self) -> ConnectivityStatus {
        self.monitor.status()
    }

    // ── Internal helpers ──────────────────────────────────────────────────────

    /// Generic byte-payload routing: try cloud, fall back to local on failure.
    fn route_bytes(
        &self,
        input: &[u8],
        cloud_fn: impl Fn(&[u8]) -> Result<Vec<u8>, anyhow::Error>,
        local_fn: impl Fn(&[u8]) -> Result<Vec<u8>, anyhow::Error>,
    ) -> RoutedResponse {
        if self.use_local_only() {
            let payload = local_fn(input).unwrap_or_default();
            return RoutedResponse { payload, degraded: false };
        }

        match cloud_fn(input) {
            Ok(payload) => RoutedResponse { payload, degraded: false },
            Err(_) => {
                self.monitor.set_degraded();
                let payload = local_fn(input).unwrap_or_default();
                RoutedResponse { payload, degraded: true }
            }
        }
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn always_local_never_uses_cloud() {
        let router = ModelRouter::always_local();
        let resp = router.route_llm("hello");
        // Local stub returns a response containing "[local]"
        assert!(resp.content.contains("[local]"));
        assert!(!resp.degraded);
    }

    #[test]
    fn prefer_cloud_falls_back_on_cloud_failure() {
        // Cloud stubs always fail, so PreferCloud should fall back to local
        // and set degraded=true.
        let monitor = ConnectivityMonitor::new();
        // Monitor starts Online by default.
        let router = ModelRouter::new(monitor, BackendPreference::PreferCloud);
        let resp = router.route_llm("test");
        // Cloud stub fails → local fallback → degraded=true
        assert!(resp.degraded);
        assert!(resp.content.contains("[local]"));
    }

    #[test]
    fn always_local_preference_bypasses_cloud_regardless_of_connectivity() {
        let monitor = ConnectivityMonitor::new();
        let router = ModelRouter::new(monitor, BackendPreference::AlwaysLocal);
        let resp = router.route_llm("query");
        assert!(!resp.degraded);
        assert!(resp.content.contains("[local]"));
    }

    #[test]
    fn route_stt_returns_payload() {
        let router = ModelRouter::always_local();
        let audio = b"audio_bytes";
        let resp = router.route_stt(audio);
        assert_eq!(resp.payload, audio);
        assert!(!resp.degraded);
    }

    #[test]
    fn route_tts_returns_payload() {
        let router = ModelRouter::always_local();
        let text = b"hello world";
        let resp = router.route_tts(text);
        assert_eq!(resp.payload, text);
        assert!(!resp.degraded);
    }

    #[test]
    fn route_vision_returns_payload() {
        let router = ModelRouter::always_local();
        let image = b"image_data";
        let resp = router.route_vision(image);
        assert_eq!(resp.payload, image);
        assert!(!resp.degraded);
    }

    #[test]
    fn cloud_failure_sets_degraded_status_on_monitor() {
        let monitor = ConnectivityMonitor::new();
        // Starts Online.
        assert_eq!(monitor.status(), ConnectivityStatus::Online);
        let router = ModelRouter::new(monitor, BackendPreference::PreferCloud);
        // Cloud stub always fails → router calls set_degraded on the monitor.
        let resp = router.route_llm("probe");
        assert!(resp.degraded);
        assert_eq!(router.connectivity_status(), ConnectivityStatus::Degraded);
    }

    // Property 17: ModelRouter never calls cloud when ConnectivityStatus=OFFLINE.
    // When offline, use_local_only() returns true for PreferCloud, so the local
    // backend is used and degraded=false (no cloud attempt was made).
    #[test]
    fn prefer_cloud_uses_local_when_offline() {
        let monitor = ConnectivityMonitor::with_status(ConnectivityStatus::Offline);
        assert_eq!(monitor.status(), ConnectivityStatus::Offline);
        let router = ModelRouter::new(monitor, BackendPreference::PreferCloud);
        let resp = router.route_llm("offline query");
        // Must use local (not cloud) → degraded=false, content from local stub.
        assert!(!resp.degraded, "must not be degraded when using local due to OFFLINE status");
        assert!(resp.content.contains("[local]"));
    }

    // Property 17 variant: AlwaysLocal never touches cloud regardless of status.
    #[test]
    fn always_local_never_degraded_regardless_of_connectivity() {
        for status_val in [
            ConnectivityStatus::Online,
            ConnectivityStatus::Offline,
            ConnectivityStatus::Degraded,
        ] {
            let monitor = ConnectivityMonitor::with_status(status_val);
            let router = ModelRouter::new(monitor, BackendPreference::AlwaysLocal);
            let resp = router.route_llm("test");
            assert!(!resp.degraded, "AlwaysLocal must never be degraded (status={status_val:?})");
        }
    }

    // Property 18: Cloud failure always falls back to local with degraded=true.
    #[test]
    fn cloud_failure_falls_back_to_local_with_degraded_true() {
        // Cloud stubs always return Err, so PreferCloud on an Online monitor
        // must fall back to local and set degraded=true.
        let monitor = ConnectivityMonitor::new();
        let router = ModelRouter::new(monitor, BackendPreference::PreferCloud);
        let resp = router.route_llm("cloud fail test");
        assert!(resp.degraded, "cloud failure must set degraded=true");
        assert!(resp.content.contains("[local]"), "must fall back to local content");
    }

    // Property 18 variant: same for STT, TTS, Vision.
    #[test]
    fn cloud_failure_falls_back_for_all_stages() {
        let monitor = ConnectivityMonitor::new();
        let router = ModelRouter::new(monitor, BackendPreference::PreferCloud);
        assert!(router.route_stt(b"audio").degraded);
        assert!(router.route_tts(b"text").degraded);
        assert!(router.route_vision(b"img").degraded);
    }
}
