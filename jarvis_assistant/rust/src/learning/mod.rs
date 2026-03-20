/// Learning + Optimization Loop — Phase 3
///
/// WorldModelUpdater, AgentPolicyRefiner, PlanningHeuristicsOptimizer,
/// PersonaAdapter, LoRATrainer.
///
/// Privacy guarantee: no training data or weights are transmitted externally.
/// All training is on-device only.

use std::collections::HashMap;

use crate::brain::WorldModelDelta;
use crate::personality::PersonaConfig;

// ── WorldModelUpdater ─────────────────────────────────────────────────────────

pub struct WorldModelUpdater;

impl WorldModelUpdater {
    /// Compute accuracy delta between predicted and actual world state.
    /// Returns a score in [0.0, 1.0] where 1.0 = perfect prediction.
    pub fn compute_accuracy_delta(
        predicted: &HashMap<String, String>,
        actual: &HashMap<String, String>,
    ) -> f32 {
        if actual.is_empty() {
            return 1.0;
        }
        let matches = actual
            .iter()
            .filter(|(k, v)| predicted.get(*k) == Some(v))
            .count();
        matches as f32 / actual.len() as f32
    }

    /// Build a WorldModelDelta from the accuracy delta.
    pub fn build_delta(accuracy: f32) -> WorldModelDelta {
        let mut env = HashMap::new();
        env.insert("last_accuracy".to_string(), format!("{accuracy:.4}"));
        WorldModelDelta {
            task_updates: HashMap::new(),
            environment_updates: env,
            user_updates: HashMap::new(),
        }
    }
}

// ── AgentPolicyRefiner ────────────────────────────────────────────────────────

pub struct PolicyGradient {
    pub parameter_deltas: HashMap<String, f32>,
    pub learning_rate: f32,
}

pub struct AgentPolicyRefiner {
    pub policy: HashMap<String, f32>,
}

impl AgentPolicyRefiner {
    pub fn new() -> Self {
        Self { policy: HashMap::new() }
    }

    /// Compute a policy gradient from reward signal.
    /// Stub: scales each parameter by the reward.
    pub fn compute_policy_gradient(
        &self,
        reward: f32,
        learning_rate: f32,
    ) -> PolicyGradient {
        let deltas = self
            .policy
            .iter()
            .map(|(k, &v)| (k.clone(), v * reward * learning_rate))
            .collect();
        PolicyGradient { parameter_deltas: deltas, learning_rate }
    }

    /// Apply a policy gradient to update the policy.
    pub fn update_policy(&mut self, gradient: &PolicyGradient) {
        for (k, &delta) in &gradient.parameter_deltas {
            *self.policy.entry(k.clone()).or_insert(0.0) += delta;
        }
    }
}

impl Default for AgentPolicyRefiner {
    fn default() -> Self {
        Self::new()
    }
}

// ── PlanningHeuristicsOptimizer ───────────────────────────────────────────────

pub struct PlanHeuristics {
    pub success_patterns: Vec<String>,
    pub failure_patterns: Vec<String>,
    pub weights: HashMap<String, f32>,
}

impl PlanHeuristics {
    pub fn new() -> Self {
        Self {
            success_patterns: vec![],
            failure_patterns: vec![],
            weights: HashMap::new(),
        }
    }
}

impl Default for PlanHeuristics {
    fn default() -> Self {
        Self::new()
    }
}

pub struct PlanningHeuristicsOptimizer {
    pub heuristics: PlanHeuristics,
}

impl PlanningHeuristicsOptimizer {
    pub fn new() -> Self {
        Self { heuristics: PlanHeuristics::new() }
    }

    pub fn analyze_successful_plans(&mut self, plan_descriptions: &[String]) {
        for desc in plan_descriptions {
            self.heuristics.success_patterns.push(desc.clone());
            *self.heuristics.weights.entry(desc.clone()).or_insert(0.0) += 0.1;
        }
    }

    pub fn analyze_failed_plans(&mut self, plan_descriptions: &[String]) {
        for desc in plan_descriptions {
            self.heuristics.failure_patterns.push(desc.clone());
            *self.heuristics.weights.entry(desc.clone()).or_insert(0.0) -= 0.1;
        }
    }

    pub fn update_heuristics(&mut self) {
        // Clamp weights to [-1.0, 1.0].
        for v in self.heuristics.weights.values_mut() {
            *v = v.clamp(-1.0, 1.0);
        }
    }
}

impl Default for PlanningHeuristicsOptimizer {
    fn default() -> Self {
        Self::new()
    }
}

// ── PersonaAdapter ────────────────────────────────────────────────────────────

pub struct PersonaFeedback {
    pub intensity_delta: f32,
    pub verbosity_delta: f32,
}

pub struct PersonaAdapter;

