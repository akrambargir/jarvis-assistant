/// Perception Engine — Phase 1 (text modality)
/// Handles multimodal input ingestion, feature extraction, modality fusion,
/// and context tagging.

// ── MultimodalInput ──────────────────────────────────────────────────────────

pub struct MultimodalInput {
    pub raw_text: Option<String>,
    pub audio_data: Option<Vec<u8>>,
    pub image_data: Option<Vec<u8>>,
    pub screen_capture: Option<Vec<u8>>,
    pub sensor_data: Option<Vec<f32>>,
}

impl MultimodalInput {
    /// Convenience constructor for text-only input.
    pub fn text(s: impl Into<String>) -> Self {
        Self {
            raw_text: Some(s.into()),
            audio_data: None,
            image_data: None,
            screen_capture: None,
            sensor_data: None,
        }
    }
}

// ── TextFeatures / FeatureExtractor ─────────────────────────────────────────

pub struct TextFeatures {
    pub tokens: Vec<String>,
    pub named_entities: Vec<String>,
    pub intent_signals: Vec<String>,
    pub language: String,
}

pub struct AudioFeatures {
    pub mfcc_coefficients: Vec<f32>,
    pub prosody_score: f32,
    pub speaker_id: Option<String>,
    pub transcription: String,
}

pub struct FeatureExtractor;

impl FeatureExtractor {
    /// Stub: returns 13 zero MFCC coefficients, prosody 0.5, no speaker, empty transcription.
    pub fn extract_audio(_audio: &[u8]) -> AudioFeatures {
        AudioFeatures {
            mfcc_coefficients: vec![0.0; 13],
            prosody_score: 0.5,
            speaker_id: None,
            transcription: String::new(),
        }
    }

    /// Stub: tokenize by whitespace, no NER, empty intent signals, language = "en".
    pub fn extract_text(input: &str) -> TextFeatures {
        let tokens: Vec<String> = input
            .split_whitespace()
            .map(|t| t.to_string())
            .collect();
        TextFeatures {
            tokens,
            named_entities: vec![],
            intent_signals: vec![],
            language: "en".to_string(),
        }
    }
}

// ── VisualFeatures ───────────────────────────────────────────────────────────

pub struct VisualFeatures {
    pub embeddings: Vec<f32>,
    pub detected_objects: Vec<String>,
    pub scene_description: String,
    pub ocr_text: String,
}

impl FeatureExtractor {
    pub fn extract_visual(image: &[u8]) -> VisualFeatures {
        let _ = image;
        VisualFeatures {
            embeddings: vec![0.0; 512],
            detected_objects: vec![],
            scene_description: String::new(),
            ocr_text: String::new(),
        }
    }

    pub fn extract_screen(screen: &[u8]) -> VisualFeatures {
        let _ = screen;
        VisualFeatures {
            embeddings: vec![0.0; 512],
            detected_objects: vec![],
            scene_description: String::new(),
            ocr_text: String::new(),
        }
    }
}

// ── FusedPercept / ModalityFusion ────────────────────────────────────────────

pub struct FusedPercept {
    pub semantic_content: String,
    pub modalities_present: Vec<String>,
    pub ambiguity_score: f32, // clamped to [0.0, 1.0]
    pub confidence: f32,
    /// Attention weight per modality (keys match modalities_present entries).
    pub attention_map: std::collections::HashMap<String, f32>,
}

pub struct ModalityFusion;

/// Clamps an ambiguity score to the valid range [0.0, 1.0].
pub fn clamp_ambiguity(score: f32) -> f32 {
    score.clamp(0.0, 1.0)
}

/// Modality priority weights used for attention computation.
/// Higher = more trusted when conflicts arise.
fn modality_base_weight(modality: &str) -> f32 {
    match modality {
        "text"   => 1.0,
        "audio"  => 0.85,
        "image"  => 0.75,
        "screen" => 0.70,
        "sensor" => 0.50,
        _        => 0.40,
    }
}

impl ModalityFusion {
    /// Compute softmax-normalised attention weights for the given modality list.
    /// Returns a map of modality → weight where all weights sum to 1.0.
    pub fn compute_attention_weights(
        modalities: &[String],
    ) -> std::collections::HashMap<String, f32> {
        if modalities.is_empty() {
            return std::collections::HashMap::new();
        }
        let raw: Vec<f32> = modalities
            .iter()
            .map(|m| modality_base_weight(m.as_str()))
            .collect();
        let sum: f32 = raw.iter().sum();
        modalities
            .iter()
            .zip(raw.iter())
            .map(|(m, &w)| (m.clone(), w / sum))
            .collect()
    }

