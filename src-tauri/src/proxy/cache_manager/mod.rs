//! Context Cache Manager — Multi-Layer Split Cache
//!
//! 维护三层独立缓存，遵循 Antigravity SignatureCache 的多层级模式。
//! 每层独立 TTL、容限、LRU 淘汰、统计 — 一层变化不影响其他层的命中。
//!
//! 架构:
//!   Layer 1 (SI Cache):        raw_instructions → sanitized_text
//!     跨 session 复用：相同 system prompt 的不同对话共享 sanitized 结果
//!   Layer 2 (Tools Cache):     raw_tools_json → processed_tools
//!     跨 session 复用：相同 tool schema 的不同对话共享处理结果
//!   Layer 3 (Prefix Tracker):  combined_key → PrefixTrackingEntry
//!     追踪 (Layer1 + Layer2) 组合的生命周期，用于 cachedContent 注入和命中统计
//!
//! 缓存生命周期:
//! 1. 请求到达 → 计算 Layer 1 key → 查找或 sanitize systemInstruction
//! 2. 计算 Layer 2 key → 查找或构建 tools
//! 3. 计算 Layer 3 key → 查找是否已有 cache_name → 注入 cachedContent
//! 4. 响应到达 → 若 cachedContentTokenCount > 0 → 更新 Layer 3 统计
//! 5. 过期条目在 insert 时被动淘汰 (LRU on limit breach)

use dashmap::DashMap;
use sha2::{Digest, Sha256};
use std::time::{Duration, Instant};

mod generic;
mod global;
mod prefix;
mod si;
mod tests;
mod tools_layer;
mod types;

pub use generic::*;
pub use global::*;
pub use prefix::*;
pub use si::*;
pub use tests::*;
pub use tools_layer::*;
pub use types::*;
