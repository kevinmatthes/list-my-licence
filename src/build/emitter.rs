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

/// The generated Rust source, relative to `OUT_DIR`.
const GENERATED: &str = "list-my-licence.rs";

/// The generated Rust source for the compressed form.
#[cfg(feature = "compression")]
const GENERATED_COMPRESSED: &str = "list-my-licence-compressed.rs";

/// The directory holding the reproduced texts, relative to `OUT_DIR`.
const TEXTS: &str = "list-my-licence-texts";

/// Writes the attribution, and checks a committed copy against it.
#[derive(Clone, Debug)]
pub struct Emitter {
    out_dir: std::path::PathBuf,
}

impl Emitter {
    /// Checks a committed attribution against what the graph would produce.
    ///
    /// This is the whole of D8.  Because both artefacts come from one pass, a
    /// dependency whose licence changed cannot reach a release without the
    /// committed file changing too — and a changed file is a reviewable diff
    /// rather than a silent difference.
    ///
    /// # Errors
    ///
    /// Returns [`crate::build::EmitError::Stale`] if the file differs or is
    /// missing.
    pub fn check(
        path: &std::path::Path,
        packages: &[crate::build::Reproduced<'_>],
    ) -> Result<(), crate::build::EmitError> {
        let expected = Self::markdown(packages);

        match std::fs::read_to_string(path) {
            Ok(found) if found == expected => Ok(()),
            _ => Err(crate::build::EmitError::Stale {
                path: path.to_path_buf(),
            }),
        }
    }

    /// Creates a directory, reporting where it failed.
    fn create(path: &std::path::Path) -> Result<(), crate::build::EmitError> {
        std::fs::create_dir_all(path).map_err(|reason| {
            crate::build::EmitError::Write {
                path: path.to_path_buf(),
                reason,
            }
        })
    }

    /// Writes the embedded form into `OUT_DIR`.
    ///
    /// # Errors
    ///
    /// Returns [`crate::build::EmitError::Write`] if anything cannot be
    /// written.
    pub fn embed(
        &self,
        packages: &[crate::build::Reproduced<'_>],
    ) -> Result<(), crate::build::EmitError> {
        let directory = self.out_dir.join(TEXTS);

        Self::create(&directory)?;

        let files = Self::intern(&directory, packages)?;

        Self::write(
            &self.out_dir.join(GENERATED),
            &crate::build::Generated {
                packages,
                files: &files,
            }
            .to_string(),
        )
    }

    /// Writes the compressed embedded form into `OUT_DIR`.
    ///
    /// The counterpart of [`Self::embed`], and independent of it:  a build
    /// script may call either, or both, and the two artefacts describe the
    /// same graph.  Licence texts are deflated;  notices are not, for the
    /// reason [`CompressedPackage`](crate::CompressedPackage) gives.
    ///
    /// # Errors
    ///
    /// Returns [`crate::build::EmitError::Write`] if anything cannot be
    /// written.
    #[cfg(feature = "compression")]
    pub fn embed_compressed(
        &self,
        packages: &[crate::build::Reproduced<'_>],
    ) -> Result<(), crate::build::EmitError> {
        let directory = self.out_dir.join(TEXTS);

        Self::create(&directory)?;

        let (files, notices) = Self::intern_compressed(&directory, packages)?;

        Self::write(
            &self.out_dir.join(GENERATED_COMPRESSED),
            &crate::build::GeneratedCompressed {
                packages,
                files: &files,
                notices: &notices,
            }
            .to_string(),
        )
    }

    /// An emitter writing into the `OUT_DIR` Cargo gives a build script.
    ///
    /// # Errors
    ///
    /// Returns [`crate::build::EmitError::Write`] if `OUT_DIR` is not set,
    /// which means this is not running as a build script.
    pub fn from_build_env() -> Result<Self, crate::build::EmitError> {
        std::env::var_os("OUT_DIR").map(Self::new).ok_or_else(|| {
            crate::build::EmitError::Write {
                path: std::path::PathBuf::from("OUT_DIR"),
                reason: std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    "OUT_DIR is unset, so this is not running as a build \
                     script",
                ),
            }
        })
    }

    /// Writes every distinct text once, and says where each went.
    ///
    /// The same MIT text is shipped by dozens of crates.  Storing it once
    /// keeps the binary a reasonable size and makes the output depend only on
    /// the set of texts, not on how many packages happen to share one.
    fn intern(
        directory: &std::path::Path,
        packages: &[crate::build::Reproduced<'_>],
    ) -> Result<
        std::collections::BTreeMap<String, String>,
        crate::build::EmitError,
    > {
        let mut files = std::collections::BTreeMap::new();

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

    /// Writes every distinct text once, deflated, and says where each went.
    ///
    /// Interning matters more here than in [`Self::intern`], not less:  the
    /// same MIT text shipped by dozens of crates compresses to the same bytes
    /// dozens of times over, and storing it once is what makes the saving
    /// worth having.
    ///
    /// Notices are written plain, so they come back as a separate map.
    #[cfg(feature = "compression")]
    fn intern_compressed(
        directory: &std::path::Path,
        packages: &[crate::build::Reproduced<'_>],
    ) -> Result<crate::build::Interned, crate::build::EmitError> {
        let mut files = std::collections::BTreeMap::new();
        let mut notices = std::collections::BTreeMap::new();

        for (_, verdict) in packages {
            for attribution in &verdict.attributions {
                if files.contains_key(&attribution.text) {
                    continue;
                }

                let name = format!("{TEXTS}/text-{:04}.deflate", files.len());
                let leaf = format!("text-{:04}.deflate", files.len());

                Self::write_bytes(
                    &directory.join(leaf),
                    &miniz_oxide::deflate::compress_to_vec(
                        attribution.text.as_bytes(),
                        10,
                    ),
                )?;
                files.insert(attribution.text.clone(), name);
            }

            for notice in &verdict.notices {
                if notices.contains_key(&notice.text) {
                    continue;
                }

                let name = format!("{TEXTS}/notice-{:04}.txt", notices.len());

                Self::write(
                    &directory.join(format!("notice-{:04}.txt", notices.len())),
                    &notice.text,
                )?;
                notices.insert(notice.text.clone(), name);
            }
        }

        Ok((files, notices))
    }

    /// Renders the human-readable attribution.
    ///
    /// Deliberately produces the same Markdown as
    /// [`Attribution::markdown`](crate::Attribution::markdown) does from the
    /// embedded form, so that the committed file and the shipped one cannot
    /// drift apart in wording.  A test holds the two together.
    #[must_use]
    pub fn markdown(packages: &[crate::build::Reproduced<'_>]) -> String {
        crate::build::Markdown(packages).to_string()
    }

    /// A licence file's own name, without the path that led to it.
    ///
    /// The full path names a directory in whoever built it, which has no place
    /// in a shipped artefact and would make the committed file differ between
    /// machines.
    fn name(path: &std::path::Path) -> String {
        path.file_name().map_or_else(String::new, |name| {
            name.to_string_lossy().into_owned()
        })
    }

    /// An emitter writing into the given directory.
    #[must_use]
    pub fn new(out_dir: impl Into<std::path::PathBuf>) -> Self {
        Self {
            out_dir: out_dir.into(),
        }
    }

    /// The runtime spelling of a provenance.
    #[must_use]
    pub fn origin(provenance: &crate::build::Provenance) -> String {
        match provenance {
            crate::build::Provenance::Distributed(path) => {
                format!("Origin::Distributed({:?})", Self::name(path))
            }
            crate::build::Provenance::Combined(path) => {
                format!("Origin::Combined({:?})", Self::name(path))
            }
            crate::build::Provenance::Canonical => {
                "Origin::Canonical".to_owned()
            }
        }
    }

    /// The human-readable spelling of a provenance.
    #[must_use]
    pub fn origin_text(provenance: &crate::build::Provenance) -> String {
        match provenance {
            crate::build::Provenance::Distributed(path) => {
                format!("as distributed, in {}", Self::name(path))
            }
            crate::build::Provenance::Combined(path) => {
                format!("as distributed, shared in {}", Self::name(path))
            }
            crate::build::Provenance::Canonical => {
                "canonical SPDX text".to_owned()
            }
        }
    }

    /// Writes the human-readable attribution to `path`.
    ///
    /// # Errors
    ///
    /// Returns [`crate::build::EmitError::Write`] if it cannot be written.
    pub fn publish(
        path: &std::path::Path,
        packages: &[crate::build::Reproduced<'_>],
    ) -> Result<(), crate::build::EmitError> {
        Self::write(path, &Self::markdown(packages))
    }

    /// Writes a file, reporting where it failed.
    ///
    /// # Errors
    ///
    /// Returns [`crate::build::EmitError::Write`] if the file cannot be
    /// written.
    pub fn write(
        path: &std::path::Path,
        contents: &str,
    ) -> Result<(), crate::build::EmitError> {
        std::fs::write(path, contents).map_err(|reason| {
            crate::build::EmitError::Write {
                path: path.to_path_buf(),
                reason,
            }
        })
    }

    /// Writes a file of bytes, reporting where it failed.
    ///
    /// [`Self::write`] takes a string, which compressed text is not.
    ///
    /// # Errors
    ///
    /// Returns [`crate::build::EmitError::Write`] if the file cannot be
    /// written.
    #[cfg(feature = "compression")]
    pub fn write_bytes(
        path: &std::path::Path,
        contents: &[u8],
    ) -> Result<(), crate::build::EmitError> {
        std::fs::write(path, contents).map_err(|reason| {
            crate::build::EmitError::Write {
                path: path.to_path_buf(),
                reason,
            }
        })
    }
}
