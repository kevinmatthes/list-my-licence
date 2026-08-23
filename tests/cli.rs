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

//! Integration tests for the clap plumbing.
//!
//! The point of these types is that they *compose*:  an application keeps its
//! own parser and gains licence reporting, rather than surrendering the call
//! that parses its arguments.  Every test below therefore goes through a
//! derived `Parser`, since that is the arrangement being claimed.

#![cfg(all(feature = "clap", feature = "build"))]

use clap::{Parser, Subcommand};
use list_my_licence::{
    Attribution, Licence, Origin, Package,
    cli::{LicenceArgs, LicenceCommand},
};

static LICENCES: [Licence; 1] = [Licence {
    identifier: "MIT",
    text: "Permission is hereby granted.\n",
    origin: Origin::Distributed("LICENCE"),
}];

static OTHER: [Licence; 1] = [Licence {
    identifier: "Apache-2.0",
    text: "Permission is granted on the Apache terms.\n",
    origin: Origin::Canonical,
}];

static PACKAGES: [Package; 2] = [
    Package {
        name: "first",
        version: "1.0.0",
        licences: &LICENCES,
        notices: &[],
    },
    Package {
        name: "second",
        version: "2.0.0",
        licences: &OTHER,
        notices: &[],
    },
];

static ATTRIBUTION: Attribution = Attribution {
    packages: &PACKAGES,
};

/// An application's own parser, with licence reporting flattened in.
#[derive(Parser)]
struct Arguments {
    /// Something the application itself cares about.
    #[arg(long)]
    verbose: bool,

    #[command(flatten)]
    licences: LicenceArgs,
}

/// An application's own subcommands, with licence reporting flattened in.
#[derive(Parser)]
struct Commanded {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Something the application itself does.
    Run,

    #[command(flatten)]
    Licence(LicenceCommand),
}

/// Parses arguments as the application's own parser would.
fn parse(arguments: &[&str]) -> Arguments {
    Arguments::try_parse_from(
        std::iter::once("myapp").chain(arguments.iter().copied()),
    )
    .expect("the arguments must parse")
}

#[test]
fn the_application_keeps_its_own_options() {
    let arguments = parse(&["--verbose"]);

    assert!(
        arguments.verbose,
        "the application's own option must survive"
    );
    assert!(
        !arguments.licences.requested(),
        "and asking for nothing must request nothing"
    );
    assert_eq!(arguments.licences.render(&ATTRIBUTION), None);
}

#[test]
fn everything_is_shown_without_an_argument() {
    let rendered = parse(&["--licences"])
        .licences
        .render(&ATTRIBUTION)
        .expect("licences were requested");

    assert!(rendered.contains("first 1.0.0"), "{rendered}");
    assert!(rendered.contains("second 2.0.0"), "{rendered}");
    assert!(
        rendered.contains("Permission is hereby granted."),
        "{rendered}"
    );
}

#[test]
fn one_crate_is_shown_when_named() {
    let rendered = parse(&["--licences", "second"])
        .licences
        .render(&ATTRIBUTION)
        .expect("licences were requested");

    assert!(rendered.contains("second 2.0.0"), "{rendered}");
    assert!(
        !rendered.contains("first"),
        "naming a crate must exclude the others:  {rendered}"
    );
}

#[test]
fn an_unknown_crate_is_an_answer_rather_than_a_failure() {
    let rendered = parse(&["--licences", "absent"])
        .licences
        .render(&ATTRIBUTION)
        .expect("licences were requested");

    assert!(
        rendered.is_empty(),
        "a dependency that is not there has no licences to report:  \
         {rendered}"
    );
}

#[test]
fn the_option_composes_with_the_applications_own() {
    let arguments = parse(&["--verbose", "--licences", "first"]);

    assert!(arguments.verbose, "both must be accepted together");
    assert!(arguments.licences.requested());
}

#[test]
fn the_subcommand_form_shows_everything() {
    let parsed = Commanded::try_parse_from(["myapp", "licences"])
        .expect("the subcommand must parse");

    let Command::Licence(licence) = parsed.command else {
        panic!("the licences subcommand must be recognised");
    };

    let rendered = licence.render(&ATTRIBUTION);

    assert!(rendered.contains("first 1.0.0"), "{rendered}");
    assert!(rendered.contains("second 2.0.0"), "{rendered}");
}

#[test]
fn the_subcommand_form_accepts_a_crate() {
    let parsed = Commanded::try_parse_from(["myapp", "licences", "first"])
        .expect("the subcommand must parse");

    let Command::Licence(licence) = parsed.command else {
        panic!("the licences subcommand must be recognised");
    };

    let rendered = licence.render(&ATTRIBUTION);

    assert!(rendered.contains("first 1.0.0"), "{rendered}");
    assert!(!rendered.contains("second"), "{rendered}");
}

#[test]
fn the_applications_own_subcommand_still_works() {
    let parsed = Commanded::try_parse_from(["myapp", "run"])
        .expect("the subcommand must parse");

    assert!(
        matches!(parsed.command, Command::Run),
        "flattening ours must not displace the application's own"
    );
}

/******************************************************************************/
