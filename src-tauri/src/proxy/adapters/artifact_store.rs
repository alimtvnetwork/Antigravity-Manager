//! Sidecar store for raw tool/search/page payloads.
//!
//! The chat history must not replay huge tool output as ordinary `tool.content`.
//! Large raw payloads are stored here and the model only receives a bounded
//! evidence summary plus an artifact id.

use rusqlite::{params, Connection, OptionalExtension};
use std::collections::HashMap;
use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

mod storedtoolartifact;
mod tests;

pub(crate) use storedtoolartifact::create_schema;
pub use storedtoolartifact::global_tool_artifact_store;
pub(crate) use storedtoolartifact::init_db;
pub(crate) use storedtoolartifact::log_artifact_warning;
pub(crate) use storedtoolartifact::new_artifact_id;
pub use storedtoolartifact::read_tool_artifact_raw;
pub(crate) use storedtoolartifact::unix_now;
pub(crate) use storedtoolartifact::ArtifactEntry;
pub(crate) use storedtoolartifact::ArtifactStoreInner;
pub use storedtoolartifact::StoredToolArtifact;
pub use storedtoolartifact::ToolArtifactRecord;
pub use storedtoolartifact::ToolArtifactStore;
#[cfg(test)]
pub(crate) use storedtoolartifact::DEFAULT_PERSISTED_TTL;
