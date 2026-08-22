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

/// Licences whose copyleft reaches only the files they cover.
const WEAK: [&str; 8] = [
    "CDDL-1.0", "CDDL-1.1", "CPL-1.0", "EPL-1.0", "EPL-2.0", "MPL-1.1",
    "MPL-2.0", "MS-PL",
];

/// How far a licence's obligations reach beyond reproducing its text.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum Strength {
    /// Nothing is required beyond reproduction, which this crate does.
    Permissive,

    /// Copyleft confined to the covered files.  Discharged by telling
    /// recipients where that source is, which is a pointer this crate can
    /// supply.
    Weak,

    /// Copyleft over a library, conditioned on the recipient being able to
    /// relink against a modified version of it.
    ///
    /// Rust links statically by default, which removes the shared-library
    /// route the LGPL offers and leaves only the other one:  publishing object
    /// files, or the application's own source.  Nothing embedded in a binary
    /// can satisfy that.
    Library,

    /// Copyleft over the whole work.  The complete corresponding source must
    /// be offered to whoever receives the binary.
    Strong,

    /// Copyleft reaching users who never receive a binary at all, but interact
    /// with the software over a network.
    ///
    /// Decisive for a server application, and invisible to any tool that
    /// reasons only about shipped artefacts.
    Network,
}

impl Strength {
    /// Whether this crate can discharge the obligation on its own.
    #[must_use]
    pub const fn is_dischargeable(self) -> bool {
        matches!(self, Self::Permissive | Self::Weak)
    }

    /// What the licence asks for beyond reproducing its text.
    #[must_use]
    pub const fn obligation(self) -> &'static str {
        match self {
            Self::Permissive => "nothing beyond the reproduction already done",
            Self::Weak => {
                "recipients must be told where the source of the covered files \
                 is;  the pointer below does that"
            }
            Self::Library => {
                "recipients must be able to relink against a modified library, \
                 which static linking makes impossible to satisfy with text \
                 alone"
            }
            Self::Strong => {
                "the complete corresponding source of the whole work must be \
                 offered to whoever receives the binary"
            }
            Self::Network => {
                "users interacting over a network must be offered the \
                 corresponding source, even if no binary is distributed"
            }
        }
    }

    /// Classifies one SPDX identifier.
    ///
    /// The GPL family is matched by prefix rather than by an exhaustive list,
    /// because SPDX carries `-only` and `-or-later` variants of each version
    /// alongside the deprecated bare forms, and real manifests use all of
    /// them.  Order matters:  `AGPL` and `LGPL` both end in `GPL`, so the
    /// narrower prefixes are tested first.
    #[must_use]
    pub fn of(identifier: &str) -> Self {
        if identifier.starts_with("AGPL-") {
            Self::Network
        } else if identifier.starts_with("LGPL-") {
            Self::Library
        } else if identifier.starts_with("GPL-") {
            Self::Strong
        } else if WEAK.contains(&identifier) {
            Self::Weak
        } else {
            Self::Permissive
        }
    }
}

impl std::fmt::Display for Strength {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::Permissive => "permissive",
            Self::Weak => "weak copyleft",
            Self::Library => "library copyleft",
            Self::Strong => "strong copyleft",
            Self::Network => "network copyleft",
        })
    }
}
