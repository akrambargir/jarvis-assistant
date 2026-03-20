/// LLM module — local inference stubs for llama.cpp integration.
///
/// Full native integration is gated behind the `llm-native` feature flag,
/// which requires native llama.cpp libraries to be present. The default build
/// uses clean stubs that compile without any native dependencies.

use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::pal::types::{DeviceClass, Platform, BATTERY_LOW_THRESHOLD};

// ── Quantization levels ───────────────────────────────────────────────────────

/// Supported GGUF quantization levels, ordered roughly by size/quality.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum QuantizationLevel {
    /// 2-bit quantization — smallest, lowest quality. Good for low-battery mobile.
    Q2K,
    /// 4-bit quantization (original). Balanced for mobile.
    Q4_0,
    /// 4-bit quantization (K-quant medium). Good default for desktop.
    Q4KM,
    /// 5-bit quantization (K-quant medium). Higher quality desktop option.
    Q5KM,
}

// ── Model configuration ───────────────────────────────────────────────────────

/// Configuration for loading a local LLM model.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelConfig {
    /// Path to the GGUF model file.
    pub model_path: String,
    /// Quantization level of the model file.
    pub quantization: QuantizationLevel,
    /// Context window size in tokens.
    pub context_size: u32,
    /// Number of CPU threads to use for inference.
    pub n_threads: u32,
}

// ── LlmModel ──────────────────────────────────────────────────────────────────

/// A loaded LLM model handle.
///
/// The inner `handle` is a placeholder (`Option<()>`) in the default build.
/// When compiled with `--features llm-native`, this would hold the real
/// llama-cpp-2 model handle.
pub struct LlmModel {
    config: ModelConfig,
    /// Placeholder for the native model handle.
    /// Replace with the real llama-cpp-2 type when `llm-native` is enabled.
    handle: Option<()>,
}

impl LlmModel {
    /// Load a model from the path specified in `config`.
    ///
    /// In the default (stub) build this logs the path and returns `Ok`.
    /// With `llm-native` this would call into llama.cpp to load the GGUF file.
    pub fn load(config: ModelConfig) -> Result<Self> {
        // Stub: log the intent; real impl would call llama_model_load_from_file.
        eprintln!(
            "[LlmModel] load stub: path={}, quant={:?}, ctx={}, threads={}",
            config.model_path, config.quantization, config.context_size, config.n_threads
        );
        Ok(Self { config, handle: Some(()) })
    }

    /// Returns `true` if the model handle is present (i.e. successfully loaded).
    pub fn is_loaded(&self) -> bool {
        self.handle.is_some()
    }

    /// Returns the model config.
    pub fn config(&self) -> &ModelConfig {
        &self.config
    }
}

// ── Inference types ───────────────────────────────────────────────────────────

/// Parameters controlling a single inference call.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InferenceParams {
    /// Maximum number of tokens to generate.
    pub max_tokens: u32,
    /// Sampling temperature (0.0 = deterministic, 1.0 = creative).
    pub temperature: f32,
    /// Nucleus sampling probability threshold.
    pub top_p: f32,
    /// Sequences that stop generation when encountered.
    pub stop_sequences: Vec<String>,
}

impl Default for InferenceParams {
    fn default() -> Self {
        Self {
            max_tokens: 512,
            temperature: 0.7,
            top_p: 0.9,
            stop_sequences: vec![],
        }
    }
}

/// Result of a completed inference call.
#[derive(Debug, Clone)]
pub struct InferenceResult {
    /// The generated text.
    pub text: String,
    /// Number of tokens that were generated.
    pub tokens_generated: u32,
    /// Identifier of the model that produced this result.
    pub model_used: String,
    /// `true` when the result came from a degraded / fallback path.
    pub degraded: bool,
}

// ── LLMCore ───────────────────────────────────────────────────────────────────

/// Core LLM inference engine.
pub struct LLMCore {
    pub model: Option<LlmModel>,
    pub config: ModelConfig,
}

impl LLMCore {
    /// Create a new `LLMCore` with the given config. The model is not loaded yet.
    pub fn new(config: ModelConfig) -> Self {
        Self { model: None, config }
    }

    /// Run inference on `prompt` with the given `params`.
    ///
    /// Stub: returns a placeholder response. Real impl would call into
    /// llama-cpp-2's sampling loop.
    pub fn infer(&self, prompt: &str, params: InferenceParams) -> Result<InferenceResult> {
        let model_name = self
            .model
            .as_ref()
            .map(|m| m.config().model_path.clone())
            .unwrap_or_else(|| self.config.model_path.clone());

        // Stub response — replace with real llama.cpp sampling when llm-native is enabled.
        let text = format!(
            "[stub inference] prompt='{}' max_tokens={} temp={}",
            prompt, params.max_tokens, params.temperature
        );

        Ok(InferenceResult {
            text,
            tokens_generated: params.max_tokens.min(16),
            model_used: model_name,
            degraded: self.model.is_none(),
        })
    }

    /// Run chain-of-thought inference by prefixing the prompt with a CoT instruction.
    ///
    /// Calls `infer()` internally with a structured reasoning prefix.
    pub fn chain_of_thought(&self, prompt: &str, steps: u32) -> Result<InferenceResult> {
        let cot_prompt = format!(
            "Think step by step in {} steps.\n\nQuestion: {}\n\nReasoning:",
            steps, prompt
        );
        self.infer(&cot_prompt, InferenceParams::default())
    }
}

