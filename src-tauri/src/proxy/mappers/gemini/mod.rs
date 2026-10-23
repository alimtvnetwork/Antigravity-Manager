// Gemini mapper module
// Responsible for v1internal packaging/unpacking

pub mod collector;
pub mod models;
pub mod wrapper; // [NEW]

// No public exports needed here if unused
pub use wrapper::*;
