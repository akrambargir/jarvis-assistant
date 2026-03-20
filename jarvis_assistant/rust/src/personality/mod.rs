/// Personality Layer — Ayanokoji Persona Engine
///
/// Applies the Ayanokoji Kiyotaka persona to response drafts:
/// calm, analytical, precise, minimal emotional expression, occasional subtle wit.

use serde::{Deserialize, Serialize};

// ── PersonaConfig ─────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersonaConfig {
    pub name: String,
    /// Persona intensity in [0.0, 1.0]. 0.0 = neutral, 1.0 = full persona.
    pub intensity: f32,
    /// Verbosity level: "minimal", "normal", "verbose"
    pub verbosity: String,
    /// Emotional expression level in [0.0, 1.0]. Ayanokoji default: 0.1
    pub emotional_expression: f32,
    /// Analytical depth in [0.0, 1.0]. Ayanokoji default: 0.9
    pub insight_depth: f32,
}

impl PersonaConfig {
    /// Default Ayanokoji Kiyotaka persona configuration.
    pub fn ayanokoji() -> Self {
        Self {
            name: "Ayanokoji Kiyotaka".to_string(),
            intensity: 0.8,
            verbosity: "minimal".to_string(),
            emotional_expression: 0.1,
            insight_depth: 0.9,
        }
    }
}

// ── Response types ────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct ResponseDraft {
    pub text: String,
}

#[derive(Debug, Clone)]
pub struct PersonaResponse {
    pub text: String,
    pub persona_applied: String,
    pub intensity: f32,
}

// ── PersonalityLayer ──────────────────────────────────────────────────────────

pub struct PersonalityLayer {
    pub config: PersonaConfig,
}

impl PersonalityLayer {
    pub fn new(config: PersonaConfig) -> Self {
        Self { config }
    }

    /// Reformat ResponseDraft through Ayanokoji style rules.
    pub fn apply_persona(&self, draft: ResponseDraft) -> PersonaResponse {
        let mut text = draft.text;

        // Rule 1: strip exclamation marks if intensity > 0.5
        if self.config.intensity > 0.5 {
            text = text.replace('!', ".");
        }

        // Rule 2: remove emotional phrases if emotional_expression < 0.3
        if self.config.emotional_expression < 0.3 {
            for phrase in &["I feel", "I'm excited", "I'm happy"] {
                text = text.replace(phrase, "");
            }
            text = text.trim().to_string();
        }

        // Rule 3: prepend logical prefix if insight_depth > 0.7
        if self.config.insight_depth > 0.7 && !text.starts_with("Logically speaking,") {
            text = format!("Logically speaking, {}", text);
        }

        // Rule 4: truncate to first 2 sentences if verbosity == "minimal"
        if self.config.verbosity == "minimal" {
            let sentences: Vec<&str> = text.splitn(3, ". ").collect();
            text = sentences.iter().take(2).cloned().collect::<Vec<&str>>().join(". ");
        }

        // Rule 5: never return empty
        text = text.trim().to_string();
        if text.is_empty() || text == "Logically speaking," || text == "Logically speaking, " {
            text = "...".to_string();
        }

        PersonaResponse {
            text,
            persona_applied: self.config.name.clone(),
            intensity: self.config.intensity,
        }
    }
}

// ── OutputFormatter ───────────────────────────────────────────────────────────

pub struct OutputFormatter;

impl OutputFormatter {
    /// Format text for display. Stub: returns text as-is.
    pub fn format_text(text: &str) -> String {
        text.to_string()
    }

    /// Synthesize speech. Stub: returns empty bytes.
    pub fn synthesize_speech(_text: &str) -> Vec<u8> {
        vec![]
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn apply_persona_returns_non_empty_for_non_empty_draft() {
        let layer = PersonalityLayer::new(PersonaConfig::ayanokoji());
        let draft = ResponseDraft { text: "This is a test response. It has two sentences.".to_string() };
        let result = layer.apply_persona(draft);
        assert!(!result.text.is_empty());
    }

    #[test]
    fn apply_persona_strips_exclamation_marks() {
        let layer = PersonalityLayer::new(PersonaConfig::ayanokoji());
        let draft = ResponseDraft { text: "Hello! Great!".to_string() };
        let result = layer.apply_persona(draft);
        assert!(!result.text.contains('!'), "Output should not contain '!'");
    }

    #[test]
    fn apply_persona_prepends_logical_prefix() {
        let layer = PersonalityLayer::new(PersonaConfig::ayanokoji());
        let draft = ResponseDraft { text: "This is a fact. It is well established.".to_string() };
        let result = layer.apply_persona(draft);
        assert!(result.text.starts_with("Logically speaking,"));
    }

    #[test]
    fn apply_persona_empty_draft_returns_ellipsis() {
        let layer = PersonalityLayer::new(PersonaConfig::ayanokoji());
        let draft = ResponseDraft { text: String::new() };
        let result = layer.apply_persona(draft);
        assert_eq!(result.text, "...");
    }

    #[test]
    fn persona_config_ayanokoji_has_correct_defaults() {
        let config = PersonaConfig::ayanokoji();
        assert_eq!(config.name, "Ayanokoji Kiyotaka");
        assert!((config.intensity - 0.8).abs() < f32::EPSILON);
        assert_eq!(config.verbosity, "minimal");
        assert!((config.emotional_expression - 0.1).abs() < f32::EPSILON);
        assert!((config.insight_depth - 0.9).abs() < f32::EPSILON);
    }

    // Property 23: applyPersona always returns non-empty PersonaResponse for non-empty draft.
    #[test]
    fn apply_persona_always_non_empty_for_non_empty_draft() {
        let layer = PersonalityLayer::new(PersonaConfig::ayanokoji());
        let drafts = vec![
            "Hello.",
            "This is a test.",
            "A",
            "Multiple sentences here. And another one. And a third.",
            "I feel excited! This is great!",
        ];
        for text in drafts {
            let draft = ResponseDraft { text: text.to_string() };
            let result = layer.apply_persona(draft);
            assert!(
                !result.text.is_empty(),
                "apply_persona must return non-empty for draft='{text}'"
            );
        }
    }

    // Property 23 variant: persona_applied field always matches config name.
    #[test]
    fn apply_persona_persona_applied_matches_config_name() {
        let config = PersonaConfig::ayanokoji();
        let name = config.name.clone();
        let layer = PersonalityLayer::new(config);
        let result = layer.apply_persona(ResponseDraft { text: "test".to_string() });
        assert_eq!(result.persona_applied, name);
    }
}
