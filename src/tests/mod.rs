//! Integration tests. Layer 0–7 mirror the plan's test architecture.
//!
//! # Layer structure
//!
//! | Layer | File | What it tests |
//! |---|---|---|
//! | 0 | `layer0_generators` | Sanity checks for the `fixtures` module |
//! | 1 | `layer1_container` | Container parsing with the public `Archive::parse` API |
//! | 2 | `layer2_dispatch` | `coder_for(method_id)` returns the right variant |
//! | 3 | `layer3_per_coder` | Sibling crate test suites pass (smoke) |
//! | 4 | `layer4_pipeline` | Full encode → container → decode round-trips |
//! | 5 | `layer5_cross` | 8z reads what 7zz writes; 7zz reads what 8z writes |
//! | 6 | `layer6_threads` | Concurrent decode/encode (placeholder) |
//! | 7 | `layer7_perf` | Wall-time bounds (informational, always ignored) |
//!
//! # Test infrastructure
//!
//! - [`fixtures`] — deterministic data generators (no external deps).
//! - [`oracle`] — hermetic `7zz` CLI wrapper; use [`require_7zz!`] macro to
//!   skip tests on machines without `7zz` installed.
//! - [`utils`] — [`assert_slices_eq!`] macro with hex-context diagnostics.

pub mod fixtures;
pub mod oracle;
pub mod utils;

mod layer0_generators;
mod layer1_container;
mod layer2_dispatch;
mod layer3_per_coder;
mod layer4_pipeline;
mod layer5_cross;
mod layer6_threads;
mod layer7_perf;
