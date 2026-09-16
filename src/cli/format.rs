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

/// Which shape licence reporting is rendered into.
///
/// `Text` is the default:  it is today's behaviour, the plain-text
/// [`Display`](std::fmt::Display), so choosing this enum a default does not
/// change what an existing caller sees.
#[derive(Clone, Copy, Debug, clap::ValueEnum, Eq, PartialEq)]
pub enum Format {
    /// A machine-readable DEP-5 `debian/copyright`.
    Dep5,

    /// Markdown, with licence texts in fenced blocks.
    Markdown,

    /// Plain text, for a terminal.
    Text,
}

impl Format {
    /// Renders the given attribution in this format.
    #[must_use]
    pub fn render(self, attribution: &crate::Attribution) -> String {
        match self {
            Self::Dep5 => attribution.dep5(),
            Self::Markdown => attribution.markdown(),
            Self::Text => attribution.to_string(),
        }
    }
}

/******************************************************************************/
