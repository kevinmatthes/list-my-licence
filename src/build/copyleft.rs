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

/// Surveys a dependency graph for obligations beyond reproduction.
///
/// # Examples
///
/// ```no_run
/// # use list_my_licence::build::{
/// #     Classifier, Copyleft, Discovery, Resolver,
/// # };
/// let mut survey = Copyleft::new().survey();
///
/// for package in Resolver::from_build_env()?.resolve()? {
///     let evidence = Discovery::new().search(&package);
///     let verdict = Classifier::new().classify(&package, &evidence);
///
///     survey.add(&package, &verdict);
/// }
///
/// for warning in survey.warnings() {
///     println!("cargo::warning={warning}");
/// }
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[derive(Clone, Copy, Debug, Default)]
pub struct Copyleft {
    _private: (),
}

impl Copyleft {
    /// A survey with the default settings.
    #[must_use]
    pub const fn new() -> Self {
        Self { _private: () }
    }

    /// An empty survey, to be filled package by package.
    #[must_use]
    pub fn survey(&self) -> crate::build::Survey {
        crate::build::Survey::default()
    }
}
