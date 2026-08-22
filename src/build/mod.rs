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

//! Build-time harvesting.
//!
//! Everything in this module runs from a `build.rs`, never from the shipped
//! binary.  It is gated behind the `build` feature so that the runtime half of
//! the crate keeps its empty dependency list.

mod discovery;
mod graph;

pub use discovery::{Discovery, Evidence, Found, MAX_BYTES, Role, Skipped};
pub use graph::{Error, ResolvedPackage, Resolver};

/******************************************************************************/
