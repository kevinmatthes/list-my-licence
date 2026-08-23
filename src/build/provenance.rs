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

/// Where a reproduced text came from.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum Provenance {
    /// The copy the author actually distributed, which is what a licence
    /// requiring its own copyright line needs.
    Distributed(std::path::PathBuf),

    /// One file covering several licence terms at once.
    Combined(std::path::PathBuf),

    /// The canonical SPDX text, used because the package shipped none.
    Canonical,
}
