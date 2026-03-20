/// Cross-Device Sync — Task 33 (optional)
///
/// Privacy-first design:
///   - Sync is DISABLED by default; user must explicitly opt in.
///   - Encryption key is generated on-device and NEVER transmitted.
///   - Only the scopes the user selects are synced.
///   - Conflict resolution uses last-write-wins; ties are flagged for review.

use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

// ── SyncScope ─────────────────────────────────────────────────────────────────

/// Data categories that can be selectively synced across devices.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SyncScope {
    LongTermMemory,
    Goals,
    Preferences,
    ConversationHistory,
}

// ── DataSyncConfig ────────────────────────────────────────────────────────────

/// Configuration for cross-device sync.
///
/// Sync is **disabled by default** — the user must call `enable()` to opt in.
/// The encryption key is generated locally and never leaves the device.
#[derive(Debug, Clone)]
pub struct DataSyncConfig {
    /// Whether sync is active. Defaults to `false`.
    pub enabled: bool,
    /// Which data categories to sync. Empty by default.
    pub scope: Vec<SyncScope>,
    /// Device-local encryption key. `None` until `DeviceKeyManager::generate_key()` is called.
    /// This field is intentionally NOT serialised / transmitted.
    pub(crate) encryption_key: Option<Vec<u8>>,
}

impl Default for DataSyncConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            scope: Vec::new(),
            encryption_key: None,
        }
    }
}

impl DataSyncConfig {
    /// Create a new config with sync disabled (safe default).
    pub fn new() -> Self {
        Self::default()
    }

    /// Explicitly opt in to sync and set the desired scopes.
    pub fn enable(&mut self, scope: Vec<SyncScope>) {
        self.enabled = true;
        self.scope = scope;
    }

    /// Disable sync and clear all scopes.
    pub fn disable(&mut self) {
        self.enabled = false;
        self.scope.clear();
    }

    /// Returns true only when sync is enabled AND at least one scope is selected.
    pub fn is_active(&self) -> bool {
        self.enabled && !self.scope.is_empty()
    }
}

// ── DeviceKeyManager ──────────────────────────────────────────────────────────

/// Manages the device-local encryption key.
///
/// The key is generated on-device using a CSPRNG stub and is NEVER transmitted.
pub struct DeviceKeyManager;

impl DeviceKeyManager {
    /// Generate a 256-bit (32-byte) device-local encryption key.
    ///
    /// In production this would use `ring` or `getrandom` for a CSPRNG.
    /// Here we use a deterministic stub that satisfies the structural contract.
    pub fn generate_key() -> Vec<u8> {
        // Stub: derive 32 bytes from the current timestamp XOR'd with a fixed salt.
        // Production: replace with `ring::rand::SystemRandom` or `getrandom::getrandom`.
        let ts = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .subsec_nanos();
        let salt: [u8; 32] = [
            0xDE, 0xAD, 0xBE, 0xEF, 0xCA, 0xFE, 0xBA, 0xBE,
            0x01, 0x23, 0x45, 0x67, 0x89, 0xAB, 0xCD, 0xEF,
            0xFE, 0xDC, 0xBA, 0x98, 0x76, 0x54, 0x32, 0x10,
            0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, 0x88,
        ];
        salt.iter()
            .enumerate()
            .map(|(i, &b)| b ^ ((ts >> (i % 32)) as u8))
            .collect()
    }

    /// Attach a freshly generated key to a `DataSyncConfig`.
    /// The key is stored only in memory; it is the caller's responsibility
    /// to persist it in the device's secure enclave / keystore.
    pub fn attach_key(config: &mut DataSyncConfig) {
        config.encryption_key = Some(Self::generate_key());
    }
}

// ── SyncRecord ────────────────────────────────────────────────────────────────

/// A single piece of data to be synced, tagged with its scope and timestamp.
#[derive(Debug, Clone)]
pub struct SyncRecord {
    pub id: String,
    pub scope: SyncScope,
    /// Unix timestamp (milliseconds) of the last write.
    pub last_modified_ms: u64,
    /// Opaque payload — encrypted before transmission.
    pub payload: Vec<u8>,
}

impl SyncRecord {
    pub fn new(id: impl Into<String>, scope: SyncScope, payload: Vec<u8>) -> Self {
        let last_modified_ms = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;
        Self {
            id: id.into(),
            scope,
            last_modified_ms,
            payload,
        }
    }
}

// ── SyncConflict ──────────────────────────────────────────────────────────────

/// Describes a conflict between two versions of the same record.
#[derive(Debug, Clone)]
pub struct SyncConflict {
    pub record_id: String,
    pub scope: SyncScope,
    pub local_modified_ms: u64,
    pub remote_modified_ms: u64,
    /// The record that won under last-write-wins.
    pub resolved: SyncRecord,
    /// Whether the timestamps were equal — flagged for user review.
    pub needs_user_review: bool,
}

