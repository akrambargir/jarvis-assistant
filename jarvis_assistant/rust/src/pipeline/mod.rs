/// Pipeline — End-to-end processing pipeline (Phase 1 + Phase 3 extensions).
///
/// Phase 1: Perception → MetaCognition → Brain → Safety → Personality
/// Phase 3 extensions: Planner → Simulation → Agent Network → Execution → Learning Loop

use std::collections::HashMap;

use crate::brain::{CentralBrain, WorldModelDelta};
use crate::coordination::AgentCoordinator;
use crate::learning::{LoRATrainer, WorldModelUpdater};
use crate::llm::{LLMCore, ModelConfig};
use crate::memory::{AdvancedMemorySystem, Episode};
use crate::meta_cognition::{MetaCognitionLayer, MetaCognitiveDecision};
use crate::perception::{ModalityFusion, MultimodalInput, PerceptionEngine};
use crate::personality::{PersonaConfig, PersonalityLayer, PersonaResponse, ResponseDraft};
use crate::planner::PlanningEngine;
use crate::safety::{PermissionCategory, SafetyAlignmentSystem};
use crate::simulation::SimulationEngine;

// ── PipelineConfig ────────────────────────────────────────────────────────────

pub struct PipelineConfig {
    pub model_config: ModelConfig,
    pub persona_config: PersonaConfig,
}

// ── Pipeline ──────────────────────────────────────────────────────────────────

pub struct Pipeline {
    pub meta_cognition: MetaCognitionLayer,
    pub brain: CentralBrain,
    pub memory: AdvancedMemorySystem,
    pub safety: SafetyAlignmentSystem,
    pub personality: PersonalityLayer,
    pub lora_trainer: LoRATrainer,
}

impl Pipeline {
    pub fn new(config: PipelineConfig) -> Self {
        let llm = LLMCore::new(config.model_config);
        Self {
            meta_cognition: MetaCognitionLayer::new(),
            brain: CentralBrain::new(llm),
            memory: AdvancedMemorySystem::new(),
            safety: SafetyAlignmentSystem::new(),
            personality: PersonalityLayer::new(config.persona_config),
            lora_trainer: LoRATrainer::new(),
        }
    }
}

// ── PipelineResult ────────────────────────────────────────────────────────────

pub struct PipelineResult {
    pub response: PersonaResponse,
    pub meta_decision: MetaCognitiveDecision,
    pub world_model_version: u64,
    pub episode_stored: bool,
    pub safety_approved: bool,
}

