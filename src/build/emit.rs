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

//! Writing the attribution out.
//!
//! Two artefacts come from one pass, which is what makes the check of D8
//! possible without a second mechanism:
//!
//! * the embedded form, written into `OUT_DIR` as Rust source, and
//! * `THIRDPARTY.md`, a human-readable file kept under version control.
//!
//! Writing Rust source rather than a binary blob is what keeps the runtime
//! half free of dependencies:  the compiler reads it, so nothing has to parse
//! anything at run time and no format can be got wrong.  The texts themselves
//! go into separate files pulled in by `include_str!`, which sidesteps
//! escaping entirely — a licence text is exactly the sort of thing that
//! contains whatever quoting scheme one might have chosen.
//!
//! Everything is emitted in a fixed order, and identical texts are stored
//! once.  Determinism is not a nicety here:  the check compares a freshly
//! rendered file against the committed one, and any instability would make it
//! fail at random.

use super::{Classification, Provenance, ResolvedPackage};
use std::{
    collections::BTreeMap,
    fmt, fs, io,
    path::{Path, PathBuf},
};

/// The generated Rust source, relative to `OUT_DIR`.
const GENERATED: &str = "list-my-licence.rs";

/// The directory holding the reproduced texts, relative to `OUT_DIR`.
const TEXTS: &str = "list-my-licence-texts";

/// Anything that can go wrong while emitting.
#[derive(Debug)]
pub enum Error {
    /// A file could not be written.
    Write {
        /// Where it should have gone.
        path: PathBuf,

        /// Why it did not.
        reason: io::Error,
    },

    /// The committed attribution is out of date.
    ///
    /// Raised only by [`Emitter::check`].  Regenerating it is the fix;  the
    /// failure exists so that a stale file cannot be merged unnoticed.
    Stale {
        /// The file that no longer matches what the graph would produce.
        path: PathBuf,
    },
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
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

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Write { reason, .. } => Some(reason),
            Self::Stale { .. } => None,
        }
    }
}

/// One package as it will be reproduced.
pub type Reproduced<'a> = (&'a ResolvedPackage, &'a Classification);

/// Writes the attribution, and checks a committed copy against it.
#[derive(Clone, Debug)]
pub struct Emitter {
    out_dir: PathBuf,
}

impl Emitter {
    /// An emitter writing into the given directory.
    #[must_use]
    pub fn new(out_dir: impl Into<PathBuf>) -> Self {
        Self {
            out_dir: out_dir.into(),
        }
    }

    /// An emitter writing into the `OUT_DIR` Cargo gives a build script.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Write`] if `OUT_DIR` is not set, which means this is
    /// not running as a build script.
    pub fn from_build_env() -> Result<Self, Error> {
        std::env::var_os("OUT_DIR").map(Self::new).ok_or_else(|| Error::Write {
            path: PathBuf::from("OUT_DIR"),
            reason: io::Error::new(
                io::ErrorKind::NotFound,
                "OUT_DIR is unset, so this is not running as a build script",
            ),
        })
    }

