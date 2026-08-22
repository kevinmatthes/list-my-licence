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
    /// Finds a text for every declared term.
    ///
    /// Returns the attributions, the terms genuinely discharged, and whether
    /// more than one term leaned on the same general file.
    fn attribute(
        terms: &[String],
        evidence: &crate::build::Evidence,
        problems: &mut Vec<crate::build::Problem>,
    ) -> (
        Vec<crate::build::Attribution>,
        std::collections::BTreeSet<String>,
        bool,
    ) {
        let general = Self::general_files(evidence);

        let mut attributions = Vec::new();
        let mut discharged = std::collections::BTreeSet::new();

        // Which attributions fell back to a general file.  A file is only
        // *combined* if more than one term actually leans on it;  a package
        // shipping `COPYING` beside `LICENSE-MIT` and `UNLICENSE` has a
        // general file, but each term still has its own text.
        let mut leaning = Vec::new();

        for term in terms {
            let (text, provenance) = if let Some(file) =
                Self::specific(evidence, term)
            {
                (
                    file.text.clone(),
                    crate::build::Provenance::Distributed(file.path.clone()),
                )
            } else if let Some(file) = general.first() {
                leaning.push(attributions.len());

                (
                    file.text.clone(),
                    crate::build::Provenance::Distributed(file.path.clone()),
                )
            } else if let Some(text) = Self::canonical(term) {
                (text, crate::build::Provenance::Canonical)
            } else {
                problems.push(crate::build::Problem::Unattributed {
                    identifier: term.clone(),
                });

                continue;
            };

            // A canonical text discharges nothing for a licence that needs its
            // own copyright line.
            if provenance == crate::build::Provenance::Canonical
                && !Self::is_standard_text(term)
            {
                problems.push(crate::build::Problem::Unattributed {
                    identifier: term.clone(),
                });

                continue;
            }

            discharged.insert(term.clone());
            attributions.push(crate::build::Attribution {
                identifier: term.clone(),
                text,
                provenance,
            });
        }

        let combined = leaning.len() > 1;

        (attributions, discharged, combined)
    }

    /// The canonical SPDX text of a licence, if the list carries one.
    fn canonical(identifier: &str) -> Option<String> {
        identifier
            .parse::<&dyn license::License>()
            .ok()
            .map(|licence| license::License::text(licence).to_owned())
    }

    /// Judges one package.
    #[must_use]
    pub fn classify(
        &self,
        package: &crate::build::ResolvedPackage,
        evidence: &crate::build::Evidence,
    ) -> crate::build::Classification {
        let mut problems: Vec<crate::build::Problem> = evidence
            .skipped
            .iter()
            .map(|(path, reason)| crate::build::Problem::Unreadable {
                path: path.clone(),
                reason: reason.clone(),
            })
            .collect();

        let notices = evidence.notices().cloned().collect();

        let Some(expression) =
            Self::declaration(package, evidence, &mut problems)
        else {
            return crate::build::Classification {
                coverage: crate::build::Coverage::Absent,
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
                !matches!(problem, crate::build::Problem::Unattributed { .. })
            });
        } else if !problems.iter().any(crate::build::Problem::is_fatal) {
            problems.push(crate::build::Problem::Unsatisfiable {
                expression: package.licence.clone().unwrap_or_default(),
            });
        }

        problems.sort();
        problems.dedup();

        crate::build::Classification {
            coverage: Self::coverage(evidence, &attributions, combined),
            attributions,
            notices,
            problems,
        }
    }

    /// How completely the evidence covered the declaration.
    fn coverage(
        evidence: &crate::build::Evidence,
        attributions: &[crate::build::Attribution],
        combined: bool,
    ) -> crate::build::Coverage {
        if evidence.licences().next().is_none() {
            return crate::build::Coverage::Absent;
        }

        if combined {
            return crate::build::Coverage::Combined;
        }

        if attributions.iter().all(|a| {
            matches!(a.provenance, crate::build::Provenance::Distributed(_))
        }) && !attributions.is_empty()
        {
            return crate::build::Coverage::Complete;
        }

        crate::build::Coverage::Partial
    }

    /// Parses the package's declaration, recording why if it cannot.
    ///
    /// Real manifests still carry the deprecated slash form and deprecated
    /// identifiers, so leniency here is a requirement rather than a
    /// convenience:  four of the twelve licence-less crates measured in the
    /// author's own cache write `MIT/Apache-2.0`.
    fn declaration(
        package: &crate::build::ResolvedPackage,
        evidence: &crate::build::Evidence,
        problems: &mut Vec<crate::build::Problem>,
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

                problems.push(crate::build::Problem::Custom {
                    identifier: synthesised.clone(),
                });

                &synthesised
            }

            None => {
                problems.push(crate::build::Problem::Undeclared);

                return None;
            }
        };

        let parsed =
            spdx::Expression::parse_mode(declared, spdx::ParseMode::LAX).ok();

        match &parsed {
            None => problems.push(crate::build::Problem::Unparsable {
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
                    .map(|requirement| crate::build::Problem::Custom {
                        identifier: Self::name(&requirement.req.license),
                    }),
            ),
        }

        parsed
    }

    /// The shipped licence files that name no particular licence.
    fn general_files(
        evidence: &crate::build::Evidence,
    ) -> Vec<&crate::build::Found> {
        evidence
            .licences()
            .filter(|file| file.identifier.is_none())
            .collect()
    }

    /// Whether a licence's canonical text discharges it without a copyright
    /// line of its own.
    fn is_standard_text(identifier: &str) -> bool {
        STANDARD_TEXT.contains(&identifier)
    }

    /// Restates shared general files as such.
    fn mark_combined(attributions: &mut [crate::build::Attribution]) {
        let shared: Vec<std::path::PathBuf> = attributions
            .iter()
            .filter_map(|attribution| match &attribution.provenance {
                crate::build::Provenance::Distributed(path) => {
                    Some(path.clone())
                }
                _ => None,
            })
            .collect();

        for attribution in attributions.iter_mut() {
            if let crate::build::Provenance::Distributed(path) =
                attribution.provenance.clone()
                && shared.iter().filter(|other| **other == path).count() > 1
            {
                attribution.provenance =
                    crate::build::Provenance::Combined(path);
            }
        }
    }

    /// What one licence item of an expression is called.
    fn name(item: &spdx::LicenseItem) -> String {
        item.id()
            .map_or_else(|| item.to_string(), |id| id.name.to_owned())
    }

    /// A classifier with the default settings.
    #[must_use]
    pub const fn new() -> Self {
        Self { _private: () }
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

    /// The shipped file naming a given licence, if there is one.
    fn specific<'a>(
        evidence: &'a crate::build::Evidence,
        term: &str,
    ) -> Option<&'a crate::build::Found> {
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
}
