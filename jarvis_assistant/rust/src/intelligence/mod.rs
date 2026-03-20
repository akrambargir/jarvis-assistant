/// Real-Time Intelligence Layer — Phase 3
///
/// RealTimeIntelligenceLayer: search, weather, finance, news, maps.
/// APIOrchestrator: register/call/batch/rate-limit external APIs.
/// EventDrivenAlertSystem: create/evaluate/dismiss alerts.
/// Response caching for all data sources.

use std::collections::HashMap;
use std::time::{Duration, Instant};

// ── CachedResponse ────────────────────────────────────────────────────────────

struct CachedResponse {
    data: String,
    cached_at: Instant,
    ttl: Duration,
}

impl CachedResponse {
    fn is_valid(&self) -> bool {
        self.cached_at.elapsed() < self.ttl
    }
}

// ── APIOrchestrator ───────────────────────────────────────────────────────────

pub struct ApiConfig {
    pub name: String,
    pub base_url: String,
    pub rate_limit_per_minute: u32,
}

pub struct APIOrchestrator {
    apis: HashMap<String, ApiConfig>,
    call_counts: HashMap<String, u32>,
    cache: HashMap<String, CachedResponse>,
}

impl APIOrchestrator {
    pub fn new() -> Self {
        Self {
            apis: HashMap::new(),
            call_counts: HashMap::new(),
            cache: HashMap::new(),
        }
    }

    /// Register an API endpoint.
    pub fn register_api(&mut self, config: ApiConfig) {
        self.apis.insert(config.name.clone(), config);
    }

    /// Call a registered API. Returns cached response if available.
    pub fn call(&mut self, api_name: &str, path: &str) -> Result<String, String> {
        let cache_key = format!("{api_name}:{path}");

        // Return cached response if valid.
        if let Some(cached) = self.cache.get(&cache_key) {
            if cached.is_valid() {
                return Ok(cached.data.clone());
            }
        }

        // Rate limit check.
        let count = self.call_counts.entry(api_name.to_string()).or_insert(0);
        let limit = self
            .apis
            .get(api_name)
            .map(|a| a.rate_limit_per_minute)
            .unwrap_or(60);
        if *count >= limit {
            return Err(format!("Rate limit exceeded for API: {api_name}"));
        }
        *count += 1;

        // Stub response.
        let response = format!("[{api_name}] response for: {path}");

        // Cache with 5-minute TTL.
        self.cache.insert(
            cache_key,
            CachedResponse {
                data: response.clone(),
                cached_at: Instant::now(),
                ttl: Duration::from_secs(300),
            },
        );

        Ok(response)
    }

    /// Call multiple APIs in batch.
    pub fn call_batch(&mut self, requests: &[(&str, &str)]) -> Vec<Result<String, String>> {
        requests
            .iter()
            .map(|(api, path)| self.call(api, path))
            .collect()
    }

    /// Reset rate limit counters (called at the start of each minute).
    pub fn reset_rate_limits(&mut self) {
        self.call_counts.clear();
    }
}

impl Default for APIOrchestrator {
    fn default() -> Self {
        Self::new()
    }
}

// ── RealTimeIntelligenceLayer ─────────────────────────────────────────────────

pub struct RealTimeIntelligenceLayer {
    orchestrator: APIOrchestrator,
}

impl RealTimeIntelligenceLayer {
    pub fn new() -> Self {
        let mut orchestrator = APIOrchestrator::new();
        orchestrator.register_api(ApiConfig {
            name: "search".to_string(),
            base_url: "https://api.search.local".to_string(),
            rate_limit_per_minute: 30,
        });
        orchestrator.register_api(ApiConfig {
            name: "weather".to_string(),
            base_url: "https://api.weather.local".to_string(),
            rate_limit_per_minute: 10,
        });
        orchestrator.register_api(ApiConfig {
            name: "finance".to_string(),
            base_url: "https://api.finance.local".to_string(),
            rate_limit_per_minute: 20,
        });
        orchestrator.register_api(ApiConfig {
            name: "news".to_string(),
            base_url: "https://api.news.local".to_string(),
            rate_limit_per_minute: 15,
        });
        orchestrator.register_api(ApiConfig {
            name: "maps".to_string(),
            base_url: "https://api.maps.local".to_string(),
            rate_limit_per_minute: 20,
        });
        Self { orchestrator }
    }

    pub fn search(&mut self, query: &str) -> Result<String, String> {
        self.orchestrator.call("search", query)
    }

    pub fn fetch_weather(&mut self, location: &str) -> Result<String, String> {
        self.orchestrator.call("weather", location)
    }

    pub fn fetch_finance(&mut self, symbol: &str) -> Result<String, String> {
        self.orchestrator.call("finance", symbol)
    }

    pub fn fetch_news(&mut self, topic: &str) -> Result<String, String> {
        self.orchestrator.call("news", topic)
    }

    pub fn fetch_maps(&mut self, query: &str) -> Result<String, String> {
        self.orchestrator.call("maps", query)
    }
}

