//! TokenManager module: account token pool management, scheduling, current limiting and quota protection.
//!
//! Split from `token_manager.rs` single file; external API remains unchanged.

mod account_load;
mod account_parse;
mod account_write;
mod health;
mod helpers;
mod lifecycle;
mod lookup;
mod oauth;
mod quota;
mod quota_models;
mod quota_protect;
mod rate_limit;
mod rate_limit_async;
mod rate_limit_mark;
mod session;
mod token_acquire;
mod token_maintain;
mod token_prefilter;
mod token_select;
mod types;

#[cfg(test)]
mod tests;

pub(crate) use types::TrackerParserMode;
pub use types::{ProxyToken, TokenManager};
