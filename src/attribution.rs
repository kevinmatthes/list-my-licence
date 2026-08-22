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

//! The embedded attribution, and how to render it.
//!
//! Everything here is compiled into the shipped binary, so it carries **no
//! dependencies whatsoever** — not even for parsing.  The build half writes
//! Rust source, which the compiler then checks;  there is no format to get
//! wrong at runtime and no failure mode for reading it back.
//!
//! The types are deliberately plain data.  A renderer is a function over them,
//! which is what lets another output format be added later without disturbing
//! anything that already works.

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

/// One licence of one package.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Licence {
    /// The SPDX identifier, or the reference invented for a custom licence.
    pub identifier: &'static str,

    /// The text, reproduced exactly.
    pub text: &'static str,

    /// Where that text came from, so the reader can judge it.
    pub origin: Origin,
}

/// One package of the dependency graph.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Package {
    /// The package name.
    pub name: &'static str,

    /// The exact version that shipped.
    pub version: &'static str,

    /// Its licences, one per discharged term.
    pub licences: &'static [Licence],

    /// Its Apache-2.0 notices, reproduced alongside rather than instead.
    pub notices: &'static [&'static str],
}

/// Everything that must be reproduced, ready to print.
///
/// Obtained from [`embed!`](crate::embed), never constructed by hand outside
/// tests.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Attribution {
    /// Every package whose licences ship, sorted by name.
    pub packages: &'static [Package],
}

impl Attribution {
    /// How many packages are covered.
    #[must_use]
    pub const fn len(&self) -> usize {
        self.packages.len()
    }

    /// Whether anything is covered at all.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.packages.is_empty()
    }

    /// The packages under a given licence.
    pub fn under(&self, identifier: &str) -> impl Iterator<Item = &Package> {
        self.packages.iter().filter(move |package| {
            package
                .licences
                .iter()
                .any(|licence| licence.identifier == identifier)
        })
    }

    /// One package by name, if it is present.
    #[must_use]
    pub fn package(&self, name: &str) -> Option<&Package> {
        self.packages.iter().find(|package| package.name == name)
    }

    /// Renders as Markdown.
    ///
    /// Licence texts go into fenced blocks, so that a text containing
    /// something Markdown would otherwise interpret survives intact.  That
    /// matters more than it sounds:  reproducing a licence *almost* verbatim
    /// is the one thing this crate exists to avoid.
    #[must_use]
    pub fn markdown(&self) -> String {
        Markdown(self).to_string()
    }
}

/// The Markdown rendering of an [`Attribution`].
///
/// A separate type rather than a method body, so that the rendering is written
/// once against [`fmt::Write`] instead of assembling a `String` by hand.
#[derive(Clone, Copy, Debug)]
pub struct Markdown<'a>(pub &'a Attribution);

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

/// Pulls in the attribution the build script wrote.
///
/// # Examples
///
/// ```ignore
/// static LICENCES: list_my_licence::Attribution = list_my_licence::embed!();
///
/// fn main() {
///     print!("{LICENCES}");
/// }
/// ```
#[macro_export]
macro_rules! embed {
    () => {{
        use $crate::{Attribution, Licence, Origin, Package};

        include!(concat!(env!("OUT_DIR"), "/list-my-licence.rs"))
    }};
}

/******************************************************************************/
