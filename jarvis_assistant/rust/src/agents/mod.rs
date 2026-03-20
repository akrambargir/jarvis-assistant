/// Multi-Agent System — Phase 3
///
/// AgentBus: message bus with publish/subscribe/request/broadcast.
/// BaseAgent: trait for all specialized agents.
/// Specialized agents: Planner, Executor, Web, Creative, Memory.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

// ── AgentMessage ──────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct AgentMessage {
    pub id: String,
    pub from: String,
    pub to: Option<String>, // None = broadcast
    pub topic: String,
    pub payload: String,
}

impl AgentMessage {
    pub fn new(
        from: impl Into<String>,
        to: impl Into<String>,
        topic: impl Into<String>,
        payload: impl Into<String>,
    ) -> Self {
        Self {
            id: uuid_stub(),
            from: from.into(),
            to: Some(to.into()),
            topic: topic.into(),
            payload: payload.into(),
        }
    }

    pub fn broadcast(
        from: impl Into<String>,
        topic: impl Into<String>,
        payload: impl Into<String>,
    ) -> Self {
        Self {
            id: uuid_stub(),
            from: from.into(),
            to: None,
            topic: topic.into(),
            payload: payload.into(),
        }
    }
}

fn uuid_stub() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let t = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    format!("msg-{t}")
}

// ── AgentStatus ───────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AgentStatus {
    Idle,
    Busy,
    Error(String),
    Offline,
}

// ── AgentBus ──────────────────────────────────────────────────────────────────

type MessageHandler = Box<dyn Fn(&AgentMessage) + Send + Sync>;

pub struct AgentBus {
    /// topic → list of subscriber agent_ids
    subscriptions: HashMap<String, Vec<String>>,
    /// agent_id → inbox
    inboxes: HashMap<String, Vec<AgentMessage>>,
    /// agent_id → status
    statuses: HashMap<String, AgentStatus>,
}

impl AgentBus {
    pub fn new() -> Self {
        Self {
            subscriptions: HashMap::new(),
            inboxes: HashMap::new(),
            statuses: HashMap::new(),
        }
    }

    /// Register an agent on the bus.
    pub fn register(&mut self, agent_id: &str) {
        self.inboxes.entry(agent_id.to_string()).or_default();
        self.statuses.insert(agent_id.to_string(), AgentStatus::Idle);
    }

    /// Subscribe an agent to a topic.
    pub fn subscribe(&mut self, agent_id: &str, topic: &str) {
        self.subscriptions
            .entry(topic.to_string())
            .or_default()
            .push(agent_id.to_string());
    }

    /// Publish a message to a specific agent.
    pub fn publish(&mut self, message: AgentMessage) {
        if let Some(to) = &message.to.clone() {
            self.inboxes
                .entry(to.clone())
                .or_default()
                .push(message);
        }
    }

    /// Broadcast a message to all subscribers of the message's topic.
    pub fn broadcast(&mut self, message: AgentMessage) {
        let subscribers = self
            .subscriptions
            .get(&message.topic)
            .cloned()
            .unwrap_or_default();
        for sub in subscribers {
            let mut msg = message.clone();
            msg.to = Some(sub.clone());
            self.inboxes.entry(sub).or_default().push(msg);
        }
    }

    /// Request-reply: publish a message and return the first reply in the sender's inbox.
    /// Stub: returns None (async reply not simulated).
    pub fn request(&mut self, message: AgentMessage) -> Option<AgentMessage> {
        self.publish(message);
        None
    }

    /// Drain all messages from an agent's inbox.
    pub fn drain_inbox(&mut self, agent_id: &str) -> Vec<AgentMessage> {
        self.inboxes
            .entry(agent_id.to_string())
            .or_default()
            .drain(..)
            .collect()
    }

    /// Get the status of an agent.
    pub fn get_agent_status(&self, agent_id: &str) -> Option<&AgentStatus> {
        self.statuses.get(agent_id)
    }

    /// Update the status of an agent.
    pub fn set_agent_status(&mut self, agent_id: &str, status: AgentStatus) {
        self.statuses.insert(agent_id.to_string(), status);
    }
}

impl Default for AgentBus {
    fn default() -> Self {
        Self::new()
    }
}

// ── BaseAgent trait ───────────────────────────────────────────────────────────

pub trait BaseAgent: Send + Sync {
    fn agent_id(&self) -> &str;
    fn capabilities(&self) -> &[String];
    fn handle_message(&mut self, message: &AgentMessage) -> Option<AgentMessage>;
    fn get_status(&self) -> AgentStatus;
    fn initialize(&mut self);
}

// ── PlannerAgent ──────────────────────────────────────────────────────────────

pub struct PlannerAgent {
    id: String,
    caps: Vec<String>,
    status: AgentStatus,
}

impl PlannerAgent {
    pub fn new() -> Self {
        Self {
            id: "planner_agent".to_string(),
            caps: vec!["planning".to_string(), "coordination".to_string()],
            status: AgentStatus::Idle,
        }
    }

    pub fn create_plan(&self, goal: &str) -> String {
        format!("[PlannerAgent] plan for: {goal}")
    }

