/// Digital Twin — User Modeling (Phase 4)
///
/// BehaviorTracker, GoalModeler, DigitalTwin, UserProfile.
/// All topicWeights are clamped to [0.0, 1.0].
/// Export and delete capability for all profile data.

use std::collections::HashMap;

// ── BehaviorPattern ───────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct BehaviorEvent {
    pub event_type: String,
    pub timestamp: u64,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone)]
pub struct BehaviorPattern {
    pub pattern_id: String,
    pub description: String,
    pub frequency: u32,
    pub last_seen: u64,
}

pub struct BehaviorTracker {
    events: Vec<BehaviorEvent>,
    patterns: Vec<BehaviorPattern>,
}

impl BehaviorTracker {
    pub fn new() -> Self {
        Self { events: vec![], patterns: vec![] }
    }

    pub fn record_event(&mut self, event: BehaviorEvent) {
        self.events.push(event);
    }

    /// Detect patterns from recorded events.
    /// Stub: groups events by type and creates a pattern for each type seen >= 2 times.
    pub fn detect_patterns(&mut self) -> Vec<BehaviorPattern> {
        let mut counts: HashMap<String, u32> = HashMap::new();
        let mut last_seen: HashMap<String, u64> = HashMap::new();

        for event in &self.events {
            *counts.entry(event.event_type.clone()).or_insert(0) += 1;
            last_seen
                .entry(event.event_type.clone())
                .and_modify(|t| *t = (*t).max(event.timestamp))
                .or_insert(event.timestamp);
        }

        self.patterns = counts
            .iter()
            .filter(|(_, &c)| c >= 2)
            .map(|(event_type, &freq)| BehaviorPattern {
                pattern_id: format!("pat-{event_type}"),
                description: format!("Repeated: {event_type}"),
                frequency: freq,
                last_seen: *last_seen.get(event_type).unwrap_or(&0),
            })
            .collect();

        self.patterns.clone()
    }

    pub fn get_active_patterns(&self) -> &[BehaviorPattern] {
        &self.patterns
    }
}

impl Default for BehaviorTracker {
    fn default() -> Self {
        Self::new()
    }
}

// ── Goal ──────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct Goal {
    pub id: String,
    pub title: String,
    pub progress: f32, // [0.0, 1.0]
    pub milestones: Vec<String>,
    pub conflicts_with: Vec<String>, // ids of conflicting goals
}

pub struct GoalModeler {
    goals: Vec<Goal>,
    next_id: u64,
}

impl GoalModeler {
    pub fn new() -> Self {
        Self { goals: vec![], next_id: 1 }
    }

    /// Infer a goal from a text description.
    pub fn infer_goal(&mut self, description: &str) -> Goal {
        let id = format!("goal-{}", self.next_id);
        self.next_id += 1;
        let goal = Goal {
            id: id.clone(),
            title: description.to_string(),
            progress: 0.0,
            milestones: vec![],
            conflicts_with: vec![],
        };
        self.goals.push(goal.clone());
        goal
    }

    /// Track progress for a goal.
    pub fn track_progress(&mut self, goal_id: &str, progress: f32) -> bool {
        if let Some(goal) = self.goals.iter_mut().find(|g| g.id == goal_id) {
            goal.progress = progress.clamp(0.0, 1.0);
            true
        } else {
            false
        }
    }

    /// Suggest the next milestone for a goal.
    pub fn suggest_milestone(&self, goal_id: &str) -> Option<String> {
        self.goals
            .iter()
            .find(|g| g.id == goal_id)
            .map(|g| format!("Next step for: {}", g.title))
    }

    /// Detect conflicts between goals (stub: any two goals with same title prefix).
    pub fn detect_goal_conflict(&self) -> Vec<(String, String)> {
        let mut conflicts: Vec<(String, String)> = vec![];
        for i in 0..self.goals.len() {
            for j in (i + 1)..self.goals.len() {
                let a = &self.goals[i];
                let b = &self.goals[j];
                // Stub: flag as conflict if titles share first word.
                let a_word = a.title.split_whitespace().next().unwrap_or("");
                let b_word = b.title.split_whitespace().next().unwrap_or("");
                if !a_word.is_empty() && a_word == b_word {
                    conflicts.push((a.id.clone(), b.id.clone()));
                }
            }
        }
        conflicts
    }

