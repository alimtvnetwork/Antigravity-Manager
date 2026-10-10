//! Antigravity Conversation Pruning, Cache Cleaning, and Undo Engine.
//!
//! Provides cross-platform discovery of Antigravity conversation databases and
//! application caches, recency-based retention pruning, safe staging in the OS
//! temporary directory for undo recovery, and transaction rollback.

pub mod daemon;
pub mod paths;
pub mod prune;
pub mod scan;
pub mod types;

pub use daemon::*;
pub use paths::*;
pub use prune::*;
pub use scan::*;
pub use types::*;
