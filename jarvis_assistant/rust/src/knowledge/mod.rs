/// Knowledge Graph
///
/// Lightweight in-process graph store implemented in Rust.
/// Nodes and edges are stored in HashMaps for O(1) lookup.
/// addEdge validates both source and target nodes exist before insertion.

use std::collections::{HashMap, HashSet};

// ── Node / Edge ───────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct Node {
    pub id: String,
    pub label: String,
    pub properties: HashMap<String, String>,
}

impl Node {
    pub fn new(id: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            properties: HashMap::new(),
        }
    }

    pub fn with_property(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.properties.insert(key.into(), value.into());
        self
    }
}

#[derive(Debug, Clone)]
pub struct Edge {
    pub id: String,
    pub source: String,
    pub target: String,
    pub relation: String,
    pub weight: f32,
}

impl Edge {
    pub fn new(
        source: impl Into<String>,
        target: impl Into<String>,
        relation: impl Into<String>,
    ) -> Self {
        let source = source.into();
        let target = target.into();
        let id = format!("{source}->{target}");
        Self { id, source, target, relation: relation.into(), weight: 1.0 }
    }
}

// ── QueryResult ───────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct QueryResult {
    pub nodes: Vec<Node>,
    pub edges: Vec<Edge>,
}

// ── KnowledgeGraph ────────────────────────────────────────────────────────────

pub struct KnowledgeGraph {
    nodes: HashMap<String, Node>,
    /// Adjacency list: source_id → list of edge ids.
    adjacency: HashMap<String, Vec<String>>,
    edges: HashMap<String, Edge>,
}

impl KnowledgeGraph {
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            adjacency: HashMap::new(),
            edges: HashMap::new(),
        }
    }

    /// Add a node. If a node with the same id already exists it is replaced.
    pub fn add_node(&mut self, node: Node) {
        self.adjacency.entry(node.id.clone()).or_default();
        self.nodes.insert(node.id.clone(), node);
    }

    /// Add a directed edge from `source` to `target`.
    ///
    /// Returns `Err` if either node does not exist (no orphaned edges).
    pub fn add_edge(&mut self, edge: Edge) -> Result<(), String> {
        if !self.nodes.contains_key(&edge.source) {
            return Err(format!("Source node '{}' does not exist", edge.source));
        }
        if !self.nodes.contains_key(&edge.target) {
            return Err(format!("Target node '{}' does not exist", edge.target));
        }
        self.adjacency
            .entry(edge.source.clone())
            .or_default()
            .push(edge.id.clone());
        self.edges.insert(edge.id.clone(), edge);
        Ok(())
    }

    /// Query nodes whose label contains `pattern` (case-insensitive).
    pub fn query(&self, pattern: &str) -> QueryResult {
        let pattern_lower = pattern.to_lowercase();
        let nodes: Vec<Node> = self
            .nodes
            .values()
            .filter(|n| n.label.to_lowercase().contains(&pattern_lower))
            .cloned()
            .collect();

        let node_ids: HashSet<&str> = nodes.iter().map(|n| n.id.as_str()).collect();
        let edges: Vec<Edge> = self
            .edges
            .values()
            .filter(|e| node_ids.contains(e.source.as_str()) || node_ids.contains(e.target.as_str()))
            .cloned()
            .collect();

        QueryResult { nodes, edges }
    }

    /// Return all direct neighbours of `node_id` (nodes reachable via one outgoing edge).
    pub fn get_neighbors(&self, node_id: &str) -> Vec<&Node> {
        let edge_ids = match self.adjacency.get(node_id) {
            Some(ids) => ids,
            None => return vec![],
        };
        edge_ids
            .iter()
            .filter_map(|eid| self.edges.get(eid))
            .filter_map(|e| self.nodes.get(&e.target))
            .collect()
    }

    /// Return the number of nodes in the graph.
    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    /// Return the number of edges in the graph.
    pub fn edge_count(&self) -> usize {
        self.edges.len()
    }

    /// Check whether a node exists.
    pub fn has_node(&self, id: &str) -> bool {
        self.nodes.contains_key(id)
    }
}

