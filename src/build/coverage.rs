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

//! Deciding whether the evidence covers the declaration.
//!
//! [`Discovery`](super::Discovery) gathers the licence files a package ships;
//! this module judges whether they discharge what its manifest declares, and
//! fills what gaps it honestly can from the canonical SPDX texts.
//!
//! Two rules govern the judgement.
//!
//! **The number of files and the number of licence terms routinely disagree.**
//! A crate declaring `MIT OR Apache-2.0` may ship two files named after the
//! two licences, or one combined file holding both, or nothing at all.
//! Reporting a missing licence for a crate that plainly ships one would
//! destroy trust in the output, so the combined case is recognised rather than
//! mistaken for a gap.
//!
//! **A canonical text does not always discharge the obligation.**  MIT, BSD and
//! ISC require *the* copyright line, which no canonical text carries;
//! substituting one would reproduce a licence with an empty
//! `Copyright (c) <year> <holders>` and satisfy nobody.  Apache-2.0, the GPL
//! family and their kin have holder-independent bodies, so for them the
//! canonical text genuinely suffices.  Only the first kind can make a build
//! fail.

use super::{Evidence, Found, ResolvedPackage, Skipped};
use license::License;
use std::{collections::BTreeSet, fmt, path::PathBuf};

/// Licences whose canonical text discharges the obligation on its own.
///
/// Every entry has a holder-independent body:  reproducing it verbatim from
/// the SPDX list says everything the licence requires, with no copyright line
/// to recover.
///
/// **Anything not listed here is treated as requiring the distributed copy.**
/// That default is deliberate and errs towards demanding more rather than
/// less:  an unlisted licence that would in fact have been satisfied by its
/// canonical text costs a build failure, whereas the reverse mistake ships an
/// unattributed dependency.  A curated list is used rather than a heuristic
/// because no reliable signal exists — `ISC`, `Zlib` and `Unicode-3.0` all
/// need the copyright line yet carry no placeholder in their canonical text
/// at all, so a placeholder test would clear them wrongly.
const STANDARD_TEXT: [&str; 17] = [
    "AGPL-3.0-only",
    "AGPL-3.0-or-later",
    "Apache-2.0",
    "Artistic-2.0",
    "BSL-1.0",
    "CC0-1.0",
    "CDDL-1.0",
    "EPL-1.0",
    "EPL-2.0",
    "EUPL-1.2",
    "GPL-2.0-only",
    "GPL-2.0-or-later",
    "GPL-3.0-only",
    "GPL-3.0-or-later",
    "LGPL-2.1-or-later",
    "LGPL-3.0-only",
    "LGPL-3.0-or-later",
];

/// Where a reproduced text came from.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum Provenance {
    /// The copy the author actually distributed, which is what a licence
    /// requiring its own copyright line needs.
    Distributed(PathBuf),

    /// One file covering several licence terms at once.
    Combined(PathBuf),

    /// The canonical SPDX text, used because the package shipped none.
    Canonical,
}

/// One licence of a package, with the text to be reproduced for it.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct Attribution {
    /// The SPDX identifier this text stands for.
    pub identifier: String,

    /// The text itself.
    pub text: String,

    /// Where it came from, recorded so that the output can say so.
    pub provenance: Provenance,
}

/// How completely the shipped files cover the declared expression.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum Coverage {
    /// Every declared term matched a file the package ships.
    Complete,

    /// One general file covers several declared terms, as `chrono` does.
    Combined,

    /// Some terms were filled from the canonical texts.
    Partial,

    /// The package ships no licence file at all.
    Absent,
}

