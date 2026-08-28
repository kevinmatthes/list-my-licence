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

/// Anything that stops a build script.
#[derive(Debug)]
pub enum Error {
    /// The dependency graph could not be resolved.
    Resolve(crate::build::ResolveError),

    /// The attribution could not be written, or the committed copy is stale.
    Emit(crate::build::EmitError),

    /// One or more packages carry an obligation that cannot be discharged.
    ///
    /// The build stops here rather than shipping an attribution known to be
    /// incomplete.  Each package is named, with what is wrong.
    Undischargeable(Vec<String>),
}

impl From<crate::build::EmitError> for Error {
    fn from(error: crate::build::EmitError) -> Self {
        Self::Emit(error)
    }
}

impl From<crate::build::ResolveError> for Error {
    fn from(error: crate::build::ResolveError) -> Self {
        Self::Resolve(error)
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Resolve(error) => Some(error),
            Self::Emit(error) => Some(error),
            Self::Undischargeable(_) => None,
        }
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Resolve(error) => write!(f, "{error}"),
            Self::Emit(error) => write!(f, "{error}"),
            Self::Undischargeable(packages) => {
                writeln!(
                    f,
                    "{} package(s) carry a licence obligation this build \
                     cannot discharge:",
                    packages.len()
                )?;

                for package in packages {
                    writeln!(f, "  {package}")?;
                }

                Ok(())
            }
        }
    }
}

/******************************************************************************/
