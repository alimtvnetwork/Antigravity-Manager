//! Outbound Email Dispatcher Module
//! Implements SMTP delivery with failover mailbox swapping,
//! anti-spam HTML templates, and machine telemetry headers.
#![allow(dead_code)]

pub mod dispatch;
pub mod formatting;
pub mod smtp;
pub mod stream;
pub mod templates_a;
pub mod templates_b;
pub mod templates_c;
pub mod tests;
pub mod types;

pub use dispatch::*;
pub use formatting::*;
pub use smtp::*;
pub use stream::*;
pub use templates_a::*;
pub use templates_b::*;
pub use templates_c::*;
pub use tests::*;
pub use types::*;