// ── PAL battery-aware quantization selection ──────────────────────────────────

/// Select the appropriate quantization level based on platform, device class,
/// and current battery level.
///
/// Rules:
/// - Mobile + battery < `BATTERY_LOW_THRESHOLD` → `Q2K`  (smallest, saves power)
/// - Mobile + battery >= `BATTERY_LOW_THRESHOLD` → `Q4_0`
/// - Desktop (any) → `Q4KM`
/// - No battery info (plugged-in desktop) → `Q4KM`
pub fn select_quantization(
    platform: Platform,
    device_class: DeviceClass,
    battery_level: Option<f32>,
) -> QuantizationLevel {
    match device_class {
        DeviceClass::Mobile => match battery_level {
            Some(level) if level < BATTERY_LOW_THRESHOLD => QuantizationLevel::Q2K,
            Some(_) => QuantizationLevel::Q4_0,
            // No battery info on mobile — be conservative.
            None => QuantizationLevel::Q4_0,
        },
        DeviceClass::Desktop => {
            // Platform is available for future fine-tuning (e.g. macOS GPU path).
            let _ = platform;
            QuantizationLevel::Q4KM
        }
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // select_quantization tests

    #[test]
    fn mobile_low_battery_selects_q2k() {
        let q = select_quantization(Platform::Android, DeviceClass::Mobile, Some(0.1));
        assert_eq!(q, QuantizationLevel::Q2K);
    }

    #[test]
    fn mobile_battery_at_threshold_boundary_selects_q4_0() {
        // Exactly at threshold is NOT below it, so Q4_0.
        let q = select_quantization(Platform::Android, DeviceClass::Mobile, Some(BATTERY_LOW_THRESHOLD));
        assert_eq!(q, QuantizationLevel::Q4_0);
    }

    #[test]
    fn mobile_normal_battery_selects_q4_0() {
        let q = select_quantization(Platform::Ios, DeviceClass::Mobile, Some(0.8));
        assert_eq!(q, QuantizationLevel::Q4_0);
    }

    #[test]
    fn desktop_selects_q4km() {
        let q = select_quantization(Platform::Linux, DeviceClass::Desktop, None);
        assert_eq!(q, QuantizationLevel::Q4KM);
    }

    #[test]
    fn desktop_with_battery_still_selects_q4km() {
        // Laptops report battery but are DeviceClass::Desktop.
        let q = select_quantization(Platform::Macos, DeviceClass::Desktop, Some(0.5));
        assert_eq!(q, QuantizationLevel::Q4KM);
    }

    #[test]
    fn mobile_no_battery_info_selects_q4_0() {
        let q = select_quantization(Platform::Android, DeviceClass::Mobile, None);
        assert_eq!(q, QuantizationLevel::Q4_0);
    }

    // Property 19: batteryLevel < BATTERY_LOW_THRESHOLD always selects Q2K on mobile.
    #[test]
    fn battery_below_threshold_always_selects_q2k_on_mobile() {
        // Test a range of values strictly below the threshold.
        let below_threshold_values = [0.0, 0.01, 0.05, 0.10, 0.15, 0.19, BATTERY_LOW_THRESHOLD - f32::EPSILON];
        for &level in &below_threshold_values {
            for platform in [Platform::Android, Platform::Ios] {
                let q = select_quantization(platform, DeviceClass::Mobile, Some(level));
                assert_eq!(
                    q, QuantizationLevel::Q2K,
                    "battery={level} on {platform:?} must select Q2K"
                );
            }
        }
    }

    // Property 19 complement: battery >= threshold selects Q4_0 on mobile.
    #[test]
    fn battery_at_or_above_threshold_selects_q4_0_on_mobile() {
        let at_or_above = [BATTERY_LOW_THRESHOLD, 0.3, 0.5, 0.8, 1.0];
        for &level in &at_or_above {
            let q = select_quantization(Platform::Android, DeviceClass::Mobile, Some(level));
            assert_eq!(q, QuantizationLevel::Q4_0, "battery={level} must select Q4_0");
        }
    }

    // LlmModel tests

    #[test]
    fn llm_model_load_stub_succeeds() {
        let config = ModelConfig {
            model_path: "/models/test.gguf".to_string(),
            quantization: QuantizationLevel::Q4KM,
            context_size: 2048,
            n_threads: 4,
        };
        let model = LlmModel::load(config).expect("load should succeed");
        assert!(model.is_loaded());
    }

    // LLMCore tests

    #[test]
    fn llmcore_infer_returns_result() {
        let config = ModelConfig {
            model_path: "/models/test.gguf".to_string(),
            quantization: QuantizationLevel::Q4_0,
            context_size: 512,
            n_threads: 2,
        };
        let core = LLMCore::new(config);
        let result = core.infer("hello", InferenceParams::default()).expect("infer should succeed");
        assert!(!result.text.is_empty());
    }

    #[test]
    fn llmcore_chain_of_thought_calls_infer() {
        let config = ModelConfig {
            model_path: "/models/test.gguf".to_string(),
            quantization: QuantizationLevel::Q4KM,
            context_size: 2048,
            n_threads: 4,
        };
        let core = LLMCore::new(config);
        let result = core.chain_of_thought("What is 2+2?", 3).expect("cot should succeed");
        // The CoT prompt should be reflected in the stub output.
        assert!(result.text.contains("step by step") || result.text.contains("stub inference"));
    }
}
