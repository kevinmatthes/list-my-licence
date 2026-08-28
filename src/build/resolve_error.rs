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

/// Anything that can go wrong while resolving the graph.
#[derive(Debug)]
pub enum ResolveError {
    /// `cargo metadata` could not be run or returned malformed output.
    Metadata(cargo_metadata::Error),

    /// `cargo metadata` returned no resolved graph.  This happens when it is
    /// invoked with `--no-deps`, which this crate never does, so it indicates
    /// a Cargo the crate does not understand.
    NoResolve,

    /// The resolved graph named no root package.  Virtual manifests — a
    /// workspace with no package of its own — have no single root, and the
    /// crate cannot guess which member is being built.
    NoRootPackage,
}

impl From<cargo_metadata::Error> for ResolveError {
    fn from(e: cargo_metadata::Error) -> Self {
        Self::Metadata(e)
    }
}

impl std::error::Error for ResolveError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Metadata(e) => Some(e),
            Self::NoResolve | Self::NoRootPackage => None,
        }
    }
}

impl std::fmt::Display for ResolveError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Metadata(e) => {
                write!(f, "could not read cargo metadata:  {e}")
            }
            Self::NoResolve => f.write_str(
                "cargo metadata returned no resolved dependency graph",
            ),
            Self::NoRootPackage => f.write_str(
                "cargo metadata named no root package; \
                 virtual workspace manifests are not supported",
            ),
        }
    }
}

/******************************************************************************/
