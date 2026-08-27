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

/// Why a candidate file was not taken.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum Skipped {
    /// The file could not be read at all.
    Unreadable(String),

    /// The file is not valid UTF-8, so its text cannot be reproduced
    /// faithfully.
    NotText,

    /// The file is larger than [`crate::build::MAX_BYTES`].
    TooLarge(u64),
}

impl std::fmt::Display for Skipped {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Unreadable(why) => write!(f, "could not be read:  {why}"),
            Self::NotText => f.write_str("is not valid UTF-8"),
            Self::TooLarge(size) => {
                write!(
                    f,
                    "is {size} bytes, larger than the {} byte limit",
                    crate::build::MAX_BYTES
                )
            }
        }
    }
}

/******************************************************************************/