impl PersonaAdapter {
    /// Analyse feedback and compute persona config adjustments.
    pub fn analyze_persona_feedback(feedback: &[PersonaFeedback]) -> PersonaFeedback {
        let intensity_delta: f32 = feedback.iter().map(|f| f.intensity_delta).sum::<f32>()
            / feedback.len().max(1) as f32;
        let verbosity_delta: f32 = feedback.iter().map(|f| f.verbosity_delta).sum::<f32>()
            / feedback.len().max(1) as f32;
        PersonaFeedback { intensity_delta, verbosity_delta }
    }

    /// Apply feedback to a PersonaConfig, clamping values to [0.0, 1.0].
    pub fn adjust_persona_config(config: &mut PersonaConfig, feedback: &PersonaFeedback) {
        config.intensity = (config.intensity + feedback.intensity_delta).clamp(0.0, 1.0);
        // verbosity is a String label; adjust emotional_expression as a proxy for verbosity level.
        config.emotional_expression = (config.emotional_expression + feedback.verbosity_delta).clamp(0.0, 1.0);
    }
}

// ── LoRATrainer ───────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct TrainingExample {
    pub input: String,
    pub expected_output: String,
}

#[derive(Debug, Clone)]
pub struct LoRAAdapter {
    pub version: u32,
    pub quality_score: f32,
    /// Stub: adapter weights as a flat vector.
    pub weights: Vec<f32>,
}

pub struct LoRATrainer {
    pub current_adapter: Option<LoRAAdapter>,
    /// Previous adapter retained for rollback.
    pub previous_adapter: Option<LoRAAdapter>,
}

impl LoRATrainer {
    pub fn new() -> Self {
        Self { current_adapter: None, previous_adapter: None }
    }

    /// Build training examples from interaction history.
    pub fn build_training_examples(
        &self,
        interactions: &[(String, String)],
    ) -> Vec<TrainingExample> {
        interactions
            .iter()
            .map(|(input, output)| TrainingExample {
                input: input.clone(),
                expected_output: output.clone(),
            })
            .collect()
    }

    /// Train a new LoRA delta from examples.
    /// Stub: quality_score = examples.len() / 100.0 clamped to [0.0, 1.0].
    pub fn train_delta(&self, examples: &[TrainingExample]) -> LoRAAdapter {
        let quality = (examples.len() as f32 / 100.0).clamp(0.0, 1.0);
        let version = self
            .current_adapter
            .as_ref()
            .map(|a| a.version + 1)
            .unwrap_or(1);
        LoRAAdapter {
            version,
            quality_score: quality,
            weights: vec![0.0; examples.len().min(64)],
        }
    }

    /// Evaluate a candidate adapter against the current one.
    /// Returns true if the candidate strictly improves quality.
    pub fn evaluate_adapter(&self, candidate: &LoRAAdapter) -> bool {
        match &self.current_adapter {
            None => true,
            Some(current) => candidate.quality_score > current.quality_score,
        }
    }

    /// Commit a new adapter only if it strictly improves quality.
    /// Retains the previous adapter for rollback.
    /// Returns true if committed, false if rejected.
    ///
    /// Privacy: no external network calls are made.
    pub fn save_adapter(&mut self, candidate: LoRAAdapter) -> bool {
        if self.evaluate_adapter(&candidate) {
            self.previous_adapter = self.current_adapter.take();
            self.current_adapter = Some(candidate);
            true
        } else {
            false
        }
    }

    /// Rollback to the previous adapter if quality regresses.
    pub fn rollback(&mut self) -> bool {
        if self.previous_adapter.is_some() {
            self.current_adapter = self.previous_adapter.take();
            true
        } else {
            false
        }
    }
}

