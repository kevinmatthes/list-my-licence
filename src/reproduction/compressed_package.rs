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

/// One package of the dependency graph, with its licences held compressed.
///
/// The counterpart of [`Package`](crate::Package) for
/// [`embed_compressed!`](crate::embed_compressed).
///
/// **Notices are not compressed.**  An Apache-2.0 `NOTICE` is a few lines
/// where a licence is tens of kilobytes, so compressing them would double
/// this type's surface to save almost nothing.  They stay plain, and stay
/// directly readable.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CompressedPackage {
    /// The package name.
    pub name: &'static str,

    /// The exact version that shipped.
    pub version: &'static str,

    /// Its licences, one per discharged term.
    pub licences: &'static [crate::CompressedLicence],

    /// Its Apache-2.0 notices, reproduced alongside rather than instead.
    pub notices: &'static [&'static str],
}

/******************************************************************************/
