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

//! Detecting copyleft obligations this crate cannot discharge.
//!
//! Reproducing a licence text discharges the reproduction obligation and
//! nothing else.  Some licences ask for more:  the corresponding source of the
//! whole work, the ability to relink against a modified library, an offer to
//! users reaching the software over a network.  None of that is expressible as
//! text embedded in a binary, and this module does not pretend otherwise.
//!
//! What it does is refuse to be silent about it.  Every licence actually
//! relied upon is classified by how far its obligations reach, and anything
//! beyond mere reproduction is reported so that a human can act on it.  A tool
//! that quietly discharged the easy half of an obligation would be worse than
//! no tool.
//!
//! Two things it deliberately is not.
//!
//! It is **not a policy engine**.  Deciding which licences a project will
//! accept is `cargo-deny`'s work, done well and at scale;  duplicating it here
//! would add a second, worse answer to a settled question.
//!
//! It is **not an oracle**.  A `license` field is a claim its author made, not
//! a fact — regularly stale, occasionally wrong, and sometimes contradicted by
//! the files beside it.  Every report below therefore says what a package
//! *declares*, never what it is.

use super::{Classification, ResolvedPackage};
use std::{collections::BTreeSet, fmt};

/// Licences whose copyleft reaches only the files they cover.
const WEAK: [&str; 8] = [
    "CDDL-1.0", "CDDL-1.1", "CPL-1.0", "EPL-1.0", "EPL-2.0", "MPL-1.1",
    "MPL-2.0", "MS-PL",
];

/// How far a licence's obligations reach beyond reproducing its text.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum Strength {
    /// Nothing is required beyond reproduction, which this crate does.
    Permissive,

    /// Copyleft confined to the covered files.  Discharged by telling
    /// recipients where that source is, which is a pointer this crate can
    /// supply.
    Weak,

    /// Copyleft over a library, conditioned on the recipient being able to
    /// relink against a modified version of it.
    ///
    /// Rust links statically by default, which removes the shared-library
    /// route the LGPL offers and leaves only the other one:  publishing object
    /// files, or the application's own source.  Nothing embedded in a binary
    /// can satisfy that.
    Library,

    /// Copyleft over the whole work.  The complete corresponding source must
    /// be offered to whoever receives the binary.
    Strong,

    /// Copyleft reaching users who never receive a binary at all, but interact
    /// with the software over a network.
    ///
    /// Decisive for a server application, and invisible to any tool that
    /// reasons only about shipped artefacts.
    Network,
}

impl Strength {
    /// What the licence asks for beyond reproducing its text.
    #[must_use]
    pub const fn obligation(self) -> &'static str {
        match self {
            Self::Permissive => "nothing beyond the reproduction already done",
            Self::Weak => {
                "recipients must be told where the source of the covered files \
                 is;  the pointer below does that"
            }
            Self::Library => {
                "recipients must be able to relink against a modified library, \
                 which static linking makes impossible to satisfy with text \
                 alone"
            }
            Self::Strong => {
                "the complete corresponding source of the whole work must be \
                 offered to whoever receives the binary"
            }
            Self::Network => {
                "users interacting over a network must be offered the \
                 corresponding source, even if no binary is distributed"
            }
        }
    }

    /// Whether this crate can discharge the obligation on its own.
    #[must_use]
    pub const fn is_dischargeable(self) -> bool {
        matches!(self, Self::Permissive | Self::Weak)
    }

    /// Classifies one SPDX identifier.
    ///
    /// The GPL family is matched by prefix rather than by an exhaustive list,
    /// because SPDX carries `-only` and `-or-later` variants of each version
    /// alongside the deprecated bare forms, and real manifests use all of
    /// them.  Order matters:  `AGPL` and `LGPL` both end in `GPL`, so the
    /// narrower prefixes are tested first.
    #[must_use]
    pub fn of(identifier: &str) -> Self {
        if identifier.starts_with("AGPL-") {
            Self::Network
        } else if identifier.starts_with("LGPL-") {
            Self::Library
        } else if identifier.starts_with("GPL-") {
            Self::Strong
        } else if WEAK.contains(&identifier) {
            Self::Weak
        } else {
            Self::Permissive
        }
    }
}

impl fmt::Display for Strength {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Permissive => "permissive",
            Self::Weak => "weak copyleft",
            Self::Library => "library copyleft",
            Self::Strong => "strong copyleft",
            Self::Network => "network copyleft",
        })
    }
}

/// One dependency carrying an obligation beyond reproduction.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct Finding {
    /// The package that declares it.
    pub package: String,

    /// Its exact version, which the source pointer needs.
    pub version: String,

    /// The licence relied upon.
    pub identifier: String,

    /// How far its obligations reach.
    pub strength: Strength,

    /// Where the covered source can be obtained, when the manifest says.
    ///
    /// This discharges MPL-2.0 §3.2 by itself.  For the stronger licences it
    /// is an ingredient rather than compliance:  knowing where upstream lives
    /// is not the same as offering the corresponding source of *your* work.
    pub source: Option<String>,
}

