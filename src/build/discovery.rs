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

//! Discovery of the licence-bearing files a package actually ships.
//!
//! A package's declared SPDX expression is a claim;  the files beside its
//! manifest are the evidence.  This module gathers that evidence.  It does not
//! judge whether the evidence covers the claim — that is the coverage model,
//! and it comes next.
//!
//! Nothing is ever dropped in silence.  A file that looks like a licence but
//! cannot be read is reported as skipped, with the reason, rather than simply
//! omitted:  a missing attribution is the failure this crate exists to
//! prevent, so the one thing discovery must never do is quietly find less than
//! there is.

/// The largest file discovery will read.
///
/// Licence texts are small;  the GPL, the longest in common use, is under
/// 40 KiB.  The limit exists to stop a package whose `LICENSE` is a symlink to
/// something enormous from being read into memory.
pub const MAX_BYTES: u64 = 1 << 20;

/// The stems that mark a file as licence-bearing, in lower case.
///
/// `LICENCE` is listed beside `LICENSE` because both spellings occur in the
/// wild, and this crate's own preference for the former is no reason to miss
/// the latter.
const STEMS: [&str; 6] = [
    "license",
    "licence",
    "copying",
    "copyright",
    "notice",
    "unlicense",
];

/// The extensions a licence file may carry, in lower case.
const EXTENSIONS: [&str; 3] = ["", "txt", "md"];

/// Directories holding licence files, by the REUSE convention.
const DIRECTORIES: [&str; 2] = ["LICENSES", "licenses"];

/// Names that are not SPDX identifiers but conventionally stand for one.
///
/// Deliberately minimal.  `BSD`, `GPL` and `UNICODE` are *not* listed:  each
/// stands for a family rather than a licence, and guessing which member was
/// meant would invent an attribution rather than find one.
const ALIASES: [(&str, &str); 2] =
    [("apache", "Apache-2.0"), ("mpl", "MPL-2.0")];

/// What part a discovered file plays.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum Role {
    /// The text of a licence.
    Licence,

    /// An Apache-2.0 §4(d) `NOTICE`, whose attribution notices must be carried
    /// into every distributed derivative work.  Cargo models no such concept,
    /// so a `NOTICE` is invisible to anything that reads only the manifest.
    Notice,
}

/// Why a candidate file was not taken.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum Skipped {
    /// The file could not be read at all.
    Unreadable(String),

    /// The file is not valid UTF-8, so its text cannot be reproduced
    /// faithfully.
    NotText,

    /// The file is larger than [`MAX_BYTES`].
    TooLarge(u64),
}

impl std::fmt::Display for Skipped {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Unreadable(why) => write!(f, "could not be read:  {why}"),
            Self::NotText => f.write_str("is not valid UTF-8"),
            Self::TooLarge(size) => {
                write!(
                    f,
                    "is {size} bytes, larger than the {MAX_BYTES} byte limit"
                )
            }
        }
    }
}

/// One licence-bearing file, with its text.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct Found {
    /// Where the file is.
    pub path: std::path::PathBuf,

    /// What part it plays.
    pub role: Role,

    /// The SPDX identifier its *name* points at, where the name names one.
    ///
    /// This is a hint drawn from the file name alone, never from the contents.
    /// `LICENSE-MIT` yields `MIT`;  a bare `LICENSE` yields nothing, because
    /// the name says nothing about which licence it holds.
    pub identifier: Option<String>,

    /// The file's text, reproduced exactly as distributed.
    pub text: String,
}

/// What discovery found for one package.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Evidence {
    /// The files taken, sorted by path so that repeated runs agree.
    pub found: Vec<Found>,

    /// Candidates that looked right but could not be taken, with the reason.
    ///
    /// Never empty without meaning:  an entry here is a licence this crate can
    /// see but not reproduce, which the caller must decide what to do about.
    pub skipped: Vec<(std::path::PathBuf, Skipped)>,
}

impl Evidence {
    /// Whether anything at all was found.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.found.is_empty()
    }

    /// The files that are licence texts rather than notices.
    pub fn licences(&self) -> impl Iterator<Item = &Found> {
        self.found.iter().filter(|file| file.role == Role::Licence)
    }

    /// The Apache-2.0 `NOTICE` files, if any.
    pub fn notices(&self) -> impl Iterator<Item = &Found> {
        self.found.iter().filter(|file| file.role == Role::Notice)
    }
}

