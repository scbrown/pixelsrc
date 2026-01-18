//! Demo tests (Phase 23)
//!
//! This is the entry point for demo tests. It includes the harness module
//! and runs tests that showcase user-visible capabilities.

mod demos;

// Re-export the harness for use by demo test submodules
pub use demos::*;
