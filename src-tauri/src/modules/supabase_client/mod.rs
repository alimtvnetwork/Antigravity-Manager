//! Supabase PostgREST Client Module
//! Provides lightweight REST communication with Supabase PostgreSQL endpoints.
#![allow(dead_code)]

pub mod client;
pub mod tests;
pub mod types;

pub use client::*;
pub use tests::*;
pub use types::*;
