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

/// Everything that must be reproduced, ready to print.
///
/// Obtained from [`embed!`](crate::embed), never constructed by hand outside
/// tests.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Attribution {
    /// Every package whose licences ship, sorted by name.
    pub packages: &'static [crate::Package],
}

impl Attribution {
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

    /// Renders as Markdown.
    ///
    /// Licence texts go into fenced blocks, so that a text containing
    /// something Markdown would otherwise interpret survives intact.  That
    /// matters more than it sounds:  reproducing a licence *almost* verbatim
    /// is the one thing this crate exists to avoid.
    #[must_use]
    pub fn markdown(&self) -> String {
        crate::Markdown(self).to_string()
    }

    /// One package by name, if it is present.
    #[must_use]
    pub fn package(&self, name: &str) -> Option<&'static crate::Package> {
        let packages: &'static [crate::Package] = self.packages;

        packages.iter().find(|package| package.name == name)
    }

    /// The packages under a given licence.
    pub fn under(
        &self,
        identifier: &str,
    ) -> impl Iterator<Item = &crate::Package> {
        self.packages.iter().filter(move |package| {
            package
                .licences
                .iter()
                .any(|licence| licence.identifier == identifier)
        })
    }
}

/// Renders as plain text, for a terminal.
impl std::fmt::Display for Attribution {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for package in self.packages {
            writeln!(f, "{} {}", package.name, package.version)?;

            for licence in package.licences {
                writeln!(f, "  {} ({})", licence.identifier, licence.origin)?;
                writeln!(f)?;

                for line in licence.text.trim_end().lines() {
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

/******************************************************************************/
