use std::collections::VecDeque;
use std::collections::HashMap;
use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// 8.1 Short-term memory — ring buffer
// ---------------------------------------------------------------------------

pub struct ShortTermMemory {
    buffer: VecDeque<String>,
    capacity: usize,
}

impl ShortTermMemory {
    pub fn new(capacity: usize) -> Self {
        Self {
            buffer: VecDeque::with_capacity(capacity),
            capacity,
        }
    }

    /// Push an item; evicts the oldest entry when the buffer is full.
    pub fn push(&mut self, item: String) {
        if self.capacity == 0 {
            return;
        }
        if self.buffer.len() == self.capacity {
            self.buffer.pop_front();
        }
        self.buffer.push_back(item);
    }

    /// Return the last `n` items (most-recent last).
    pub fn recent(&self, n: usize) -> Vec<&str> {
        let skip = self.buffer.len().saturating_sub(n);
        self.buffer.iter().skip(skip).map(|s| s.as_str()).collect()
    }

    pub fn len(&self) -> usize {
        self.buffer.len()
    }

    pub fn is_empty(&self) -> bool {
        self.buffer.is_empty()
    }
}

// ---------------------------------------------------------------------------
// 8.2 Long-term memory — SQLite-backed (stub)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LongTermEntry {
    pub key: String,
    pub value: String,
    pub confidence: f32,
    pub source: String,
    /// Unix timestamp millis; None = never expires.
    pub expiry: Option<u64>,
}

pub struct LongTermMemory {
    // Stub: in-memory HashMap; real impl would use SQLite via rusqlite.
    store: HashMap<String, LongTermEntry>,
}

impl LongTermMemory {
    pub fn new() -> Self {
        Self {
            store: HashMap::new(),
        }
    }

    pub fn store(&mut self, entry: LongTermEntry) {
        self.store.insert(entry.key.clone(), entry);
    }

    pub fn get(&self, key: &str) -> Option<&LongTermEntry> {
        self.store.get(key)
    }

    pub fn remove(&mut self, key: &str) -> Option<LongTermEntry> {
        self.store.remove(key)
    }

    pub fn len(&self) -> usize {
        self.store.len()
    }

    /// Iterate over all entries.
    pub fn entries(&self) -> impl Iterator<Item = &LongTermEntry> {
        self.store.values()
    }
}

// ---------------------------------------------------------------------------
// 8.3 Episodic memory — Episode with embedding
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct Episode {
    pub id: String,
    pub content: String,
    pub embedding: Vec<f32>,
    pub timestamp: u64,
    pub tags: Vec<String>,
}

pub struct EpisodicMemory {
    episodes: Vec<Episode>,
}

impl EpisodicMemory {
    pub fn new() -> Self {
        Self { episodes: Vec::new() }
    }

    pub fn store_episode(&mut self, episode: Episode) {
        self.episodes.push(episode);
    }

    pub fn get_by_id(&self, id: &str) -> Option<&Episode> {
        self.episodes.iter().find(|e| e.id == id)
    }

    pub fn len(&self) -> usize {
        self.episodes.len()
    }
}

// ---------------------------------------------------------------------------
// 8.4 Vector store (FAISS stub)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct VectorEntry {
    pub id: String,
    pub embedding: Vec<f32>,
    pub payload: String,
}

/// Stub vector store. Real impl would use FAISS via Rust bindings.
pub struct VectorStore {
    entries: Vec<VectorEntry>,
}

fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    if a.len() != b.len() || a.is_empty() {
        return 0.0;
    }
    let dot: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();
    if norm_a == 0.0 || norm_b == 0.0 {
        return 0.0;
    }
    dot / (norm_a * norm_b)
}

impl VectorStore {
    pub fn new() -> Self {
        Self { entries: Vec::new() }
    }

    pub fn upsert(&mut self, entry: VectorEntry) {
        if let Some(existing) = self.entries.iter_mut().find(|e| e.id == entry.id) {
            *existing = entry;
        } else {
            self.entries.push(entry);
        }
    }

    pub fn delete(&mut self, id: &str) -> bool {
        let before = self.entries.len();
        self.entries.retain(|e| e.id != id);
        self.entries.len() < before
    }

