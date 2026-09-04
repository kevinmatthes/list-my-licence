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

//! Integration tests for the coverage classifier.
//!
//! The cases are the ones measured in the author's own registry cache:  the
//! dominant two-file layout, `chrono`'s single combined file, and the twelve
//! crates that ship no licence text at all — eight of which stay
//! satisfiable through a standard-text branch, while four do not.

#![cfg(feature = "build")]

use list_my_licence::build::{
    Classifier, Coverage, Discovery, Evidence, Problem, Provenance,
    ResolvedPackage,
};
use std::{fs, path::Path};

/// Builds a package declaring `licence` and rooted at `directory`.
fn package(directory: &Path, licence: Option<&str>) -> ResolvedPackage {
    ResolvedPackage {
        name: "fixture".to_owned(),
        version: "0.0.0".to_owned(),
        manifest_dir: directory.to_path_buf(),
        licence: licence.map(ToOwned::to_owned),
        licence_file: None,
        authors: Vec::new(),
        repository: None,
    }
}

/// Creates a directory holding the named files.
fn fixture(names: &[&str]) -> tempfile::TempDir {
    let directory = tempfile::tempdir().expect("a temporary directory");

    for name in names {
        let path = directory.path().join(name);
        fs::write(&path, format!("text of {name}\n")).expect("a fixture file");
    }

    directory
}

/// Classifies a fixture directory against a declaration.
fn classify(
    names: &[&str],
    licence: Option<&str>,
) -> (list_my_licence::build::Classification, tempfile::TempDir) {
    let directory = fixture(names);
    let described = package(directory.path(), licence);
    let evidence: Evidence = Discovery::new().search(&described);
    let verdict = Classifier::new().classify(&described, &evidence);

    (verdict, directory)
}

#[test]
fn the_dominant_layout_is_complete() {
    let (verdict, _keep) = classify(
        &["LICENSE-MIT", "LICENSE-APACHE"],
        Some("MIT OR Apache-2.0"),
    );

    assert_eq!(verdict.coverage, Coverage::Complete);
    assert_eq!(verdict.attributions.len(), 2, "one text per declared term");
    assert!(
        verdict
            .attributions
            .iter()
            .all(|a| matches!(a.provenance(), Provenance::Distributed(_))),
        "both texts come from the copies the author distributed"
    );
    assert!(verdict.problems.is_empty(), "{:?}", verdict.problems);
}

#[test]
fn a_single_combined_file_is_not_a_gap() {
    let (verdict, _keep) = classify(&["LICENSE"], Some("MIT OR Apache-2.0"));

    assert_eq!(
        verdict.coverage,
        Coverage::Combined,
        "chrono ships two licences in one file, and reporting that as missing \
         would destroy trust in the output"
    );
    assert_eq!(
        verdict.attributions.len(),
        2,
        "both terms are covered by it"
    );
    assert!(
        verdict
            .attributions
            .iter()
            .all(|a| matches!(a.provenance(), Provenance::Combined(_))),
        "and the output must be able to say the file was shared"
    );
    assert!(verdict.problems.is_empty(), "{:?}", verdict.problems);
}

#[test]
fn one_branch_of_an_or_suffices() {
    let (verdict, _keep) =
        classify(&["LICENSE-APACHE"], Some("MIT OR Apache-2.0"));

    assert!(
        !verdict.is_fatal(),
        "MIT is unattributable here, but the expression offers a branch that \
         is fully dischargeable, so nothing fails: {:?}",
        verdict.problems
    );
    assert_eq!(verdict.attributions.len(), 1, "only the branch taken");
    assert_eq!(verdict.attributions[0].identifier(), "Apache-2.0");
}

#[test]
fn a_missing_file_is_survivable_for_a_standard_text_licence() {
    let (verdict, _keep) = classify(&[], Some("MIT OR Apache-2.0"));

    assert_eq!(verdict.coverage, Coverage::Absent, "nothing was shipped");
    assert!(
        !verdict.is_fatal(),
        "eight of the twelve licence-less crates measured declare a variant of \
         this, and Apache-2.0's canonical text discharges them: {:?}",
        verdict.problems
    );
    assert_eq!(
        verdict.attributions[0].provenance(),
        Provenance::Canonical,
        "the text is the canonical one, and the output must say so"
    );
    assert_eq!(verdict.attributions[0].identifier(), "Apache-2.0");
}

