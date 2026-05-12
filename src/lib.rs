//! # 7zippy — Pure-Rust 7z archive implementation
//!
//! Umbrella over the [zippy family of codec crates](https://github.com/JackDanger/7zippy#coders).
//! This crate parses the 7z container and dispatches each folder's coder pipeline to the
//! appropriate sibling crate.
//!
//! ## Status
//!
//! Scaffolding phase — see [`STATUS.md`](https://github.com/JackDanger/7zippy/blob/main/STATUS.md).

#![deny(unsafe_op_in_unsafe_fn)]

pub mod analyze;
pub mod cli;
pub mod container;
pub mod error;
pub mod pipeline;
mod read;
mod write;

pub use error::{SevenZippyError, SevenZippyResult};
pub use read::{Archive, ArchiveReader};
pub use write::ArchiveBuilder;

#[cfg(test)]
mod tests;
