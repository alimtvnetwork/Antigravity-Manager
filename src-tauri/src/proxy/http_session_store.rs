use crate::proxy::handlers::openai::get_cached_tool_call;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::{Arc, OnceLock};
use std::time::{Duration, Instant};
use tokio::sync::Mutex;

mod httpsessionentry;
mod tests;

pub(crate) use httpsessionentry::dedupe_input_items;
pub use httpsessionentry::get_session;
pub use httpsessionentry::get_session_with_parent;
pub(crate) use httpsessionentry::materialize_history;
pub use httpsessionentry::merge_history_with_new_input;
pub use httpsessionentry::prepare_session_input;
pub use httpsessionentry::prepare_session_input_with_storage;
pub(crate) use httpsessionentry::repair_tool_calls;
pub use httpsessionentry::save_session;
pub use httpsessionentry::save_session_delta;
pub(crate) use httpsessionentry::store;
pub use httpsessionentry::HttpSessionEntry;
pub(crate) use httpsessionentry::HttpSessionStore;
pub use httpsessionentry::PreparedSessionInput;
pub use httpsessionentry::SessionParent;
pub(crate) use httpsessionentry::SessionNode;
pub(crate) use httpsessionentry::StoredSession;
