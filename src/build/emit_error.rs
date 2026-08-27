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

/// Anything that can go wrong while emitting.
#[derive(Debug)]
pub enum EmitError {
    /// A file could not be written.
    Write {
        /// Where it should have gone.
        path: std::path::PathBuf,

        /// Why it did not.
        reason: std::io::Error,
    },

    /// The committed attribution is out of date.
    ///
    /// Raised only by [`crate::build::Emitter::check`].  Regenerating it is
    /// the fix;  the failure exists so that a stale file cannot be merged
    /// unnoticed.
    Stale {
        /// The file that no longer matches what the graph would produce.
        path: std::path::PathBuf,
    },
}

impl std::error::Error for EmitError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Write { reason, .. } => Some(reason),
            Self::Stale { .. } => None,
        }
    }
}

impl std::fmt::Display for EmitError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Write { path, reason } => {
                write!(f, "could not write {}:  {reason}", path.display())
            }
            Self::Stale { path } => write!(
                f,
                "{} is out of date;  the dependency graph has changed since it \
                 was written",
                path.display()
            ),
        }
    }
}

/******************************************************************************/
