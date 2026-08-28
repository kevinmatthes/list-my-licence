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

/// One licence of one package.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Licence {
    /// The SPDX identifier, or the reference invented for a custom licence.
    pub identifier: &'static str,

    /// The text, reproduced exactly.
    pub text: &'static str,

    /// Where that text came from, so the reader can judge it.
    pub origin: crate::Origin,
}

/******************************************************************************/
