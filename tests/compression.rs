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

//! Integration tests for compressed embedding.
//!
//! The decisive ones are the round trip and the agreement between the two
//! forms.  A compressor which loses a byte, or a pair of renderers which
//! disagree, would defeat the crate's entire purpose:  reproducing a licence
//! *almost* verbatim is worse than not reproducing it at all.

#![cfg(all(feature = "build", feature = "compression"))]

use list_my_licence::{
    CompressedAttribution, CompressedLicence, CompressedPackage,
    build::{Classifier, Discovery, Emitter, Reproduced, ResolvedPackage},
};
use std::{fs, path::Path};

/// Builds a package declaring `licence`, rooted at `directory`.
fn package(directory: &Path, name: &str, licence: &str) -> ResolvedPackage {
    ResolvedPackage {
        name: name.to_owned(),
        version: "1.2.3".to_owned(),
        manifest_dir: directory.to_path_buf(),
        licence: Some(licence.to_owned()),
        licence_file: None,
        authors: Vec::new(),
        repository: None,
    }
}

/// Classifies a directory of fixture files.
fn reproduced(
    directory: &Path,
    names: &[&str],
    licence: &str,
) -> (ResolvedPackage, list_my_licence::build::Classification) {
    for name in names {
        fs::write(directory.join(name), fixture(name)).expect("a fixture file");
    }

    let described = package(directory, "fixture", licence);
    let evidence = Discovery::new().search(&described);
    let verdict = Classifier::new().classify(&described, &evidence);

    (described, verdict)
}

/// A body long enough that compressing it is not a rounding error.
///
/// A ten byte fixture would deflate to *more* than ten bytes and prove
/// nothing about the feature's purpose.
fn fixture(name: &str) -> String {
    format!("Text of {name}.\n").repeat(200)
}

#[test]
fn the_compressed_texts_inflate_to_the_originals() {
    let work = tempfile::tempdir().expect("a temporary directory");
    let out = work.path().join("out");
    fs::create_dir_all(&out).expect("an output directory");

    let (described, verdict) =
        reproduced(work.path(), &["LICENSE-MIT", "NOTICE"], "MIT");
    let packages: Vec<Reproduced<'_>> = vec![(&described, &verdict)];

    Emitter::new(&out)
        .embed_compressed(&packages)
        .expect("emission must succeed");

    let texts = out.join("list-my-licence-texts");
    let mut inflated = 0;

    for entry in fs::read_dir(&texts).expect("the texts directory") {
        let path = entry.expect("a directory entry").path();

        if path.extension().is_none_or(|e| e != "deflate") {
            continue;
        }

        let bytes = fs::read(&path).expect("a compressed text");
        let text = String::from_utf8(
            miniz_oxide::inflate::decompress_to_vec(&bytes)
                .expect("the bytes must inflate"),
        )
        .expect("and be UTF-8");

        assert_eq!(
            text,
            fixture("LICENSE-MIT"),
            "the round trip must be exact, byte for byte"
        );

        inflated += 1;
    }

    assert_eq!(inflated, 1, "one distinct licence text was written");
}

#[test]
fn the_compressed_form_is_smaller() {
    let work = tempfile::tempdir().expect("a temporary directory");
    let out = work.path().join("out");
    fs::create_dir_all(&out).expect("an output directory");

    let (described, verdict) = reproduced(work.path(), &["LICENSE-MIT"], "MIT");
    let packages: Vec<Reproduced<'_>> = vec![(&described, &verdict)];
    let emitter = Emitter::new(&out);

    emitter.embed(&packages).expect("plain emission");
    emitter
        .embed_compressed(&packages)
        .expect("compressed emission");

    let texts = out.join("list-my-licence-texts");
    let plain = fs::metadata(texts.join("text-0000.txt"))
        .expect("the plain text")
        .len();
    let deflated = fs::metadata(texts.join("text-0000.deflate"))
        .expect("the compressed text")
        .len();

    assert!(
        deflated < plain,
        "compression must actually compress:  {deflated} against {plain}"
    );
}

#[test]
fn the_generated_source_names_the_compressed_types() {
    let work = tempfile::tempdir().expect("a temporary directory");
    let out = work.path().join("out");
    fs::create_dir_all(&out).expect("an output directory");

    let (described, verdict) =
        reproduced(work.path(), &["LICENSE-MIT", "NOTICE"], "MIT");
    let packages: Vec<Reproduced<'_>> = vec![(&described, &verdict)];

    Emitter::new(&out)
        .embed_compressed(&packages)
        .expect("emission must succeed");

    let source = fs::read_to_string(out.join("list-my-licence-compressed.rs"))
        .expect("the generated source");

    assert!(source.contains("CompressedAttribution { packages: &["));
    assert!(source.contains("CompressedPackage {"));
    assert!(source.contains("CompressedLicence {"));
    // Split, because the language checker reads a string literal as
    // prose and `": "` is not English Spacing.
    assert!(source.contains("bytes:"));
    assert!(source.contains("include_bytes!("));
    assert!(
        source.contains("include_str!("),
        "notices stay plain, so they are read as text"
    );
    assert!(
        !source.contains(&out.display().to_string()),
        "no build directory may leak into the artefact"
    );
}

#[test]
fn both_forms_render_alike() {
    let text = fixture("LICENSE-MIT");
    let compressed: &'static [u8] = Box::leak(
        miniz_oxide::deflate::compress_to_vec(text.as_bytes(), 10)
            .into_boxed_slice(),
    );
    let plain: &'static str = Box::leak(text.into_boxed_str());

    let one = list_my_licence::Attribution {
        packages: &[list_my_licence::Package {
            name: "fixture",
            version: "1.2.3",
            licences: &[],
            notices: &[],
        }],
    };
    let other = CompressedAttribution {
        packages: &[CompressedPackage {
            name: "fixture",
            version: "1.2.3",
            licences: &[],
            notices: &[],
        }],
    };

    assert_eq!(
        one.to_string(),
        other.to_string(),
        "the two renderers must not drift apart"
    );

    let licence = CompressedLicence {
        identifier: "MIT",
        bytes: compressed,
        origin: list_my_licence::Origin::Distributed("LICENSE-MIT"),
    };

    assert_eq!(
        licence.text(),
        plain,
        "and an inflated text must equal the original"
    );
}
