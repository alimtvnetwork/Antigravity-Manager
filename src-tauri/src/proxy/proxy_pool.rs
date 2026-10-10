use crate::proxy::config::{ProxyEntry, ProxyPoolConfig, ProxySelectionStrategy};
use dashmap::DashMap;
use futures::{stream, StreamExt};
use rquest::Client;
use rquest_util::Emulation;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::sync::OnceLock;
use std::time::Duration;
use tokio::sync::RwLock;

mod get_global_proxy_pool;
mod proxypoolmanager_impl;

pub use get_global_proxy_pool::get_global_proxy_pool;
pub use get_global_proxy_pool::init_global_proxy_pool;
pub use get_global_proxy_pool::PoolProxyConfig;
pub use get_global_proxy_pool::ProxyPoolManager;
pub(crate) use proxypoolmanager_impl::tests;
