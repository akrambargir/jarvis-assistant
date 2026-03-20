/// Proactive Intelligence Engine — Phase 4
///
/// NeedPredictor, ProactiveIntelligenceEngine, ProactiveLevel.
/// All surfaced suggestions have urgency >= PROACTIVE_URGENCY_THRESHOLD.
/// All proactive actions pass SafetyAlignmentSystem.validate() before execution.
/// Proactive event log with reversibility.

use crate::digital_twin::PredictedNeed;
use crate::safety::{PermissionCategory, SafetyAlignmentSystem};

// ── Constants ─────────────────────────────────────────────────────────────────

pub const PROACTIVE_URGENCY_THRESHOLD: f32 = 0.5;

// ── ProactiveLevel ────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProactiveLevel {
    Off,
    SuggestionsOnly,
    SemiAuto,
    FullAuto,
}

// ── ProactiveSuggestion ───────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct ProactiveSuggestion {
    pub id: String,
    pub description: String,
    pub urgency: f32, // [0.0, 1.0]
    pub reversible: bool,
}

// ── ProactiveEvent ────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct ProactiveEvent {
    pub id: String,
    pub action: String,
    pub reversed: bool,
}

// ── NeedPredictor ─────────────────────────────────────────────────────────────

pub struct NeedPredictor;

impl NeedPredictor {
    /// Predict needs from a list of predicted needs, filtering by preference.
    pub fn predict(needs: &[PredictedNeed]) -> Vec<&PredictedNeed> {
        needs.iter().collect()
    }

    /// Score urgency — returns the urgency value clamped to [0.0, 1.0].
    pub fn score_urgency(need: &PredictedNeed) -> f32 {
        need.urgency.clamp(0.0, 1.0)
    }

    /// Filter needs by preference: only return those above the urgency threshold.
    pub fn filter_by_preference(needs: &[PredictedNeed]) -> Vec<&PredictedNeed> {
        needs
            .iter()
            .filter(|n| n.urgency >= PROACTIVE_URGENCY_THRESHOLD)
            .collect()
    }
}

// ── ProactiveIntelligenceEngine ───────────────────────────────────────────────

pub struct ProactiveIntelligenceEngine {
    pub level: ProactiveLevel,
    pub safety: SafetyAlignmentSystem,
    event_log: Vec<ProactiveEvent>,
    next_id: u64,
}

impl ProactiveIntelligenceEngine {
    pub fn new(level: ProactiveLevel) -> Self {
        Self {
            level,
            safety: SafetyAlignmentSystem::new(),
            event_log: vec![],
            next_id: 1,
        }
    }

    /// Analyse context and generate a suggestion if urgency >= threshold.
    /// Returns None if level is Off or no needs meet the threshold.
    pub fn generate_suggestion(
        &self,
        needs: &[PredictedNeed],
    ) -> Option<ProactiveSuggestion> {
        if self.level == ProactiveLevel::Off {
            return None;
        }
        let filtered = NeedPredictor::filter_by_preference(needs);
        filtered.first().map(|need| ProactiveSuggestion {
            id: format!("sug-{}", need.description.len()),
            description: need.description.clone(),
            urgency: need.urgency.clamp(0.0, 1.0),
            reversible: true,
        })
    }

    /// Execute a proactive action — gated through Safety.
    /// Logs the event with reversibility.
    pub fn execute_proactive_action(&mut self, action: &str) -> bool {
        let result = self.safety.validate(action, PermissionCategory::Other);
        if !result.approved {
            return false;
        }
        let id = format!("pe-{}", self.next_id);
        self.next_id += 1;
        self.event_log.push(ProactiveEvent {
            id,
            action: action.to_string(),
            reversed: false,
        });
        true
    }

    /// Reverse a proactive action by id.
    pub fn reverse_action(&mut self, event_id: &str) -> bool {
        if let Some(event) = self.event_log.iter_mut().find(|e| e.id == event_id) {
            event.reversed = true;
            true
        } else {
            false
        }
    }

