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

//! Integration tests for dependency graph resolution.
//!
//! These resolve *this* crate's own graph, which makes them self-checking:
//! the expectations below are statements about `Cargo.toml` in the repository
//! root, and they fail if that manifest and this file drift apart.

#![cfg(feature = "build")]

use list_my_licence::build::Resolver;

/// Names of the resolved packages, for terse assertions.
fn resolve_names(resolver: &Resolver) -> Vec<String> {
    resolver
        .resolve()
        .expect("resolving this crate's own graph must succeed")
        .into_iter()
        .map(|package| package.name)
        .collect()
}

#[test]
fn resolves_own_shipping_dependencies() {
    let names = resolve_names(&Resolver::new().features(["build"]));

    for expected in ["cargo_metadata", "license", "spdx"] {
        assert!(
            names.iter().any(|name| name == expected),
            "{expected} is a dependency of the build feature and must \
             be resolved, got {names:?}"
        );
    }
}

#[test]
fn optional_dependencies_are_absent_without_their_feature() {
    let names = resolve_names(&Resolver::new().features(Vec::<String>::new()));

    for gated in ["cargo_metadata", "license", "spdx"] {
        assert!(
            !names.iter().any(|name| name == gated),
            "{gated} is gated behind the build feature and must not \
             be resolved when that feature is off, got {names:?}"
        );
    }
}

#[test]
fn feature_selection_changes_the_resolved_set() {
    let without =
        resolve_names(&Resolver::new().features(Vec::<String>::new()));
    let with = resolve_names(&Resolver::new().features(["build"]));

    assert!(
        with.len() > without.len(),
        "enabling a feature that pulls optional dependencies must widen \
         the graph; without = {without:?}, with = {with:?}"
    );
}

#[test]
fn excludes_dev_dependencies() {
    let names = resolve_names(&Resolver::new());

    assert!(
        !names.iter().any(|name| name == "tempfile"),
        "tempfile is a dev-dependency and never ships, so it must not \
         be resolved, got {names:?}"
    );
}

#[test]
fn includes_the_root_package_by_default() {
    let names = resolve_names(&Resolver::new());

    assert!(
        names.iter().any(|name| name == "list-my-licence"),
        "an application must reproduce its own licence too, got {names:?}"
    );
}

#[test]
fn root_can_be_excluded() {
    let names = resolve_names(&Resolver::new().include_root(false));

    assert!(
        !names.iter().any(|name| name == "list-my-licence"),
        "include_root(false) must drop the root package, got {names:?}"
    );
}

#[test]
fn result_is_sorted_and_deduplicated() {
    let packages = Resolver::new().resolve().expect("resolution must succeed");

    let mut sorted = packages.clone();
    sorted.sort();
    assert_eq!(packages, sorted, "the result must be sorted");

    let mut deduplicated = packages.clone();
    deduplicated.dedup();
    assert_eq!(
        packages, deduplicated,
        "the result must contain no duplicates"
    );
}

#[test]
fn every_package_has_a_manifest_directory_that_exists() {
    for package in Resolver::new().resolve().expect("resolution must succeed") {
        assert!(
            package.manifest_dir.is_dir(),
            "{} {} points at {}, which is not a directory; \
             licence discovery starts there and would find nothing",
            package.name,
            package.version,
            package.manifest_dir.display()
        );
    }
}

#[test]
fn a_missing_manifest_is_reported_rather_than_panicking() {
    let error = Resolver::new()
        .manifest_path("/nonexistent/Cargo.toml")
        .resolve()
        .expect_err("a manifest that does not exist cannot resolve");

    assert!(
        matches!(error, list_my_licence::build::ResolveError::Metadata(_)),
        "expected a metadata error, got {error:?}"
    );
}

#[test]
fn resolves_against_the_host_triple() {
    let rustc = std::process::Command::new("rustc")
        .arg("-vV")
        .output()
        .expect("rustc must be runnable to learn the host triple");
    let verbose =
        String::from_utf8(rustc.stdout).expect("rustc -vV emits UTF-8");
    let host = verbose
        .lines()
        .find_map(|line| line.strip_prefix("host: "))
        .expect("rustc -vV reports a host triple");

    let filtered =
        resolve_names(&Resolver::new().features(["build"]).target(host));
    let unfiltered = resolve_names(&Resolver::new().features(["build"]));

    assert!(
        !filtered.is_empty(),
        "filtering by the host triple must not empty the graph"
    );
    assert!(
        filtered.len() <= unfiltered.len(),
        "filtering by a platform can only narrow the graph; \
         filtered = {filtered:?}, unfiltered = {unfiltered:?}"
    );
    assert!(
        filtered.iter().any(|name| name == "cargo_metadata"),
        "a dependency of every platform must survive filtering, \
         got {filtered:?}"
    );
}

/******************************************************************************/
