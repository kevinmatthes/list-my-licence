/*********************** GNU General Public License 3.0 ***********************\
|                                                                              |
|  Copyright (C) 2026 Kevin Matthes                                            |
|                                                                              |
|  This program is free software: you can redistribute it and/or modify        |
|  it under the terms of the GNU General Public License as published by        |
|  the Free Software Foundation, either version 3 of the License, or           |
|  (at your option) any later version.                                         |
|                                                                              |
|  This program is distributed in the hope that it will be useful,             |
|  but WITHOUT ANY WARRANTY; without even the implied warranty of              |
|  MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the               |
|  GNU General Public License for more details.                                |
|                                                                              |
|  You should have received a copy of the GNU General Public License           |
|  along with this program.  If not, see <https://www.gnu.org/licenses/>.      |
|                                                                              |
\******************************************************************************/

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

#![deny(
    clippy::all,
    clippy::cargo,
    clippy::complexity,
    clippy::correctness,
    clippy::nursery,
    clippy::pedantic,
    clippy::perf,
    clippy::suspicious,
    clippy::style,
    dead_code,
    deprecated,
    missing_docs,
    rustdoc::broken_intra_doc_links,
    unreachable_code,
    unused_assignments,
    unused_imports,
    unused_macros,
    unused_must_use,
    unused_mut,
    unused_parens,
    unused_variables
)]
#![allow(clippy::multiple_crate_versions)]

#[cfg(feature = "build")]
pub mod build;

/******************************************************************************/
