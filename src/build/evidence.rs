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

/// What discovery found for one package.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Evidence {
    /// The files taken, sorted by path so that repeated runs agree.
    pub found: Vec<crate::build::Found>,

    /// Candidates that looked right but could not be taken, with the reason.
    ///
    /// Never empty without meaning:  an entry here is a licence this crate can
    /// see but not reproduce, which the caller must decide what to do about.
    pub skipped: Vec<(std::path::PathBuf, crate::build::Skipped)>,
}

impl Evidence {
    /// Whether anything at all was found.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.found.is_empty()
    }

    /// The files that are licence texts rather than notices.
    pub fn licences(&self) -> impl Iterator<Item = &crate::build::Found> {
        self.found
            .iter()
            .filter(|file| file.role == crate::build::Role::Licence)
    }

    /// The Apache-2.0 `NOTICE` files, if any.
    pub fn notices(&self) -> impl Iterator<Item = &crate::build::Found> {
        self.found
            .iter()
            .filter(|file| file.role == crate::build::Role::Notice)
    }
}
