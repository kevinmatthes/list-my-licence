//! Embed the verbatim licences of a crate and its dependencies into the
//! binary that ships them.
//!
//! Many open source licences oblige a distributed application to reproduce
//! their text word for word.  Doing so by hand does not scale past a handful
//! of dependencies and silently rots as the dependency graph moves.  This
//! crate harvests the licences at build time and embeds them, so the
//! obligation is discharged by the build rather than by memory.
//!
//! # Shape
//!
//! The crate has two halves, separated by the `build` feature:
//!
//! * The **runtime half**, compiled by default, has *no dependencies at all*.
//!   It holds the embedded data and renders it.
//! * The **build half**, behind `features = ["build"]`, does the harvesting.
//!   It is meant to be used from a `build.rs`, never from the shipped binary.
//!
//! A consuming crate therefore names this crate twice:
//!
//! ```toml
//! [build-dependencies]
//! list-my-licence = { version = "0.1", features = ["build"] }
//!
//! [dependencies]
//! list-my-licence = "0.1"
//! ```
//!
//! This keeps the SPDX tables, the manifest parser and the canonical licence
//! texts out of the shipped binary entirely:  only the harvested result
//! crosses over.

#[cfg(feature = "build")]
pub mod build;