    /// Returns up to `top_k` entries ordered by cosine similarity to `query` (descending).
    pub fn search(&self, query: &[f32], top_k: usize) -> Vec<(&VectorEntry, f32)> {
        let mut scored: Vec<(&VectorEntry, f32)> = self
            .entries
            .iter()
            .map(|e| (e, cosine_similarity(query, &e.embedding)))
            .collect();
        scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        scored.truncate(top_k);
        scored
    }
}

// ---------------------------------------------------------------------------
// 8.5 & 8.6 AdvancedMemorySystem
// ---------------------------------------------------------------------------

pub struct SearchResult {
    pub entry: VectorEntry,
    pub similarity: f32,
}

pub struct AdvancedMemorySystem {
    pub short_term: ShortTermMemory,
    pub long_term: LongTermMemory,
    pub episodic: EpisodicMemory,
    pub vector_store: VectorStore,
    pub knowledge_graph: crate::knowledge::KnowledgeGraph,
}

impl AdvancedMemorySystem {
    pub fn new() -> Self {
        Self {
            short_term: ShortTermMemory::new(64),
            long_term: LongTermMemory::new(),
            episodic: EpisodicMemory::new(),
            vector_store: VectorStore::new(),
            knowledge_graph: crate::knowledge::KnowledgeGraph::new(),
        }
    }

    /// Returns top_k results ordered by cosine similarity (non-increasing).
    pub fn vector_search(&self, query: &[f32], top_k: usize) -> Vec<SearchResult> {
        self.vector_store
            .search(query, top_k)
            .into_iter()
            .map(|(entry, similarity)| SearchResult {
                entry: entry.clone(),
                similarity,
            })
            .collect()
    }

    /// Store an episode in episodic memory AND upsert a node in the knowledge graph.
    pub fn store_episode_with_graph(&mut self, episode: Episode) {
        // Upsert a knowledge graph node for this episode.
        let node = crate::knowledge::Node::new(
            episode.id.clone(),
            episode.content.clone(),
        );
        self.knowledge_graph.add_node(node);

        // Also upsert into vector store.
        let ve = VectorEntry {
            id: episode.id.clone(),
            embedding: episode.embedding.clone(),
            payload: episode.content.clone(),
        };
        self.vector_store.upsert(ve);

        self.episodic.store_episode(episode);
    }

    /// Background consolidation stub: moves short-term items to long-term.
    /// Real impl would run in a background thread.
    pub fn consolidate_memory(&mut self) {
        let items: Vec<String> = self.short_term.buffer.drain(..).collect();
        for item in items {
            let entry = LongTermEntry {
                key: item.clone(),
                value: item,
                confidence: 0.5,
                source: "consolidation".to_string(),
                expiry: None,
            };
            self.long_term.store(entry);
        }
    }

    // ── Semantic memory tier (Task 32) ────────────────────────────────────────

    /// Store a semantic fact in long-term memory with high confidence.
    pub fn store_semantic(&mut self, key: impl Into<String>, value: impl Into<String>) {
        let k = key.into();
        let v = value.into();
        // Also upsert into vector store for semantic search.
        let embedding: Vec<f32> = k
            .bytes()
            .take(8)
            .map(|b| b as f32 / 255.0)
            .collect();
        let ve = VectorEntry {
            id: format!("sem:{k}"),
            embedding,
            payload: v.clone(),
        };
        self.vector_store.upsert(ve);
        self.long_term.store(LongTermEntry {
            key: k,
            value: v,
            confidence: 0.95,
            source: "semantic".to_string(),
            expiry: None,
        });
    }

