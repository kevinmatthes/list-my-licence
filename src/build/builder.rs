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

/// Runs the whole pass.
///
/// # Examples
///
/// A complete `build.rs`:
///
/// ```no_run
/// list_my_licence::build::Builder::new()
///     .publish("THIRDPARTY.md")
///     .run()
///     .unwrap_or_else(|error| panic!("{error}"));
/// ```
#[derive(Clone, Debug, Default)]
pub struct Builder {
    published: Option<std::path::PathBuf>,
    checking: bool,
}

impl Builder {
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

    /// A pass that embeds the attribution and nothing more.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Also writes the human-readable attribution to `path`.
    #[must_use]
    pub fn publish(mut self, path: impl Into<std::path::PathBuf>) -> Self {
        self.published = Some(path.into());
        self
    }

    /// Resolves, discovers, classifies, surveys and emits.
    ///
    /// # Errors
    ///
    /// Returns [`crate::build::Error::Undischargeable`] if any package
    /// carries an obligation that cannot be discharged,
    /// [`crate::build::Error::Resolve`] if the graph cannot be read, and
    /// [`crate::build::Error::Emit`] if the artefacts cannot be written or
    /// the published file is stale.
    pub fn run(&self) -> Result<crate::build::Outcome, crate::build::Error> {
        println!("cargo::rerun-if-changed=Cargo.lock");

        let resolver = crate::build::Resolver::from_build_env()?;
        let discovery = crate::build::Discovery::new();
        let classifier = crate::build::Classifier::new();

        let mut packages = Vec::new();
        let mut survey = crate::build::Copyleft::new().survey();
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
            return Err(crate::build::Error::Undischargeable(undischargeable));
        }

        let borrowed: Vec<crate::build::Reproduced<'_>> = packages
            .iter()
            .map(|(package, verdict)| (package, verdict))
            .collect();

        crate::build::Emitter::from_build_env()?.embed(&borrowed)?;

        if let Some(path) = &self.published {
            if self.checking {
                crate::build::Emitter::check(path, &borrowed)?;
            } else {
                crate::build::Emitter::publish(path, &borrowed)?;
            }
        }

        Ok(crate::build::Outcome { packages, survey })
    }
}
