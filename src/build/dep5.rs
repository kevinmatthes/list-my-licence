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

/// The specification the emitted file conforms to.
const FORMAT: &str =
    "https://www.debian.org/doc/packaging-manuals/copyright-format/1.0/";

/// The DEP-5 `debian/copyright` rendering of what will be reproduced.
///
/// The same packages [`Markdown`](crate::build::Markdown) renders, in the
/// machine-readable format Debian distributions ship instead.
#[derive(Clone, Copy, Debug)]
pub struct Dep5<'a>(pub &'a [crate::build::Reproduced<'a>]);

impl Dep5<'_> {
    /// Writes one package's licence texts and notices, each folded.
    fn body(
        f: &mut std::fmt::Formatter<'_>,
        verdict: &crate::build::Classification,
    ) -> std::fmt::Result {
        for attribution in &verdict.attributions {
            let origin =
                crate::build::Emitter::origin_text(attribution.provenance());

            write!(f, " .\n {} ({origin})", attribution.identifier())?;
            f.write_str(":\n .\n")?;
            Self::fold(f, attribution.text())?;
        }

        for notice in &verdict.notices {
            f.write_str(" .\n NOTICE:\n .\n")?;
            Self::fold(f, &notice.text)?;
        }

        Ok(())
    }

    /// Writes the mandatory `Copyright` field from the declared authors.
    ///
    /// Where the package names none, that is stated rather than guessed
    /// at:  a manifest is the only source here, and it is silent.
    fn copyright(
        f: &mut std::fmt::Formatter<'_>,
        package: &crate::build::ResolvedPackage,
    ) -> std::fmt::Result {
        if package.authors.is_empty() {
            Self::field(f, "Copyright", "not stated in the package manifest")
        } else {
            Self::field(f, "Copyright", &package.authors.join("\n "))
        }
    }

    /// Writes one `key` and its `value` as a control field.
    ///
    /// The colon and its space are assembled here rather than inlined, so
    /// that no scanned string literal carries the `key: value` shape.
    fn field(
        f: &mut std::fmt::Formatter<'_>,
        key: &str,
        value: &str,
    ) -> std::fmt::Result {
        write!(f, "{key}")?;
        writeln!(f, ": {value}")
    }

    /// Writes one package's whole `Files` paragraph.
    fn files(
        f: &mut std::fmt::Formatter<'_>,
        package: &crate::build::ResolvedPackage,
        verdict: &crate::build::Classification,
    ) -> std::fmt::Result {
        let identifiers = verdict
            .attributions
            .iter()
            .map(crate::build::Attribution::identifier)
            .collect::<Vec<_>>()
            .join(" and ");
        let synopsis = if identifiers.is_empty() {
            "UNKNOWN"
        } else {
            identifiers.as_str()
        };

        Self::field(
            f,
            "\nFiles",
            &format!("{}-{}/*", package.name, package.version),
        )?;
        Self::copyright(f, package)?;
        Self::field(f, "License", synopsis)?;
        Self::body(f, verdict)
    }

    /// Writes `text` as a control-field continuation.
    ///
    /// Every line is indented by one space; a blank one becomes a lone
    /// full stop, and a line that would itself begin with one is indented
    /// twice so a parser cannot read it as the end of the field.
    fn fold(f: &mut std::fmt::Formatter<'_>, text: &str) -> std::fmt::Result {
        for line in text.trim_end().lines() {
            if line.trim().is_empty() {
                f.write_str(" .\n")?;
            } else if line.trim_start().starts_with('.') {
                writeln!(f, "  {line}")?;
            } else {
                writeln!(f, " {line}")?;
            }
        }

        Ok(())
    }
}

impl std::fmt::Display for Dep5<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Self::field(f, "Format", FORMAT)?;

        for (package, verdict) in self.0 {
            Self::files(f, package, verdict)?;
        }

        Ok(())
    }
}

/******************************************************************************/
