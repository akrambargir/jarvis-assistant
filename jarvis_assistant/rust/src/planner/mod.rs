/// Planner System
///
/// Decomposes goals into SubTask dependency graphs, validates acyclicity,
/// matches tasks to agent capabilities, and supports replanning on failure.

use std::collections::{HashMap, HashSet, VecDeque};

// ── SubTask ───────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TaskStatus {
    Pending,
    InProgress,
    Completed,
    Failed,
}

#[derive(Debug, Clone)]
pub struct SubTask {
    pub id: String,
    pub description: String,
    /// IDs of tasks that must complete before this one can start.
    pub depends_on: Vec<String>,
    pub required_capability: Option<String>,
    pub status: TaskStatus,
    pub priority: u8, // 0 = lowest, 255 = highest
}

impl SubTask {
    pub fn new(id: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            description: description.into(),
            depends_on: vec![],
            required_capability: None,
            status: TaskStatus::Pending,
            priority: 128,
        }
    }

    pub fn with_deps(mut self, deps: Vec<&str>) -> Self {
        self.depends_on = deps.iter().map(|s| s.to_string()).collect();
        self
    }

    pub fn with_capability(mut self, cap: impl Into<String>) -> Self {
        self.required_capability = Some(cap.into());
        self
    }
}

// ── Plan ─────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct Plan {
    pub goal: String,
    pub tasks: Vec<SubTask>,
}

impl Plan {
    pub fn new(goal: impl Into<String>, tasks: Vec<SubTask>) -> Self {
        Self { goal: goal.into(), tasks }
    }

    /// Returns a reference to a task by id.
    pub fn get_task(&self, id: &str) -> Option<&SubTask> {
        self.tasks.iter().find(|t| t.id == id)
    }

    /// Returns a mutable reference to a task by id.
    pub fn get_task_mut(&mut self, id: &str) -> Option<&mut SubTask> {
        self.tasks.iter_mut().find(|t| t.id == id)
    }
}

// ── PlanningEngine ────────────────────────────────────────────────────────────

pub struct PlanningEngine;

impl PlanningEngine {
    /// Decompose a goal string into a list of SubTasks.
    ///
    /// Stub: produces a linear chain of 3 tasks derived from the goal.
    /// Real implementation would use LLM-based decomposition.
    pub fn decompose(goal: &str) -> Plan {
        let t1 = SubTask::new("t1", format!("Analyse: {goal}"));
        let t2 = SubTask::new("t2", format!("Plan: {goal}")).with_deps(vec!["t1"]);
        let t3 = SubTask::new("t3", format!("Execute: {goal}")).with_deps(vec!["t2"]);
        Plan::new(goal, vec![t1, t2, t3])
    }

    /// Validate that the dependency graph is acyclic using Kahn's algorithm
    /// (topological sort). Returns `Ok(sorted_ids)` or `Err(cycle_description)`.
    pub fn validate_acyclic(plan: &Plan) -> Result<Vec<String>, String> {
        let mut in_degree: HashMap<&str, usize> = HashMap::new();
        let mut adj: HashMap<&str, Vec<&str>> = HashMap::new();

        for task in &plan.tasks {
            in_degree.entry(task.id.as_str()).or_insert(0);
            adj.entry(task.id.as_str()).or_default();
        }

        for task in &plan.tasks {
            for dep in &task.depends_on {
                *in_degree.entry(task.id.as_str()).or_insert(0) += 1;
                adj.entry(dep.as_str()).or_default().push(task.id.as_str());
            }
        }

        let mut queue: VecDeque<&str> = in_degree
            .iter()
            .filter(|(_, &d)| d == 0)
            .map(|(&id, _)| id)
            .collect();

        let mut sorted: Vec<String> = vec![];

        while let Some(node) = queue.pop_front() {
            sorted.push(node.to_string());
            if let Some(neighbors) = adj.get(node) {
                for &neighbor in neighbors {
                    let deg = in_degree.get_mut(neighbor).unwrap();
                    *deg -= 1;
                    if *deg == 0 {
                        queue.push_back(neighbor);
                    }
                }
            }
        }

        if sorted.len() == plan.tasks.len() {
            Ok(sorted)
        } else {
            Err("Dependency graph contains a cycle".to_string())
        }
    }

