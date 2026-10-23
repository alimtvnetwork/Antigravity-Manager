//! Context Cache Manager — Multi-Layer Split Cache
//!
//! Maintain three levels of independent cache, following the multi-level model of Antigravity SignatureCache.
//! Each layer has independent TTL, tolerance, LRU elimination, statistics - changes in one layer do not affect hits in other layers.
//!
//! Architecture:
//!   Layer 1 (SI Cache):        raw_instructions → sanitized_text
//!     Cross-session reuse: different conversations with the same system prompt share sanitized results
//!   Layer 2 (Tools Cache):     raw_tools_json → processed_tools
//!     Cross-session reuse: different conversations with the same tool schema share processing results
//!   Layer 3 (Prefix Tracker):  combined_key → PrefixTrackingEntry
//!     Track the life cycle of (Layer1 + Layer2) combination for cachedContent injection and hit statistics
//!
//! Cache life cycle:
//! 1. The request arrives → calculate Layer 1 key → find or sanitize systemInstruction
//! 2. Calculate Layer 2 key → Find or build tools
//! 3. Calculate Layer 3 key → Find whether cache_name already exists → Inject cachedContent
//! 4. Response arrives → if cachedContentTokenCount > 0 → Update Layer 3 statistics
//! 5. Expired entries are passively eliminated during insert (LRU on limit breach)

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
