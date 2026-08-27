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

/// The Markdown rendering of an [`crate::reproduction::Attribution`].
///
/// A separate type rather than a method body, so that the rendering is written
/// once against [`std::fmt::Write`] instead of assembling a `String` by hand.
#[derive(Clone, Copy, Debug)]
pub struct Markdown<'a>(pub &'a crate::Attribution);

impl std::fmt::Display for Markdown<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("# Third party licences\n")?;

        for package in self.0.packages {
            write!(f, "\n## {} {}\n", package.name, package.version)?;

            for licence in package.licences {
                write!(
                    f,
                    "\n### {} ({})\n\n```text\n{}\n```\n",
                    licence.identifier,
                    licence.origin,
                    licence.text.trim_end(),
                )?;
            }

            for notice in package.notices {
                write!(
                    f,
                    "\n### NOTICE\n\n```text\n{}\n```\n",
                    notice.trim_end()
                )?;
            }
        }

        Ok(())
    }
}

/******************************************************************************/