/// Something standing between a package and a discharged obligation.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum Problem {
    /// The manifest declares no licence, and the package ships no text
    /// either.
    ///
    /// The most dangerous case of all, because it fails silently:  the
    /// obligation cannot even be identified, so nothing is reproduced and
    /// there is nothing to object to.
    Undeclared,

    /// The licence is a custom one:  either declared as an SPDX `LicenseRef`,
    /// or not declared at all while a text is nonetheless shipped.
    ///
    /// Not a fault.  A copyright holder is entitled to write their own terms,
    /// and this crate's business is reproducing them faithfully, not insisting
    /// they be drawn from a list.  It is recorded so that the output can say
    /// the licence was not identifiable, since no canonical text exists to
    /// check it against.
    Custom {
        /// What the licence is called, as declared or as invented for it.
        identifier: String,
    },

    /// The declared expression could not be parsed, even leniently.
    Unparsable {
        /// The expression as written.
        expression: String,
    },

    /// A licence needing its own copyright line has no distributed text, so
    /// the canonical text cannot stand in for it.
    Unattributed {
        /// The licence that cannot be discharged.
        identifier: String,
    },

    /// No combination of what is available satisfies the declared expression.
    Unsatisfiable {
        /// The expression as written.
        expression: String,
    },

    /// A file was found but could not be read.
    Unreadable {
        /// Where it is.
        path: PathBuf,

        /// Why it was refused.
        reason: Skipped,
    },
}

impl Problem {
    /// Whether this must fail the build.
    ///
    /// Only two things do.  An unattributed notice-style licence cannot be
    /// discharged at all, and an unsatisfiable expression means nothing
    /// available covers what was declared.  Everything else is reported and
    /// survivable:  a canonical text stood in, or the package said nothing and
    /// the human must look.
    #[must_use]
    pub const fn is_fatal(&self) -> bool {
        matches!(
            self,
            Self::Undeclared
                | Self::Unparsable { .. }
                | Self::Unattributed { .. }
                | Self::Unsatisfiable { .. }
        )
    }
}

impl fmt::Display for Problem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Undeclared => {
                f.write_str("declares no licence and ships no licence text")
            }
            Self::Custom { identifier } => {
                write!(f, "is under the custom licence {identifier}")
            }
            Self::Unparsable { expression } => {
                write!(
                    f,
                    "declares {expression:?}, which is not a licence expression"
                )
            }
            Self::Unattributed { identifier } => write!(
                f,
                "ships no {identifier} text, and {identifier} requires its own \
                 copyright line, which the canonical text cannot supply"
            ),
            Self::Unsatisfiable { expression } => {
                write!(
                    f,
                    "declares {expression}, which nothing available satisfies"
                )
            }
            Self::Unreadable { path, reason } => {
                write!(f, "{} {reason}", path.display())
            }
        }
    }
}

/// What one package's licences amount to.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Classification {
    /// How completely the shipped files cover the declaration.
    pub coverage: Coverage,

    /// The texts to reproduce, one per discharged licence term.
    pub attributions: Vec<Attribution>,

    /// Apache-2.0 notices, which are reproduced alongside rather than instead.
    pub notices: Vec<Found>,

    /// Everything that stood in the way, fatal or not.
    pub problems: Vec<Problem>,
}

impl Classification {
    /// Whether anything here must fail the build.
    #[must_use]
    pub fn is_fatal(&self) -> bool {
        self.problems.iter().any(Problem::is_fatal)
    }
}

/// Judges evidence against a declaration.
///
/// # Examples
///
/// ```no_run
/// # use list_my_licence::build::{Classifier, Discovery, Resolver};
/// for package in Resolver::from_build_env()?.resolve()? {
///     let evidence = Discovery::new().search(&package);
///     let verdict = Classifier::new().classify(&package, &evidence);
///
///     if verdict.is_fatal() {
///         for problem in &verdict.problems {
///             println!("{} {problem}", package.name);
///         }
///     }
/// }
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[derive(Clone, Copy, Debug, Default)]
pub struct Classifier {
    _private: (),
}

impl Classifier {
    /// A classifier with the default settings.
    #[must_use]
    pub const fn new() -> Self {
        Self { _private: () }
    }

