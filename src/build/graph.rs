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

//! Resolution of the dependency graph that actually ships.
//!
//! The set of packages whose licences must be reproduced is narrower than
//! "everything in `Cargo.lock`".  Only what is *distributed* counts:
//!
//! * normal dependencies, and
//! * build dependencies, whose code is compiled into the artefacts that
//!   produce the binary,
//!
//! both restricted to the target platform actually being compiled for, and
//! **not** dev-dependencies, which never leave the developer's machine.

use cargo_metadata::{
    DependencyKind, Metadata, MetadataCommand, Node, Package, PackageId,
};
use std::{
    collections::{BTreeMap, BTreeSet, VecDeque},
    fmt,
    path::PathBuf,
};

/// Anything that can go wrong while resolving the graph.
#[derive(Debug)]
pub enum Error {
    /// `cargo metadata` could not be run or returned malformed output.
    Metadata(cargo_metadata::Error),

    /// `cargo metadata` returned no resolved graph.  This happens when it is
    /// invoked with `--no-deps`, which this crate never does, so it indicates
    /// a Cargo the crate does not understand.
    NoResolve,

    /// The resolved graph named no root package.  Virtual manifests — a
    /// workspace with no package of its own — have no single root, and the
    /// crate cannot guess which member is being built.
    NoRootPackage,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Metadata(e) => {
                write!(f, "could not read cargo metadata:  {e}")
            }
            Self::NoResolve => f.write_str(
                "cargo metadata returned no resolved dependency graph",
            ),
            Self::NoRootPackage => f.write_str(
                "cargo metadata named no root package; \
                 virtual workspace manifests are not supported",
            ),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Metadata(e) => Some(e),
            Self::NoResolve | Self::NoRootPackage => None,
        }
    }
}

impl From<cargo_metadata::Error> for Error {
    fn from(e: cargo_metadata::Error) -> Self {
        Self::Metadata(e)
    }
}

/// One package of the resolved graph, reduced to what licence harvesting
/// needs.
///
/// The fields are deliberately owned rather than borrowed:  the
/// [`cargo_metadata::Metadata`] they come from is large, and holding it alive
/// for the whole of a build script would be wasteful.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct ResolvedPackage {
    /// The package name as Cargo knows it.
    pub name: String,

    /// The exact resolved version.
    pub version: String,

    /// Directory containing the package's `Cargo.toml`.
    ///
    /// This is where the search for `LICENSE`, `COPYING` and `NOTICE` files
    /// begins.  For registry dependencies it points into
    /// `~/.cargo/registry/src/`; for path and workspace members it points at
    /// the source tree.
    pub manifest_dir: PathBuf,

    /// The SPDX expression the package *declares*, verbatim from its
    /// `license` field.
    ///
    /// This is a claim, not a fact:  it is regularly absent, occasionally
    /// wrong, and may disagree with the licence files actually shipped.  It is
    /// deliberately kept as written rather than normalised here.
    pub licence: Option<String>,

    /// The package's `license-file` field, resolved against
    /// [`Self::manifest_dir`], for packages that point at a file instead of
    /// naming an expression.
    pub licence_file: Option<PathBuf>,

    /// The declared authors, used to recover a copyright line where no
    /// licence file carries one.
    pub authors: Vec<String>,

    /// The declared repository, which discharges the MPL-2.0 source-pointer
    /// obligation and is a useful ingredient elsewhere.
    pub repository: Option<String>,
}

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
    manifest_path: Option<PathBuf>,
    target: Option<String>,
    include_root: bool,
    features: Option<Vec<String>>,
}

impl Default for Resolver {
    fn default() -> Self {
        Self::new()
    }
}

impl Resolver {
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
    /// Returns [`Error::Metadata`] if the manifest's own feature list cannot
    /// be read, which is needed to undo Cargo's lossy environment-variable
    /// naming.
    pub fn from_build_env() -> Result<Self, Error> {
        let mut resolver = Self::new();

        if let Ok(dir) = std::env::var("CARGO_MANIFEST_DIR") {
            resolver.manifest_path =
                Some(PathBuf::from(dir).join("Cargo.toml"));
        }

        if let Ok(target) = std::env::var("TARGET") {
            resolver.target = Some(target);
        }

        resolver.features = Some(resolver.features_from_env()?);

        Ok(resolver)
    }

