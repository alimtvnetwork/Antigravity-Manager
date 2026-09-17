//! Full-duplex unified model thinking and response streaming pipeline architecture
//!
//! 1. Canonical Intermediate Representation (Canonical IR): Google Gemini standard format (`contents` + `generationConfig`)
//! 2. Protocol Inbound Strategy Adapters (`InboundThinkingPipeline`, `ProxyProtocol`)
//! 3. Unified Usage and Cache Computation & Protocol Diffusion (`CanonicalUsage`)
//! 4. Unified Outbound Extraction, Reverse Ingestion & Protocol Diffusion for Streaming/Non-Streaming (`OutboundThinkingPipeline`, `CanonicalStreamEvent`)

pub mod canonical;
pub mod events;
pub mod inbound;
pub mod outbound;
pub mod policy;
pub mod usage;

pub use events::CanonicalStreamEvent;
pub use inbound::InboundThinkingPipeline;
pub use outbound::{CanonicalEgressPayload, OutboundThinkingPipeline};
pub use policy::ProxyProtocol;
pub use usage::CanonicalUsage;
