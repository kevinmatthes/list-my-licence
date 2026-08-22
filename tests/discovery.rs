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

//! Integration tests for licence file discovery.
//!
//! The fixtures are drawn from the layouts actually measured across the
//! author's registry cache, so they are a sample of what the ecosystem does
//! rather than of what it ought to do.

#![cfg(feature = "build")]

use list_my_licence::build::{Discovery, ResolvedPackage, Role, Skipped};
use std::{fs, path::Path, path::PathBuf};

/// Builds a package rooted at `directory`, with no declared `license-file`.
fn package(directory: &Path) -> ResolvedPackage {
    ResolvedPackage {
        name: "fixture".to_owned(),
        version: "0.0.0".to_owned(),
        manifest_dir: directory.to_path_buf(),
        licence: Some("MIT OR Apache-2.0".to_owned()),
        licence_file: None,
        authors: Vec::new(),
        repository: None,
    }
}

/// Creates a directory holding the named files, each with trivial contents.
fn fixture(names: &[&str]) -> tempfile::TempDir {
    let directory = tempfile::tempdir().expect("a temporary directory");

    for name in names {
        let path = directory.path().join(name);

        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).expect("a parent directory");
        }

        fs::write(&path, format!("text of {name}\n")).expect("a fixture file");
    }

    directory
}

/// The file names discovery took, relative to the fixture root.
fn names(directory: &Path) -> Vec<String> {
    Discovery::new()
        .search(&package(directory))
        .found
        .into_iter()
        .map(|file| {
            file.path
                .strip_prefix(directory)
                .unwrap_or(&file.path)
                .to_string_lossy()
                .into_owned()
        })
        .collect()
}

#[test]
fn finds_the_dominant_two_file_layout() {
    let directory = fixture(&["LICENSE-APACHE", "LICENSE-MIT"]);

    assert_eq!(
        names(directory.path()),
        vec!["LICENSE-APACHE".to_owned(), "LICENSE-MIT".to_owned()],
        "the layout of 72 per cent of the measured crates must be found"
    );
}

#[test]
fn recognises_the_identifier_a_name_points_at() {
    let directory = fixture(&["LICENSE-APACHE", "LICENSE-MIT", "UNLICENSE"]);
    let evidence = Discovery::new().search(&package(directory.path()));

    let mut identifiers: Vec<_> = evidence
        .found
        .iter()
        .filter_map(|file| file.identifier.clone())
        .collect();
    identifiers.sort();

    assert_eq!(
        identifiers,
        vec![
            "Apache-2.0".to_owned(),
            "MIT".to_owned(),
            "Unlicense".to_owned()
        ],
        "APACHE resolves through the alias table, MIT and UNLICENSE directly"
    );
}

#[test]
fn a_bare_licence_file_claims_no_identifier() {
    let directory = fixture(&["LICENSE"]);
    let evidence = Discovery::new().search(&package(directory.path()));

    assert_eq!(evidence.found.len(), 1, "the file must be found");
    assert_eq!(
        evidence.found[0].identifier, None,
        "a bare name says nothing about which licence it holds, and guessing \
         would invent an attribution rather than find one"
    );
}

#[test]
fn understands_an_expression_written_into_the_name() {
    let directory = fixture(&["LICENSE-Apache-2.0_WITH_LLVM-exception"]);
    let evidence = Discovery::new().search(&package(directory.path()));

    assert_eq!(
        evidence.found[0].identifier,
        Some("Apache-2.0 WITH LLVM-exception".to_owned()),
        "the underscore convention of the Rust ecosystem must be understood"
    );
}

#[test]
fn declines_to_guess_at_a_licence_family() {
    for name in ["LICENSE-UNICODE", "LICENSE-BSD", "LICENSE-MIT-ATTY"] {
        let directory = fixture(&[name]);
        let evidence = Discovery::new().search(&package(directory.path()));

        assert_eq!(evidence.found.len(), 1, "{name} must still be found");
        assert_eq!(
            evidence.found[0].identifier, None,
            "{name} names a family or a variant, not a licence, so no \
             identifier may be invented for it"
        );
    }
}

#[test]
fn separates_a_notice_from_a_licence() {
    let directory = fixture(&["LICENSE-APACHE", "NOTICE"]);
    let evidence = Discovery::new().search(&package(directory.path()));

    assert_eq!(
        evidence.notices().count(),
        1,
        "an Apache-2.0 NOTICE must be recognised as such, since Cargo models \
         no such concept and it is invisible to anything reading the manifest"
    );
    assert_eq!(
        evidence.licences().count(),
        1,
        "the licence is not a notice"
    );
    assert_eq!(evidence.notices().next().unwrap().role, Role::Notice);
}

#[test]
fn finds_the_reuse_directory() {
    let directory = fixture(&["LICENSES/MIT.txt", "LICENSES/Apache-2.0.txt"]);
    let evidence = Discovery::new().search(&package(directory.path()));

    let mut identifiers: Vec<_> = evidence
        .found
        .iter()
        .filter_map(|file| file.identifier.clone())
        .collect();
    identifiers.sort();

    assert_eq!(
        identifiers,
        vec!["Apache-2.0".to_owned(), "MIT".to_owned()],
        "REUSE names each file after the identifier it holds, so the stem rule \
         never reaches them and a separate rule is needed"
    );
}