    /// Recovers the enabled feature names from `CARGO_FEATURE_*`.
    ///
    /// Cargo exports each enabled feature as `CARGO_FEATURE_<NAME>` with the
    /// name upper-cased and hyphens turned into underscores.  That mapping is
    /// lossy — `my-feat` and `my_feat` both become `MY_FEAT` — so the names
    /// cannot simply be lower-cased back.  Instead the manifest's declared
    /// features are read and normalised the same way, which recovers the
    /// original spelling exactly.
    fn features_from_env(&self) -> Result<Vec<String>, Error> {
        fn normalise(name: &str) -> String {
            name.to_uppercase().replace('-', "_")
        }

        let enabled: BTreeSet<String> = std::env::vars()
            .filter_map(|(key, _)| {
                key.strip_prefix("CARGO_FEATURE_").map(ToOwned::to_owned)
            })
            .collect();

        if enabled.is_empty() {
            return Ok(Vec::new());
        }

        let mut command = MetadataCommand::new();
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

    /// Overrides the manifest to resolve.
    #[must_use]
    pub fn manifest_path(mut self, path: impl Into<PathBuf>) -> Self {
        self.manifest_path = Some(path.into());
        self
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

    /// Whether the root package itself appears in the result.
    ///
    /// Defaults to `true`:  an application must reproduce its own licence just
    /// as much as its dependencies'.
    #[must_use]
    pub const fn include_root(mut self, include: bool) -> Self {
        self.include_root = include;
        self
    }

    /// Resolves the graph.
    ///
    /// The result is sorted by name and then version, and contains no
    /// duplicates.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Metadata`] if `cargo metadata` cannot be run,
    /// [`Error::NoResolve`] if it returns no graph, and
    /// [`Error::NoRootPackage`] for a virtual workspace manifest.
    pub fn resolve(&self) -> Result<Vec<ResolvedPackage>, Error> {
        let metadata = self.metadata()?;
        let resolve = metadata.resolve.as_ref().ok_or(Error::NoResolve)?;
        let root = resolve.root.as_ref().ok_or(Error::NoRootPackage)?;

        let nodes: BTreeMap<&PackageId, &Node> =
            resolve.nodes.iter().map(|node| (&node.id, node)).collect();
        let packages: BTreeMap<&PackageId, &Package> =
            metadata.packages.iter().map(|pkg| (&pkg.id, pkg)).collect();

        let reachable = Self::walk(root, &nodes);

        let mut resolved: Vec<ResolvedPackage> = reachable
            .iter()
            .filter(|id| self.include_root || *id != &root)
            .filter_map(|id| packages.get(id).map(|pkg| Self::describe(pkg)))
            .collect();

        resolved.sort();
        resolved.dedup();

        Ok(resolved)
    }

    /// Runs `cargo metadata`, restricted to the configured target.
    fn metadata(&self) -> Result<Metadata, Error> {
        let mut command = MetadataCommand::new();

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

    /// Breadth-first walk from the root over shipping edges only.
    ///
    /// Normal and build dependencies are followed at *every* level, not just
    /// from the root:  a build dependency's own dependencies are compiled in
    /// order to build it, so they ship just as surely.  Dev-dependencies are
    /// followed nowhere, including from the root, because a test-only crate is
    /// never distributed.
    fn walk<'id>(
        root: &'id PackageId,
        nodes: &BTreeMap<&'id PackageId, &'id Node>,
    ) -> BTreeSet<&'id PackageId> {
        let mut seen = BTreeSet::new();
        let mut queue = VecDeque::new();

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
                    DependencyKind::Normal | DependencyKind::Build
                )
            })
    }

    /// Reduces a Cargo package to the fields harvesting needs.
    fn describe(package: &Package) -> ResolvedPackage {
        let manifest_dir = package.manifest_path.parent().map_or_else(
            || PathBuf::from("."),
            |dir| dir.as_std_path().to_path_buf(),
        );

        let licence_file = package
            .license_file
            .as_ref()
            .map(|file| manifest_dir.join(file.as_std_path()));

        ResolvedPackage {
            name: package.name.to_string(),
            version: package.version.to_string(),
            manifest_dir,
            licence: package.license.clone(),
            licence_file,
            authors: package.authors.clone(),
            repository: package.repository.clone(),
        }
    }
}

/******************************************************************************/