    /// Judges one package.
    #[must_use]
    pub fn classify(
        &self,
        package: &ResolvedPackage,
        evidence: &Evidence,
    ) -> Classification {
        let mut problems: Vec<Problem> = evidence
            .skipped
            .iter()
            .map(|(path, reason)| Problem::Unreadable {
                path: path.clone(),
                reason: reason.clone(),
            })
            .collect();

        let notices = evidence.notices().cloned().collect();

        let Some(expression) =
            Self::declaration(package, evidence, &mut problems)
        else {
            return Classification {
                coverage: Coverage::Absent,
                attributions: Vec::new(),
                notices,
                problems,
            };
        };

        let (mut attributions, discharged, combined) =
            Self::attribute(&Self::terms(&expression), evidence, &mut problems);

        if combined {
            Self::mark_combined(&mut attributions);
        }

        // An `OR` needs only one of its branches.  A term that could not be
        // discharged therefore matters only if the expression cannot be
        // satisfied without it, which is why this is checked against the whole
        // expression rather than term by term.
        let satisfied = expression.evaluate(|requirement| {
            discharged.contains(&Self::name(&requirement.license))
        });

        if satisfied {
            problems.retain(|problem| {
                !matches!(problem, Problem::Unattributed { .. })
            });
        } else if !problems.iter().any(Problem::is_fatal) {
            problems.push(Problem::Unsatisfiable {
                expression: package.licence.clone().unwrap_or_default(),
            });
        }

        problems.sort();
        problems.dedup();

        Classification {
            coverage: Self::coverage(evidence, &attributions, combined),
            attributions,
            notices,
            problems,
        }
    }

    /// Parses the package's declaration, recording why if it cannot.
    ///
    /// Real manifests still carry the deprecated slash form and deprecated
    /// identifiers, so leniency here is a requirement rather than a
    /// convenience:  four of the twelve licence-less crates measured in the
    /// author's own cache write `MIT/Apache-2.0`.
    fn declaration(
        package: &ResolvedPackage,
        evidence: &Evidence,
        problems: &mut Vec<Problem>,
    ) -> Option<spdx::Expression> {
        let synthesised;

        let declared = match package.licence.as_deref() {
            Some(declared) => declared,

            // Cargo's own way of saying "my terms are not on any list":  set
            // `license-file` and leave `license` empty.  A copyright holder is
            // entitled to write their own licence, so this is not a fault —
            // but SPDX has no identifier for it, and one is needed before the
            // rest of the classifier can treat it like any other term.  The
            // reference SPDX reserves for exactly this purpose is invented
            // from the package's own name.
            None if !evidence.is_empty() || package.licence_file.is_some() => {
                synthesised = Self::reference(&package.name);

                problems.push(Problem::Custom {
                    identifier: synthesised.clone(),
                });

                &synthesised
            }

            None => {
                problems.push(Problem::Undeclared);

                return None;
            }
        };

        let parsed =
            spdx::Expression::parse_mode(declared, spdx::ParseMode::LAX).ok();

        match &parsed {
            None => problems.push(Problem::Unparsable {
                expression: declared.to_owned(),
            }),

            // A declared `LicenseRef` is custom too, and is noted for the same
            // reason:  no canonical text exists to check it against.
            Some(expression) => problems.extend(
                expression
                    .requirements()
                    .filter(|requirement| {
                        requirement.req.license.id().is_none()
                    })
                    .map(|requirement| Problem::Custom {
                        identifier: Self::name(&requirement.req.license),
                    }),
            ),
        }

        parsed
    }

    /// Invents the SPDX reference for a package's own custom licence.
    ///
    /// An SPDX reference admits letters, digits, full stops and hyphens only,
    /// so anything else in the package name — an underscore, most commonly —
    /// becomes a hyphen.
    fn reference(name: &str) -> String {
        let sanitised: String = name
            .chars()
            .map(|character| {
                if character.is_ascii_alphanumeric() || character == '.' {
                    character
                } else {
                    '-'
                }
            })
            .collect();

        format!("LicenseRef-{sanitised}")
    }

