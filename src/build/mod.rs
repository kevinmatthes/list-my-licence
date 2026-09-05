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

//! Builder-time harvesting.
//!
//! Everything in this module runs from a `build.rs`, never from the
//! shipped binary.  It is gated behind the `build` feature so that the
//! runtime half of the crate keeps its empty dependency list.

mod attribution;
mod builder;
mod classification;
mod classifier;
mod copyleft;
mod coverage;
mod dep5;
mod discovery;
mod emit_error;
mod emitter;
mod error;
mod evidence;
mod finding;
mod found;
mod generated;
#[cfg(feature = "compression")]
mod generated_compressed;
#[cfg(feature = "compression")]
mod interned;
mod markdown;
mod outcome;
mod problem;
mod provenance;
mod reproduced;
mod resolve_error;
mod resolved_package;
mod resolver;
mod role;
mod skipped;
mod strength;
mod survey;

pub use crate::build::{
    attribution::Attribution, builder::Builder, classification::Classification,
    classifier::Classifier, copyleft::Copyleft, coverage::Coverage,
    discovery::Discovery, discovery::MAX_BYTES, emit_error::EmitError,
    emitter::Emitter, error::Error, evidence::Evidence, finding::Finding,
    found::Found, outcome::Outcome, problem::Problem, provenance::Provenance,
    reproduced::Reproduced, resolve_error::ResolveError,
    resolved_package::ResolvedPackage, resolver::Resolver, role::Role,
    skipped::Skipped, strength::Strength, survey::Survey,
};

pub(crate) use crate::build::{
    dep5::Dep5, generated::Generated, markdown::Markdown,
};

#[cfg(feature = "compression")]
pub(crate) use crate::build::{
    generated_compressed::GeneratedCompressed, interned::Interned,
};

/******************************************************************************/