    pub fn goals(&self) -> &[Goal] {
        &self.goals
    }
}

impl Default for GoalModeler {
    fn default() -> Self {
        Self::new()
    }
}

// ── UserProfile ───────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct ScheduleModel {
    pub wake_hour: u8,
    pub sleep_hour: u8,
    pub peak_productivity_hour: u8,
}

impl Default for ScheduleModel {
    fn default() -> Self {
        Self { wake_hour: 7, sleep_hour: 23, peak_productivity_hour: 10 }
    }
}

#[derive(Debug, Clone)]
pub struct UserProfile {
    pub user_id: String,
    pub habits: Vec<String>,
    pub goals: Vec<Goal>,
    pub behavior_patterns: Vec<BehaviorPattern>,
    /// All values clamped to [0.0, 1.0].
    pub topic_weights: HashMap<String, f32>,
    pub schedule_model: ScheduleModel,
}

impl UserProfile {
    pub fn new(user_id: impl Into<String>) -> Self {
        Self {
            user_id: user_id.into(),
            habits: vec![],
            goals: vec![],
            behavior_patterns: vec![],
            topic_weights: HashMap::new(),
            schedule_model: ScheduleModel::default(),
        }
    }

    /// Update a topic weight, clamping to [0.0, 1.0].
    pub fn update_topic_weight(&mut self, topic: impl Into<String>, weight: f32) {
        self.topic_weights.insert(topic.into(), weight.clamp(0.0, 1.0));
    }

    /// Export all profile data as a JSON-like string (stub).
    pub fn export(&self) -> String {
        format!(
            r#"{{"user_id":"{}","habits":{},"topic_weights":{}}}"#,
            self.user_id,
            self.habits.len(),
            self.topic_weights.len()
        )
    }

    /// Delete all profile data.
    pub fn delete_all(&mut self) {
        self.habits.clear();
        self.goals.clear();
        self.behavior_patterns.clear();
        self.topic_weights.clear();
    }
}

// ── PredictedNeed ─────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct PredictedNeed {
    pub description: String,
    pub urgency: f32, // [0.0, 1.0]
}

// ── DigitalTwin ───────────────────────────────────────────────────────────────

pub struct DigitalTwin {
    pub profile: UserProfile,
    behavior_tracker: BehaviorTracker,
    goal_modeler: GoalModeler,
}

impl DigitalTwin {
    pub fn new(user_id: impl Into<String>) -> Self {
        Self {
            profile: UserProfile::new(user_id),
            behavior_tracker: BehaviorTracker::new(),
            goal_modeler: GoalModeler::new(),
        }
    }

    /// Update the twin from a new interaction.
    pub fn update_from_interaction(&mut self, input: &str, topic: &str) {
        // Increment topic weight slightly.
        let current = self.profile.topic_weights.get(topic).copied().unwrap_or(0.0);
        self.profile.update_topic_weight(topic, current + 0.05);

        // Record a behavior event.
        self.behavior_tracker.record_event(BehaviorEvent {
            event_type: "interaction".to_string(),
            timestamp: 0,
            metadata: {
                let mut m = HashMap::new();
                m.insert("input".to_string(), input.to_string());
                m
            },
        });
    }

    /// Update the twin from a behavior event.
    pub fn update_from_behavior(&mut self, event: BehaviorEvent) {
        self.behavior_tracker.record_event(event);
        self.behavior_tracker.detect_patterns();
        self.profile.behavior_patterns =
            self.behavior_tracker.get_active_patterns().to_vec();
    }

    /// Get the current user profile.
    pub fn get_user_profile(&self) -> &UserProfile {
        &self.profile
    }