    /// Resolve conflicts between modalities by returning the semantic content
    /// from the highest-weighted modality.
    ///
    /// `candidates` is a list of `(modality, content)` pairs.
    /// Returns the content of the modality with the highest attention weight.
    pub fn resolve_conflicts(
        candidates: &[(String, String)],
        attention_map: &std::collections::HashMap<String, f32>,
    ) -> String {
        candidates
            .iter()
            .max_by(|(a, _), (b, _)| {
                let wa = attention_map.get(a.as_str()).copied().unwrap_or(0.0);
                let wb = attention_map.get(b.as_str()).copied().unwrap_or(0.0);
                wa.partial_cmp(&wb).unwrap_or(std::cmp::Ordering::Equal)
            })
            .map(|(_, content)| content.clone())
            .unwrap_or_default()
    }

    /// Fuse N >= 1 modalities with attention weighting.
    ///
    /// For N == 1 the single modality is used directly.
    /// For N >= 2 attention weights are computed and conflicts resolved by
    /// selecting the highest-weighted modality's semantic content.
    pub fn fuse(input: &MultimodalInput) -> FusedPercept {
        // Collect present modalities and their candidate semantic content.
        let mut candidates: Vec<(String, String)> = vec![];

        if let Some(text) = &input.raw_text {
            candidates.push(("text".to_string(), text.clone()));
        }
        if let Some(audio) = &input.audio_data {
            let af = FeatureExtractor::extract_audio(audio);
            candidates.push(("audio".to_string(), af.transcription));
        }
        if let Some(image) = &input.image_data {
            let vf = FeatureExtractor::extract_visual(image);
            candidates.push(("image".to_string(), vf.scene_description));
        }
        if let Some(screen) = &input.screen_capture {
            let sf = FeatureExtractor::extract_screen(screen);
            candidates.push(("screen".to_string(), sf.ocr_text));
        }
        if let Some(sensor) = &input.sensor_data {
            let summary = format!("sensor:{}", sensor.len());
            candidates.push(("sensor".to_string(), summary));
        }

        let modalities_present: Vec<String> =
            candidates.iter().map(|(m, _)| m.clone()).collect();

        let attention_map = Self::compute_attention_weights(&modalities_present);

        let semantic_content = if candidates.is_empty() {
            String::new()
        } else {
            Self::resolve_conflicts(&candidates, &attention_map)
        };

        // Ambiguity is lower when text is present; higher when only non-text modalities.
        let has_text = input.raw_text.is_some();
        let n = modalities_present.len();
        let raw_ambiguity = if n == 0 {
            1.0
        } else if has_text {
            0.1_f32 / (n as f32).sqrt()
        } else {
            0.5_f32 / (n as f32).sqrt()
        };

        // Confidence scales with number of modalities (more = more confident).
        let confidence = if n == 0 {
            0.0
        } else {
            (0.6 + 0.1 * (n as f32).min(4.0)).min(1.0)
        };

        FusedPercept {
            semantic_content,
            modalities_present,
            ambiguity_score: clamp_ambiguity(raw_ambiguity),
            confidence,
            attention_map,
        }
    }
}

// ── EnvironmentState / TaggedPercept / PerceptionEngine ──────────────────────

pub struct EnvironmentState {
    pub time_of_day: String,
    pub active_app: Option<String>,
    pub tags: Vec<String>,
}

pub struct TaggedPercept {
    pub percept: FusedPercept,
    pub environment: EnvironmentState,
}

pub struct PerceptionEngine;

impl PerceptionEngine {
    pub fn tag_context(percept: FusedPercept) -> TaggedPercept {
        TaggedPercept {
            percept,
            environment: EnvironmentState {
                time_of_day: "unknown".to_string(),
                active_app: None,
                tags: vec![],
            },
        }
    }
}

// ── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn text_input_produces_non_empty_semantic_content() {
        let input = MultimodalInput::text("hello");
        let percept = ModalityFusion::fuse(&input);
        assert!(!percept.semantic_content.is_empty());
    }

    #[test]
    fn ambiguity_score_always_clamped() {
        for &score in &[-0.5_f32, 0.5, 1.5] {
            let clamped = clamp_ambiguity(score);
            assert!(
                (0.0..=1.0).contains(&clamped),
                "score {score} clamped to {clamped}, expected [0.0, 1.0]"
            );
        }
    }

    #[test]
    fn empty_input_has_max_ambiguity() {
        let input = MultimodalInput {
            raw_text: None,
            audio_data: None,
            image_data: None,
            screen_capture: None,
            sensor_data: None,
        };
        let percept = ModalityFusion::fuse(&input);
        assert_eq!(percept.ambiguity_score, 1.0);
    }

    #[test]
    fn extract_text_tokenizes() {
        let features = FeatureExtractor::extract_text("hello world");
        assert_eq!(features.tokens.len(), 2);
    }

    // Task 15: attention map tests

    #[test]
    fn attention_map_has_n_entries_for_n_modalities() {
        let input = MultimodalInput {
            raw_text: Some("hello".to_string()),
            audio_data: Some(vec![0u8; 4]),
            image_data: Some(vec![0u8; 4]),
            screen_capture: None,
            sensor_data: None,
        };
        let percept = ModalityFusion::fuse(&input);
        assert_eq!(percept.attention_map.len(), percept.modalities_present.len());
    }

    #[test]
    fn attention_weights_sum_to_one() {
        let modalities = vec!["text".to_string(), "audio".to_string(), "image".to_string()];
        let weights = ModalityFusion::compute_attention_weights(&modalities);
        let sum: f32 = weights.values().sum();
        assert!((sum - 1.0).abs() < 1e-5, "weights sum {sum} != 1.0");
    }

    #[test]
    fn single_modality_attention_weight_is_one() {
        let modalities = vec!["text".to_string()];
        let weights = ModalityFusion::compute_attention_weights(&modalities);
        assert!((weights["text"] - 1.0).abs() < 1e-5);
    }

    #[test]
    fn resolve_conflicts_picks_highest_weight() {
        let mut attention_map = std::collections::HashMap::new();
        attention_map.insert("audio".to_string(), 0.3);
        attention_map.insert("text".to_string(), 0.7);
        let candidates = vec![
            ("audio".to_string(), "audio content".to_string()),
            ("text".to_string(), "text content".to_string()),
        ];
        let result = ModalityFusion::resolve_conflicts(&candidates, &attention_map);
        assert_eq!(result, "text content");
    }

    #[test]
    fn multimodal_fuse_attention_map_covers_all_modalities() {
        let input = MultimodalInput {
            raw_text: Some("hi".to_string()),
            audio_data: Some(vec![0u8; 4]),
            image_data: None,
            screen_capture: None,
            sensor_data: Some(vec![1.0, 2.0]),
        };
        let percept = ModalityFusion::fuse(&input);
        // 3 modalities: text, audio, sensor
        assert_eq!(percept.modalities_present.len(), 3);
        assert_eq!(percept.attention_map.len(), 3);
        for m in &percept.modalities_present {
            assert!(percept.attention_map.contains_key(m), "missing key: {m}");
        }
    }

    // Property 1: ambiguityScore always in [0.0, 1.0] for any valid input.
    #[test]
    fn ambiguity_score_always_in_range_for_any_input() {
        let inputs = vec![
            MultimodalInput::text(""),
            MultimodalInput::text("hello"),
            MultimodalInput::text("a very long sentence with many words and tokens"),
            MultimodalInput {
                raw_text: Some("text".to_string()),
                audio_data: Some(vec![0u8; 8]),
                image_data: Some(vec![0u8; 8]),
                screen_capture: None,
                sensor_data: None,
            },
            MultimodalInput {
                raw_text: None,
                audio_data: None,
                image_data: None,
                screen_capture: None,
                sensor_data: None,
            },
        ];
        for input in inputs {
            let percept = ModalityFusion::fuse(&input);
            assert!(
                (0.0..=1.0).contains(&percept.ambiguity_score),
                "ambiguity_score {} out of [0.0, 1.0]",
                percept.ambiguity_score
            );
        }
    }

    // Property 2 (implied): fusedPercept.semanticContent is always non-empty for text input.
    #[test]
    fn fused_percept_semantic_content_non_empty_for_text_input() {
        let inputs = vec![
            MultimodalInput::text("hello"),
            MultimodalInput::text("what is the weather?"),
            MultimodalInput::text("x"),
        ];
        for input in inputs {
            let percept = ModalityFusion::fuse(&input);
            assert!(
                !percept.semantic_content.is_empty(),
                "semantic_content must not be empty for text input"
            );
        }
    }

    // Property 2 (Task 15.3): N-modality input produces attentionMap with exactly N entries.
    #[test]
    fn n_modality_input_produces_attention_map_with_n_entries() {
        // 1 modality
        let input1 = MultimodalInput::text("hello");
        let p1 = ModalityFusion::fuse(&input1);
        assert_eq!(p1.attention_map.len(), p1.modalities_present.len());

        // 2 modalities
        let input2 = MultimodalInput {
            raw_text: Some("hello".to_string()),
            audio_data: Some(vec![0u8; 4]),
            image_data: None,
            screen_capture: None,
            sensor_data: None,
        };
        let p2 = ModalityFusion::fuse(&input2);
        assert_eq!(p2.attention_map.len(), p2.modalities_present.len());
        assert_eq!(p2.modalities_present.len(), 2);

        // 3 modalities
        let input3 = MultimodalInput {
            raw_text: Some("hello".to_string()),
            audio_data: Some(vec![0u8; 4]),
            image_data: Some(vec![0u8; 4]),
            screen_capture: None,
            sensor_data: None,
        };
        let p3 = ModalityFusion::fuse(&input3);
        assert_eq!(p3.attention_map.len(), p3.modalities_present.len());
        assert_eq!(p3.modalities_present.len(), 3);
    }
}
