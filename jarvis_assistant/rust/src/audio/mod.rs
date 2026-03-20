/// Audio modality — Voice Input Pipeline
/// Provides platform-specific audio capture, wake word detection, STT, and
/// a high-level VoicePipeline that wires them together.

use crate::pal::types::Platform;

// ── AudioBackend / AudioAdapter ──────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AudioBackend {
    MediaRecorder, // Android
    AvAudioEngine, // iOS
    Wasapi,        // Windows
    CoreAudio,     // macOS
    PipeWire,      // Linux
}

pub struct AudioAdapter {
    pub platform: Platform,
    pub backend: AudioBackend,
}

impl AudioAdapter {
    pub fn for_platform(platform: Platform) -> Self {
        let backend = match platform {
            Platform::Android => AudioBackend::MediaRecorder,
            Platform::Ios => AudioBackend::AvAudioEngine,
            Platform::Windows => AudioBackend::Wasapi,
            Platform::Macos => AudioBackend::CoreAudio,
            Platform::Linux => AudioBackend::PipeWire,
        };
        Self { platform, backend }
    }

    /// Start audio capture. Stub: returns Ok(()).
    pub fn start_capture(&self) -> anyhow::Result<()> {
        Ok(())
    }

    /// Stop audio capture. Stub: returns Ok(()).
    pub fn stop_capture(&self) -> anyhow::Result<()> {
        Ok(())
    }

    /// Read captured audio bytes. Stub: returns empty vec.
    pub fn read_audio(&self) -> Vec<u8> {
        vec![]
    }
}

// ── WakeWordDetector ─────────────────────────────────────────────────────────

pub struct WakeWordDetector {
    pub keyword: String,
}

impl WakeWordDetector {
    pub fn new(keyword: impl Into<String>) -> Self {
        Self {
            keyword: keyword.into(),
        }
    }

    /// Process audio chunk. Stub: always returns false (no wake word detected).
    pub fn process_chunk(&self, _audio: &[u8]) -> bool {
        false
    }
}

// ── SttEngine ────────────────────────────────────────────────────────────────

pub struct SttEngine;

impl SttEngine {
    /// Transcribe audio to text. Stub: returns empty string.
    pub fn transcribe(&self, _audio: &[u8]) -> String {
        String::new()
    }
}

// ── VoicePipeline ────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VoicePipelineState {
    Idle,
    Listening,
    Processing,
    Transcribing,
}

pub struct VoicePipeline {
    pub adapter: AudioAdapter,
    pub wake_word: WakeWordDetector,
    pub stt: SttEngine,
    pub state: VoicePipelineState,
}

impl VoicePipeline {
    pub fn new(platform: Platform) -> Self {
        Self {
            adapter: AudioAdapter::for_platform(platform),
            wake_word: WakeWordDetector::new("jarvis"),
            stt: SttEngine,
            state: VoicePipelineState::Idle,
        }
    }

    pub fn start_listening(&mut self) -> anyhow::Result<()> {
        self.state = VoicePipelineState::Listening;
        self.adapter.start_capture()
    }

    pub fn on_wake_word_detected(&mut self) {
        self.state = VoicePipelineState::Processing;
    }

    pub fn on_transcription(&mut self, audio: &[u8]) -> String {
        self.state = VoicePipelineState::Transcribing;
        let text = self.stt.transcribe(audio);
        self.state = VoicePipelineState::Idle;
        text
    }
}

// ── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn audio_adapter_for_android_uses_media_recorder() {
        let adapter = AudioAdapter::for_platform(Platform::Android);
        assert_eq!(adapter.backend, AudioBackend::MediaRecorder);
    }

    #[test]
    fn voice_pipeline_starts_in_idle_state() {
        let pipeline = VoicePipeline::new(Platform::Linux);
        assert_eq!(pipeline.state, VoicePipelineState::Idle);
    }

    #[test]
    fn wake_word_detector_stub_returns_false() {
        let detector = WakeWordDetector::new("jarvis");
        assert!(!detector.process_chunk(&[0u8; 64]));
    }
}
