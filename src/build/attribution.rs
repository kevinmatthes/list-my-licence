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

/// One licence of a package, with the text to be reproduced for it.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct Attribution {
    identifier: String,
    text: String,

    /// Where it came from, recorded so that the output can say so.
    provenance: crate::build::Provenance,
}

impl Attribution {
    /// Retrieve the SPDX identifier this text stands for.
    #[must_use]
    pub fn identifier(&self) -> &str {
        &self.identifier
    }

    /// Create a new instance.
    #[must_use]
    pub const fn new(
        identifier: String,
        text: String,
        provenance: crate::build::Provenance,
    ) -> Self {
        Self {
            identifier,
            text,
            provenance,
        }
    }

    /// Retrieve the provenance.
    #[must_use]
    pub fn provenance(&self) -> crate::build::Provenance {
        self.provenance.clone()
    }

    /// Retrieve the text itself.
    #[must_use]
    pub fn text(&self) -> &str {
        &self.text
    }

    /// Change the SPDX identifier this text stands for.
    pub fn with_identifier(&mut self, identifier: &str) -> &mut Self {
        self.identifier = identifier.to_string();

        self
    }

    /// Change the provenance.
    pub fn with_provenance(
        &mut self,
        provenance: crate::build::Provenance,
    ) -> &mut Self {
        self.provenance = provenance;

        self
    }

    /// Change the text itself.
    pub fn with_text(&mut self, text: &str) -> &mut Self {
        self.text = text.to_string();

        self
    }
}

/******************************************************************************/