    /// Return all proactive events.
    pub fn event_log(&self) -> &[ProactiveEvent] {
        &self.event_log
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn make_needs(urgencies: &[f32]) -> Vec<PredictedNeed> {
        urgencies
            .iter()
            .enumerate()
            .map(|(i, &u)| PredictedNeed {
                description: format!("need-{i}"),
                urgency: u,
            })
            .collect()
    }

    #[test]
    fn filter_by_preference_only_returns_above_threshold() {
        let needs = make_needs(&[0.3, 0.6, 0.8, 0.1]);
        let filtered = NeedPredictor::filter_by_preference(&needs);
        for n in &filtered {
            assert!(n.urgency >= PROACTIVE_URGENCY_THRESHOLD);
        }
        assert_eq!(filtered.len(), 2);
    }

    #[test]
    fn generate_suggestion_returns_none_when_off() {
        let engine = ProactiveIntelligenceEngine::new(ProactiveLevel::Off);
        let needs = make_needs(&[0.9]);
        assert!(engine.generate_suggestion(&needs).is_none());
    }

    #[test]
    fn generate_suggestion_urgency_above_threshold() {
        let engine = ProactiveIntelligenceEngine::new(ProactiveLevel::SuggestionsOnly);
        let needs = make_needs(&[0.8, 0.3]);
        let sug = engine.generate_suggestion(&needs);
        assert!(sug.is_some());
        assert!(sug.unwrap().urgency >= PROACTIVE_URGENCY_THRESHOLD);
    }

    #[test]
    fn generate_suggestion_returns_none_when_no_needs_above_threshold() {
        let engine = ProactiveIntelligenceEngine::new(ProactiveLevel::FullAuto);
        let needs = make_needs(&[0.1, 0.2]);
        assert!(engine.generate_suggestion(&needs).is_none());
    }

    #[test]
    fn execute_proactive_action_logs_event() {
        let mut engine = ProactiveIntelligenceEngine::new(ProactiveLevel::FullAuto);
        // Safety defaults to denied for OsControl, but Other is approved.
        engine.safety.permissions.grant(crate::safety::PermissionCategory::Other);
        let ok = engine.execute_proactive_action("send reminder");
        assert!(ok);
        assert_eq!(engine.event_log().len(), 1);
    }

    #[test]
    fn reverse_action_marks_reversed() {
        let mut engine = ProactiveIntelligenceEngine::new(ProactiveLevel::FullAuto);
        engine.safety.permissions.grant(crate::safety::PermissionCategory::Other);
        engine.execute_proactive_action("do something");
        let event_id = engine.event_log()[0].id.clone();
        assert!(engine.reverse_action(&event_id));
        assert!(engine.event_log()[0].reversed);
    }

    // Property 14: all surfaced suggestions have urgency >= PROACTIVE_URGENCY_THRESHOLD.
    #[test]
    fn all_surfaced_suggestions_have_urgency_above_threshold() {
        let engine = ProactiveIntelligenceEngine::new(ProactiveLevel::SuggestionsOnly);
        let needs = make_needs(&[0.1, 0.4, 0.5, 0.6, 0.9]);
        // Only needs with urgency >= threshold should be surfaced.
        let filtered = NeedPredictor::filter_by_preference(&needs);
        for n in &filtered {
            assert!(
                n.urgency >= PROACTIVE_URGENCY_THRESHOLD,
                "surfaced need urgency {} < threshold {}",
                n.urgency, PROACTIVE_URGENCY_THRESHOLD
            );
        }
        // generate_suggestion must also respect the threshold.
        if let Some(sug) = engine.generate_suggestion(&needs) {
            assert!(
                sug.urgency >= PROACTIVE_URGENCY_THRESHOLD,
                "suggestion urgency {} < threshold",
                sug.urgency
            );
        }
    }

    // Property 14 variant: no suggestion generated when all needs are below threshold.
    #[test]
    fn no_suggestion_when_all_needs_below_threshold() {
        let engine = ProactiveIntelligenceEngine::new(ProactiveLevel::FullAuto);
        let needs = make_needs(&[0.0, 0.1, 0.2, 0.3, 0.49]);
        assert!(engine.generate_suggestion(&needs).is_none());
    }

    // Property 31.7: proactive actions always pass Safety validation before execution.
    #[test]
    fn proactive_actions_pass_safety_before_execution() {
        let mut engine = ProactiveIntelligenceEngine::new(ProactiveLevel::FullAuto);
        // Without granting permission, execute_proactive_action must return false.
        let ok = engine.execute_proactive_action("unauthorized action");
        assert!(!ok, "action without permission must not execute");
        assert_eq!(engine.event_log().len(), 0, "no event must be logged for rejected action");

        // With permission granted, it must succeed.
        engine.safety.permissions.grant(crate::safety::PermissionCategory::Other);
        let ok2 = engine.execute_proactive_action("allowed action");
        assert!(ok2, "action with permission must execute");
        assert_eq!(engine.event_log().len(), 1);
    }
}
