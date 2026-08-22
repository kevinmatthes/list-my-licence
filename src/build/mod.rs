//! Build-time harvesting.
//!
//! Everything in this module runs from a `build.rs`, never from the shipped
//! binary.  It is gated behind the `build` feature so that the runtime half of
//! the crate keeps its empty dependency list.

mod graph;

pub use graph::{Error, ResolvedPackage, Resolver};