impl Default for RealTimeIntelligenceLayer {
    fn default() -> Self {
        Self::new()
    }
}

// ── Alert ─────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AlertStatus {
    Active,
    Triggered,
    Dismissed,
}

#[derive(Debug, Clone)]
pub struct Alert {
    pub id: String,
    pub condition: String,
    pub message: String,
    pub status: AlertStatus,
}

// ── EventDrivenAlertSystem ────────────────────────────────────────────────────

pub struct EventDrivenAlertSystem {
    alerts: Vec<Alert>,
    next_id: u64,
}

impl EventDrivenAlertSystem {
    pub fn new() -> Self {
        Self { alerts: vec![], next_id: 1 }
    }

    /// Create a new alert with a condition expression.
    pub fn create_alert(&mut self, condition: impl Into<String>, message: impl Into<String>) -> String {
        let id = format!("alert-{}", self.next_id);
        self.next_id += 1;
        self.alerts.push(Alert {
            id: id.clone(),
            condition: condition.into(),
            message: message.into(),
            status: AlertStatus::Active,
        });
        id
    }

    /// Evaluate all active alerts against a context map.
    /// Stub: triggers any alert whose condition key exists in context with value "true".
    pub fn evaluate_conditions(&mut self, context: &HashMap<String, String>) -> Vec<String> {
        let mut triggered: Vec<String> = vec![];
        for alert in &mut self.alerts {
            if alert.status != AlertStatus::Active {
                continue;
            }
            if context.get(&alert.condition).map(|v| v == "true").unwrap_or(false) {
                alert.status = AlertStatus::Triggered;
                triggered.push(alert.id.clone());
            }
        }
        triggered
    }

    /// Dismiss an alert by id.
    pub fn dismiss_alert(&mut self, id: &str) -> bool {
        if let Some(alert) = self.alerts.iter_mut().find(|a| a.id == id) {
            alert.status = AlertStatus::Dismissed;
            true
        } else {
            false
        }
    }

    /// Return all active alerts.
    pub fn active_alerts(&self) -> Vec<&Alert> {
        self.alerts
            .iter()
            .filter(|a| a.status == AlertStatus::Active)
            .collect()
    }
}

impl Default for EventDrivenAlertSystem {
    fn default() -> Self {
        Self::new()
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn api_orchestrator_call_returns_response() {
        let mut orch = APIOrchestrator::new();
        orch.register_api(ApiConfig {
            name: "test".to_string(),
            base_url: "http://test.local".to_string(),
            rate_limit_per_minute: 5,
        });
        let result = orch.call("test", "/endpoint");
        assert!(result.is_ok());
    }

    #[test]
    fn api_orchestrator_caches_response() {
        let mut orch = APIOrchestrator::new();
        orch.register_api(ApiConfig {
            name: "cached".to_string(),
            base_url: "http://cached.local".to_string(),
            rate_limit_per_minute: 1,
        });
        let r1 = orch.call("cached", "/data").unwrap();
        // Second call should hit cache (rate limit = 1, but cache returns before counting).
        let r2 = orch.call("cached", "/data").unwrap();
        assert_eq!(r1, r2);
    }

    #[test]
    fn api_orchestrator_rate_limit_enforced() {
        let mut orch = APIOrchestrator::new();
        orch.register_api(ApiConfig {
            name: "limited".to_string(),
            base_url: "http://limited.local".to_string(),
            rate_limit_per_minute: 1,
        });
        // First call succeeds.
        assert!(orch.call("limited", "/a").is_ok());
        // Second call to different path (no cache) should fail.
        let result = orch.call("limited", "/b");
        assert!(result.is_err());
    }

    #[test]
    fn intelligence_layer_search_returns_ok() {
        let mut layer = RealTimeIntelligenceLayer::new();
        assert!(layer.search("rust programming").is_ok());
    }

    #[test]
    fn alert_system_create_and_trigger() {
        let mut sys = EventDrivenAlertSystem::new();
        let id = sys.create_alert("rain", "It's raining");
        let mut ctx = HashMap::new();
        ctx.insert("rain".to_string(), "true".to_string());
        let triggered = sys.evaluate_conditions(&ctx);
        assert!(triggered.contains(&id));
    }

    #[test]
    fn alert_system_dismiss() {
        let mut sys = EventDrivenAlertSystem::new();
        let id = sys.create_alert("wind", "Windy");
        assert_eq!(sys.active_alerts().len(), 1);
        sys.dismiss_alert(&id);
        assert_eq!(sys.active_alerts().len(), 0);
    }

    #[test]
    fn batch_call_returns_results_for_each_request() {
        let mut orch = APIOrchestrator::new();
        orch.register_api(ApiConfig {
            name: "batch".to_string(),
            base_url: "http://batch.local".to_string(),
            rate_limit_per_minute: 100,
        });
        let results = orch.call_batch(&[("batch", "/a"), ("batch", "/b")]);
        assert_eq!(results.len(), 2);
    }
}