#[test]
fn a_missing_file_is_fatal_for_a_notice_style_licence() {
    let (verdict, _keep) = classify(&[], Some("MIT"));

    assert!(
        verdict.is_fatal(),
        "bech32, cookie-factory, pcsc and pcsc-sys are MIT-only and ship no \
         text; the canonical MIT carries an empty copyright line and \
         discharges nothing"
    );
    assert!(
        verdict.problems.iter().any(|problem| matches!(
            problem,
            Problem::Unattributed { identifier } if identifier == "MIT"
        )),
        "and the report must name the licence:  {:?}",
        verdict.problems
    );
}

#[test]
fn an_and_expression_needs_every_term() {
    let (verdict, _keep) =
        classify(&["LICENSE-APACHE"], Some("MIT AND Apache-2.0"));

    assert!(
        verdict.is_fatal(),
        "AND offers no choice of branch, so the unattributable MIT is fatal: \
         {:?}",
        verdict.problems
    );
}

#[test]
fn the_deprecated_slash_form_is_understood() {
    let (verdict, _keep) =
        classify(&["LICENSE-MIT", "LICENSE-APACHE"], Some("MIT/Apache-2.0"));

    assert_eq!(
        verdict.coverage,
        Coverage::Complete,
        "four of the twelve measured crates still write the slash form, and \
         refusing it would report them as having no parsable licence"
    );
    assert!(verdict.problems.is_empty(), "{:?}", verdict.problems);
}

#[test]
fn a_deprecated_identifier_is_understood() {
    let (verdict, _keep) = classify(&["LICENSE"], Some("GPL-3.0"));

    assert!(
        !verdict.is_fatal(),
        "aeruginous-rs itself declares the deprecated GPL-3.0: {:?}",
        verdict.problems
    );
}

#[test]
fn declaring_nothing_but_shipping_a_text_is_survivable() {
    let (verdict, _keep) = classify(&["LICENSE"], None);

    assert!(
        !verdict.is_fatal(),
        "the text is there to reproduce, so the reproduction obligation is \
         discharged even though the licence cannot be named: {:?}",
        verdict.problems
    );
    assert!(
        verdict
            .problems
            .iter()
            .any(|problem| matches!(problem, Problem::Custom { .. })),
        "but it is recorded as custom rather than passed over in silence"
    );
}

#[test]
fn an_unparsable_declaration_is_reported() {
    let (verdict, _keep) = classify(&["LICENSE"], Some("banana"));

    assert!(
        verdict
            .problems
            .iter()
            .any(|problem| matches!(problem, Problem::Unparsable { .. })),
        "{:?}",
        verdict.problems
    );
    assert!(verdict.attributions.is_empty(), "nothing can be reproduced");
}

#[test]
fn an_exception_matches_the_licence_it_qualifies() {
    let (verdict, _keep) = classify(
        &["LICENSE-Apache-2.0_WITH_LLVM-exception"],
        Some("Apache-2.0 WITH LLVM-exception"),
    );

    assert_eq!(verdict.attributions.len(), 1);
    assert!(
        matches!(
            verdict.attributions[0].provenance(),
            Provenance::Distributed(_)
        ),
        "the file names the exception, the term names the licence, and they \
         must still be recognised as the same thing"
    );
}

#[test]
fn a_notice_is_carried_alongside_rather_than_instead() {
    let (verdict, _keep) =
        classify(&["LICENSE-APACHE", "NOTICE"], Some("Apache-2.0"));

    assert_eq!(verdict.notices.len(), 1, "the NOTICE must survive");
    assert_eq!(
        verdict.attributions.len(),
        1,
        "and must not be mistaken for a licence text"
    );
}

#[test]
fn every_standard_text_licence_really_is_holder_independent() {
    use license::License;

    for identifier in [
        "Apache-2.0",
        "GPL-3.0-or-later",
        "LGPL-3.0-only",
        "AGPL-3.0-only",
        "EPL-2.0",
        "CC0-1.0",
        "BSL-1.0",
    ] {
        let licence: &dyn License = identifier
            .parse()
            .unwrap_or_else(|_| panic!("{identifier} must be a known licence"));
        let text = licence.text();
        let operative = &text[..text.len() * 9 / 10];

        for marker in ["<year>", "<owner>", "<copyright holders>", "[yyyy]"] {
            assert!(
                !operative.contains(marker),
                "{identifier} carries {marker} in its operative text, so its \
                 canonical form cannot discharge the obligation and it does \
                 not belong on the standard-text list"
            );
        }
    }
}