    /// Finds a text for every declared term.
    ///
    /// Returns the attributions, the terms genuinely discharged, and whether
    /// more than one term leaned on the same general file.
    fn attribute(
        terms: &[String],
        evidence: &Evidence,
        problems: &mut Vec<Problem>,
    ) -> (Vec<Attribution>, BTreeSet<String>, bool) {
        let general = Self::general_files(evidence);

        let mut attributions = Vec::new();
        let mut discharged = BTreeSet::new();

        // Which attributions fell back to a general file.  A file is only
        // *combined* if more than one term actually leans on it;  a package
        // shipping `COPYING` beside `LICENSE-MIT` and `UNLICENSE` has a
        // general file, but each term still has its own text.
        let mut leaning = Vec::new();

        for term in terms {
            let (text, provenance) =
                if let Some(file) = Self::specific(evidence, term) {
                    (
                        file.text.clone(),
                        Provenance::Distributed(file.path.clone()),
                    )
                } else if let Some(file) = general.first() {
                    leaning.push(attributions.len());

                    (
                        file.text.clone(),
                        Provenance::Distributed(file.path.clone()),
                    )
                } else if let Some(text) = Self::canonical(term) {
                    (text, Provenance::Canonical)
                } else {
                    problems.push(Problem::Unattributed {
                        identifier: term.clone(),
                    });

                    continue;
                };

            // A canonical text discharges nothing for a licence that needs its
            // own copyright line.
            if provenance == Provenance::Canonical
                && !Self::is_standard_text(term)
            {
                problems.push(Problem::Unattributed {
                    identifier: term.clone(),
                });

                continue;
            }

            discharged.insert(term.clone());
            attributions.push(Attribution {
                identifier: term.clone(),
                text,
                provenance,
            });
        }

        let combined = leaning.len() > 1;

        (attributions, discharged, combined)
    }

    /// Restates shared general files as such.
    fn mark_combined(attributions: &mut [Attribution]) {
        let shared: Vec<PathBuf> = attributions
            .iter()
            .filter_map(|attribution| match &attribution.provenance {
                Provenance::Distributed(path) => Some(path.clone()),
                _ => None,
            })
            .collect();

        for attribution in attributions.iter_mut() {
            if let Provenance::Distributed(path) =
                attribution.provenance.clone()
                && shared.iter().filter(|other| **other == path).count() > 1
            {
                attribution.provenance = Provenance::Combined(path);
            }
        }
    }

    /// The licence identifiers a parsed expression names, in order.
    ///
    /// A custom licence carries no SPDX identifier, so it is named by its
    /// reference instead — `LicenseRef-Whatever`.  Naming it is what lets the
    /// rest of the classifier treat it like any other term.
    fn terms(expression: &spdx::Expression) -> Vec<String> {
        let mut terms: Vec<String> = expression
            .requirements()
            .map(|requirement| Self::name(&requirement.req.license))
            .collect();

        terms.dedup();
        terms
    }

    /// What one licence item of an expression is called.
    fn name(item: &spdx::LicenseItem) -> String {
        item.id()
            .map_or_else(|| item.to_string(), |id| id.name.to_owned())
    }

    /// The shipped file naming a given licence, if there is one.
    fn specific<'a>(evidence: &'a Evidence, term: &str) -> Option<&'a Found> {
        evidence.licences().find(|file| {
            file.identifier.as_deref().is_some_and(|identifier| {
                identifier == term
                    || spdx::Expression::parse_mode(
                        identifier,
                        spdx::ParseMode::LAX,
                    )
                    .is_ok_and(|parsed| {
                        parsed
                            .requirements()
                            .filter_map(|r| r.req.license.id())
                            .any(|id| id.name == term)
                    })
            })
        })
    }

    /// The shipped licence files that name no particular licence.
    fn general_files(evidence: &Evidence) -> Vec<&Found> {
        evidence
            .licences()
            .filter(|file| file.identifier.is_none())
            .collect()
    }

    /// The canonical SPDX text of a licence, if the list carries one.
    fn canonical(identifier: &str) -> Option<String> {
        identifier
            .parse::<&dyn License>()
            .ok()
            .map(|licence| licence.text().to_owned())
    }

    /// Whether a licence's canonical text discharges it without a copyright
    /// line of its own.
    fn is_standard_text(identifier: &str) -> bool {
        STANDARD_TEXT.contains(&identifier)
    }

    /// How completely the evidence covered the declaration.
    fn coverage(
        evidence: &Evidence,
        attributions: &[Attribution],
        combined: bool,
    ) -> Coverage {
        if evidence.licences().next().is_none() {
            return Coverage::Absent;
        }

        if combined {
            return Coverage::Combined;
        }

        if attributions
            .iter()
            .all(|a| matches!(a.provenance, Provenance::Distributed(_)))
            && !attributions.is_empty()
        {
            return Coverage::Complete;
        }

        Coverage::Partial
    }
}

/******************************************************************************/