    pub fn coordinate_agents(&self, agents: &[&str]) -> String {
        format!("[PlannerAgent] coordinating: {}", agents.join(", "))
    }

    pub fn synthesize_results(&self, results: &[String]) -> String {
        format!("[PlannerAgent] synthesized {} results", results.len())
    }
}

impl Default for PlannerAgent {
    fn default() -> Self {
        Self::new()
    }
}

impl BaseAgent for PlannerAgent {
    fn agent_id(&self) -> &str { &self.id }
    fn capabilities(&self) -> &[String] { &self.caps }
    fn get_status(&self) -> AgentStatus { self.status.clone() }
    fn initialize(&mut self) { self.status = AgentStatus::Idle; }
    fn handle_message(&mut self, msg: &AgentMessage) -> Option<AgentMessage> {
        let reply = self.create_plan(&msg.payload);
        Some(AgentMessage::new(&self.id, &msg.from, "plan_result", reply))
    }
}

// ── ExecutorAgent ─────────────────────────────────────────────────────────────

pub struct ExecutorAgent {
    id: String,
    caps: Vec<String>,
    status: AgentStatus,
}

impl ExecutorAgent {
    pub fn new() -> Self {
        Self {
            id: "executor_agent".to_string(),
            caps: vec!["execution".to_string(), "os_control".to_string()],
            status: AgentStatus::Idle,
        }
    }

    pub fn execute_action(&self, action: &str) -> String {
        format!("[ExecutorAgent] executed: {action}")
    }

    pub fn execute_workflow(&self, workflow_id: &str) -> String {
        format!("[ExecutorAgent] workflow: {workflow_id}")
    }

    pub fn report_progress(&self, task_id: &str, pct: u8) -> String {
        format!("[ExecutorAgent] task {task_id}: {pct}%")
    }
}

impl Default for ExecutorAgent {
    fn default() -> Self {
        Self::new()
    }
}

impl BaseAgent for ExecutorAgent {
    fn agent_id(&self) -> &str { &self.id }
    fn capabilities(&self) -> &[String] { &self.caps }
    fn get_status(&self) -> AgentStatus { self.status.clone() }
    fn initialize(&mut self) { self.status = AgentStatus::Idle; }
    fn handle_message(&mut self, msg: &AgentMessage) -> Option<AgentMessage> {
        let result = self.execute_action(&msg.payload);
        Some(AgentMessage::new(&self.id, &msg.from, "execution_result", result))
    }
}

// ── WebAgent ──────────────────────────────────────────────────────────────────

pub struct WebAgent {
    id: String,
    caps: Vec<String>,
    status: AgentStatus,
}

impl WebAgent {
    pub fn new() -> Self {
        Self {
            id: "web_agent".to_string(),
            caps: vec!["search".to_string(), "scrape".to_string(), "fetch_api".to_string()],
            status: AgentStatus::Idle,
        }
    }

    pub fn search(&self, query: &str) -> String {
        format!("[WebAgent] search results for: {query}")
    }

    pub fn scrape(&self, url: &str) -> String {
        format!("[WebAgent] scraped: {url}")
    }

    pub fn monitor_feed(&self, url: &str) -> String {
        format!("[WebAgent] monitoring feed: {url}")
    }

    pub fn fetch_api(&self, endpoint: &str) -> String {
        format!("[WebAgent] fetched: {endpoint}")
    }
}

impl Default for WebAgent {
    fn default() -> Self {
        Self::new()
    }
}

impl BaseAgent for WebAgent {
    fn agent_id(&self) -> &str { &self.id }
    fn capabilities(&self) -> &[String] { &self.caps }
    fn get_status(&self) -> AgentStatus { self.status.clone() }
    fn initialize(&mut self) { self.status = AgentStatus::Idle; }
    fn handle_message(&mut self, msg: &AgentMessage) -> Option<AgentMessage> {
        let result = self.search(&msg.payload);
        Some(AgentMessage::new(&self.id, &msg.from, "search_result", result))
    }
}

// ── CreativeAgent ─────────────────────────────────────────────────────────────

pub struct CreativeAgent {
    id: String,
    caps: Vec<String>,
    status: AgentStatus,
}

impl CreativeAgent {
    pub fn new() -> Self {
        Self {
            id: "creative_agent".to_string(),
            caps: vec!["generate".to_string(), "edit".to_string(), "summarize".to_string()],
            status: AgentStatus::Idle,
        }
    }

    pub fn generate(&self, prompt: &str) -> String {
        format!("[CreativeAgent] generated content for: {prompt}")
    }

    pub fn edit(&self, content: &str) -> String {
        format!("[CreativeAgent] edited: {content}")
    }

    pub fn design(&self, spec: &str) -> String {
        format!("[CreativeAgent] designed: {spec}")
    }

    pub fn summarize(&self, text: &str) -> String {
        format!("[CreativeAgent] summary of {} chars", text.len())
    }
}

impl Default for CreativeAgent {
    fn default() -> Self {
        Self::new()
    }
}

