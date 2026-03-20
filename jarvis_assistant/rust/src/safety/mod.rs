use std::collections::HashMap;

// ── Permission Layer ──────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PermissionCategory {
    OsControl,
    Browser,
    Iot,
    ApiCall,
    FileSystem,
    Email,
    Calendar,
    Financial,
    Other,
}

pub struct PermissionLayer {
    grants: HashMap<PermissionCategory, bool>,
}

impl PermissionLayer {
    /// All permissions denied by default.
    pub fn new() -> Self {
        Self {
            grants: HashMap::new(),
        }
    }

    pub fn grant(&mut self, category: PermissionCategory) {
        self.grants.insert(category, true);
    }

    pub fn revoke(&mut self, category: PermissionCategory) {
        self.grants.insert(category, false);
    }

    pub fn is_granted(&self, category: PermissionCategory) -> bool {
        *self.grants.get(&category).unwrap_or(&false)
    }
}

impl Default for PermissionLayer {
    fn default() -> Self {
        Self::new()
    }
}

// ── Risk Detector ─────────────────────────────────────────────────────────────

pub struct RiskDetector;

impl RiskDetector {
    /// Returns a risk score in [0.0, 1.0].
    /// Stub: returns 0.1 for all actions (low risk by default).
    pub fn score(action: &str) -> f32 {
        let _ = action;
        0.1_f32.clamp(0.0, 1.0)
    }
}

// ── Ethics Engine ─────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct EthicsViolation {
    pub rule: String,
    pub description: String,
}

pub struct EthicsEngine;

impl EthicsEngine {
    /// Check hard constraints. Returns None if no violation, Some(violation) if violated.
    /// Hard constraints: no deception, no unauthorized access.
    pub fn check(action: &str) -> Option<EthicsViolation> {
        if action.contains("deceive") {
            return Some(EthicsViolation {
                rule: "no_deception".to_string(),
                description: "Action involves deception, which is prohibited.".to_string(),
            });
        }
        if action.contains("unauthorized") {
            return Some(EthicsViolation {
                rule: "no_unauthorized_access".to_string(),
                description: "Action involves unauthorized access, which is prohibited.".to_string(),
            });
        }
        None
    }
}

// ── Audit Log ─────────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct AuditEntry {
    pub id: u64,
    pub action: String,
    pub category: PermissionCategory,
    pub approved: bool,
    pub risk_score: f32,
    /// Stub: uses audit_counter as timestamp.
    pub timestamp: u64,
}

// ── Validation Result ─────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct ValidationResult {
    pub approved: bool,
    pub risk_score: f32,
    pub ethics_violation: Option<EthicsViolation>,
    pub permission_denied: bool,
    pub audit_id: u64,
}

// ── Safety Alignment System ───────────────────────────────────────────────────

pub struct SafetyAlignmentSystem {
    pub permissions: PermissionLayer,
    audit_counter: u64,
    audit_log: Vec<AuditEntry>,
}

impl SafetyAlignmentSystem {
    pub fn new() -> Self {
        Self {
            permissions: PermissionLayer::new(),
            audit_counter: 0,
            audit_log: Vec::new(),
        }
    }

    /// Permission → risk → ethics pipeline.
    ///
    /// INVARIANT: every call to `validate()` writes exactly one entry to the
    /// audit log, regardless of the outcome.
    pub fn validate(
        &mut self,
        action: &str,
        category: PermissionCategory,
    ) -> ValidationResult {
        // 1. Permission check
        let permission_denied = !self.permissions.is_granted(category);

        // 2. Risk score
        let risk_score = RiskDetector::score(action);

        // 3. Ethics check
        let ethics_violation = EthicsEngine::check(action);

        // 4. Approval decision
        let approved = !permission_denied && ethics_violation.is_none();

        // 5. Increment counter and write audit entry
        self.audit_counter += 1;
        let id = self.audit_counter;
        self.audit_log.push(AuditEntry {
            id,
            action: action.to_string(),
            category,
            approved,
            risk_score,
            timestamp: id, // stub: counter doubles as timestamp
        });

        // 6. Return result
        ValidationResult {
            approved,
            risk_score,
            ethics_violation,
            permission_denied,
            audit_id: id,
        }
    }

