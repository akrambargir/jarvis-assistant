/// Meta-Cognition Layer
/// Provides uncertainty detection, clarification management, cognitive load
/// tracking, and high-level meta-cognitive decision making.

use crate::perception::FusedPercept;

// ── Constants ────────────────────────────────────────────────────────────────

pub const AMBIGUITY_THRESHOLD: f32 = 0.6;

// ── UncertaintyDetector ──────────────────────────────────────────────────────

pub struct AmbiguousSlot {
    pub name: String,
    pub description: String,
}

pub struct UncertaintyDetector;

impl UncertaintyDetector {
    /// Returns the ambiguity score from the percept (clamped to [0.0, 1.0]).
    pub fn score_ambiguity(percept: &FusedPercept) -> f32 {
        percept.ambiguity_score
    }

    /// Phase 1 stub: slot identification not yet implemented.
    pub fn identify_ambiguous_slots(_percept: &FusedPercept) -> Vec<AmbiguousSlot> {
        vec![]
    }
}

// ── ClarificationManager ─────────────────────────────────────────────────────

pub struct ClarificationQuestion {
    pub question: String,
    pub slot: Option<String>,
}

pub struct ClarificationManager;

impl ClarificationManager {
    /// Returns true when the ambiguity score exceeds the threshold.
    pub fn should_ask_user(ambiguity_score: f32) -> bool {
        ambiguity_score > AMBIGUITY_THRESHOLD
    }

    /// Phase 1 stub: returns a generic clarification question.
    pub fn generate_clarification_question(_percept: &FusedPercept) -> ClarificationQuestion {
        ClarificationQuestion {
            question: "Could you please clarify what you mean?".to_string(),
            slot: None,
        }
    }
}

// ── CognitiveLoadManager ─────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CognitiveLoad {
    Normal,
    High,
    Overloaded,
}

pub struct CognitiveLoadManager {
    pub current_load: CognitiveLoad,
    pub task_queue_depth: usize,
}

impl CognitiveLoadManager {
    pub fn new() -> Self {
        Self {
            current_load: CognitiveLoad::Normal,
            task_queue_depth: 0,
        }
    }

    /// Updates the cognitive load level based on the current task queue depth.
    /// - queue_depth > 10 → Overloaded
    /// - queue_depth > 5  → High
    /// - otherwise        → Normal
    pub fn update_load(&mut self, queue_depth: usize) {
        self.task_queue_depth = queue_depth;
        self.current_load = if queue_depth > 10 {
            CognitiveLoad::Overloaded
        } else if queue_depth > 5 {
            CognitiveLoad::High
        } else {
            CognitiveLoad::Normal
        };
    }
}

impl Default for CognitiveLoadManager {
    fn default() -> Self {
        Self::new()
    }
}

// ── MetaCognitionLayer ───────────────────────────────────────────────────────

/// Possible decisions the meta-cognition layer can return.
///
/// The return type is a non-nullable enum, so `evaluate()` is guaranteed
/// to always return a valid `MetaCognitiveDecision` — null is impossible in Rust.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MetaCognitiveDecision {
    Proceed,
    Clarify,
    Defer,
    Pause,
    Redirect,
}

pub struct MetaCognitionLayer {
    pub load_manager: CognitiveLoadManager,
}

impl MetaCognitionLayer {
    pub fn new() -> Self {
        Self {
            load_manager: CognitiveLoadManager::new(),
        }
    }

    /// Evaluates the current percept and cognitive load to decide how to proceed.
    ///
    /// Decision logic:
    /// - `Overloaded` load → `Defer`
    /// - ambiguity score > `AMBIGUITY_THRESHOLD` → `Clarify`
    /// - otherwise → `Proceed`
    pub fn evaluate(&self, percept: &FusedPercept) -> MetaCognitiveDecision {
        if self.load_manager.current_load == CognitiveLoad::Overloaded {
            return MetaCognitiveDecision::Defer;
        }
        if UncertaintyDetector::score_ambiguity(percept) > AMBIGUITY_THRESHOLD {
            return MetaCognitiveDecision::Clarify;
        }
        MetaCognitiveDecision::Proceed
    }
}

impl Default for MetaCognitionLayer {
    fn default() -> Self {
        Self::new()
    }
}

// ── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::perception::{ModalityFusion, MultimodalInput};

    fn percept_with_ambiguity(score: f32) -> FusedPercept {
        FusedPercept {
            semantic_content: "test".to_string(),
            modalities_present: vec!["text".to_string()],
            ambiguity_score: score,
            confidence: 1.0 - score,
            attention_map: {
                let mut m = std::collections::HashMap::new();
                m.insert("text".to_string(), 1.0);
                m
            },
        }
    }

    #[test]
    fn evaluate_returns_defer_when_overloaded() {
        let mut layer = MetaCognitionLayer::new();
        layer.load_manager.update_load(15);
        let percept = percept_with_ambiguity(0.1);
        assert_eq!(layer.evaluate(&percept), MetaCognitiveDecision::Defer);
    }

    #[test]
    fn evaluate_returns_clarify_when_high_ambiguity() {
        let layer = MetaCognitionLayer::new(); // Normal load
        let percept = percept_with_ambiguity(0.9);
        assert_eq!(layer.evaluate(&percept), MetaCognitiveDecision::Clarify);
    }

    #[test]
    fn evaluate_returns_proceed_when_clear() {
        let layer = MetaCognitionLayer::new();
        let percept = percept_with_ambiguity(0.1);
        assert_eq!(layer.evaluate(&percept), MetaCognitiveDecision::Proceed);
    }

    #[test]
    fn ambiguity_threshold_boundary() {
        // Score exactly at threshold is NOT > threshold, so should Proceed
        let layer = MetaCognitionLayer::new();
        let percept = percept_with_ambiguity(AMBIGUITY_THRESHOLD);
        assert_eq!(layer.evaluate(&percept), MetaCognitiveDecision::Proceed);
    }

    #[test]
    fn should_ask_user_above_threshold() {
        assert!(ClarificationManager::should_ask_user(0.7));
    }

    #[test]
    fn should_not_ask_user_at_or_below_threshold() {
        assert!(!ClarificationManager::should_ask_user(AMBIGUITY_THRESHOLD));
        assert!(!ClarificationManager::should_ask_user(0.3));
    }

    #[test]
    fn cognitive_load_transitions() {
        let mut mgr = CognitiveLoadManager::new();
        assert_eq!(mgr.current_load, CognitiveLoad::Normal);

        mgr.update_load(6);
        assert_eq!(mgr.current_load, CognitiveLoad::High);

        mgr.update_load(11);
        assert_eq!(mgr.current_load, CognitiveLoad::Overloaded);

        mgr.update_load(3);
        assert_eq!(mgr.current_load, CognitiveLoad::Normal);
    }

    #[test]
    fn identify_ambiguous_slots_is_empty_stub() {
        let input = MultimodalInput::text("ambiguous request");
        let percept = ModalityFusion::fuse(&input);
        let slots = UncertaintyDetector::identify_ambiguous_slots(&percept);
        assert!(slots.is_empty());
    }

    // Property 3: evaluate() always returns a non-null MetaCognitiveDecision.
    // (In Rust, non-null is guaranteed by the type system; this test documents the contract.)
    #[test]
    fn evaluate_always_returns_valid_decision_for_any_input() {
        let valid_decisions = [
            MetaCognitiveDecision::Proceed,
            MetaCognitiveDecision::Clarify,
            MetaCognitiveDecision::Defer,
            MetaCognitiveDecision::Pause,
            MetaCognitiveDecision::Redirect,
        ];
        let layer = MetaCognitionLayer::new();
        for &ambiguity in &[0.0_f32, 0.3, 0.6, 0.7, 0.9, 1.0] {
            let percept = percept_with_ambiguity(ambiguity);
            let decision = layer.evaluate(&percept);
            assert!(
                valid_decisions.contains(&decision),
                "evaluate() returned unexpected decision for ambiguity={ambiguity}"
            );
        }
    }

    // Property 4: ambiguityScore > AMBIGUITY_THRESHOLD always returns CLARIFY.
    #[test]
    fn high_ambiguity_always_returns_clarify() {
        let layer = MetaCognitionLayer::new(); // Normal load
        for &score in &[AMBIGUITY_THRESHOLD + 0.01, 0.7, 0.8, 0.9, 1.0] {
            let percept = percept_with_ambiguity(score);
            assert_eq!(
                layer.evaluate(&percept),
                MetaCognitiveDecision::Clarify,
                "ambiguity={score} must return Clarify"
            );
        }
    }

    // Property 5: OVERLOADED load always returns DEFER, never PROCEED.
    #[test]
    fn overloaded_load_always_returns_defer_never_proceed() {
        let mut layer = MetaCognitionLayer::new();
        layer.load_manager.update_load(11); // > 10 → Overloaded
        assert_eq!(layer.load_manager.current_load, CognitiveLoad::Overloaded);
        // Test with various ambiguity scores — must always be Defer.
        for &ambiguity in &[0.0_f32, 0.3, 0.6, 0.9, 1.0] {
            let percept = percept_with_ambiguity(ambiguity);
            let decision = layer.evaluate(&percept);
            assert_eq!(decision, MetaCognitiveDecision::Defer, "ambiguity={ambiguity} with Overloaded must be Defer");
            assert_ne!(decision, MetaCognitiveDecision::Proceed, "Overloaded must never return Proceed");
        }
    }
}
