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

/// One package of the resolved graph, reduced to what licence harvesting
/// needs.
///
/// The fields are deliberately owned rather than borrowed:  the
/// [`cargo_metadata::Metadata`] they come from is large, and holding it alive
/// for the whole of a build script would be wasteful.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct ResolvedPackage {
    /// The package name as Cargo knows it.
    pub name: String,

    /// The exact resolved version.
    pub version: String,

    /// Directory containing the package's `Cargo.toml`.
    ///
    /// This is where the search for `LICENSE`, `COPYING` and `NOTICE` files
    /// begins.  For registry dependencies it points into
    /// `~/.cargo/registry/src/`; for path and workspace members it points at
    /// the source tree.
    pub manifest_dir: std::path::PathBuf,

    /// The SPDX expression the package *declares*, verbatim from its
    /// `license` field.
    ///
    /// This is a claim, not a fact:  it is regularly absent, occasionally
    /// wrong, and may disagree with the licence files actually shipped.  It is
    /// deliberately kept as written rather than normalised here.
    pub licence: Option<String>,

    /// The package's `license-file` field, resolved against
    /// [`Self::manifest_dir`], for packages that point at a file instead of
    /// naming an expression.
    pub licence_file: Option<std::path::PathBuf>,

    /// The declared authors, used to recover a copyright line where no
    /// licence file carries one.
    pub authors: Vec<String>,

    /// The declared repository, which discharges the MPL-2.0 source-pointer
    /// obligation and is a useful ingredient elsewhere.
    pub repository: Option<String>,
}

/******************************************************************************/