    /// Query semantic memory by key prefix.
    pub fn query_semantic(&self, prefix: &str) -> Vec<&LongTermEntry> {
        self.long_term
            .entries()
            .filter(|e| e.source == "semantic" && e.key.starts_with(prefix))
            .collect()
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn short_term_ring_buffer_evicts_oldest() {
        let capacity = 3;
        let mut stm = ShortTermMemory::new(capacity);
        stm.push("a".to_string());
        stm.push("b".to_string());
        stm.push("c".to_string());
        // Push one more — "a" should be evicted
        stm.push("d".to_string());
        assert_eq!(stm.len(), capacity);
        let recent = stm.recent(capacity);
        assert!(!recent.contains(&"a"), "oldest item should have been evicted");
        assert!(recent.contains(&"b"));
        assert!(recent.contains(&"c"));
        assert!(recent.contains(&"d"));
    }

    #[test]
    fn vector_search_returns_at_most_top_k() {
        let mut vs = VectorStore::new();
        for i in 0..5u32 {
            vs.upsert(VectorEntry {
                id: format!("e{}", i),
                embedding: vec![i as f32, 0.0],
                payload: format!("payload{}", i),
            });
        }
        let query = vec![1.0, 0.0];
        let results = vs.search(&query, 3);
        assert!(results.len() <= 3);
    }

    #[test]
    fn vector_search_ordered_by_similarity() {
        let mut vs = VectorStore::new();
        vs.upsert(VectorEntry { id: "a".into(), embedding: vec![1.0, 0.0], payload: "a".into() });
        vs.upsert(VectorEntry { id: "b".into(), embedding: vec![0.0, 1.0], payload: "b".into() });
        vs.upsert(VectorEntry { id: "c".into(), embedding: vec![1.0, 1.0], payload: "c".into() });
        let query = vec![1.0, 0.0];
        let results = vs.search(&query, 3);
        for window in results.windows(2) {
            assert!(
                window[0].1 >= window[1].1,
                "results must be in non-increasing similarity order"
            );
        }
    }

    #[test]
    fn episode_retrievable_by_id() {
        let mut em = EpisodicMemory::new();
        let ep = Episode {
            id: "ep1".to_string(),
            content: "test content".to_string(),
            embedding: vec![0.1, 0.2],
            timestamp: 1000,
            tags: vec!["tag1".to_string()],
        };
        em.store_episode(ep);
        let found = em.get_by_id("ep1");
        assert!(found.is_some());
        assert_eq!(found.unwrap().content, "test content");
    }

    #[test]
    fn cosine_similarity_zero_vector_returns_zero() {
        let mut vs = VectorStore::new();
        vs.upsert(VectorEntry { id: "x".into(), embedding: vec![1.0, 2.0], payload: "x".into() });
        let zero_query = vec![0.0, 0.0];
        let results = vs.search(&zero_query, 5);
        for (_, sim) in &results {
            assert_eq!(*sim, 0.0, "zero query vector should yield similarity 0.0");
        }
    }

    // Property 11: vectorSearch returns at most topK results in non-increasing similarity order.
    #[test]
    fn vector_search_returns_at_most_top_k_in_non_increasing_order() {
        let mut sys = AdvancedMemorySystem::new();
        for i in 0..10u32 {
            sys.vector_store.upsert(VectorEntry {
                id: format!("e{i}"),
                embedding: vec![i as f32, 0.0],
                payload: format!("p{i}"),
            });
        }
        let query = vec![1.0, 0.0];
        for top_k in [1, 3, 5, 10, 20] {
            let results = sys.vector_search(&query, top_k);
            assert!(results.len() <= top_k, "got {} results for top_k={top_k}", results.len());
            for window in results.windows(2) {
                assert!(
                    window[0].similarity >= window[1].similarity,
                    "results not in non-increasing order"
                );
            }
        }
    }

    // Property 10: stored episode is retrievable via vector search with its own embedding.
    #[test]
    fn stored_episode_retrievable_via_vector_search() {
        let mut sys = AdvancedMemorySystem::new();
        let embedding = vec![1.0, 0.0, 0.0, 0.0];
        let ep = Episode {
            id: "ep-test".to_string(),
            content: "test content".to_string(),
            embedding: embedding.clone(),
            timestamp: 0,
            tags: vec![],
        };
        sys.store_episode_with_graph(ep);
        let results = sys.vector_search(&embedding, 1);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].entry.id, "ep-test");
        // Cosine similarity of identical vectors = 1.0
        assert!((results[0].similarity - 1.0).abs() < 1e-5);
    }
}