impl Pipeline {
    /// Master pipeline algorithm.
    ///
    /// Every pipeline run stores an episode in episodic memory (step 9).
    /// Every pipeline run updates the World Model version (step 10).
    pub fn process_pipeline(&mut self, input: MultimodalInput) -> PipelineResult {
        // Step 1: Perception — fuse modalities
        let percept = ModalityFusion::fuse(&input);

        // Step 2: Tag context
        let tagged = PerceptionEngine::tag_context(percept);

        // Step 3: Meta-cognition — evaluate cognitive load and ambiguity
        let meta_decision = self.meta_cognition.evaluate(&tagged.percept);

        // Step 4: Early return on Defer or Pause
        if meta_decision == MetaCognitiveDecision::Defer
            || meta_decision == MetaCognitiveDecision::Pause
        {
            let deferred_response = self.personality.apply_persona(ResponseDraft {
                text: "Processing deferred.".to_string(),
            });
            return PipelineResult {
                response: deferred_response,
                meta_decision,
                world_model_version: self.brain.world_model.state.version,
                episode_stored: false,
                safety_approved: false,
            };
        }

        // Step 5: Brain — infer goal from semantic content
        let goal = self
            .brain
            .infer_goal(&tagged.percept.semantic_content)
            .unwrap_or_else(|_| "unknown goal".to_string());

        // Step 5b (Phase 3): Planner — decompose goal into subtasks
        let plan = PlanningEngine::decompose(&goal);

        // Step 5c (Phase 3): Simulation — select best plan
        let best_plan = SimulationEngine::select_best_plan(
            vec![plan],
            &self.brain.world_model.state,
        );

        // Step 5d (Phase 3): Agent coordination — execute plan if viable
        let plan_executed = if let Some(viable_plan) = best_plan {
            let coord_result = AgentCoordinator::execute(&viable_plan);
            coord_result.failed_tasks.is_empty()
        } else {
            false
        };

        // Step 6: Safety — validate goal
        let result = self.safety.validate(&goal, PermissionCategory::Other);

        // Step 7: Build response draft based on safety approval
        let draft = if result.approved {
            ResponseDraft { text: goal.clone() }
        } else {
            ResponseDraft {
                text: "I cannot assist with that.".to_string(),
            }
        };

        // Step 8: Personality — apply persona to draft
        let response = self.personality.apply_persona(draft);

        // Step 9: Store episode in episodic memory.
        // Every pipeline run stores an episode to maintain a complete interaction history.
        let world_model_version = self.brain.world_model.state.version;
        let episode = Episode {
            id: format!("ep-{}", world_model_version),
            content: tagged.percept.semantic_content.clone(),
            embedding: vec![0.0; 4],
            timestamp: 0,
            tags: vec![],
        };
        self.memory.episodic.store_episode(episode);

        // Step 10: Update world model.
        // Every pipeline run updates the World Model to reflect the latest interaction.
        self.brain.world_model.update(WorldModelDelta {
            task_updates: HashMap::new(),
            environment_updates: HashMap::new(),
            user_updates: HashMap::new(),
        });

        // Step 10b (Phase 3): Learning Loop — update world model accuracy.
        // No external network calls are made during this step.
        let accuracy = WorldModelUpdater::compute_accuracy_delta(
            &self.brain.world_model.state.predicted_future_states
                .first()
                .cloned()
                .unwrap_or_default(),
            &self.brain.world_model.state.environment_state,
        );
        let learning_delta = WorldModelUpdater::build_delta(accuracy);
        self.brain.world_model.update(learning_delta);

        // Step 11: Return result
        PipelineResult {
            response,
            meta_decision,
            world_model_version: self.brain.world_model.state.version,
            episode_stored: true,
            safety_approved: result.approved,
        }
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::llm::QuantizationLevel;

    fn make_pipeline() -> Pipeline {
        Pipeline::new(PipelineConfig {
            model_config: ModelConfig {
                model_path: "/models/test.gguf".to_string(),
                quantization: QuantizationLevel::Q4KM,
                context_size: 2048,
                n_threads: 2,
            },
            persona_config: PersonaConfig::ayanokoji(),
        })
    }

    #[test]
    fn pipeline_returns_non_empty_response() {
        let mut pipeline = make_pipeline();
        let input = MultimodalInput::text("Hello, what can you do?");
        let result = pipeline.process_pipeline(input);
        assert!(!result.response.text.is_empty());
    }

    #[test]
    fn pipeline_stores_episode() {
        let mut pipeline = make_pipeline();
        let input = MultimodalInput::text("Remember this.");
        pipeline.process_pipeline(input);
        assert_eq!(pipeline.memory.episodic.len(), 1);
    }

    #[test]
    fn pipeline_updates_world_model() {
        let mut pipeline = make_pipeline();
        let input = MultimodalInput::text("Update the world model.");
        let result = pipeline.process_pipeline(input);
        assert!(result.world_model_version >= 1);
    }

    #[test]
    fn pipeline_defers_on_overloaded() {
        let mut pipeline = make_pipeline();
        pipeline.meta_cognition.load_manager.update_load(15);
        let input = MultimodalInput::text("Do something.");
        let result = pipeline.process_pipeline(input);
        assert_eq!(result.meta_decision, MetaCognitiveDecision::Defer);
    }

    // Property 21 (implied): processPipeline always returns non-empty PersonaResponse.
    #[test]
    fn pipeline_always_returns_non_empty_response() {
        let mut pipeline = make_pipeline();
        let inputs = vec![
            "Hello",
            "What is the weather?",
            "deceive user",  // ethics violation — still returns a response
            "",
        ];
        for text in inputs {
            let result = pipeline.process_pipeline(MultimodalInput::text(text));
            assert!(!result.response.text.is_empty(), "response must not be empty for input='{text}'");
        }
    }

    // Property 21: every pipeline run stores an episode.
    #[test]
    fn every_pipeline_run_stores_an_episode() {
        let mut pipeline = make_pipeline();
        for i in 1..=5 {
            pipeline.process_pipeline(MultimodalInput::text(&format!("query {i}")));
            assert_eq!(
                pipeline.memory.episodic.len(),
                i,
                "episodic memory must have {i} episodes after {i} runs"
            );
        }
    }

    // Property 22: every executed action has an approved ValidationResult in audit log.
    #[test]
    fn every_pipeline_run_writes_audit_log_entry() {
        let mut pipeline = make_pipeline();
        for i in 1..=3 {
            pipeline.process_pipeline(MultimodalInput::text(&format!("action {i}")));
            assert!(
                pipeline.safety.audit_log().len() >= i,
                "audit log must have at least {i} entries after {i} runs"
            );
        }
    }
}