    /// Writes the embedded form into `OUT_DIR`.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Write`] if anything cannot be written.
    pub fn embed(&self, packages: &[Reproduced<'_>]) -> Result<(), Error> {
        let directory = self.out_dir.join(TEXTS);

        Self::create(&directory)?;

        let files = Self::intern(&directory, packages)?;

        Self::write(
            &self.out_dir.join(GENERATED),
            &Generated {
                packages,
                files: &files,
            }
            .to_string(),
        )
    }

    /// Writes every distinct text once, and says where each went.
    ///
    /// The same MIT text is shipped by dozens of crates.  Storing it once
    /// keeps the binary a reasonable size and makes the output depend only on
    /// the set of texts, not on how many packages happen to share one.
    fn intern(
        directory: &Path,
        packages: &[Reproduced<'_>],
    ) -> Result<BTreeMap<String, String>, Error> {
        let mut files = BTreeMap::new();

        let texts = packages.iter().flat_map(|(_, verdict)| {
            verdict
                .attributions
                .iter()
                .map(|attribution| attribution.text.as_str())
                .chain(
                    verdict.notices.iter().map(|notice| notice.text.as_str()),
                )
        });

        for text in texts {
            if files.contains_key(text) {
                continue;
            }

            let name = format!("{TEXTS}/text-{:04}.txt", files.len());

            Self::write(
                &directory.join(format!("text-{:04}.txt", files.len())),
                text,
            )?;
            files.insert(text.to_owned(), name);
        }

        Ok(files)
    }

    /// Renders the human-readable attribution.
    ///
    /// Deliberately produces the same Markdown as
    /// [`Attribution::markdown`](crate::Attribution::markdown) does from the
    /// embedded form, so that the committed file and the shipped one cannot
    /// drift apart in wording.  A test holds the two together.
    #[must_use]
    pub fn markdown(packages: &[Reproduced<'_>]) -> String {
        Markdown(packages).to_string()
    }

    /// Writes the human-readable attribution to `path`.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Write`] if it cannot be written.
    pub fn publish(
        path: &Path,
        packages: &[Reproduced<'_>],
    ) -> Result<(), Error> {
        Self::write(path, &Self::markdown(packages))
    }

    /// Checks a committed attribution against what the graph would produce.
    ///
    /// This is the whole of D8.  Because both artefacts come from one pass, a
    /// dependency whose licence changed cannot reach a release without the
    /// committed file changing too — and a changed file is a reviewable diff
    /// rather than a silent difference.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Stale`] if the file differs or is missing.
    pub fn check(
        path: &Path,
        packages: &[Reproduced<'_>],
    ) -> Result<(), Error> {
        let expected = Self::markdown(packages);

        match fs::read_to_string(path) {
            Ok(found) if found == expected => Ok(()),
            _ => Err(Error::Stale {
                path: path.to_path_buf(),
            }),
        }
    }

    /// The runtime spelling of a provenance.
    fn origin(provenance: &Provenance) -> String {
        match provenance {
            Provenance::Distributed(path) => {
                format!("Origin::Distributed({:?})", Self::name(path))
            }
            Provenance::Combined(path) => {
                format!("Origin::Combined({:?})", Self::name(path))
            }
            Provenance::Canonical => "Origin::Canonical".to_owned(),
        }
    }

    /// The human-readable spelling of a provenance.
    fn origin_text(provenance: &Provenance) -> String {
        match provenance {
            Provenance::Distributed(path) => {
                format!("as distributed, in {}", Self::name(path))
            }
            Provenance::Combined(path) => {
                format!("as distributed, shared in {}", Self::name(path))
            }
            Provenance::Canonical => "canonical SPDX text".to_owned(),
        }
    }

    /// A licence file's own name, without the path that led to it.
    ///
    /// The full path names a directory in whoever built it, which has no place
    /// in a shipped artefact and would make the committed file differ between
    /// machines.
    fn name(path: &Path) -> String {
        path.file_name().map_or_else(String::new, |name| {
            name.to_string_lossy().into_owned()
        })
    }

    /// Creates a directory, reporting where it failed.
    fn create(path: &Path) -> Result<(), Error> {
        fs::create_dir_all(path).map_err(|reason| Error::Write {
            path: path.to_path_buf(),
            reason,
        })
    }

    /// Writes a file, reporting where it failed.
    fn write(path: &Path, contents: &str) -> Result<(), Error> {
        fs::write(path, contents).map_err(|reason| Error::Write {
            path: path.to_path_buf(),
            reason,
        })
    }
}

/// The Markdown rendering of what will be reproduced.
#[derive(Clone, Copy, Debug)]
struct Markdown<'a>(&'a [Reproduced<'a>]);

impl fmt::Display for Markdown<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("# Third party licences\n")?;

        for (package, verdict) in self.0 {
            write!(f, "\n## {} {}\n", package.name, package.version)?;

            for attribution in &verdict.attributions {
                write!(
                    f,
                    "\n### {} ({})\n\n```text\n{}\n```\n",
                    attribution.identifier,
                    Emitter::origin_text(&attribution.provenance),
                    attribution.text.trim_end(),
                )?;
            }

            for notice in &verdict.notices {
                write!(
                    f,
                    "\n### NOTICE\n\n```text\n{}\n```\n",
                    notice.text.trim_end()
                )?;
            }
        }

        Ok(())
    }
}

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
struct Generated<'a> {
    packages: &'a [Reproduced<'a>],
    files: &'a BTreeMap<String, String>,
}

impl fmt::Display for Generated<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
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
                    Emitter::origin(&attribution.provenance),
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
