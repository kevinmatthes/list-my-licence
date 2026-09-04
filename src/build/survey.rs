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

/// What a survey of the dependency graph turned up.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Survey {
    /// Every dependency carrying an obligation beyond reproduction, sorted.
    pub findings: Vec<crate::build::Finding>,
}

impl Survey {
    /// Whether the graph is free of obligations beyond reproduction.
    #[must_use]
    pub const fn is_clear(&self) -> bool {
        self.findings.is_empty()
    }

    /// The furthest-reaching obligation present, if any.
    #[must_use]
    pub fn strongest(&self) -> Option<crate::build::Strength> {
        self.findings.iter().map(|finding| finding.strength).max()
    }

    /// The findings this crate cannot discharge on its own.
    pub fn undischargeable(
        &self,
    ) -> impl Iterator<Item = &crate::build::Finding> {
        self.findings
            .iter()
            .filter(|finding| !finding.strength.is_dischargeable())
    }

    /// Lines for a build script to emit as `cargo::warning=` messages.
    ///
    /// Phrased as what each package *declares*, because that is all a manifest
    /// can tell anyone.
    pub fn warnings(&self) -> impl Iterator<Item = String> {
        self.findings.iter().map(ToString::to_string)
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
    pub fn add(
        &mut self,
        package: &crate::build::ResolvedPackage,
        verdict: &crate::build::Classification,
    ) {
        let discharged: std::collections::BTreeSet<&str> = verdict
            .attributions
            .iter()
            .map(crate::build::Attribution::identifier)
            .collect();

        let expression = package.licence.as_deref().and_then(|declared| {
            spdx::Expression::parse_mode(declared, spdx::ParseMode::LAX).ok()
        });

        for attribution in &verdict.attributions {
            let strength =
                crate::build::Strength::of(attribution.identifier());

            if strength == crate::build::Strength::Permissive {
                continue;
            }

            if Self::avoidable(
                expression.as_ref(),
                &discharged,
                attribution.identifier(),
            ) {
                continue;
            }

            self.findings.push(crate::build::Finding {
                package: package.name.clone(),
                version: package.version.clone(),
                identifier: attribution.identifier().to_string(),
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
        discharged: &std::collections::BTreeSet<&str>,
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
    fn source(package: &crate::build::ResolvedPackage) -> Option<String> {
        package.repository.as_ref().map(|repository| {
            format!("{repository} at version {}", package.version)
        })
    }
}

/******************************************************************************/