impl fmt::Display for Finding {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} {} declares {} ({}):  {}",
            self.package,
            self.version,
            self.identifier,
            self.strength,
            self.strength.obligation(),
        )?;

        if let Some(source) = &self.source {
            write!(f, "  Source:  {source}")?;
        }

        Ok(())
    }
}

/// What a survey of the dependency graph turned up.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Survey {
    /// Every dependency carrying an obligation beyond reproduction, sorted.
    pub findings: Vec<Finding>,
}

impl Survey {
    /// Whether the graph is free of obligations beyond reproduction.
    #[must_use]
    pub const fn is_clear(&self) -> bool {
        self.findings.is_empty()
    }

    /// The findings this crate cannot discharge on its own.
    pub fn undischargeable(&self) -> impl Iterator<Item = &Finding> {
        self.findings
            .iter()
            .filter(|finding| !finding.strength.is_dischargeable())
    }

    /// The furthest-reaching obligation present, if any.
    #[must_use]
    pub fn strongest(&self) -> Option<Strength> {
        self.findings.iter().map(|finding| finding.strength).max()
    }

    /// Lines for a build script to emit as `cargo::warning=` messages.
    ///
    /// Phrased as what each package *declares*, because that is all a manifest
    /// can tell anyone.
    pub fn warnings(&self) -> impl Iterator<Item = String> {
        self.findings.iter().map(ToString::to_string)
    }
}

/// Surveys a dependency graph for obligations beyond reproduction.
///
/// # Examples
///
/// ```no_run
/// # use list_my_licence::build::{
/// #     Classifier, Copyleft, Discovery, Resolver,
/// # };
/// let mut survey = Copyleft::new().survey();
///
/// for package in Resolver::from_build_env()?.resolve()? {
///     let evidence = Discovery::new().search(&package);
///     let verdict = Classifier::new().classify(&package, &evidence);
///
///     survey.add(&package, &verdict);
/// }
///
/// for warning in survey.warnings() {
///     println!("cargo::warning={warning}");
/// }
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[derive(Clone, Copy, Debug, Default)]
pub struct Copyleft {
    _private: (),
}

impl Copyleft {
    /// A survey with the default settings.
    #[must_use]
    pub const fn new() -> Self {
        Self { _private: () }
    }

    /// An empty survey, to be filled package by package.
    #[must_use]
    pub fn survey(&self) -> Survey {
        Survey::default()
    }
}

impl Survey {
    /// Adds one package's verdict to the survey.
    ///
    /// The licences examined are the ones the classification actually *relied
    /// upon*, not everything the manifest names.  A crate offering
    /// `MIT OR GPL-3.0` imposes no copyleft on anyone who takes the MIT
    /// branch, and warning about it would be crying wolf — the surest way to
    /// have a real warning ignored later.
    pub fn add(&mut self, package: &ResolvedPackage, verdict: &Classification) {
        let discharged: BTreeSet<&str> = verdict
            .attributions
            .iter()
            .map(|attribution| attribution.identifier.as_str())
            .collect();

        let expression = package.licence.as_deref().and_then(|declared| {
            spdx::Expression::parse_mode(declared, spdx::ParseMode::LAX).ok()
        });

        for attribution in &verdict.attributions {
            let strength = Strength::of(&attribution.identifier);

            if strength == Strength::Permissive {
                continue;
            }

            if Self::avoidable(
                expression.as_ref(),
                &discharged,
                &attribution.identifier,
            ) {
                continue;
            }

            self.findings.push(Finding {
                package: package.name.clone(),
                version: package.version.clone(),
                identifier: attribution.identifier.clone(),
                strength,
                source: Self::source(package),
            });
        }

        self.findings.sort();
        self.findings.dedup();
    }

    /// Whether the declaration is satisfied without relying on this licence.
    ///
    /// Step three discharges every branch it can, which over-reproduces
    /// harmlessly.  An obligation, though, attaches only to the branch
    /// actually relied upon:  nobody electing the `MIT` half of
    /// `MIT OR GPL-3.0-or-later` owes anything under the GPL.  The question is
    /// therefore whether the expression still holds once this licence is
    /// struck from what is available — and if it does, there is nothing to
    /// warn about.
    fn avoidable(
        expression: Option<&spdx::Expression>,
        discharged: &BTreeSet<&str>,
        identifier: &str,
    ) -> bool {
        expression.is_some_and(|expression| {
            expression.evaluate(|requirement| {
                let name = requirement.license.id().map_or_else(
                    || requirement.license.to_string(),
                    |id| id.name.to_owned(),
                );

                name != identifier && discharged.contains(name.as_str())
            })
        })
    }

    /// Where the covered source can be obtained.
    ///
    /// The repository the manifest names, qualified by the exact version, so
    /// that the pointer identifies the code actually used rather than whatever
    /// the default branch holds today.
    fn source(package: &ResolvedPackage) -> Option<String> {
        package.repository.as_ref().map(|repository| {
            format!("{repository} at version {}", package.version)
        })
    }
}

/******************************************************************************/
