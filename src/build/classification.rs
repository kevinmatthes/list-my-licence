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

/// What one package's licences amount to.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Classification {
    /// How completely the shipped files cover the declaration.
    pub coverage: crate::build::Coverage,

    /// The texts to reproduce, one per discharged licence term.
    pub attributions: Vec<crate::build::Attribution>,

    /// Apache-2.0 notices, which are reproduced alongside rather than instead.
    pub notices: Vec<crate::build::Found>,

    /// Everything that stood in the way, fatal or not.
    pub problems: Vec<crate::build::Problem>,
}

impl Classification {
    /// Whether anything here must fail the build.
    #[must_use]
    pub fn is_fatal(&self) -> bool {
        self.problems.iter().any(crate::build::Problem::is_fatal)
    }
}
