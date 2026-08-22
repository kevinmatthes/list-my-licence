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

/// One licence-bearing file, with its text.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct Found {
    /// Where the file is.
    pub path: std::path::PathBuf,

    /// What part it plays.
    pub role: crate::build::Role,

    /// The SPDX identifier its *name* points at, where the name names one.
    ///
    /// This is a hint drawn from the file name alone, never from the contents.
    /// `LICENSE-MIT` yields `MIT`;  a bare `LICENSE` yields nothing, because
    /// the name says nothing about which licence it holds.
    pub identifier: Option<String>,

    /// The file's text, reproduced exactly as distributed.
    pub text: String,
}