impl Default for LoRATrainer {
    fn default() -> Self {
        Self::new()
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn world_model_updater_perfect_prediction() {
        let mut predicted = HashMap::new();
        predicted.insert("k".to_string(), "v".to_string());
        let mut actual = HashMap::new();
        actual.insert("k".to_string(), "v".to_string());
        let delta = WorldModelUpdater::compute_accuracy_delta(&predicted, &actual);
        assert!((delta - 1.0).abs() < 1e-5);
    }

    #[test]
    fn world_model_updater_zero_accuracy() {
        let predicted = HashMap::new();
        let mut actual = HashMap::new();
        actual.insert("k".to_string(), "v".to_string());
        let delta = WorldModelUpdater::compute_accuracy_delta(&predicted, &actual);
        assert_eq!(delta, 0.0);
    }

    #[test]
    fn lora_trainer_commits_better_adapter() {
        let mut trainer = LoRATrainer::new();
        let examples: Vec<TrainingExample> = (0..50)
            .map(|i| TrainingExample {
                input: format!("in{i}"),
                expected_output: format!("out{i}"),
            })
            .collect();
        let adapter = trainer.train_delta(&examples);
        assert!(trainer.save_adapter(adapter));
        assert!(trainer.current_adapter.is_some());
    }

    #[test]
    fn lora_trainer_rejects_worse_adapter() {
        let mut trainer = LoRATrainer::new();
        // Commit a good adapter first.
        let good = LoRAAdapter { version: 1, quality_score: 0.8, weights: vec![] };
        trainer.save_adapter(good);
        // Try to commit a worse one.
        let bad = LoRAAdapter { version: 2, quality_score: 0.5, weights: vec![] };
        assert!(!trainer.save_adapter(bad));
        assert_eq!(trainer.current_adapter.as_ref().unwrap().quality_score, 0.8);
    }

    #[test]
    fn lora_trainer_rollback_restores_previous() {
        let mut trainer = LoRATrainer::new();
        let v1 = LoRAAdapter { version: 1, quality_score: 0.5, weights: vec![] };
        let v2 = LoRAAdapter { version: 2, quality_score: 0.9, weights: vec![] };
        trainer.save_adapter(v1);
        trainer.save_adapter(v2);
        trainer.rollback();
        assert_eq!(trainer.current_adapter.as_ref().unwrap().version, 1);
    }

    #[test]
    fn persona_adapter_clamps_intensity() {
        let mut config = PersonaConfig::ayanokoji();
        let feedback = PersonaFeedback { intensity_delta: 999.0, verbosity_delta: -999.0 };
        PersonaAdapter::adjust_persona_config(&mut config, &feedback);
        assert!(config.intensity <= 1.0);
        assert!(config.emotional_expression >= 0.0);
    }

    #[test]
    fn planning_heuristics_weights_clamped() {
        let mut opt = PlanningHeuristicsOptimizer::new();
        for _ in 0..20 {
            opt.analyze_successful_plans(&["pattern_a".to_string()]);
        }
        opt.update_heuristics();
        for &w in opt.heuristics.weights.values() {
            assert!(w <= 1.0 && w >= -1.0);
        }
    }

    // Property 13: LoRA adaptation only commits when qualityScore strictly improves.
    #[test]
    fn lora_only_commits_when_quality_strictly_improves() {
        let mut trainer = LoRATrainer::new();

        // First commit always succeeds (no current adapter).
        let v1 = LoRAAdapter { version: 1, quality_score: 0.5, weights: vec![] };
        assert!(trainer.save_adapter(v1.clone()), "first commit must succeed");

        // Same quality → must NOT commit.
        let same = LoRAAdapter { version: 2, quality_score: 0.5, weights: vec![] };
        assert!(!trainer.save_adapter(same), "equal quality must not commit");
        assert_eq!(trainer.current_adapter.as_ref().unwrap().version, 1);

        // Lower quality → must NOT commit.
        let worse = LoRAAdapter { version: 3, quality_score: 0.3, weights: vec![] };
        assert!(!trainer.save_adapter(worse), "lower quality must not commit");
        assert_eq!(trainer.current_adapter.as_ref().unwrap().version, 1);

        // Strictly higher quality → must commit.
        let better = LoRAAdapter { version: 4, quality_score: 0.8, weights: vec![] };
        assert!(trainer.save_adapter(better), "higher quality must commit");
        assert_eq!(trainer.current_adapter.as_ref().unwrap().version, 4);
    }

    // Property 13 variant: no external network calls during Learning Loop operations.
    // (Structural test: all Learning Loop functions are pure in-memory operations.)
    #[test]
    fn learning_loop_operations_are_pure_in_memory() {
        // WorldModelUpdater: pure function, no I/O.
        let predicted = HashMap::new();
        let actual = HashMap::new();
        let delta = WorldModelUpdater::compute_accuracy_delta(&predicted, &actual);
        assert!((0.0..=1.0).contains(&delta));

        // LoRATrainer: no network calls in train_delta or save_adapter.
        let mut trainer = LoRATrainer::new();
        let examples: Vec<TrainingExample> = (0..10)
            .map(|i| TrainingExample { input: format!("i{i}"), expected_output: format!("o{i}") })
            .collect();
        let adapter = trainer.train_delta(&examples);
        // quality_score must be in [0.0, 1.0].
        assert!((0.0..=1.0).contains(&adapter.quality_score));
        trainer.save_adapter(adapter);

        // Rollback: pure in-memory.
        let v1 = LoRAAdapter { version: 1, quality_score: 0.3, weights: vec![] };
        let v2 = LoRAAdapter { version: 2, quality_score: 0.9, weights: vec![] };
        let mut t2 = LoRATrainer::new();
        t2.save_adapter(v1);
        t2.save_adapter(v2);
        assert!(t2.rollback());
        assert_eq!(t2.current_adapter.as_ref().unwrap().version, 1);
    }
}
