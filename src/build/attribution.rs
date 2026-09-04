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

/// One licence of a package, with the text to be reproduced for it.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct Attribution {
    /// The SPDX identifier this text stands for.
    pub identifier: String,

    /// The text itself.
    pub text: String,

    /// Where it came from, recorded so that the output can say so.
    pub provenance: crate::build::Provenance,
}

impl Attribution {
    /// Create a new instance.
    pub fn new(
        identifier: String,
        text: String,
        provenance: crate::build::Provenance,
    ) -> Attribution {
        Attribution {
            identifier,
            text,
            provenance,
        }
    }
}

/******************************************************************************/
