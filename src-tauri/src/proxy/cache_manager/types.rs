use super::*;

// ===== Layer Limits (following SignatureCache pattern) =====
pub const SI_CACHE_LIMIT: usize = 200;
pub const TOOLS_CACHE_LIMIT: usize = 100;
pub const PREFIX_TRACKER_LIMIT: usize = 500;

// ===== TTL Constants =====
/// Layer 1 & 2 TTL: 30 min — longer than the final content cache because it does not change with the session
pub const LAYER_12_TTL: Duration = Duration::from_secs(30 * 60);
/// Layer 3 TTL: 1 hour — Aligned Gemini explicit cache default TTL
pub const LAYER_3_TTL: Duration = Duration::from_secs(3600);

// ===== Layer 1: System Instruction Cache =====

/// Layer 1 entry: raw system instructions → sanitized text
#[derive(Debug, Clone)]
pub(crate) struct SiCacheEntry {
    /// Cleaned system instruction text
    pub(crate) sanitized_text: String,
    /// Create/update time
    pub(crate) timestamp: Instant,
    /// Number of hits
    pub(crate) hit_count: u64,
}

// ===== Layer 2: Tools Cache =====

/// Layer 2 entry: raw tools JSON → processed tools
#[derive(Debug, Clone)]
pub(crate) struct ToolsCacheEntry {
    /// Processed tools JSON (serialized to string, deserialized when used)
    pub(crate) tools_json: String,
    /// Create/update time
    pub(crate) timestamp: Instant,
    /// Number of hits
    pub(crate) hit_count: u64,
}

// ===== Layer 3: Prefix Tracker =====

/// Layer 3 entry: Tracking (Layer1_hash + Layer2_hash) combination
#[derive(Debug, Clone)]
pub(crate) struct PrefixTrackingEntry {
    /// Layer 1 hash (for correlation)
    pub(crate) si_hash: String,
    /// Layer 2 hash (for correlation)
    pub(crate) tools_hash: String,
    /// Gemini cached resource name (cachedContents/xxx)
    pub(crate) cache_name: String,
    /// creation time
    pub(crate) created_at: Instant,
    /// Expiration time
    pub(crate) expires_at: Instant,
    /// Number of implicit cache hits (cachedContentTokenCount > 0)
    pub(crate) implicit_hit_count: u64,
    /// Number of explicit cache hits (successfully injected cachedContent)
    pub(crate) explicit_hit_count: u64,
    /// Model name
    pub(crate) model: String,
}

// ===== Stats =====

/// Hierarchical statistics
#[derive(Debug, Clone, Default)]
pub struct LayerStats {
    /// Layer 1: SI cache statistics
    pub si_total: u64,
    pub si_hits: u64,
    pub si_misses: u64,
    /// Layer 2: Tools cache statistics
    pub tools_total: u64,
    pub tools_hits: u64,
    pub tools_misses: u64,
    /// Layer 3: Prefix tracking statistics
    pub prefix_total: u64,
    pub prefix_hits: u64,
    pub prefix_misses: u64,
    /// Total implicit cache hits
    pub total_implicit_hits: u64,
    /// Total explicit cache hits
    pub total_explicit_hits: u64,
    /// Current number of active entries
    pub active_si_entries: usize,
    pub active_tools_entries: usize,
    pub active_prefix_entries: usize,
}

// ===== CacheManager =====

/// Context Cache Manager — multi-level cache singleton
pub struct CacheManager {
    /// Layer 1: raw SI hash → sanitized text
    pub(crate) si_cache: DashMap<String, SiCacheEntry>,
    /// Layer 1 Statistics
    pub(crate) si_stats: std::sync::RwLock<(u64, u64, u64)>, // (total, hits, misses)

    /// Layer 2: raw tools hash → processed tools JSON
    pub(crate) tools_cache: DashMap<String, ToolsCacheEntry>,
    /// Layer 2 Statistics
    pub(crate) tools_stats: std::sync::RwLock<(u64, u64, u64)>,

    /// Layer 3: combined hash → tracking entry
    pub(crate) prefix_tracker: DashMap<String, PrefixTrackingEntry>,
    /// Layer 3 Statistics
    pub(crate) prefix_stats: std::sync::RwLock<(u64, u64, u64)>,
}
