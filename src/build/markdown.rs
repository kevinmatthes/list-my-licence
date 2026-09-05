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

/// The Markdown rendering of what will be reproduced.
#[derive(Clone, Copy, Debug)]
pub struct Markdown<'a>(pub &'a [crate::build::Reproduced<'a>]);

impl std::fmt::Display for Markdown<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("# Third party licences\n")?;

        for (package, verdict) in self.0 {
            write!(f, "\n## {} {}\n", package.name, package.version)?;

            for attribution in &verdict.attributions {
                write!(
                    f,
                    "\n### {} ({})\n\n```text\n{}\n```\n",
                    attribution.identifier(),
                    crate::build::Emitter::origin_text(
                        attribution.provenance()
                    ),
                    attribution.text().trim_end(),
                )?;
            }

            for notice in &verdict.notices {
                write!(
                    f,
                    "\n### NOTICE\n\n```text\n{}\n```\n",
                    notice.text.trim_end()
                )?;
            }
        }

        Ok(())
    }
}

/******************************************************************************/
