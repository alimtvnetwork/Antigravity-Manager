//! TokenManager 模块：账号 Token 池管理、调度、限流与配额保护。
//!
//! 由 `token_manager.rs` 单文件拆分而来；对外 API 保持不变。

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

pub use types::{ProxyToken, TokenManager};
