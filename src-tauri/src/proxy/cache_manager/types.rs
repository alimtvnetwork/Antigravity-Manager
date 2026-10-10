use super::*;

// ===== Layer Limits (following SignatureCache pattern) =====
const SI_CACHE_LIMIT: usize = 200;
const TOOLS_CACHE_LIMIT: usize = 100;
const PREFIX_TRACKER_LIMIT: usize = 500;

// ===== TTL Constants =====
/// Layer 1 & 2 TTL: 30 min — 比最终内容缓存更长，因为不随 session 变化
const LAYER_12_TTL: Duration = Duration::from_secs(30 * 60);
/// Layer 3 TTL: 1 hour — 对齐 Gemini 显式缓存默认 TTL
const LAYER_3_TTL: Duration = Duration::from_secs(3600);

// ===== Layer 1: System Instruction Cache =====

/// Layer 1 条目: raw system instructions → sanitized text
#[derive(Debug, Clone)]
struct SiCacheEntry {
    /// 清洗后的 system instruction 文本
    sanitized_text: String,
    /// 创建/更新时间
    timestamp: Instant,
    /// 命中次数
    hit_count: u64,
}

// ===== Layer 2: Tools Cache =====

/// Layer 2 条目: raw tools JSON → processed tools
#[derive(Debug, Clone)]
struct ToolsCacheEntry {
    /// 处理后的 tools JSON (序列化为字符串，使用时反序列化)
    tools_json: String,
    /// 创建/更新时间
    timestamp: Instant,
    /// 命中次数
    hit_count: u64,
}

// ===== Layer 3: Prefix Tracker =====

/// Layer 3 条目: 追踪 (Layer1_hash + Layer2_hash) 组合
#[derive(Debug, Clone)]
struct PrefixTrackingEntry {
    /// Layer 1 的 hash (用于关联)
    si_hash: String,
    /// Layer 2 的 hash (用于关联)
    tools_hash: String,
    /// Gemini 缓存的资源名 (cachedContents/xxx)
    cache_name: String,
    /// 创建时间
    created_at: Instant,
    /// 过期时间
    expires_at: Instant,
    /// 隐式缓存命中次数 (cachedContentTokenCount > 0)
    implicit_hit_count: u64,
    /// 显式缓存命中次数 (成功注入 cachedContent)
    explicit_hit_count: u64,
    /// 模型名
    model: String,
}

// ===== Stats =====

/// 分层统计信息
#[derive(Debug, Clone, Default)]
pub struct LayerStats {
    /// Layer 1: SI 缓存统计
    pub si_total: u64,
    pub si_hits: u64,
    pub si_misses: u64,
    /// Layer 2: Tools 缓存统计
    pub tools_total: u64,
    pub tools_hits: u64,
    pub tools_misses: u64,
    /// Layer 3: Prefix 跟踪统计
    pub prefix_total: u64,
    pub prefix_hits: u64,
    pub prefix_misses: u64,
    /// 总隐式缓存命中次数
    pub total_implicit_hits: u64,
    /// 总显式缓存命中次数
    pub total_explicit_hits: u64,
    /// 当前活跃条目数
    pub active_si_entries: usize,
    pub active_tools_entries: usize,
    pub active_prefix_entries: usize,
}

// ===== CacheManager =====

/// Context Cache Manager — 多层级缓存单例
pub struct CacheManager {
    /// Layer 1: raw SI hash → sanitized text
    si_cache: DashMap<String, SiCacheEntry>,
    /// Layer 1 统计
    si_stats: std::sync::RwLock<(u64, u64, u64)>, // (total, hits, misses)

    /// Layer 2: raw tools hash → processed tools JSON
    tools_cache: DashMap<String, ToolsCacheEntry>,
    /// Layer 2 统计
    tools_stats: std::sync::RwLock<(u64, u64, u64)>,

    /// Layer 3: combined hash → tracking entry
    prefix_tracker: DashMap<String, PrefixTrackingEntry>,
    /// Layer 3 统计
    prefix_stats: std::sync::RwLock<(u64, u64, u64)>,
}