    /// Returns an immutable view of the audit log.
    pub fn audit_log(&self) -> &[AuditEntry] {
        &self.audit_log
    }
}

impl Default for SafetyAlignmentSystem {
    fn default() -> Self {
        Self::new()
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn permission_denied_returns_not_approved() {
        let mut sys = SafetyAlignmentSystem::new();
        // FileSystem not granted
        let result = sys.validate("read file", PermissionCategory::FileSystem);
        assert!(!result.approved);
        assert!(result.permission_denied);
    }

    #[test]
    fn ethics_violation_returns_not_approved() {
        let mut sys = SafetyAlignmentSystem::new();
        sys.permissions.grant(PermissionCategory::Other);
        let result = sys.validate("deceive user", PermissionCategory::Other);
        assert!(!result.approved);
        assert!(result.ethics_violation.is_some());
    }

    #[test]
    fn valid_action_returns_approved() {
        let mut sys = SafetyAlignmentSystem::new();
        sys.permissions.grant(PermissionCategory::FileSystem);
        let result = sys.validate("read file", PermissionCategory::FileSystem);
        assert!(result.approved);
        assert!(!result.permission_denied);
        assert!(result.ethics_violation.is_none());
    }

    #[test]
    fn every_validate_writes_audit_entry() {
        let mut sys = SafetyAlignmentSystem::new();
        sys.validate("action one", PermissionCategory::Other);
        sys.validate("action two", PermissionCategory::Other);
        sys.validate("action three", PermissionCategory::Other);
        assert_eq!(sys.audit_log().len(), 3);
    }

    #[test]
    fn risk_score_clamped() {
        // Stub always returns 0.1, but verify it is within [0.0, 1.0]
        let score = RiskDetector::score("any action");
        assert!((0.0..=1.0).contains(&score));
    }

    // Property 16: ethics violation always returns approved=false.
    #[test]
    fn ethics_violation_always_returns_not_approved() {
        let mut sys = SafetyAlignmentSystem::new();
        // Grant all permissions so only ethics can block.
        for cat in [
            PermissionCategory::OsControl, PermissionCategory::Browser,
            PermissionCategory::Iot, PermissionCategory::ApiCall,
            PermissionCategory::FileSystem, PermissionCategory::Email,
            PermissionCategory::Calendar, PermissionCategory::Financial,
            PermissionCategory::Other,
        ] {
            sys.permissions.grant(cat);
        }
        // Both hard-constraint violations must return approved=false.
        let r1 = sys.validate("deceive the user", PermissionCategory::Other);
        assert!(!r1.approved, "deception must not be approved");
        assert!(r1.ethics_violation.is_some());

        let r2 = sys.validate("unauthorized access to system", PermissionCategory::Other);
        assert!(!r2.approved, "unauthorized access must not be approved");
        assert!(r2.ethics_violation.is_some());
    }

    // Property 15: every validate() call produces an audit log entry.
    #[test]
    fn every_validate_call_produces_audit_log_entry() {
        let mut sys = SafetyAlignmentSystem::new();
        let actions = ["action one", "deceive user", "read file", "unauthorized access"];
        for (i, action) in actions.iter().enumerate() {
            sys.validate(action, PermissionCategory::Other);
            assert_eq!(
                sys.audit_log().len(),
                i + 1,
                "audit log must have {} entries after {} calls",
                i + 1, i + 1
            );
        }
    }

    // Property 15 variant: audit log entries are append-only (ids strictly increase).
    #[test]
    fn audit_log_ids_strictly_increase() {
        let mut sys = SafetyAlignmentSystem::new();
        for _ in 0..5 {
            sys.validate("test", PermissionCategory::Other);
        }
        let ids: Vec<u64> = sys.audit_log().iter().map(|e| e.id).collect();
        for window in ids.windows(2) {
            assert!(window[1] > window[0], "audit log ids must strictly increase");
        }
    }
}
