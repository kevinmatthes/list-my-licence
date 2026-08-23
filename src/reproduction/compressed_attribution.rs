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

/// Everything that must be reproduced, held compressed.
///
/// The counterpart of [`Attribution`](crate::Attribution) for
/// [`embed_compressed!`](crate::embed_compressed).  Obtained from that macro,
/// never constructed by hand outside tests.
///
/// There is deliberately no `markdown` here.  Rendering Markdown means
/// inflating every text anyway, at which point the plain
/// [`Attribution`](crate::Attribution) is the better shape;  this type exists
/// to keep a binary small, not to render from.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CompressedAttribution {
    /// Every package whose licences ship, sorted by name.
    pub packages: &'static [crate::CompressedPackage],
}

impl CompressedAttribution {
    /// Whether anything is covered at all.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.packages.is_empty()
    }

    /// How many packages are covered.
    #[must_use]
    pub const fn len(&self) -> usize {
        self.packages.len()
    }

    /// One package by name, if it is present.
    #[must_use]
    pub fn package(
        &self,
        name: &str,
    ) -> Option<&'static crate::CompressedPackage> {
        let packages: &'static [crate::CompressedPackage] = self.packages;

        packages.iter().find(|package| package.name == name)
    }

    /// The packages under a given licence.
    pub fn under(
        &self,
        identifier: &str,
    ) -> impl Iterator<Item = &crate::CompressedPackage> {
        self.packages.iter().filter(move |package| {
            package
                .licences
                .iter()
                .any(|licence| licence.identifier == identifier)
        })
    }
}

/// Renders as plain text, for a terminal.
///
/// Every text is inflated as it is written, so this costs what the
/// compression saved.  That is the intended trade:  a binary which never
/// prints its licences never pays it.
impl std::fmt::Display for CompressedAttribution {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for package in self.packages {
            writeln!(f, "{} {}", package.name, package.version)?;

            for licence in package.licences {
                writeln!(f, "  {} ({})", licence.identifier, licence.origin)?;
                writeln!(f)?;

                for line in licence.text().trim_end().lines() {
                    writeln!(f, "    {line}")?;
                }

                writeln!(f)?;
            }

            for notice in package.notices {
                writeln!(f, "  NOTICE")?;
                writeln!(f)?;

                for line in notice.trim_end().lines() {
                    writeln!(f, "    {line}")?;
                }

                writeln!(f)?;
            }
        }

        Ok(())
    }
}
