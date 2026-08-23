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

/// Resolves the dependency graph that ships.
///
/// # Examples
///
/// From a `build.rs`:
///
/// ```no_run
/// # use list_my_licence::build::Resolver;
/// let packages = Resolver::from_build_env()?.resolve()?;
/// for package in &packages {
///     println!("cargo:warning={} {}", package.name, package.version);
/// }
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[derive(Clone, Debug)]
pub struct Resolver {
    manifest_path: Option<std::path::PathBuf>,
    target: Option<String>,
    include_root: bool,
    features: Option<Vec<String>>,
}

impl Resolver {
    /// Reduces a Cargo package to the fields harvesting needs.
    fn describe(
        package: &cargo_metadata::Package,
    ) -> crate::build::ResolvedPackage {
        let manifest_dir = package.manifest_path.parent().map_or_else(
            || std::path::PathBuf::from("."),
            |dir| dir.as_std_path().to_path_buf(),
        );

        let licence_file = package
            .license_file
            .as_ref()
            .map(|file| manifest_dir.join(file.as_std_path()));

        crate::build::ResolvedPackage {
            name: package.name.to_string(),
            version: package.version.to_string(),
            manifest_dir,
            licence: package.license.clone(),
            licence_file,
            authors: package.authors.clone(),
            repository: package.repository.clone(),
        }
    }

    /// Sets the feature selection to resolve with, replacing any default.
    ///
    /// Passing an empty list resolves with no features at all.  Leaving this
    /// unset resolves with Cargo's defaults, which is rarely what a build
    /// script wants — see [`Self::from_build_env`].
    #[must_use]
    pub fn features<I, S>(mut self, features: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.features = Some(features.into_iter().map(Into::into).collect());
        self
    }

    /// Recovers the enabled feature names from `CARGO_FEATURE_*`.
    ///
    /// Cargo exports each enabled feature as `CARGO_FEATURE_<NAME>` with the
    /// name upper-cased and hyphens turned into underscores.  That mapping is
    /// lossy — `my-feat` and `my_feat` both become `MY_FEAT` — so the names
    /// cannot simply be lower-cased back.  Instead the manifest's declared
    /// features are read and normalised the same way, which recovers the
    /// original spelling exactly.
    fn features_from_env(
        &self,
    ) -> Result<Vec<String>, crate::build::ResolveError> {
        fn normalise(name: &str) -> String {
            name.to_uppercase().replace('-', "_")
        }

        let enabled: std::collections::BTreeSet<String> = std::env::vars()
            .filter_map(|(key, _)| {
                key.strip_prefix("CARGO_FEATURE_").map(ToOwned::to_owned)
            })
            .collect();

        if enabled.is_empty() {
            return Ok(Vec::new());
        }

        let mut command = cargo_metadata::MetadataCommand::new();
        command.no_deps();

        if let Some(path) = &self.manifest_path {
            command.manifest_path(path);
        }

        let metadata = command.exec()?;

        Ok(metadata
            .packages
            .iter()
            .flat_map(|package| package.features.keys())
            .filter(|declared| enabled.contains(&normalise(declared)))
            .cloned()
            .collect())
    }

    /// A resolver configured from the environment Cargo gives a build script.
    ///
    /// `CARGO_MANIFEST_DIR` locates the manifest, `TARGET` names the platform
    /// actually being compiled for, and the `CARGO_FEATURE_*` variables say
    /// which features are enabled.  Together these restrict the graph to the
    /// dependencies that genuinely ship.
    ///
    /// Mirroring the feature selection is not a refinement but a correctness
    /// requirement.  An optional dependency is absent from `cargo metadata`'s
    /// resolved graph unless the feature enabling it is selected, so a
    /// resolver that ignores features silently under-reports — and a missing
    /// attribution is precisely the failure this crate exists to prevent.
    ///
    /// # Errors
    ///
    /// Returns [`crate::build::ResolveError::Metadata`] if the manifest's
    /// own feature list cannot be read, which is needed to undo Cargo's
    /// lossy environment-variable naming.
    pub fn from_build_env() -> Result<Self, crate::build::ResolveError> {
        let mut resolver = Self::new();

        if let Ok(dir) = std::env::var("CARGO_MANIFEST_DIR") {
            resolver.manifest_path =
                Some(std::path::PathBuf::from(dir).join("Cargo.toml"));
        }

        if let Ok(target) = std::env::var("TARGET") {
            resolver.target = Some(target);
        }

        resolver.features = Some(resolver.features_from_env()?);

        Ok(resolver)
    }

    /// Whether the root package itself appears in the result.
    ///
    /// Defaults to `true`:  an application must reproduce its own licence just
    /// as much as its dependencies'.
    #[must_use]
    pub const fn include_root(mut self, include: bool) -> Self {
        self.include_root = include;
        self
    }

    /// Overrides the manifest to resolve.
    #[must_use]
    pub fn manifest_path(
        mut self,
        path: impl Into<std::path::PathBuf>,
    ) -> Self {
        self.manifest_path = Some(path.into());
        self
    }

