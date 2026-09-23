//! Full-duplex unified model thinking and response streaming pipeline architecture
//!
//! 1. Canonical Intermediate Representation (Canonical IR): Google Gemini standard format (`contents` + `generationConfig`)
//! 2. Protocol Inbound Strategy Adapters (`InboundThinkingPipeline`, `ProxyProtocol`, `UpstreamClassification`)
//! 3. Unified Usage and Cache Computation & Protocol Diffusion (`CanonicalUsage`)
//!
//! Architectural Design: Outbound diffusion is independently implemented by each protocol mapper adapter
//! (Gemini -> each protocol wire format). No unified outbound pipeline is maintained to preserve protocol
//! flexibility and streaming stability.

pub mod inbound;
pub mod policy;
pub mod usage;

pub use inbound::InboundThinkingPipeline;
pub use policy::{ProxyProtocol, UpstreamClassification};
pub use usage::CanonicalUsage;