    /// Predict the user's next needs based on topic weights.
    /// Returns needs sorted by urgency descending.
    pub fn predict_next_need(&self) -> Vec<PredictedNeed> {
        let mut needs: Vec<PredictedNeed> = self
            .profile
            .topic_weights
            .iter()
            .map(|(topic, &weight)| PredictedNeed {
                description: format!("Continue with: {topic}"),
                urgency: weight.clamp(0.0, 1.0),
            })
            .collect();
        needs.sort_by(|a, b| {
            b.urgency
                .partial_cmp(&a.urgency)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        needs
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn topic_weights_always_clamped() {
        let mut profile = UserProfile::new("user1");
        profile.update_topic_weight("rust", 999.0);
        profile.update_topic_weight("flutter", -999.0);
        for &w in profile.topic_weights.values() {
            assert!(w >= 0.0 && w <= 1.0, "weight {w} out of range");
        }
    }

    #[test]
    fn predict_next_need_sorted_descending() {
        let mut twin = DigitalTwin::new("user1");
        twin.profile.update_topic_weight("rust", 0.9);
        twin.profile.update_topic_weight("flutter", 0.3);
        twin.profile.update_topic_weight("ai", 0.7);
        let needs = twin.predict_next_need();
        for i in 1..needs.len() {
            assert!(needs[i - 1].urgency >= needs[i].urgency);
        }
    }

    #[test]
    fn predict_next_need_urgency_in_range() {
        let mut twin = DigitalTwin::new("user1");
        twin.profile.update_topic_weight("topic", 0.5);
        let needs = twin.predict_next_need();
        for need in &needs {
            assert!(need.urgency >= 0.0 && need.urgency <= 1.0);
        }
    }

    #[test]
    fn behavior_tracker_detects_repeated_events() {
        let mut tracker = BehaviorTracker::new();
        for _ in 0..3 {
            tracker.record_event(BehaviorEvent {
                event_type: "click".to_string(),
                timestamp: 0,
                metadata: HashMap::new(),
            });
        }
        let patterns = tracker.detect_patterns();
        assert!(!patterns.is_empty());
        assert_eq!(patterns[0].frequency, 3);
    }

    #[test]
    fn goal_modeler_track_progress_clamped() {
        let mut gm = GoalModeler::new();
        let goal = gm.infer_goal("learn rust");
        gm.track_progress(&goal.id, 1.5);
        let g = gm.goals().iter().find(|g| g.id == goal.id).unwrap();
        assert!(g.progress <= 1.0);
    }

    #[test]
    fn user_profile_delete_clears_all() {
        let mut profile = UserProfile::new("u1");
        profile.habits.push("exercise".to_string());
        profile.update_topic_weight("rust", 0.8);
        profile.delete_all();
        assert!(profile.habits.is_empty());
        assert!(profile.topic_weights.is_empty());
    }

    #[test]
    fn digital_twin_update_from_interaction_increments_weight() {
        let mut twin = DigitalTwin::new("u1");
        twin.update_from_interaction("hello", "rust");
        let w = twin.profile.topic_weights.get("rust").copied().unwrap_or(0.0);
        assert!(w > 0.0);
    }

    // Property: all topicWeights remain in [0.0, 1.0] after any sequence of updates.
    #[test]
    fn topic_weights_always_in_range_after_any_sequence_of_updates() {
        let mut twin = DigitalTwin::new("u1");
        // Apply many interactions to drive weights up.
        for _ in 0..30 {
            twin.update_from_interaction("msg", "rust");
            twin.update_from_interaction("msg", "flutter");
            twin.update_from_interaction("msg", "ai");
        }
        for (&ref topic, &w) in &twin.profile.topic_weights {
            assert!(
                w >= 0.0 && w <= 1.0,
                "topic '{topic}' weight {w} out of [0.0, 1.0]"
            );
        }
    }

    // Property 12: predictNextNeed returns urgency values in [0.0, 1.0] sorted descending.
    #[test]
    fn predict_next_need_urgency_in_range_and_sorted_descending() {
        let mut twin = DigitalTwin::new("u1");
        twin.profile.update_topic_weight("rust", 0.9);
        twin.profile.update_topic_weight("flutter", 0.3);
        twin.profile.update_topic_weight("ai", 0.7);
        twin.profile.update_topic_weight("music", 0.1);

        let needs = twin.predict_next_need();
        for need in &needs {
            assert!(
                need.urgency >= 0.0 && need.urgency <= 1.0,
                "urgency {} out of [0.0, 1.0]",
                need.urgency
            );
        }
        for window in needs.windows(2) {
            assert!(
                window[0].urgency >= window[1].urgency,
                "needs must be sorted descending by urgency"
            );
        }
    }
}