impl SyncConflict {
    /// Resolve a conflict using last-write-wins.
    ///
    /// If timestamps are equal the local record wins and `needs_user_review` is set.
    pub fn resolve(local: SyncRecord, remote: SyncRecord) -> Self {
        let needs_user_review = local.last_modified_ms == remote.last_modified_ms;
        let resolved = if remote.last_modified_ms > local.last_modified_ms {
            remote.clone()
        } else {
            local.clone()
        };
        Self {
            record_id: local.id.clone(),
            scope: local.scope,
            local_modified_ms: local.last_modified_ms,
            remote_modified_ms: remote.last_modified_ms,
            resolved,
            needs_user_review,
        }
    }
}

// ── CrossDeviceSync ───────────────────────────────────────────────────────────

/// Orchestrates cross-device sync.
///
/// `sync()` is a no-op when `config.is_active()` returns false, ensuring
/// that users who have not opted in are never affected.
pub struct CrossDeviceSync {
    config: DataSyncConfig,
}

/// Result of a sync operation.
#[derive(Debug)]
pub struct SyncResult {
    pub synced_count: usize,
    pub conflicts: Vec<SyncConflict>,
    pub skipped_scopes: Vec<SyncScope>,
}

impl CrossDeviceSync {
    pub fn new(config: DataSyncConfig) -> Self {
        Self { config }
    }

