/// Central Brain — LLM + World Model
///
/// Provides the WorldModel (state tracking + future prediction),
/// ReasoningEngine (chain-of-thought via LLMCore), DecisionEngine
/// (best-action selection), and CentralBrain (top-level coordinator).

use std::collections::HashMap;

use crate::llm::{InferenceParams, LLMCore, ModelConfig, QuantizationLevel};

// ── WorldState ────────────────────────────────────────────────────────────────

pub struct WorldState {
    pub task_state: HashMap<String, String>,
    pub environment_state: HashMap<String, String>,
    pub user_state: HashMap<String, String>,
    pub predicted_future_states: Vec<HashMap<String, String>>,
    pub version: u64,
    /// Unix timestamp millis (stub: incremented by 1 on each update).
    pub last_updated_at: u64,
}

// ── WorldModelDelta ───────────────────────────────────────────────────────────

pub struct WorldModelDelta {
    pub task_updates: HashMap<String, String>,
    pub environment_updates: HashMap<String, String>,
    pub user_updates: HashMap<String, String>,
}

// ── WorldModel ────────────────────────────────────────────────────────────────

pub struct WorldModel {
    pub state: WorldState,
}

impl WorldModel {
    pub fn new() -> Self {
        Self {
            state: WorldState {
                task_state: HashMap::new(),
                environment_state: HashMap::new(),
                user_state: HashMap::new(),
                predicted_future_states: vec![],
                version: 0,
                last_updated_at: 0,
            },
        }
    }

    /// Increment version by 1, update last_updated_at, merge delta into state,
    /// then recompute future state predictions.
    pub fn update(&mut self, delta: WorldModelDelta) {
        self.state.version += 1;
        self.state.last_updated_at += 1; // stub: increment by 1
        self.state.task_state.extend(delta.task_updates);
        self.state.environment_state.extend(delta.environment_updates);
        self.state.user_state.extend(delta.user_updates);
        self.recompute_future_states();
    }

    /// Stub: same as update() — delegates directly.
    pub fn update_from_learning(&mut self, delta: WorldModelDelta) {
        self.update(delta);
    }

    /// Stub: clear predictions and push one empty prediction.
    fn recompute_future_states(&mut self) {
        self.state.predicted_future_states.clear();
        self.state.predicted_future_states.push(HashMap::new());
    }
}

impl Default for WorldModel {
    fn default() -> Self {
        Self::new()
    }
}

// ── ReasoningResult ───────────────────────────────────────────────────────────

pub struct ReasoningResult {
    pub goal: String,
    pub reasoning_steps: Vec<String>,
    pub confidence: f32,
}

// ── ReasoningEngine ───────────────────────────────────────────────────────────

pub struct ReasoningEngine {
    pub llm: LLMCore,
}

impl ReasoningEngine {
    pub fn new(llm: LLMCore) -> Self {
        Self { llm }
    }

    /// Run chain-of-thought reasoning over `prompt` with `steps` reasoning steps.
    pub fn chain_of_thought(&self, prompt: &str, steps: u32) -> anyhow::Result<ReasoningResult> {
        let result = self.llm.chain_of_thought(prompt, steps)?;
        Ok(ReasoningResult {
            goal: result.text,
            reasoning_steps: vec![],
            confidence: 0.8,
        })
    }
}

// ── ActionOption / DecisionEngine ────────────────────────────────────────────

pub struct ActionOption {
    pub id: String,
    pub description: String,
    pub score: f32,
}

pub struct DecisionEngine;

impl DecisionEngine {
    /// Returns the action with the highest score. Returns `None` if `options` is empty.
    pub fn select_best(options: Vec<ActionOption>) -> Option<ActionOption> {
        options
            .into_iter()
            .reduce(|best, candidate| {
                if candidate.score > best.score {
                    candidate
                } else {
                    best
                }
            })
    }
}

// ── CentralBrain ──────────────────────────────────────────────────────────────

pub struct CentralBrain {
    pub world_model: WorldModel,
    pub reasoning_engine: ReasoningEngine,
}

impl CentralBrain {
    pub fn new(llm: LLMCore) -> Self {
        Self {
            world_model: WorldModel::new(),
            reasoning_engine: ReasoningEngine::new(llm),
        }
    }

    /// Infer the user's goal from `prompt` using chain-of-thought reasoning.
    pub fn infer_goal(&self, prompt: &str) -> anyhow::Result<String> {
        let result = self.reasoning_engine.chain_of_thought(prompt, 3)?;
        Ok(result.goal)
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn make_llm() -> LLMCore {
        LLMCore::new(ModelConfig {
            model_path: "/models/test.gguf".to_string(),
            quantization: QuantizationLevel::Q4KM,
            context_size: 2048,
            n_threads: 2,
        })
    }

    fn empty_delta() -> WorldModelDelta {
        WorldModelDelta {
            task_updates: HashMap::new(),
            environment_updates: HashMap::new(),
            user_updates: HashMap::new(),
        }
    }

    #[test]
    fn world_model_version_increments() {
        let mut wm = WorldModel::new();
        wm.update(empty_delta());
        wm.update(empty_delta());
        assert_eq!(wm.state.version, 2);
    }

    #[test]
    fn world_model_version_strictly_monotonic() {
        let mut wm = WorldModel::new();
        let before = wm.state.version;
        wm.update(empty_delta());
        let after = wm.state.version;
        assert!(after > before);
    }

    #[test]
    fn decision_engine_selects_highest_score() {
        let options = vec![
            ActionOption { id: "a".to_string(), description: "low".to_string(), score: 0.2 },
            ActionOption { id: "b".to_string(), description: "high".to_string(), score: 0.9 },
            ActionOption { id: "c".to_string(), description: "mid".to_string(), score: 0.5 },
        ];
        let best = DecisionEngine::select_best(options).expect("should return Some");
        assert_eq!(best.id, "b");
    }

    #[test]
    fn decision_engine_empty_returns_none() {
        let result = DecisionEngine::select_best(vec![]);
        assert!(result.is_none());
    }

    #[test]
    fn central_brain_infer_goal_returns_string() {
        let brain = CentralBrain::new(make_llm());
        let goal = brain.infer_goal("test").expect("infer_goal should succeed");
        assert!(!goal.is_empty());
    }

    // Property 6: WorldModel.version is strictly monotonically increasing.
    #[test]
    fn world_model_version_strictly_monotonically_increasing() {
        let mut wm = WorldModel::new();
        let mut prev = wm.state.version;
        for _ in 0..10 {
            wm.update(empty_delta());
            let curr = wm.state.version;
            assert!(curr > prev, "version must strictly increase: {prev} -> {curr}");
            prev = curr;
        }
    }

    // Property 6 variant: update_from_learning also increments version.
    #[test]
    fn world_model_update_from_learning_increments_version() {
        let mut wm = WorldModel::new();
        let before = wm.state.version;
        wm.update_from_learning(empty_delta());
        assert!(wm.state.version > before);
    }
}
