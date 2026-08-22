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

//! The whole pass, for a build script to call once.
//!
//! Resolving the graph, finding the licences, judging them, surveying the
//! copyleft and writing both artefacts are separate concerns with separate
//! types, and each can be driven directly.  Almost nobody wants to:  a build
//! script wants the obligation discharged and the build stopped if it cannot
//! be.  That is what this is.

use super::{
    Classification, Classifier, Copyleft, Discovery, EmitError, Emitter,
    Reproduced, ResolveError, ResolvedPackage, Survey,
};
use std::{fmt, path::PathBuf};

/// Anything that stops a build script.
#[derive(Debug)]
pub enum Error {
    /// The dependency graph could not be resolved.
    Resolve(ResolveError),

    /// The attribution could not be written, or the committed copy is stale.
    Emit(EmitError),

    /// One or more packages carry an obligation that cannot be discharged.
    ///
    /// The build stops here rather than shipping an attribution known to be
    /// incomplete.  Each package is named, with what is wrong.
    Undischargeable(Vec<String>),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Resolve(error) => write!(f, "{error}"),
            Self::Emit(error) => write!(f, "{error}"),
            Self::Undischargeable(packages) => {
                writeln!(
                    f,
                    "{} package(s) carry a licence obligation this build \
                     cannot discharge:",
                    packages.len()
                )?;

                for package in packages {
                    writeln!(f, "  {package}")?;
                }

                Ok(())
            }
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Resolve(error) => Some(error),
            Self::Emit(error) => Some(error),
            Self::Undischargeable(_) => None,
        }
    }
}

impl From<ResolveError> for Error {
    fn from(error: ResolveError) -> Self {
        Self::Resolve(error)
    }
}

impl From<EmitError> for Error {
    fn from(error: EmitError) -> Self {
        Self::Emit(error)
    }
}

/// What one pass produced.
#[derive(Clone, Debug)]
pub struct Outcome {
    /// Every package examined, with its verdict.
    pub packages: Vec<(ResolvedPackage, Classification)>,

    /// The copyleft obligations found, if any.
    pub survey: Survey,
}

impl Outcome {
    /// The lines a build script should emit as `cargo::warning=`.
    ///
    /// Everything worth a human's attention that did not stop the build:  the
    /// copyleft obligations text cannot discharge, and the survivable
    /// complaints about individual packages.
    #[must_use]
    pub fn warnings(&self) -> Vec<String> {
        let mut warnings: Vec<String> = self.survey.warnings().collect();

        for (package, verdict) in &self.packages {
            warnings.extend(
                verdict
                    .problems
                    .iter()
                    .map(|problem| format!("{} {problem}", package.name)),
            );
        }

        warnings
    }
}

/// Runs the whole pass.
///
/// # Examples
///
/// A complete `build.rs`:
///
/// ```no_run
/// list_my_licence::build::Build::new()
///     .publish("THIRDPARTY.md")
///     .run()
///     .unwrap_or_else(|error| panic!("{error}"));
/// ```
#[derive(Clone, Debug, Default)]
pub struct Build {
    published: Option<PathBuf>,
    checking: bool,
}

impl Build {
    /// A pass that embeds the attribution and nothing more.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Also writes the human-readable attribution to `path`.
    #[must_use]
    pub fn publish(mut self, path: impl Into<PathBuf>) -> Self {
        self.published = Some(path.into());
        self
    }

    /// Fails instead of rewriting the published file when it is out of date.
    ///
    /// This is the whole of the drift check.  Left off, a build refreshes the
    /// committed attribution;  turned on, in continuous integration, it
    /// refuses to proceed while that file disagrees with the graph — so a
    /// dependency whose licence changed cannot be merged unnoticed.
    #[must_use]
    pub const fn checking(mut self, checking: bool) -> Self {
        self.checking = checking;
        self
    }

    /// Resolves, discovers, classifies, surveys and emits.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Undischargeable`] if any package carries an obligation
    /// that cannot be discharged, [`Error::Resolve`] if the graph cannot be
    /// read, and [`Error::Emit`] if the artefacts cannot be written or the
    /// published file is stale.
    pub fn run(&self) -> Result<Outcome, Error> {
        println!("cargo::rerun-if-changed=Cargo.lock");

        let resolver = super::Resolver::from_build_env()?;
        let discovery = Discovery::new();
        let classifier = Classifier::new();

        let mut packages = Vec::new();
        let mut survey = Copyleft::new().survey();
        let mut undischargeable = Vec::new();

        for package in resolver.resolve()? {
            let evidence = discovery.search(&package);
            let verdict = classifier.classify(&package, &evidence);

            survey.add(&package, &verdict);

            if verdict.is_fatal() {
                undischargeable.extend(
                    verdict
                        .problems
                        .iter()
                        .filter(|problem| problem.is_fatal())
                        .map(|problem| {
                            format!(
                                "{} {} {problem}",
                                package.name, package.version
                            )
                        }),
                );
            }

            packages.push((package, verdict));
        }

        if !undischargeable.is_empty() {
            return Err(Error::Undischargeable(undischargeable));
        }

        let borrowed: Vec<Reproduced<'_>> = packages
            .iter()
            .map(|(package, verdict)| (package, verdict))
            .collect();

        Emitter::from_build_env()?.embed(&borrowed)?;

        if let Some(path) = &self.published {
            if self.checking {
                Emitter::check(path, &borrowed)?;
            } else {
                Emitter::publish(path, &borrowed)?;
            }
        }

        Ok(Outcome { packages, survey })
    }
}

/******************************************************************************/
