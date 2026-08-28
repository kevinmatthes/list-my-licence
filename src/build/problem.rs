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

/// Something standing between a package and a discharged obligation.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum Problem {
    /// The manifest declares no licence, and the package ships no text
    /// either.
    ///
    /// The most dangerous case of all, because it fails silently:  the
    /// obligation cannot even be identified, so nothing is reproduced and
    /// there is nothing to object to.
    Undeclared,

    /// The licence is a custom one:  either declared as an SPDX `LicenseRef`,
    /// or not declared at all while a text is nonetheless shipped.
    ///
    /// Not a fault.  A copyright holder is entitled to write their own terms,
    /// and this crate's business is reproducing them faithfully, not insisting
    /// they be drawn from a list.  It is recorded so that the output can say
    /// the licence was not identifiable, since no canonical text exists to
    /// check it against.
    Custom {
        /// What the licence is called, as declared or as invented for it.
        identifier: String,
    },

    /// The declared expression could not be parsed, even leniently.
    Unparsable {
        /// The expression as written.
        expression: String,
    },

    /// A licence needing its own copyright line has no distributed text, so
    /// the canonical text cannot stand in for it.
    Unattributed {
        /// The licence that cannot be discharged.
        identifier: String,
    },

    /// No combination of what is available satisfies the declared expression.
    Unsatisfiable {
        /// The expression as written.
        expression: String,
    },

    /// A file was found but could not be read.
    Unreadable {
        /// Where it is.
        path: std::path::PathBuf,

        /// Why it was refused.
        reason: crate::build::Skipped,
    },
}

impl Problem {
    /// Whether this must fail the build.
    ///
    /// Only two things do.  An unattributed notice-style licence cannot be
    /// discharged at all, and an unsatisfiable expression means nothing
    /// available covers what was declared.  Everything else is reported and
    /// survivable:  a canonical text stood in, or the package said nothing and
    /// the human must look.
    #[must_use]
    pub const fn is_fatal(&self) -> bool {
        matches!(
            self,
            Self::Undeclared
                | Self::Unparsable { .. }
                | Self::Unattributed { .. }
                | Self::Unsatisfiable { .. }
        )
    }
}

impl std::fmt::Display for Problem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Undeclared => {
                f.write_str("declares no licence and ships no licence text")
            }
            Self::Custom { identifier } => {
                write!(f, "is under the custom licence {identifier}")
            }
            Self::Unparsable { expression } => {
                write!(
                    f,
                    "declares {expression:?}, which is not a licence expression"
                )
            }
            Self::Unattributed { identifier } => write!(
                f,
                "ships no {identifier} text, and {identifier} requires its own \
                 copyright line, which the canonical text cannot supply"
            ),
            Self::Unsatisfiable { expression } => {
                write!(
                    f,
                    "declares {expression}, which nothing available satisfies"
                )
            }
            Self::Unreadable { path, reason } => {
                write!(f, "{} {reason}", path.display())
            }
        }
    }
}

/******************************************************************************/