    /// Match tasks to agent capabilities.
    /// Returns a map of task_id → agent_id for tasks that have a required_capability.
    pub fn match_capabilities(
        plan: &Plan,
        available_agents: &HashMap<String, Vec<String>>,
    ) -> HashMap<String, String> {
        let mut assignments: HashMap<String, String> = HashMap::new();
        for task in &plan.tasks {
            if let Some(cap) = &task.required_capability {
                for (agent_id, caps) in available_agents {
                    if caps.contains(cap) {
                        assignments.insert(task.id.clone(), agent_id.clone());
                        break;
                    }
                }
            }
        }
        assignments
    }
}

// ── ReflectionSystem ─────────────────────────────────────────────────────────

pub struct ConsistencyReport {
    pub is_consistent: bool,
    pub issues: Vec<String>,
}

pub struct ReflectionSystem;

impl ReflectionSystem {
    /// Check plan consistency — read-only, never mutates the plan.
    ///
    /// Checks:
    /// 1. All dependency IDs reference existing tasks.
    /// 2. Dependency graph is acyclic.
    pub fn check_consistency(plan: &Plan) -> ConsistencyReport {
        let mut issues: Vec<String> = vec![];
        let task_ids: HashSet<&str> =
            plan.tasks.iter().map(|t| t.id.as_str()).collect();

        // Check all dep references are valid.
        for task in &plan.tasks {
            for dep in &task.depends_on {
                if !task_ids.contains(dep.as_str()) {
                    issues.push(format!(
                        "Task '{}' depends on unknown task '{}'",
                        task.id, dep
                    ));
                }
            }
        }

        // Check acyclicity.
        if let Err(e) = PlanningEngine::validate_acyclic(plan) {
            issues.push(e);
        }

        ConsistencyReport {
            is_consistent: issues.is_empty(),
            issues,
        }
    }
}

// ── PlannerSystem ─────────────────────────────────────────────────────────────

pub struct PlannerSystem;

