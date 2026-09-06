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

//! The runtime DEP-5 renderer, over the embedded model alone.
//!
//! The build half has its own DEP-5 tests against a resolved graph; these
//! exercise the same output shape from the plain data a binary carries,
//! with no feature enabled.

use list_my_licence::{Attribution, Licence, Origin, Package};

static PROSE: [Licence; 1] = [Licence {
    identifier: "MIT",
    text: "First paragraph.\n\n.hidden line\nLast paragraph.\n",
    origin: Origin::Distributed("LICENSE-MIT"),
}];

static DUAL: [Licence; 2] = [
    Licence {
        identifier: "MIT",
        text: "MIT text.\n",
        origin: Origin::Canonical,
    },
    Licence {
        identifier: "Apache-2.0",
        text: "Apache text.\n",
        origin: Origin::Canonical,
    },
];

static UNDECLARED: [Licence; 0] = [];

static PACKAGES: [Package; 3] = [
    Package {
        name: "alpha",
        version: "1.0.0",
        licences: &PROSE,
        notices: &[],
    },
    Package {
        name: "beta",
        version: "2.3.4",
        licences: &DUAL,
        notices: &["Notice body.\n"],
    },
    Package {
        name: "gamma",
        version: "0.1.0",
        licences: &UNDECLARED,
        notices: &[],
    },
];

static ATTRIBUTION: Attribution = Attribution {
    packages: &PACKAGES,
};

#[test]
fn it_opens_with_the_format_stanza() {
    let rendered = ATTRIBUTION.dep5();

    assert!(
        rendered.starts_with("Format:")
            && rendered.contains(
                "https://www.debian.org/doc/packaging-manuals/\
                 copyright-format/1.0/"
            ),
        "a DEP-5 file is identified by its Format field:  {rendered}"
    );
}

#[test]
fn each_package_is_a_files_paragraph_keyed_by_its_directory() {
    let rendered = ATTRIBUTION.dep5();

    assert!(rendered.contains("\nFiles:"), "{rendered}");

    for keyed in [" alpha-1.0.0/*\n", " beta-2.3.4/*\n", " gamma-0.1.0/*\n"] {
        assert!(rendered.contains(keyed), "missing {keyed:?}:  {rendered}");
    }
}

#[test]
fn the_synopsis_joins_the_identifiers_and_admits_when_there_are_none() {
    let rendered = ATTRIBUTION.dep5();

    assert!(rendered.contains("\nLicense:"), "{rendered}");

    for synopsis in [" MIT\n", " MIT and Apache-2.0\n", " UNKNOWN\n"] {
        assert!(
            rendered.contains(synopsis),
            "missing {synopsis:?}:  {rendered}"
        );
    }
}

#[test]
fn the_copyright_field_states_the_absence_rather_than_guessing() {
    let rendered = ATTRIBUTION.dep5();

    assert!(
        rendered.contains("Copyright:")
            && rendered.contains(" not stated in the package manifest"),
        "the Copyright field is mandatory, and the embedded model has no \
         author data to fill it with:  {rendered}"
    );
}

#[test]
fn a_blank_line_folds_to_a_lone_full_stop() {
    assert!(
        ATTRIBUTION.dep5().contains("\n .\n"),
        "a blank line must fold to a lone full stop, or a control-file \
         parser reads the field as having ended"
    );
}

#[test]
fn a_line_that_would_start_with_a_full_stop_is_indented_twice() {
    assert!(
        ATTRIBUTION.dep5().contains("\n  .hidden line\n"),
        "an inner full stop at the margin would otherwise end the field"
    );
}

#[test]
fn a_notice_is_folded_under_its_own_heading() {
    let rendered = ATTRIBUTION.dep5();

    assert!(rendered.contains(" .\n NOTICE:\n .\n"), "{rendered}");
    assert!(rendered.contains(" Notice body.\n"), "{rendered}");
}

#[test]
fn every_line_is_a_field_or_a_continuation() {
    for line in ATTRIBUTION.dep5().lines() {
        assert!(
            line.is_empty()
                || line.starts_with(' ')
                || line.split_once(": ").is_some_and(|(key, _)| key
                    .chars()
                    .all(|c| c.is_ascii_alphanumeric() || c == '-')),
            "a bare text line leaked out of a folded field:  {line:?}"
        );
    }
}

#[test]
fn the_output_is_deterministic() {
    assert_eq!(
        ATTRIBUTION.dep5(),
        ATTRIBUTION.dep5(),
        "an unstable rendering would fail a committed-copy check at random"
    );
}

/******************************************************************************/