/// Finds the licence-bearing files of a package.
///
/// # Examples
///
/// ```no_run
/// # use list_my_licence::build::{Discovery, Resolver};
/// for package in Resolver::from_build_env()?.resolve()? {
///     let evidence = Discovery::new().search(&package);
///
///     for file in evidence.licences() {
///         println!("{} -> {}", package.name, file.path.display());
///     }
/// }
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[derive(Clone, Copy, Debug, Default)]
pub struct Discovery {
    _private: (),
}

impl Discovery {
    /// A discovery with the default settings.
    #[must_use]
    pub const fn new() -> Self {
        Self { _private: () }
    }

    /// Searches one package for licence-bearing files.
    ///
    /// The search covers the directory holding the manifest, the `LICENSES`
    /// directory beside it where the REUSE convention puts them, and whatever
    /// the manifest's own `license-file` key points at.  It does **not**
    /// recurse:  a full walk would pick up the licence fixtures that many
    /// projects keep under `tests/`, and attribute another project's licence to
    /// this one.
    #[must_use]
    pub fn search(&self, package: &crate::build::ResolvedPackage) -> Evidence {
        let mut evidence = Evidence::default();
        let mut candidates = Vec::new();

        Self::collect(&package.manifest_dir, &mut candidates);

        for directory in DIRECTORIES {
            Self::collect_reuse(
                &package.manifest_dir.join(directory),
                &mut candidates,
            );
        }

        // A `license-file` is taken on the manifest's word, whatever it is
        // called:  the author has told us where their licence is.
        if let Some(declared) = &package.licence_file
            && !candidates.contains(declared)
        {
            candidates.push(declared.clone());
        }

        candidates.sort();
        candidates.dedup();

        for path in candidates {
            match Self::read(&path) {
                Ok(text) => evidence.found.push(Found {
                    role: Self::role(&path),
                    identifier: Self::identifier(&path),
                    path,
                    text,
                }),
                Err(why) => evidence.skipped.push((path, why)),
            }
        }

        evidence.found.sort();
        evidence.skipped.sort();

        evidence
    }

    /// Adds every licence-looking file of one directory to `candidates`.
    fn collect(
        directory: &std::path::Path,
        candidates: &mut Vec<std::path::PathBuf>,
    ) {
        let Ok(entries) = std::fs::read_dir(directory) else {
            return;
        };

        for entry in entries.flatten() {
            let path = entry.path();

            if path.is_file() && Self::is_licence_name(&path) {
                candidates.push(path);
            }
        }
    }

    /// Adds the licence files of a REUSE `LICENSES` directory.
    ///
    /// The REUSE specification names each file after the SPDX identifier it
    /// holds — `LICENSES/MIT.txt`, not `LICENSES/LICENSE-MIT` — so the stem
    /// rule of [`Self::collect`] never matches there and a separate rule is
    /// needed.
    fn collect_reuse(
        directory: &std::path::Path,
        candidates: &mut Vec<std::path::PathBuf>,
    ) {
        let Ok(entries) = std::fs::read_dir(directory) else {
            return;
        };

        for entry in entries.flatten() {
            let path = entry.path();

            if path.is_file() && Self::reuse_identifier(&path).is_some() {
                candidates.push(path);
            }
        }
    }

    /// The SPDX identifier a REUSE file name is, if it is one.
    fn reuse_identifier(path: &std::path::Path) -> Option<String> {
        let name = path.file_name()?.to_str()?;
        let base = Self::strip_extension(name);

        Self::canonical(&base)
            .or_else(|| Self::expression(&base.replace('_', " ")))
    }

    /// Whether a file name marks it as licence-bearing.
    ///
    /// The name is split at the first `-`, `_` or `.` that follows a known
    /// stem, so that `LICENSE`, `LICENSE.txt`, `LICENSE-MIT` and
    /// `LICENSE-Apache-2.0_WITH_LLVM-exception` are all recognised, while
    /// `licensing-policy.md` is not.
    fn is_licence_name(path: &std::path::Path) -> bool {
        Self::split_name(path).is_some()
    }

