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

/// What one pass produced.
#[derive(Clone, Debug)]
pub struct Outcome {
    /// Every package examined, with its verdict.
    pub packages:
        Vec<(crate::build::ResolvedPackage, crate::build::Classification)>,

    /// The copyleft obligations found, if any.
    pub survey: crate::build::Survey,
}

impl Outcome {
    /// The lines a build script should emit as `cargo::warning=`.
    ///
    /// Everything worth a human's attention that did not stop the build:  the
    /// copyleft obligations text cannot discharge, and the survivable
    /// complaints about individual packages.
    #[must_use]
    pub fn warnings(&self) -> Vec<String> {
        let mut warnings: Vec<String> = self.survey.warnings().collect();

        for (package, verdict) in &self.packages {
            warnings.extend(
                verdict
                    .problems
                    .iter()
                    .map(|problem| format!("{} {problem}", package.name)),
            );
        }

        warnings
    }
}

/******************************************************************************/