impl BaseAgent for CreativeAgent {
    fn agent_id(&self) -> &str { &self.id }
    fn capabilities(&self) -> &[String] { &self.caps }
    fn get_status(&self) -> AgentStatus { self.status.clone() }
    fn initialize(&mut self) { self.status = AgentStatus::Idle; }
    fn handle_message(&mut self, msg: &AgentMessage) -> Option<AgentMessage> {
        let result = self.generate(&msg.payload);
        Some(AgentMessage::new(&self.id, &msg.from, "creative_result", result))
    }
}

// ── MemoryAgent ───────────────────────────────────────────────────────────────

pub struct MemoryAgent {
    id: String,
    caps: Vec<String>,
    status: AgentStatus,
    store: Vec<(String, String)>, // (key, value)
}

impl MemoryAgent {
    pub fn new() -> Self {
        Self {
            id: "memory_agent".to_string(),
            caps: vec!["store".to_string(), "retrieve".to_string(), "consolidate".to_string()],
            status: AgentStatus::Idle,
            store: vec![],
        }
    }

    pub fn store(&mut self, key: &str, value: &str) {
        self.store.push((key.to_string(), value.to_string()));
    }

    pub fn retrieve(&self, key: &str) -> Option<&str> {
        self.store
            .iter()
            .rev()
            .find(|(k, _)| k == key)
            .map(|(_, v)| v.as_str())
    }

    pub fn consolidate(&mut self) -> usize {
        // Stub: deduplicate by key, keep last value.
        let mut seen: HashMap<String, String> = HashMap::new();
        for (k, v) in self.store.drain(..) {
            seen.insert(k, v);
        }
        let count = seen.len();
        self.store = seen.into_iter().collect();
        count
    }

    pub fn build_context(&self, keys: &[&str]) -> String {
        keys.iter()
            .filter_map(|k| self.retrieve(k))
            .collect::<Vec<_>>()
            .join("; ")
    }
}

impl Default for MemoryAgent {
    fn default() -> Self {
        Self::new()
    }
}

impl BaseAgent for MemoryAgent {
    fn agent_id(&self) -> &str { &self.id }
    fn capabilities(&self) -> &[String] { &self.caps }
    fn get_status(&self) -> AgentStatus { self.status.clone() }
    fn initialize(&mut self) { self.status = AgentStatus::Idle; }
    fn handle_message(&mut self, msg: &AgentMessage) -> Option<AgentMessage> {
        // Treat payload as a retrieval key.
        let result = self.retrieve(&msg.payload)
            .unwrap_or("not found")
            .to_string();
        Some(AgentMessage::new(&self.id, &msg.from, "memory_result", result))
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn agent_bus_publish_delivers_to_inbox() {
        let mut bus = AgentBus::new();
        bus.register("agent_a");
        bus.register("agent_b");

        let msg = AgentMessage::new("agent_a", "agent_b", "task", "hello");
        bus.publish(msg);

        let inbox = bus.drain_inbox("agent_b");
        assert_eq!(inbox.len(), 1);
        assert_eq!(inbox[0].payload, "hello");
    }

    #[test]
    fn agent_bus_broadcast_delivers_to_subscribers() {
        let mut bus = AgentBus::new();
        bus.register("agent_a");
        bus.register("agent_b");
        bus.register("agent_c");
        bus.subscribe("agent_b", "news");
        bus.subscribe("agent_c", "news");

        let msg = AgentMessage::broadcast("agent_a", "news", "breaking");
        bus.broadcast(msg);

        assert_eq!(bus.drain_inbox("agent_b").len(), 1);
        assert_eq!(bus.drain_inbox("agent_c").len(), 1);
        assert_eq!(bus.drain_inbox("agent_a").len(), 0);
    }

    #[test]
    fn agent_bus_status_tracking() {
        let mut bus = AgentBus::new();
        bus.register("agent_x");
        assert_eq!(bus.get_agent_status("agent_x"), Some(&AgentStatus::Idle));
        bus.set_agent_status("agent_x", AgentStatus::Busy);
        assert_eq!(bus.get_agent_status("agent_x"), Some(&AgentStatus::Busy));
    }

    #[test]
    fn planner_agent_handles_message() {
        let mut agent = PlannerAgent::new();
        let msg = AgentMessage::new("user", "planner_agent", "plan", "write a report");
        let reply = agent.handle_message(&msg);
        assert!(reply.is_some());
        assert!(!reply.unwrap().payload.is_empty());
    }

    #[test]
    fn memory_agent_store_and_retrieve() {
        let mut agent = MemoryAgent::new();
        agent.store("key1", "value1");
        assert_eq!(agent.retrieve("key1"), Some("value1"));
        assert_eq!(agent.retrieve("missing"), None);
    }

    #[test]
    fn memory_agent_consolidate_deduplicates() {
        let mut agent = MemoryAgent::new();
        agent.store("k", "v1");
        agent.store("k", "v2");
        let count = agent.consolidate();
        assert_eq!(count, 1);
    }

    #[test]
    fn web_agent_capabilities_include_search() {
        let agent = WebAgent::new();
        assert!(agent.capabilities().contains(&"search".to_string()));
    }
}
