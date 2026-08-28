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

/// Where a reproduced licence text came from.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Origin {
    /// The copy the author distributed, named by the file it was read from.
    Distributed(&'static str),

    /// One file the author distributed, covering several licences at once.
    Combined(&'static str),

    /// The canonical SPDX text, because the package shipped none of its own.
    Canonical,
}

impl std::fmt::Display for Origin {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Distributed(file) => write!(f, "as distributed, in {file}"),
            Self::Combined(file) => {
                write!(f, "as distributed, shared in {file}")
            }
            Self::Canonical => f.write_str("canonical SPDX text"),
        }
    }
}

/******************************************************************************/
