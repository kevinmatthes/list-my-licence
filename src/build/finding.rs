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

/// One dependency carrying an obligation beyond reproduction.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct Finding {
    /// The package that declares it.
    pub package: String,

    /// Its exact version, which the source pointer needs.
    pub version: String,

    /// The licence relied upon.
    pub identifier: String,

    /// How far its obligations reach.
    pub strength: crate::build::Strength,

    /// Where the covered source can be obtained, when the manifest says.
    ///
    /// This discharges MPL-2.0 §3.2 by itself.  For the stronger licences it
    /// is an ingredient rather than compliance:  knowing where upstream lives
    /// is not the same as offering the corresponding source of *your* work.
    pub source: Option<String>,
}

impl std::fmt::Display for Finding {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} {} declares {} ({}):  {}",
            self.package,
            self.version,
            self.identifier,
            self.strength,
            self.strength.obligation(),
        )?;

        if let Some(source) = &self.source {
            write!(f, "  Source:  {source}")?;
        }

        Ok(())
    }
}