impl Default for KnowledgeGraph {
    fn default() -> Self {
        Self::new()
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn make_graph() -> KnowledgeGraph {
        let mut g = KnowledgeGraph::new();
        g.add_node(Node::new("person:alice", "Alice"));
        g.add_node(Node::new("person:bob", "Bob"));
        g.add_node(Node::new("topic:rust", "Rust programming"));
        g
    }

    #[test]
    fn add_node_increases_count() {
        let mut g = KnowledgeGraph::new();
        g.add_node(Node::new("n1", "Node 1"));
        assert_eq!(g.node_count(), 1);
    }

    #[test]
    fn add_edge_valid_nodes_succeeds() {
        let mut g = make_graph();
        let result = g.add_edge(Edge::new("person:alice", "topic:rust", "knows"));
        assert!(result.is_ok());
        assert_eq!(g.edge_count(), 1);
    }

    #[test]
    fn add_edge_missing_source_fails() {
        let mut g = make_graph();
        let result = g.add_edge(Edge::new("ghost", "person:bob", "knows"));
        assert!(result.is_err());
        assert_eq!(g.edge_count(), 0);
    }

    #[test]
    fn add_edge_missing_target_fails() {
        let mut g = make_graph();
        let result = g.add_edge(Edge::new("person:alice", "ghost", "knows"));
        assert!(result.is_err());
        assert_eq!(g.edge_count(), 0);
    }

    #[test]
    fn no_orphaned_edges_after_failed_add() {
        let mut g = make_graph();
        let _ = g.add_edge(Edge::new("person:alice", "nonexistent", "rel"));
        // Verify no edge was stored.
        assert_eq!(g.edge_count(), 0);
    }

    #[test]
    fn query_returns_matching_nodes() {
        let g = make_graph();
        let result = g.query("rust");
        assert_eq!(result.nodes.len(), 1);
        assert_eq!(result.nodes[0].id, "topic:rust");
    }

    #[test]
    fn query_case_insensitive() {
        let g = make_graph();
        let result = g.query("ALICE");
        assert_eq!(result.nodes.len(), 1);
    }

    #[test]
    fn get_neighbors_returns_connected_nodes() {
        let mut g = make_graph();
        g.add_edge(Edge::new("person:alice", "topic:rust", "knows")).unwrap();
        g.add_edge(Edge::new("person:alice", "person:bob", "friends_with")).unwrap();
        let neighbors = g.get_neighbors("person:alice");
        assert_eq!(neighbors.len(), 2);
    }

    #[test]
    fn get_neighbors_unknown_node_returns_empty() {
        let g = make_graph();
        let neighbors = g.get_neighbors("nobody");
        assert!(neighbors.is_empty());
    }

    // Property: addEdge never creates orphaned edges (both nodes must exist first).
    #[test]
    fn add_edge_never_creates_orphaned_edges() {
        let mut g = KnowledgeGraph::new();
        g.add_node(Node::new("a", "A"));

        // Missing target → must fail, no edge stored.
        let r1 = g.add_edge(Edge::new("a", "missing", "rel"));
        assert!(r1.is_err());
        assert_eq!(g.edge_count(), 0, "no orphaned edge must be created");

        // Missing source → must fail, no edge stored.
        let r2 = g.add_edge(Edge::new("missing", "a", "rel"));
        assert!(r2.is_err());
        assert_eq!(g.edge_count(), 0, "no orphaned edge must be created");

        // Both present → must succeed.
        g.add_node(Node::new("b", "B"));
        let r3 = g.add_edge(Edge::new("a", "b", "rel"));
        assert!(r3.is_ok());
        assert_eq!(g.edge_count(), 1);
    }

    // Property: every edge references existing nodes (invariant check after multiple ops).
    #[test]
    fn all_edges_reference_existing_nodes() {
        let mut g = make_graph();
        g.add_edge(Edge::new("person:alice", "topic:rust", "knows")).unwrap();
        g.add_edge(Edge::new("person:bob", "topic:rust", "knows")).unwrap();
        // Verify via neighbors: all neighbors of alice and bob must be existing nodes.
        for src in ["person:alice", "person:bob"] {
            for neighbor in g.get_neighbors(src) {
                assert!(g.has_node(&neighbor.id), "neighbor '{}' must exist", neighbor.id);
            }
        }
        assert_eq!(g.edge_count(), 2);
    }
}
