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

/// Licence reporting, ready to flatten into an existing argument parser.
///
/// Deliberately additive.  It contributes options to whatever parser an
/// application already has, rather than replacing the call that parses them —
/// which is what lets it compose with a derived `Parser` instead of fighting
/// it.
///
/// # Examples
///
/// ```no_run
/// # use clap::Parser;
/// #[derive(Parser)]
/// struct Arguments {
///     #[command(flatten)]
///     licences: list_my_licence::cli::LicenceArgs,
/// }
///
/// // In an application this comes from `list_my_licence::embed!()`;  it is
/// // written out here so that the example compiles on its own.
/// static LICENCES: list_my_licence::Attribution =
///     list_my_licence::Attribution { packages: &[] };
///
/// let arguments = Arguments::parse();
///
/// arguments.licences.handle_and_exit(&LICENCES);
/// ```
#[derive(Clone, Debug, clap::Args)]
pub struct LicenceArgs {
    /// Show the licences of this application and its dependencies.
    ///
    /// Given a crate name, only that crate's licences are shown.
    #[arg(
        default_missing_value = "",
        long = "licences",
        num_args = 0..=1,
        value_name = "CRATE"
    )]
    licences: Option<String>,
}

impl LicenceArgs {
    /// Prints the requested licences and leaves, if any were requested.
    ///
    /// Returns normally when the option was not given, so that a caller can
    /// simply place this before its own work.
    ///
    /// # Panics
    ///
    /// Never panics.  It does not return when the option was given:  the
    /// process exits with a success status, since asking for a licence and
    /// receiving it is not an error.
    pub fn handle_and_exit(&self, attribution: &crate::Attribution) {
        if let Some(rendered) = self.render(attribution) {
            print!("{rendered}");
            std::process::exit(0);
        }
    }

    /// What the given options ask to be shown, if anything.
    ///
    /// An unknown crate name yields an empty rendering rather than an error:
    /// a dependency that is not there has no licences to report, which is an
    /// answer rather than a failure.
    #[must_use]
    pub fn render(&self, attribution: &crate::Attribution) -> Option<String> {
        let wanted = self.licences.as_deref()?;

        if wanted.is_empty() {
            return Some(attribution.to_string());
        }

        Some(
            attribution
                .package(wanted)
                .map_or_else(String::new, |package| {
                    crate::Attribution {
                        packages: std::slice::from_ref(package),
                    }
                    .to_string()
                }),
        )
    }

    /// Whether licences were asked for at all.
    #[must_use]
    pub const fn requested(&self) -> bool {
        self.licences.is_some()
    }
}

/******************************************************************************/