impl PlannerSystem {
    /// Replan after a SubTask failure.
    ///
    /// Marks the failed task and all tasks that transitively depend on it
    /// as Failed, then returns a new Plan with only the remaining Pending tasks.
    pub fn replan(plan: &mut Plan, failed_task_id: &str) -> Plan {
        // Collect all tasks that transitively depend on the failed task.
        let mut failed_ids: HashSet<String> = HashSet::new();
        failed_ids.insert(failed_task_id.to_string());

        // BFS over dependents.
        let mut queue: VecDeque<String> = VecDeque::new();
        queue.push_back(failed_task_id.to_string());

        while let Some(current) = queue.pop_front() {
            for task in &plan.tasks {
                if task.depends_on.contains(&current) && !failed_ids.contains(&task.id) {
                    failed_ids.insert(task.id.clone());
                    queue.push_back(task.id.clone());
                }
            }
        }

        // Mark failed tasks.
        for task in &mut plan.tasks {
            if failed_ids.contains(&task.id) {
                task.status = TaskStatus::Failed;
            }
        }

        // Build new plan with only pending tasks.
        let remaining: Vec<SubTask> = plan
            .tasks
            .iter()
            .filter(|t| t.status == TaskStatus::Pending)
            .cloned()
            .collect();

        Plan::new(plan.goal.clone(), remaining)
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decompose_produces_acyclic_graph() {
        let plan = PlanningEngine::decompose("write a report");
        let result = PlanningEngine::validate_acyclic(&plan);
        assert!(result.is_ok(), "expected acyclic, got: {:?}", result.err());
    }

    #[test]
    fn decompose_produces_three_tasks() {
        let plan = PlanningEngine::decompose("test goal");
        assert_eq!(plan.tasks.len(), 3);
    }

    #[test]
    fn cyclic_graph_detected() {
        let t1 = SubTask::new("a", "A").with_deps(vec!["b"]);
        let t2 = SubTask::new("b", "B").with_deps(vec!["a"]);
        let plan = Plan::new("cycle", vec![t1, t2]);
        let result = PlanningEngine::validate_acyclic(&plan);
        assert!(result.is_err());
    }

    #[test]
    fn topological_sort_respects_dependencies() {
        let plan = PlanningEngine::decompose("ordered goal");
        let sorted = PlanningEngine::validate_acyclic(&plan).unwrap();
        // t1 must come before t2, t2 before t3
        let pos: HashMap<&str, usize> = sorted
            .iter()
            .enumerate()
            .map(|(i, id)| (id.as_str(), i))
            .collect();
        assert!(pos["t1"] < pos["t2"]);
        assert!(pos["t2"] < pos["t3"]);
    }

    #[test]
    fn reflection_system_is_read_only() {
        let plan = PlanningEngine::decompose("check consistency");
        let original_len = plan.tasks.len();
        let report = ReflectionSystem::check_consistency(&plan);
        assert!(report.is_consistent);
        assert_eq!(plan.tasks.len(), original_len, "plan must not be mutated");
    }

    #[test]
    fn reflection_detects_unknown_dependency() {
        let t1 = SubTask::new("t1", "task 1").with_deps(vec!["ghost"]);
        let plan = Plan::new("bad plan", vec![t1]);
        let report = ReflectionSystem::check_consistency(&plan);
        assert!(!report.is_consistent);
        assert!(!report.issues.is_empty());
    }

    #[test]
    fn replan_removes_failed_and_dependents() {
        let mut plan = PlanningEngine::decompose("replan test");
        let new_plan = PlannerSystem::replan(&mut plan, "t1");
        // t1 failed → t2 and t3 also fail (they depend on t1 transitively)
        assert!(new_plan.tasks.is_empty());
    }

    #[test]
    fn capability_matching_assigns_correct_agent() {
        let mut plan = PlanningEngine::decompose("cap test");
        plan.tasks[0].required_capability = Some("search".to_string());

        let mut agents: HashMap<String, Vec<String>> = HashMap::new();
        agents.insert("web_agent".to_string(), vec!["search".to_string()]);

        let assignments = PlanningEngine::match_capabilities(&plan, &agents);
        assert_eq!(assignments.get("t1").map(|s| s.as_str()), Some("web_agent"));
    }

    // Property 7: decompose always produces acyclic dependency graph.
    #[test]
    fn decompose_always_produces_acyclic_graph_for_any_goal() {
        let goals = ["", "simple goal", "a very complex multi-step goal with many dependencies"];
        for goal in goals {
            let plan = PlanningEngine::decompose(goal);
            let result = PlanningEngine::validate_acyclic(&plan);
            assert!(result.is_ok(), "decompose must produce acyclic graph for goal='{goal}'");
        }
    }

    // Property 7 variant: checkConsistency never mutates the input plan.
    #[test]
    fn check_consistency_never_mutates_plan() {
        let plan = PlanningEngine::decompose("immutability test");
        let original_task_count = plan.tasks.len();
        let original_goal = plan.goal.clone();
        let original_ids: Vec<String> = plan.tasks.iter().map(|t| t.id.clone()).collect();

        let _report = ReflectionSystem::check_consistency(&plan);

        assert_eq!(plan.tasks.len(), original_task_count, "task count must not change");
        assert_eq!(plan.goal, original_goal, "goal must not change");
        let ids_after: Vec<String> = plan.tasks.iter().map(|t| t.id.clone()).collect();
        assert_eq!(ids_after, original_ids, "task ids must not change");
    }
}