    /// Splits a file name into its stem and whatever follows it.
    ///
    /// The stem is matched without regard to case, but the qualifier is
    /// returned **exactly as written**.  Lower-casing it would destroy the
    /// spelling that both the SPDX identifier table and the expression parser
    /// need:  `Apache-2.0_WITH_LLVM-exception` is meaningful, whereas
    /// `apache-2.0_with_llvm-exception` parses as nothing at all.
    fn split_name(path: &std::path::Path) -> Option<(&'static str, String)> {
        let name = path.file_name()?.to_str()?;
        let lowered = name.to_ascii_lowercase();

        for stem in STEMS {
            if !lowered.starts_with(stem) {
                continue;
            }

            let rest = &name[stem.len()..];

            if rest.is_empty() {
                return Some((stem, String::new()));
            }

            // An extension alone, such as `LICENSE.md`.
            if let Some(extension) = rest.strip_prefix('.')
                && EXTENSIONS.contains(&extension.to_ascii_lowercase().as_str())
            {
                return Some((stem, String::new()));
            }

            // A qualifier, such as `LICENSE-MIT` or `LICENSE_APACHE.txt`.
            if let Some(qualifier) = rest.strip_prefix(['-', '_', '.']) {
                return Some((stem, Self::strip_extension(qualifier)));
            }
        }

        None
    }

    /// Removes a trailing `.txt` or `.md` from a qualifier, whatever its case.
    fn strip_extension(qualifier: &str) -> String {
        let lowered = qualifier.to_ascii_lowercase();

        for extension in EXTENSIONS {
            if extension.is_empty() {
                continue;
            }

            if lowered.ends_with(&format!(".{extension}")) {
                return qualifier[..qualifier.len() - extension.len() - 1]
                    .to_owned();
            }
        }

        qualifier.to_owned()
    }

    /// What part a file plays, judged by its name.
    fn role(path: &std::path::Path) -> Role {
        match Self::split_name(path) {
            Some(("notice", _)) => Role::Notice,
            _ => Role::Licence,
        }
    }

    /// Whether a path sits inside a REUSE `LICENSES` directory.
    fn is_reuse(path: &std::path::Path) -> bool {
        path.parent()
            .and_then(std::path::Path::file_name)
            .and_then(|name| name.to_str())
            .is_some_and(|name| DIRECTORIES.contains(&name))
    }

    /// The SPDX identifier a file name points at, if it points at one.
    ///
    /// The qualifier is tried as an SPDX identifier, then as a whole SPDX
    /// expression with `_` read as a space, which is how the Rust ecosystem
    /// writes `LICENSE-Apache-2.0_WITH_LLVM-exception`.  Only then is the small
    /// alias table consulted.  A stem that is itself an identifier, such as
    /// `UNLICENSE`, is recognised too.
    fn identifier(path: &std::path::Path) -> Option<String> {
        if Self::is_reuse(path) {
            return Self::reuse_identifier(path);
        }

        let (stem, qualifier) = Self::split_name(path)?;

        if qualifier.is_empty() {
            return Self::canonical(stem);
        }

        Self::canonical(&qualifier)
            .or_else(|| Self::expression(&qualifier.replace('_', " ")))
            .or_else(|| {
                ALIASES
                    .iter()
                    .find(|(alias, _)| qualifier.eq_ignore_ascii_case(alias))
                    .map(|(_, identifier)| (*identifier).to_owned())
            })
    }

    /// The canonically spelled SPDX identifier matching `name`, if any.
    fn canonical(name: &str) -> Option<String> {
        spdx::identifiers::LICENSES
            .iter()
            .find(|licence| licence.name.eq_ignore_ascii_case(name))
            .map(|licence| licence.name.to_owned())
    }

    /// The given text, if it is a valid SPDX expression.
    fn expression(text: &str) -> Option<String> {
        spdx::Expression::parse(text)
            .ok()
            .map(|parsed| parsed.to_string())
    }

    /// Reads a file, refusing anything too large or not textual.
    fn read(path: &std::path::Path) -> Result<String, Skipped> {
        let metadata = path
            .metadata()
            .map_err(|error| Skipped::Unreadable(error.to_string()))?;

        if metadata.len() > MAX_BYTES {
            return Err(Skipped::TooLarge(metadata.len()));
        }

        let bytes = std::fs::read(path)
            .map_err(|error| Skipped::Unreadable(error.to_string()))?;

        String::from_utf8(bytes).map_err(|_| Skipped::NotText)
    }
}

/******************************************************************************/
