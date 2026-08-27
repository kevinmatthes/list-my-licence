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

/// The generated Rust source.
///
/// Paths in `include_str!` are **relative** to the generated file, so that no
/// directory belonging to whoever built the binary ends up inside it.  That
/// keeps the artefact reproducible and free of anybody's home directory.
///
/// The type names are written bare rather than as `$crate::Attribution`.
/// `$crate` is substituted while a macro's *body* is expanded, and these
/// tokens come from a file that `include!` reads at that point, so the
/// substitution never reaches them.  [`embed!`](crate::embed) brings the names
/// into scope instead, which also survives a consumer renaming the
/// dependency.
#[derive(Clone, Copy, Debug)]
pub struct Generated<'a> {
    pub packages: &'a [crate::build::Reproduced<'a>],
    pub files: &'a std::collections::BTreeMap<String, String>,
}

impl std::fmt::Display for Generated<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Attribution { packages: &[\n")?;

        for (package, verdict) in self.packages {
            write!(
                f,
                "    Package {{\n        name: {:?},\n        version: \
                 {:?},\n        licences: &[\n",
                package.name, package.version,
            )?;

            for attribution in &verdict.attributions {
                let file = self
                    .files
                    .get(&attribution.text)
                    .map_or("", String::as_str);

                write!(
                    f,
                    "            Licence {{\n                identifier: \
                     {:?},\n                text: \
                     include_str!({file:?}),\n                origin: \
                     {},\n            }},\n",
                    attribution.identifier,
                    crate::build::Emitter::origin(&attribution.provenance),
                )?;
            }

            f.write_str("        ],\n        notices: &[\n")?;

            for notice in &verdict.notices {
                let file =
                    self.files.get(&notice.text).map_or("", String::as_str);

                writeln!(f, "            include_str!({file:?}),")?;
            }

            f.write_str("        ],\n    },\n")?;
        }

        f.write_str("] }\n")
    }
}

/******************************************************************************/
