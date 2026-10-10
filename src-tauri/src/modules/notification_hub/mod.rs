//! Notification Hub Module
//! Coordinates unified notifications across Email and Telegram channels
//! for critical events such as account switching, quota alerts, and workspace updates.
#![allow(dead_code)]

pub mod channels;
pub mod config_alerts;
pub mod email_alerts;
pub mod state;
pub mod switch_context;
pub mod system;
pub mod telegram_alerts;
pub mod telemetry;
pub mod tests;

pub use channels::*;
pub use config_alerts::*;
pub use email_alerts::*;
pub use state::*;
pub use switch_context::*;
pub use system::*;
pub use telegram_alerts::*;
pub use telemetry::*;
pub use tests::*;
