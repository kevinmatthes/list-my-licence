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

//! Integration tests for the copyleft survey.
//!
//! The survey exists to make an unexpected obligation impossible to miss, in a
//! dependency graph the author keeps deliberately permissive.  Its accuracy
//! therefore matters most in the negative direction:  a warning that fires on
//! a crate imposing nothing is the surest way to have a real one ignored.

#![cfg(feature = "build")]

use list_my_licence::build::{
    Classifier, Copyleft, Discovery, ResolvedPackage, Strength,
};
use std::{fs, path::Path};

/// Builds a package declaring `licence`, rooted at `directory`.
fn package(directory: &Path, licence: &str) -> ResolvedPackage {
    ResolvedPackage {
        name: "fixture".to_owned(),
        version: "1.2.3".to_owned(),
        manifest_dir: directory.to_path_buf(),
        licence: Some(licence.to_owned()),
        licence_file: None,
        authors: Vec::new(),
        repository: Some("https://example.invalid/fixture".to_owned()),
    }
}

/// Surveys one package built from the named files and declaration.
fn survey(
    names: &[&str],
    licence: &str,
) -> (list_my_licence::build::Survey, tempfile::TempDir) {
    let directory = tempfile::tempdir().expect("a temporary directory");

    for name in names {
        fs::write(directory.path().join(name), "licence text\n")
            .expect("a fixture file");
    }

    let described = package(directory.path(), licence);
    let evidence = Discovery::new().search(&described);
    let verdict = Classifier::new().classify(&described, &evidence);
    let mut survey = Copyleft::new().survey();
    survey.add(&described, &verdict);

    (survey, directory)
}

#[test]
fn the_gpl_family_is_told_apart() {
    assert_eq!(Strength::of("AGPL-3.0-only"), Strength::Network);
    assert_eq!(Strength::of("LGPL-3.0-or-later"), Strength::Library);
    assert_eq!(Strength::of("GPL-3.0-or-later"), Strength::Strong);
    assert_eq!(
        Strength::of("GPL-3.0"),
        Strength::Strong,
        "the deprecated bare form still occurs, and aeruginous-rs uses it"
    );
    assert_eq!(Strength::of("MIT"), Strength::Permissive);
    assert_eq!(Strength::of("MPL-2.0"), Strength::Weak);
}

#[test]
fn the_narrower_prefixes_win() {
    assert_ne!(
        Strength::of("AGPL-3.0-only"),
        Strength::Strong,
        "AGPL ends in GPL, and misreading it as merely strong would hide the \
         one obligation that reaches users who receive no binary at all"
    );
    assert_ne!(
        Strength::of("LGPL-3.0-only"),
        Strength::Strong,
        "LGPL ends in GPL too, and its obligation is a different one"
    );
}

#[test]
fn a_permissive_graph_produces_no_warning() {
    let (survey, _keep) = survey(&["LICENSE-MIT"], "MIT");

    assert!(survey.is_clear(), "{:?}", survey.findings);
    assert_eq!(survey.warnings().count(), 0);
}

#[test]
fn an_unused_copyleft_branch_does_not_warn() {
    let (survey, _keep) = survey(&["LICENSE-MIT"], "MIT OR GPL-3.0-or-later");

    assert!(
        survey.is_clear(),
        "nobody taking the MIT branch owes anything under the GPL, and crying \
         wolf here is the surest way to have a real warning ignored: {:?}",
        survey.findings
    );
}

#[test]
fn a_relied_upon_copyleft_branch_does_warn() {
    let (survey, _keep) = survey(&["LICENSE"], "GPL-3.0-or-later");

    assert_eq!(survey.findings.len(), 1);
    assert_eq!(survey.strongest(), Some(Strength::Strong));
    assert_eq!(survey.undischargeable().count(), 1);
}

#[test]
fn weak_copyleft_is_discharged_by_the_pointer() {
    let (survey, _keep) = survey(&["LICENSE"], "MPL-2.0");

    assert_eq!(survey.findings.len(), 1, "it is still reported");
    assert_eq!(
        survey.undischargeable().count(),
        0,
        "but MPL-2.0 section 3.2 asks only that recipients be told where the \
         source is, which the pointer does"
    );
    assert_eq!(
        survey.findings[0].source.as_deref(),
        Some("https://example.invalid/fixture at version 1.2.3"),
        "and the pointer names the exact version, not whatever the default \
         branch holds today"
    );
}

#[test]
fn the_report_says_what_a_package_declares() {
    let (survey, _keep) = survey(&["LICENSE"], "AGPL-3.0-only");
    let warning = survey.warnings().next().expect("a warning");

    assert!(
        warning.contains("declares"),
        "a `license` field is a claim its author made, never a fact:  {warning}"
    );
    assert!(
        warning.contains("network"),
        "and the obligation must be named, not merely hinted at:  {warning}"
    );
}

#[test]
fn the_strongest_obligation_is_reported() {
    let directory = tempfile::tempdir().expect("a temporary directory");
    fs::write(directory.path().join("LICENSE"), "text\n").expect("a file");

    let mut survey = Copyleft::new().survey();

    for licence in ["MPL-2.0", "AGPL-3.0-only", "LGPL-3.0-only"] {
        let described = package(directory.path(), licence);
        let evidence = Discovery::new().search(&described);
        let verdict = Classifier::new().classify(&described, &evidence);
        survey.add(&described, &verdict);
    }

    assert_eq!(
        survey.strongest(),
        Some(Strength::Network),
        "network copyleft reaches furthest and must not be masked by the rest"
    );
    assert_eq!(survey.findings.len(), 3);
}

#[test]
fn only_weak_and_permissive_are_dischargeable() {
    assert!(Strength::Permissive.is_dischargeable());
    assert!(Strength::Weak.is_dischargeable());
    assert!(
        !Strength::Library.is_dischargeable(),
        "static linking removes the shared-library route the LGPL offers"
    );
    assert!(!Strength::Strong.is_dischargeable());
    assert!(!Strength::Network.is_dischargeable());
}

#[test]
fn a_conjunction_leaves_no_branch_to_elect() {
    let (survey, _keep) = survey(&["LICENSE"], "MIT AND GPL-3.0-or-later");

    assert_eq!(
        survey.findings.len(),
        1,
        "AND offers no choice, so the GPL obligation attaches however the \
         permissive half is treated: {:?}",
        survey.findings
    );
    assert_eq!(survey.strongest(), Some(Strength::Strong));
}

/******************************************************************************/
