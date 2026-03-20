/// Multi-Agent Coordination Algorithm — Phase 3
///
/// Topological sort of SubTask dependency graph, concurrent dispatch of
/// independent tasks, dependency ordering enforcement, and CoordinationResult.

use std::collections::{HashMap, HashSet, VecDeque};

use crate::planner::{Plan, SubTask, TaskStatus};

// ── TaskResult ────────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct TaskResult {
    pub task_id: String,
    pub output: String,
    pub success: bool,
}

// ── CoordinationResult ────────────────────────────────────────────────────────

#[derive(Debug)]
pub struct CoordinationResult {
    pub completed_results: Vec<TaskResult>,
    pub failed_tasks: Vec<String>,
}

impl CoordinationResult {
    /// Total tasks accounted for = completed + failed.
    pub fn total(&self) -> usize {
        self.completed_results.len() + self.failed_tasks.len()
    }
}

// ── AgentCoordinator ──────────────────────────────────────────────────────────

pub struct AgentCoordinator;

impl AgentCoordinator {
    /// Topological sort of the plan's dependency graph.
    /// Returns `Ok(sorted_ids)` or `Err(cycle_description)`.
    pub fn topological_sort(plan: &Plan) -> Result<Vec<String>, String> {
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

    /// Execute a plan respecting dependency ordering.
    ///
    /// Tasks are dispatched in topological order. Independent tasks (same
    /// dependency depth) are conceptually concurrent — in this stub they are
    /// executed sequentially but grouped by wave.
    ///
    /// Invariant: no task is dispatched before all its `depends_on` tasks
    /// appear in `completed_results`.
    pub fn execute(plan: &Plan) -> CoordinationResult {
        let sorted = match Self::topological_sort(plan) {
            Ok(s) => s,
            Err(e) => {
                return CoordinationResult {
                    completed_results: vec![],
                    failed_tasks: plan.tasks.iter().map(|t| t.id.clone()).collect(),
                };
            }
        };

        let task_map: HashMap<&str, &SubTask> =
            plan.tasks.iter().map(|t| (t.id.as_str(), t)).collect();

        let mut completed_results: Vec<TaskResult> = vec![];
        let mut failed_tasks: Vec<String> = vec![];
        let mut completed_ids: HashSet<String> = HashSet::new();

        for task_id in &sorted {
            let task = match task_map.get(task_id.as_str()) {
                Some(t) => t,
                None => continue,
            };

            // Enforce dependency ordering: all depends_on must be completed.
            let deps_satisfied = task
                .depends_on
                .iter()
                .all(|dep| completed_ids.contains(dep));

            if !deps_satisfied {
                failed_tasks.push(task_id.clone());
                continue;
            }

            // Stub execution: always succeeds.
            let result = TaskResult {
                task_id: task_id.clone(),
                output: format!("completed: {}", task.description),
                success: true,
            };
            completed_ids.insert(task_id.clone());
            completed_results.push(result);
        }

        CoordinationResult { completed_results, failed_tasks }
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::planner::{PlanningEngine, SubTask};

    #[test]
    fn topological_sort_respects_dependencies() {
        let plan = PlanningEngine::decompose("coord test");
        let sorted = AgentCoordinator::topological_sort(&plan).unwrap();
        let pos: HashMap<&str, usize> = sorted
            .iter()
            .enumerate()
            .map(|(i, id)| (id.as_str(), i))
            .collect();
        assert!(pos["t1"] < pos["t2"]);
        assert!(pos["t2"] < pos["t3"]);
    }

    #[test]
    fn execute_completes_all_tasks() {
        let plan = PlanningEngine::decompose("execute all");
        let result = AgentCoordinator::execute(&plan);
        assert_eq!(result.completed_results.len(), 3);
        assert!(result.failed_tasks.is_empty());
    }

    #[test]
    fn completed_plus_failed_equals_total_tasks() {
        let plan = PlanningEngine::decompose("accounting test");
        let total = plan.tasks.len();
        let result = AgentCoordinator::execute(&plan);
        assert_eq!(result.total(), total);
    }

    #[test]
    fn no_task_dispatched_before_dependencies_complete() {
        // Build a plan where t2 depends on t1.
        let t1 = SubTask::new("t1", "first");
        let t2 = SubTask::new("t2", "second").with_deps(vec!["t1"]);
        let plan = Plan::new("dep test", vec![t1, t2]);

        let result = AgentCoordinator::execute(&plan);
        // Both should complete.
        assert_eq!(result.completed_results.len(), 2);
        // t1 must appear before t2 in results.
        let ids: Vec<&str> = result.completed_results.iter().map(|r| r.task_id.as_str()).collect();
        let pos_t1 = ids.iter().position(|&id| id == "t1").unwrap();
        let pos_t2 = ids.iter().position(|&id| id == "t2").unwrap();
        assert!(pos_t1 < pos_t2);
    }

    #[test]
    fn cyclic_plan_fails_all_tasks() {
        let t1 = SubTask::new("a", "A").with_deps(vec!["b"]);
        let t2 = SubTask::new("b", "B").with_deps(vec!["a"]);
        let plan = Plan::new("cycle", vec![t1, t2]);
        let result = AgentCoordinator::execute(&plan);
        assert_eq!(result.failed_tasks.len(), 2);
        assert!(result.completed_results.is_empty());
    }

    // Property 9: no SubTask dispatched before its dependsOn tasks complete.
    #[test]
    fn no_task_dispatched_before_all_dependencies_complete() {
        // Build a diamond dependency: t1 → t2, t1 → t3, t2+t3 → t4.
        let t1 = SubTask::new("t1", "root");
        let t2 = SubTask::new("t2", "branch-a").with_deps(vec!["t1"]);
        let t3 = SubTask::new("t3", "branch-b").with_deps(vec!["t1"]);
        let t4 = SubTask::new("t4", "merge").with_deps(vec!["t2", "t3"]);
        let plan = Plan::new("diamond", vec![t1, t2, t3, t4]);

        let result = AgentCoordinator::execute(&plan);
        assert_eq!(result.completed_results.len(), 4);
        assert!(result.failed_tasks.is_empty());

        // Verify ordering: t1 before t2 and t3; t2 and t3 before t4.
        let ids: Vec<&str> = result.completed_results.iter().map(|r| r.task_id.as_str()).collect();
        let pos = |id: &str| ids.iter().position(|&x| x == id).unwrap();
        assert!(pos("t1") < pos("t2"));
        assert!(pos("t1") < pos("t3"));
        assert!(pos("t2") < pos("t4"));
        assert!(pos("t3") < pos("t4"));
    }

    // Property 9 variant: completedResults + failedTasks always equals total task count.
    #[test]
    fn completed_plus_failed_always_equals_total_for_any_plan() {
        let plans = vec![
            PlanningEngine::decompose("plan a"),
            PlanningEngine::decompose("plan b"),
            Plan::new("empty", vec![]),
        ];
        for plan in plans {
            let total = plan.tasks.len();
            let result = AgentCoordinator::execute(&plan);
            assert_eq!(
                result.total(), total,
                "completed + failed must equal total tasks"
            );
        }
    }
}