#[test]
fn a_general_file_beside_specific_ones_is_not_combined() {
    let (verdict, _keep) = classify(
        &["COPYING", "LICENSE-MIT", "UNLICENSE"],
        Some("Unlicense OR MIT"),
    );

    assert_eq!(
        verdict.coverage,
        Coverage::Complete,
        "six crates in the measured cache ship exactly this trio; each term \
         has its own text, so the presence of a general file does not make \
         anything shared"
    );
    assert!(
        verdict
            .attributions
            .iter()
            .all(|a| !matches!(a.provenance(), Provenance::Combined(_))),
        "nothing here leans on the general file:  {:?}",
        verdict.attributions
    );
}

/******************************************************************************/

#[test]
fn a_custom_licence_declared_by_file_is_reproduced() {
    let directory = tempfile::tempdir().expect("a temporary directory");
    let path = directory.path().join("LICENCE");
    fs::write(&path, "Bespoke terms, written by the copyright holder.\n")
        .expect("a fixture file");

    let mut described = package(directory.path(), None);
    described.licence_file = Some(path);
    described.name = "my_crate".to_owned();

    let evidence = Discovery::new().search(&described);
    let verdict = Classifier::new().classify(&described, &evidence);

    assert!(
        !verdict.is_fatal(),
        "a copyright holder may write their own terms, and Cargo's way of \
         saying so is license-file with no license: {:?}",
        verdict.problems
    );
    assert_eq!(verdict.attributions.len(), 1, "the text must be reproduced");
    assert_eq!(
        verdict.attributions[0].identifier(), "LicenseRef-my-crate",
        "an underscore is not admissible in an SPDX reference"
    );
    assert!(
        verdict.attributions[0].text().contains("Bespoke terms"),
        "and the text must be the author's own, not a canonical stand-in"
    );
    assert!(
        verdict
            .problems
            .iter()
            .any(|problem| matches!(problem, Problem::Custom { .. })),
        "recorded as custom, since no canonical text exists to check it against"
    );
}

#[test]
fn a_declared_licence_reference_is_reproduced() {
    let (verdict, _keep) =
        classify(&["LICENCE"], Some("LicenseRef-Acme-Proprietary"));

    assert!(!verdict.is_fatal(), "{:?}", verdict.problems);
    assert_eq!(verdict.attributions.len(), 1);
    assert_eq!(
        verdict.attributions[0].identifier(), "LicenseRef-Acme-Proprietary",
        "the reference the author chose must be preserved verbatim"
    );
    assert!(verdict.problems.iter().any(|problem| matches!(
        problem,
        Problem::Custom { identifier }
            if identifier == "LicenseRef-Acme-Proprietary"
    )));
}

#[test]
fn a_custom_licence_without_its_text_is_fatal() {
    let (verdict, _keep) = classify(&[], Some("LicenseRef-Acme-Proprietary"));

    assert!(
        verdict.is_fatal(),
        "no canonical text can ever exist for a custom licence, so the \
         distributed copy is the only possible source: {:?}",
        verdict.problems
    );
}

#[test]
fn a_custom_branch_does_not_spoil_a_standard_one() {
    let (verdict, _keep) =
        classify(&["LICENSE-MIT"], Some("MIT OR LicenseRef-Acme"));

    assert!(
        !verdict.is_fatal(),
        "the MIT branch is fully discharged by the shipped file:  {:?}",
        verdict.problems
    );
    assert_eq!(verdict.attributions[0].identifier(), "MIT");
}

#[test]
fn declaring_nothing_and_shipping_nothing_is_fatal() {
    let (verdict, _keep) = classify(&["README.md"], None);

    assert!(
        verdict.problems.contains(&Problem::Undeclared),
        "{:?}",
        verdict.problems
    );
    assert!(
        verdict.is_fatal(),
        "the obligation cannot even be identified, so nothing is reproduced \
         and there is nothing to object to — the quietest failure of all"
    );
}

#[test]
fn an_unparsable_declaration_is_fatal() {
    let (verdict, _keep) = classify(&["LICENSE"], Some("banana"));

    assert!(
        verdict.is_fatal(),
        "lenient parsing already accepts every real-world quirk, so what is \
         left is genuinely unreadable: {:?}",
        verdict.problems
    );
}

/******************************************************************************/