#[test]
fn accepts_whatever_the_manifest_points_at() {
    let directory = tempfile::tempdir().expect("a temporary directory");
    let unusual = directory.path().join("terms-of-use.rst");
    fs::write(&unusual, "bespoke terms\n").expect("a fixture file");

    let mut described = package(directory.path());
    described.licence_file = Some(unusual.clone());

    let evidence = Discovery::new().search(&described);

    assert_eq!(
        evidence.found.len(),
        1,
        "a license-file is taken on the manifest's word, whatever it is called"
    );
    assert_eq!(evidence.found[0].path, unusual);
}

#[test]
fn ignores_files_that_merely_mention_licensing() {
    let directory = fixture(&["licensing-policy.md", "src.rs", "README.md"]);

    assert!(
        names(directory.path()).is_empty(),
        "only licence-bearing names count, got {:?}",
        names(directory.path())
    );
}

#[test]
fn does_not_recurse_into_subdirectories() {
    let directory = fixture(&["LICENSE", "tests/assets/GPL-3.0/LICENSE"]);

    assert_eq!(
        names(directory.path()),
        vec!["LICENSE".to_owned()],
        "a full walk would attribute another project's licence fixture to this \
         package, which several real crates keep under tests/"
    );
}

#[test]
fn reports_an_unreadable_candidate_rather_than_dropping_it() {
    let directory = tempfile::tempdir().expect("a temporary directory");
    let path = directory.path().join("LICENSE");
    fs::write(&path, [0x00, 0xff, 0xfe]).expect("a fixture file");

    let evidence = Discovery::new().search(&package(directory.path()));

    assert!(evidence.found.is_empty(), "the file cannot be reproduced");
    assert_eq!(
        evidence.skipped,
        vec![(path, Skipped::NotText)],
        "a licence that can be seen but not reproduced must be reported, \
         never silently omitted"
    );
}

#[test]
fn nothing_at_all_is_reported_as_nothing() {
    let directory = fixture(&["README.md"]);
    let evidence = Discovery::new().search(&package(directory.path()));

    assert!(evidence.is_empty(), "no licence file means no evidence");
    assert!(
        evidence.skipped.is_empty(),
        "and nothing was refused either"
    );
}

#[test]
fn the_result_is_deterministic() {
    let directory = fixture(&["UNLICENSE", "LICENSE-MIT", "COPYING", "NOTICE"]);

    let first = Discovery::new().search(&package(directory.path()));
    let second = Discovery::new().search(&package(directory.path()));

    assert_eq!(first, second, "repeated runs must agree, or D8 cannot work");
    assert_eq!(
        names(directory.path()),
        vec![
            "COPYING".to_owned(),
            "LICENSE-MIT".to_owned(),
            "NOTICE".to_owned(),
            "UNLICENSE".to_owned()
        ],
        "the order must be stable, not whatever the file system returns"
    );
}

/// `chrono` combines two licences into a single file, which is the case that
/// would make a naive checker report a missing licence for a crate that plainly
/// ships one.
#[test]
fn handles_the_combined_file_of_a_real_crate() {
    let Some(chrono) = registry_package("chrono-") else {
        eprintln!("chrono is not in the registry cache; skipping");
        return;
    };

    let evidence = Discovery::new().search(&package(&chrono));

    assert_eq!(
        evidence.found.len(),
        1,
        "chrono ships exactly one combined licence file, found {:?}",
        evidence.found.iter().map(|f| &f.path).collect::<Vec<_>>()
    );
    assert!(
        evidence.found[0].text.contains("Apache License")
            && evidence.found[0].text.contains("MIT"),
        "and that one file carries both sets of terms"
    );
}

/// The newest cached copy of a package whose directory starts with `prefix`.
fn registry_package(prefix: &str) -> Option<PathBuf> {
    let home = match std::env::var_os("CARGO_HOME") {
        Some(cargo_home) => PathBuf::from(cargo_home),
        None => PathBuf::from(std::env::var_os("HOME")?).join(".cargo"),
    };

    let mut matches: Vec<PathBuf> = fs::read_dir(home.join("registry/src"))
        .ok()?
        .flatten()
        .filter_map(|registry| fs::read_dir(registry.path()).ok())
        .flatten()
        .flatten()
        .map(|entry| entry.path())
        .filter(|path| {
            path.file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| name.starts_with(prefix))
        })
        .collect();

    matches.sort();
    matches.pop()
}

#[test]
fn an_extension_may_be_written_in_any_case() {
    let directory = fixture(&["LICENSE-MIT.TXT", "LICENCE.Md"]);
    let evidence = Discovery::new().search(&package(directory.path()));

    assert_eq!(evidence.found.len(), 2, "both spellings must be found");
    assert!(
        evidence
            .found
            .iter()
            .any(|file| file.identifier == Some("MIT".to_owned())),
        "an upper-case extension must not hide the identifier behind it"
    );
}

/******************************************************************************/