    /// Runs `cargo metadata`, restricted to the configured target.
    fn metadata(
        &self,
    ) -> Result<cargo_metadata::Metadata, crate::build::ResolveError> {
        let mut command = cargo_metadata::MetadataCommand::new();

        if let Some(path) = &self.manifest_path {
            command.manifest_path(path);
        }

        let mut options = Vec::new();

        if let Some(target) = &self.target {
            options.push("--filter-platform".to_owned());
            options.push(target.clone());
        }

        if let Some(features) = &self.features {
            options.push("--no-default-features".to_owned());

            if !features.is_empty() {
                options.push("--features".to_owned());
                options.push(features.join(","));
            }
        }

        if !options.is_empty() {
            command.other_options(options);
        }

        Ok(command.exec()?)
    }

    /// A resolver for the current directory, every target, root included.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            manifest_path: None,
            target: None,
            include_root: true,
            features: None,
        }
    }

    /// Resolves the graph.
    ///
    /// The result is sorted by name and then version, and contains no
    /// duplicates.
    ///
    /// # Errors
    ///
    /// Returns [`crate::build::ResolveError::Metadata`] if `cargo metadata`
    /// cannot be run, [`crate::build::ResolveError::NoResolve`] if it
    /// returns no graph, and [`crate::build::ResolveError::NoRootPackage`]
    /// for a virtual workspace manifest.
    pub fn resolve(
        &self,
    ) -> Result<Vec<crate::build::ResolvedPackage>, crate::build::ResolveError>
    {
        let metadata = self.metadata()?;
        let resolve = metadata
            .resolve
            .as_ref()
            .ok_or(crate::build::ResolveError::NoResolve)?;
        let root = resolve
            .root
            .as_ref()
            .ok_or(crate::build::ResolveError::NoRootPackage)?;

        let nodes: std::collections::BTreeMap<
            &cargo_metadata::PackageId,
            &cargo_metadata::Node,
        > = resolve.nodes.iter().map(|node| (&node.id, node)).collect();
        let packages: std::collections::BTreeMap<
            &cargo_metadata::PackageId,
            &cargo_metadata::Package,
        > = metadata.packages.iter().map(|pkg| (&pkg.id, pkg)).collect();

        let reachable = Self::walk(root, &nodes);

        let mut resolved: Vec<crate::build::ResolvedPackage> = reachable
            .iter()
            .filter(|id| self.include_root || *id != &root)
            .filter_map(|id| packages.get(id).map(|pkg| Self::describe(pkg)))
            .collect();

        resolved.sort();
        resolved.dedup();

        Ok(resolved)
    }

    /// Whether an edge represents code that ends up being distributed.
    ///
    /// An edge carries one [`cargo_metadata::DepKindInfo`] per kind it was
    /// declared under, so a package that is both a normal and a
    /// dev-dependency ships on the strength of the normal edge alone.  An
    /// empty list is treated as shipping:  older Cargo versions omitted the
    /// field, and over-reporting a licence is the safe failure direction.
    fn ships(dependency: &cargo_metadata::NodeDep) -> bool {
        dependency.dep_kinds.is_empty()
            || dependency.dep_kinds.iter().any(|kind| {
                matches!(
                    kind.kind,
                    cargo_metadata::DependencyKind::Normal
                        | cargo_metadata::DependencyKind::Build
                )
            })
    }

    /// Restricts the graph to a target triple.
    ///
    /// Without this, dependencies of platforms that are not being built for
    /// are included, and their licences would be reproduced without their code
    /// ever shipping.
    #[must_use]
    pub fn target(mut self, target: impl Into<String>) -> Self {
        self.target = Some(target.into());
        self
    }

    /// Breadth-first walk from the root over shipping edges only.
    ///
    /// Normal and build dependencies are followed at *every* level, not just
    /// from the root:  a build dependency's own dependencies are compiled in
    /// order to build it, so they ship just as surely.  Dev-dependencies are
    /// followed nowhere, including from the root, because a test-only crate is
    /// never distributed.
    fn walk<'id>(
        root: &'id cargo_metadata::PackageId,
        nodes: &std::collections::BTreeMap<
            &'id cargo_metadata::PackageId,
            &'id cargo_metadata::Node,
        >,
    ) -> std::collections::BTreeSet<&'id cargo_metadata::PackageId> {
        let mut seen = std::collections::BTreeSet::new();
        let mut queue = std::collections::VecDeque::new();

        seen.insert(root);
        queue.push_back(root);

        while let Some(current) = queue.pop_front() {
            let Some(node) = nodes.get(current) else {
                continue;
            };

            for dependency in &node.deps {
                if !Self::ships(dependency) {
                    continue;
                }

                if let Some((id, _)) = nodes.get_key_value(&dependency.pkg)
                    && seen.insert(*id)
                {
                    queue.push_back(*id);
                }
            }
        }

        seen
    }
}

impl Default for Resolver {
    fn default() -> Self {
        Self::new()
    }
}
