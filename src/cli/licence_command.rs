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

/// Licence reporting as a subcommand, for applications that prefer one.
///
/// Flattened into an application's own subcommand enumeration, so that
/// `myapp licences` works without the application writing the plumbing.
///
/// # Examples
///
/// ```no_run
/// # use clap::{Parser, Subcommand};
/// #[derive(Parser)]
/// struct Arguments {
///     #[command(subcommand)]
///     command: Command,
/// }
///
/// #[derive(Subcommand)]
/// enum Command {
///     Run,
///
///     #[command(flatten)]
///     Licence(list_my_licence::cli::LicenceCommand),
/// }
///
/// // In an application this comes from `list_my_licence::embed!()`; it is
/// // written out here so that the example compiles on its own.
/// static LICENCES: list_my_licence::Attribution =
///     list_my_licence::Attribution { packages: &[] };
///
/// match Arguments::parse().command {
///     Command::Run => todo!(),
///     Command::Licence(licence) => print!("{}", licence.render(&LICENCES)),
/// }
/// ```
#[derive(Clone, Debug, clap::Subcommand)]
pub enum LicenceCommand {
    /// Show the licences of this application and its dependencies.
    Licences {
        /// Show only this crate's licences.
        #[arg(value_name = "CRATE")]
        wanted: Option<String>,
    },
}

impl LicenceCommand {
    /// What this subcommand asks to be shown.
    ///
    /// An unknown crate name yields an empty rendering rather than an error:
    /// a dependency that is not there has no licences to report, which is an
    /// answer rather than a failure.
    #[must_use]
    pub fn render(&self, attribution: &crate::Attribution) -> String {
        let Self::Licences { wanted } = self;

        wanted.as_deref().map_or_else(
            || attribution.to_string(),
            |name| {
                attribution
                    .package(name)
                    .map_or_else(String::new, |package| {
                        crate::Attribution {
                            packages: std::slice::from_ref(package),
                        }
                        .to_string()
                    })
            },
        )
    }
}

/******************************************************************************/