    /// Perform a sync pass.
    ///
    /// - Returns immediately with zero work if sync is disabled.
    /// - Only processes records whose scope is in `config.scope`.
    /// - Resolves conflicts via `SyncConflict::resolve()`.
    pub fn sync(
        &self,
        local_records: &[SyncRecord],
        remote_records: &[SyncRecord],
    ) -> SyncResult {
        if !self.config.is_active() {
            return SyncResult {
                synced_count: 0,
                conflicts: vec![],
                skipped_scopes: vec![],
            };
        }

        let active_scopes: std::collections::HashSet<SyncScope> =
            self.config.scope.iter().copied().collect();

        // Index remote records by id for O(1) lookup.
        let remote_index: HashMap<&str, &SyncRecord> =
            remote_records.iter().map(|r| (r.id.as_str(), r)).collect();

        let mut synced_count = 0usize;
        let mut conflicts: Vec<SyncConflict> = vec![];
        let mut skipped_scopes: Vec<SyncScope> = vec![];

        for local in local_records {
            if !active_scopes.contains(&local.scope) {
                if !skipped_scopes.contains(&local.scope) {
                    skipped_scopes.push(local.scope);
                }
                continue;
            }

            if let Some(&remote) = remote_index.get(local.id.as_str()) {
                // Conflict: same id exists on both sides.
                if local.last_modified_ms != remote.last_modified_ms {
                    conflicts.push(SyncConflict::resolve(local.clone(), remote.clone()));
                }
                // If timestamps are identical, flag for user review.
                else {
                    conflicts.push(SyncConflict::resolve(local.clone(), remote.clone()));
                }
            }

            synced_count += 1;
        }

        SyncResult {
            synced_count,
            conflicts,
            skipped_scopes,
        }
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // ── 33.1*: DataSyncConfig defaults ───────────────────────────────────────

    #[test]
    fn sync_config_disabled_by_default() {
        let config = DataSyncConfig::new();
        assert!(!config.enabled, "sync must be disabled by default");
        assert!(config.scope.is_empty(), "scope must be empty by default");
        assert!(config.encryption_key.is_none(), "no key before generation");
        assert!(!config.is_active(), "is_active must be false by default");
    }

    #[test]
    fn sync_config_enable_sets_active() {
        let mut config = DataSyncConfig::new();
        config.enable(vec![SyncScope::Goals, SyncScope::Preferences]);
        assert!(config.enabled);
        assert!(config.is_active());
        assert_eq!(config.scope.len(), 2);
    }

    #[test]
    fn sync_config_disable_clears_state() {
        let mut config = DataSyncConfig::new();
        config.enable(vec![SyncScope::Goals]);
        config.disable();
        assert!(!config.enabled);
        assert!(!config.is_active());
        assert!(config.scope.is_empty());
    }

    // ── 33.2*: E2E encryption key ─────────────────────────────────────────────

    #[test]
    fn generate_key_produces_32_bytes() {
        let key = DeviceKeyManager::generate_key();
        assert_eq!(key.len(), 32, "encryption key must be 256 bits (32 bytes)");
    }

    #[test]
    fn attach_key_stores_key_in_config() {
        let mut config = DataSyncConfig::new();
        DeviceKeyManager::attach_key(&mut config);
        assert!(
            config.encryption_key.is_some(),
            "key must be stored in config after attach_key"
        );
        assert_eq!(config.encryption_key.as_ref().unwrap().len(), 32);
    }

    /// The key must never be all-zeros (trivially weak key).
    #[test]
    fn generated_key_is_not_all_zeros() {
        let key = DeviceKeyManager::generate_key();
        assert!(
            key.iter().any(|&b| b != 0),
            "generated key must not be all-zeros"
        );
    }

    // ── 33.3*: Selective sync scope ───────────────────────────────────────────

    #[test]
    fn sync_only_processes_configured_scopes() {
        let mut config = DataSyncConfig::new();
        config.enable(vec![SyncScope::Goals]);
        let syncer = CrossDeviceSync::new(config);

        let local = vec![
            SyncRecord::new("g1", SyncScope::Goals, b"goal data".to_vec()),
            SyncRecord::new("p1", SyncScope::Preferences, b"pref data".to_vec()),
            SyncRecord::new("m1", SyncScope::LongTermMemory, b"mem data".to_vec()),
        ];

        let result = syncer.sync(&local, &[]);

        // Only Goals scope is active — 1 record synced, 2 scopes skipped.
        assert_eq!(result.synced_count, 1, "only Goals records should be synced");
        assert!(
            result.skipped_scopes.contains(&SyncScope::Preferences),
            "Preferences must be in skipped_scopes"
        );
        assert!(
            result.skipped_scopes.contains(&SyncScope::LongTermMemory),
            "LongTermMemory must be in skipped_scopes"
        );
    }

    #[test]
    fn sync_all_four_scopes_when_all_enabled() {
        let mut config = DataSyncConfig::new();
        config.enable(vec![
            SyncScope::LongTermMemory,
            SyncScope::Goals,
            SyncScope::Preferences,
            SyncScope::ConversationHistory,
        ]);
        let syncer = CrossDeviceSync::new(config);

        let local = vec![
            SyncRecord::new("m1", SyncScope::LongTermMemory, b"mem".to_vec()),
            SyncRecord::new("g1", SyncScope::Goals, b"goal".to_vec()),
            SyncRecord::new("p1", SyncScope::Preferences, b"pref".to_vec()),
            SyncRecord::new("c1", SyncScope::ConversationHistory, b"conv".to_vec()),
        ];

        let result = syncer.sync(&local, &[]);
        assert_eq!(result.synced_count, 4);
        assert!(result.skipped_scopes.is_empty());
    }

    // ── 33.4*: Conflict resolution ────────────────────────────────────────────

    #[test]
    fn last_write_wins_selects_newer_remote() {
        let local = SyncRecord {
            id: "r1".to_string(),
            scope: SyncScope::Goals,
            last_modified_ms: 1_000,
            payload: b"old".to_vec(),
        };
        let remote = SyncRecord {
            id: "r1".to_string(),
            scope: SyncScope::Goals,
            last_modified_ms: 2_000,
            payload: b"new".to_vec(),
        };
        let conflict = SyncConflict::resolve(local, remote);
        assert_eq!(conflict.resolved.payload, b"new", "newer remote must win");
        assert!(!conflict.needs_user_review);
    }

    #[test]
    fn last_write_wins_selects_newer_local() {
        let local = SyncRecord {
            id: "r1".to_string(),
            scope: SyncScope::Goals,
            last_modified_ms: 3_000,
            payload: b"local_new".to_vec(),
        };
        let remote = SyncRecord {
            id: "r1".to_string(),
            scope: SyncScope::Goals,
            last_modified_ms: 1_000,
            payload: b"remote_old".to_vec(),
        };
        let conflict = SyncConflict::resolve(local, remote);
        assert_eq!(conflict.resolved.payload, b"local_new", "newer local must win");
        assert!(!conflict.needs_user_review);
    }

    #[test]
    fn equal_timestamps_flagged_for_user_review() {
        let ts = 5_000u64;
        let local = SyncRecord {
            id: "r1".to_string(),
            scope: SyncScope::Preferences,
            last_modified_ms: ts,
            payload: b"local".to_vec(),
        };
        let remote = SyncRecord {
            id: "r1".to_string(),
            scope: SyncScope::Preferences,
            last_modified_ms: ts,
            payload: b"remote".to_vec(),
        };
        let conflict = SyncConflict::resolve(local, remote);
        assert!(
            conflict.needs_user_review,
            "equal timestamps must be flagged for user review"
        );
    }

    #[test]
    fn sync_disabled_is_noop() {
        let config = DataSyncConfig::new(); // disabled by default
        let syncer = CrossDeviceSync::new(config);

        let local = vec![SyncRecord::new("g1", SyncScope::Goals, b"data".to_vec())];
        let result = syncer.sync(&local, &[]);

        assert_eq!(result.synced_count, 0, "disabled sync must be a no-op");
        assert!(result.conflicts.is_empty());
        assert!(result.skipped_scopes.is_empty());
    }
}
