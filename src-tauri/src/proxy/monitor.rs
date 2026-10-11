use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tauri::Emitter;
use tokio::sync::{RwLock, Semaphore};

mod proxymonitor_impl;
pub(crate) mod proxyrequestlog;

#[cfg(test)]
pub(crate) use proxyrequestlog::prompt_log_tests;
pub(crate) use proxyrequestlog::sanitize_upstream_debug_value;
pub use proxyrequestlog::ProxyMonitor;
pub use proxyrequestlog::ProxyRequestLog;
pub use proxyrequestlog::ProxyStats;
pub(crate) use proxyrequestlog::UpstreamCapture;
pub use proxyrequestlog::UpstreamRequestBodyHolder;
pub use proxyrequestlog::CURRENT_UPSTREAM_CAPTURE;
