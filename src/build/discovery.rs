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
    /// The canonically spelled SPDX identifier matching `name`, if any.
    fn canonical(name: &str) -> Option<String> {
        spdx::identifiers::LICENSES
            .iter()
            .find(|licence| licence.name.eq_ignore_ascii_case(name))
            .map(|licence| licence.name.to_owned())
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

    /// The given text, if it is a valid SPDX expression.
    fn expression(text: &str) -> Option<String> {
        spdx::Expression::parse(text)
            .ok()
            .map(|parsed| parsed.to_string())
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

    /// Whether a file name marks it as licence-bearing.
    ///
    /// The name is split at the first `-`, `_` or `.` that follows a known
    /// stem, so that `LICENSE`, `LICENSE.txt`, `LICENSE-MIT` and
    /// `LICENSE-Apache-2.0_WITH_LLVM-exception` are all recognised, while
    /// `licensing-policy.md` is not.
    fn is_licence_name(path: &std::path::Path) -> bool {
        Self::split_name(path).is_some()
    }

    /// Whether a path sits inside a REUSE `LICENSES` directory.
    fn is_reuse(path: &std::path::Path) -> bool {
        path.parent()
            .and_then(std::path::Path::file_name)
            .and_then(|name| name.to_str())
            .is_some_and(|name| DIRECTORIES.contains(&name))
    }

    /// A discovery with the default settings.
    #[must_use]
    pub const fn new() -> Self {
        Self { _private: () }
    }

    /// Reads a file, refusing anything too large or not textual.
    fn read(path: &std::path::Path) -> Result<String, crate::build::Skipped> {
        let metadata = path.metadata().map_err(|error| {
            crate::build::Skipped::Unreadable(error.to_string())
        })?;

        if metadata.len() > crate::build::MAX_BYTES {
            return Err(crate::build::Skipped::TooLarge(metadata.len()));
        }

        let bytes = std::fs::read(path).map_err(|error| {
            crate::build::Skipped::Unreadable(error.to_string())
        })?;

        String::from_utf8(bytes).map_err(|_| crate::build::Skipped::NotText)
    }

    /// The SPDX identifier a REUSE file name is, if it is one.
    fn reuse_identifier(path: &std::path::Path) -> Option<String> {
        let name = path.file_name()?.to_str()?;
        let base = Self::strip_extension(name);

        Self::canonical(&base)
            .or_else(|| Self::expression(&base.replace('_', " ")))
    }

    /// What part a file plays, judged by its name.
    fn role(path: &std::path::Path) -> crate::build::Role {
        match Self::split_name(path) {
            Some(("notice", _)) => crate::build::Role::Notice,
            _ => crate::build::Role::Licence,
        }
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
    pub fn search(
        &self,
        package: &crate::build::ResolvedPackage,
    ) -> crate::build::Evidence {
        let mut evidence = crate::build::Evidence::default();
        let mut candidates = Vec::new();

        Self::collect(&package.manifest_dir, &mut candidates);

        for directory in DIRECTORIES {
            Self::collect_reuse(
                &package.manifest_dir.join(directory),
                &mut candidates,
            );
        }

        if let Some(declared) = &package.licence_file
            && !candidates.contains(declared)
        {
            candidates.push(declared.clone());
        }

        candidates.sort();
        candidates.dedup();

        for path in candidates {
            match Self::read(&path) {
                Ok(text) => evidence.found.push(crate::build::Found {
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

            if let Some(extension) = rest.strip_prefix('.')
                && EXTENSIONS.contains(&extension.to_ascii_lowercase().as_str())
            {
                return Some((stem, String::new()));
            }

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
}

/******************************************************************************/
