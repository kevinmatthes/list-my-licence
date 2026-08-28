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

//! Integration tests for emission.
//!
//! The decisive one compiles the generated source with a real `rustc`.  Every
//! other check here could pass while the artefact failed to build, and an
//! attribution that does not compile is not an attribution.

#![cfg(feature = "build")]

use list_my_licence::build::{
    Classifier, Discovery, Emitter, Reproduced, ResolvedPackage,
};
use std::{fs, path::Path, process::Command};

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
        fs::write(directory.join(name), format!("Text of {name}.\n"))
            .expect("a fixture file");
    }

    let described = package(directory, "fixture", licence);
    let evidence = Discovery::new().search(&described);
    let verdict = Classifier::new().classify(&described, &evidence);

    (described, verdict)
}

#[test]
fn the_generated_source_compiles_and_runs() {
    let work = tempfile::tempdir().expect("a temporary directory");
    let out = work.path().join("out");
    fs::create_dir_all(&out).expect("an output directory");

    let (described, verdict) =
        reproduced(work.path(), &["LICENSE-MIT", "NOTICE"], "MIT");
    let packages: Vec<Reproduced<'_>> = vec![(&described, &verdict)];

    Emitter::new(&out)
        .embed(&packages)
        .expect("emission must succeed");

    let driver = work.path().join("driver.rs");
    fs::write(
        &driver,
        format!(
            "mod attribution {{ {} }}\n\
             use attribution::{{Attribution, Licence, Origin, Package}};\n\
             fn main() {{\n\
             \x20   let a: Attribution = include!({:?});\n\
             \x20   assert_eq!(a.packages.len(), 1);\n\
             \x20   assert_eq!(a.packages[0].name, \"fixture\");\n\
             \x20   assert_eq!(a.packages[0].licences.len(), 1);\n\
             \x20   let text = a.packages[0].licences[0].text;\n\
             \x20   assert!(text.contains(\"LICENSE-MIT\"));\n\
             \x20   assert_eq!(a.packages[0].notices.len(), 1);\n\
             }}",
            stand_in(),
            out.join("list-my-licence.rs"),
        ),
    )
    .expect("a driver");

    let binary = work.path().join("driver");
    let compile = Command::new("rustc")
        .args(["--edition", "2021", "-o"])
        .arg(&binary)
        .arg(&driver)
        .output()
        .expect("rustc must be runnable");

    assert!(
        compile.status.success(),
        "the generated source must compile:\n{}",
        String::from_utf8_lossy(&compile.stderr)
    );

    let run = Command::new(&binary).output().expect("the driver must run");

    assert!(
        run.status.success(),
        "and its assertions must hold:\n{}",
        String::from_utf8_lossy(&run.stderr)
    );
}

/// A minimal stand-in for the runtime types the generated source names.
const fn stand_in() -> &'static str {
    "
    pub enum Origin {
        Distributed(&'static str),
        Combined(&'static str),
        Canonical,
    }
    pub struct Licence {
        pub identifier: &'static str,
        pub text: &'static str,
        pub origin: Origin,
    }
    pub struct Package {
        pub name: &'static str,
        pub version: &'static str,
        pub licences: &'static [Licence],
        pub notices: &'static [&'static str],
    }
    pub struct Attribution {
        pub packages: &'static [Package],
    }
    "
}

#[test]
fn the_check_passes_on_a_freshly_written_file() {
    let work = tempfile::tempdir().expect("a temporary directory");
    let (described, verdict) = reproduced(work.path(), &["LICENSE-MIT"], "MIT");
    let packages: Vec<Reproduced<'_>> = vec![(&described, &verdict)];
    let committed = work.path().join("THIRDPARTY.md");

    Emitter::publish(&committed, &packages).expect("publication must succeed");

    assert!(
        Emitter::check(&committed, &packages).is_ok(),
        "what was just written must satisfy the check"
    );
}

#[test]
fn the_check_fails_on_a_stale_file() {
    let work = tempfile::tempdir().expect("a temporary directory");
    let (described, verdict) = reproduced(work.path(), &["LICENSE-MIT"], "MIT");
    let packages: Vec<Reproduced<'_>> = vec![(&described, &verdict)];
    let committed = work.path().join("THIRDPARTY.md");

    fs::write(&committed, "# Third party licences\n\nout of date\n")
        .expect("a stale file");

    assert!(
        Emitter::check(&committed, &packages).is_err(),
        "a dependency whose licence changed must not reach a release without \
         the committed file changing too"
    );
}

#[test]
fn the_check_fails_when_the_file_is_missing() {
    let work = tempfile::tempdir().expect("a temporary directory");
    let (described, verdict) = reproduced(work.path(), &["LICENSE-MIT"], "MIT");
    let packages: Vec<Reproduced<'_>> = vec![(&described, &verdict)];

    assert!(
        Emitter::check(&work.path().join("absent.md"), &packages).is_err(),
        "never having written it is not the same as it being up to date"
    );
}

#[test]
fn emission_is_deterministic() {
    let work = tempfile::tempdir().expect("a temporary directory");
    let (described, verdict) = reproduced(work.path(), &["LICENSE-MIT"], "MIT");
    let packages: Vec<Reproduced<'_>> = vec![(&described, &verdict)];

    let first = Emitter::markdown(&packages);
    let second = Emitter::markdown(&packages);

    assert_eq!(
        first, second,
        "the check compares a fresh rendering against a committed one, so any \
         instability would fail at random"
    );
}

#[test]
fn no_build_directory_leaks_into_the_output() {
    let work = tempfile::tempdir().expect("a temporary directory");
    let out = work.path().join("out");
    fs::create_dir_all(&out).expect("an output directory");

    let (described, verdict) = reproduced(work.path(), &["LICENSE-MIT"], "MIT");
    let packages: Vec<Reproduced<'_>> = vec![(&described, &verdict)];

    Emitter::new(&out)
        .embed(&packages)
        .expect("emission must succeed");

    let generated = fs::read_to_string(out.join("list-my-licence.rs"))
        .expect("the generated source");
    let published = Emitter::markdown(&packages);
    let temporary = work.path().to_string_lossy().into_owned();

    for (what, text) in
        [("generated source", &generated), ("markdown", &published)]
    {
        assert!(
            !text.contains(&temporary),
            "the {what} names the directory it was built in, which belongs to \
             whoever built it and not in a shipped artefact"
        );
    }
}

#[test]
fn both_renderers_agree() {
    use list_my_licence::{Attribution, Licence, Origin, Package};

    let work = tempfile::tempdir().expect("a temporary directory");
    let (described, verdict) = reproduced(work.path(), &["LICENSE-MIT"], "MIT");
    let packages: Vec<Reproduced<'_>> = vec![(&described, &verdict)];

    let licences: &'static [Licence] = Box::leak(Box::new([Licence {
        identifier: Box::leak(
            verdict.attributions[0].identifier.clone().into_boxed_str(),
        ),
        text: Box::leak(verdict.attributions[0].text.clone().into_boxed_str()),
        origin: Origin::Distributed("LICENSE-MIT"),
    }]));
    #[allow(clippy::redundant_clone)]
    let embedded: &'static [Package] = Box::leak(Box::new([Package {
        name: Box::leak(described.name.clone().into_boxed_str()),
        version: Box::leak(described.version.clone().into_boxed_str()),
        licences,
        notices: &[],
    }]));

    assert_eq!(
        Attribution { packages: embedded }.markdown(),
        Emitter::markdown(&packages),
        "the committed file and the shipped one must not drift apart in \
         wording, so the two renderers are held together here"
    );
}

/******************************************************************************/
