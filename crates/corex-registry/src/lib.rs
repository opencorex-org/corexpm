//! npm registry client abstraction and mock client implementation.

use corex_errors::{Diagnostic, ErrorFamily};
use corex_manifest::PackageName;
use corex_semver::Version;
use std::collections::BTreeMap;
use std::path::PathBuf;

/// Package distribution metadata.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct RegistryDist {
    /// URL of the package tarball.
    pub tarball: String,
    /// Expected integrity hash (e.g., sha512).
    pub integrity: String,
}

/// Metadata returned from the registry for a specific version.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct RegistryVersionMetadata {
    /// Semantic version.
    pub version: Version,
    /// Runtime dependencies.
    #[serde(default)]
    pub dependencies: BTreeMap<PackageName, String>,
    /// Development-only dependencies.
    #[serde(default, rename = "devDependencies")]
    pub dev_dependencies: BTreeMap<PackageName, String>,
    /// Optional dependencies.
    #[serde(default, rename = "optionalDependencies")]
    pub optional_dependencies: BTreeMap<PackageName, String>,
    /// Peer dependencies.
    #[serde(default, rename = "peerDependencies")]
    pub peer_dependencies: BTreeMap<PackageName, String>,
    /// Distribution information.
    pub dist: RegistryDist,
    /// Target engine constraints.
    #[serde(default)]
    pub engines: BTreeMap<String, String>,
    /// Target operating systems.
    #[serde(default)]
    pub os: Vec<String>,
    /// Target CPU architectures.
    #[serde(default)]
    pub cpu: Vec<String>,
}

/// Registry metadata for all versions of a package.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct RegistryPackageMetadata {
    /// Original package name.
    pub name: PackageName,
    /// Distribution tags (e.g. latest -> 1.0.0).
    #[serde(default, rename = "dist-tags")]
    pub dist_tags: BTreeMap<String, String>,
    /// Mapping of all versions to their metadata.
    pub versions: BTreeMap<Version, RegistryVersionMetadata>,
}

/// Client interface for interacting with npm registry metadata.
pub trait RegistryClient: Send + Sync {
    /// Fetches package metadata from the registry.
    ///
    /// # Errors
    ///
    /// Returns a [`Diagnostic`] when the request or parsing fails.
    fn fetch_metadata(&self, name: &PackageName) -> Result<RegistryPackageMetadata, Diagnostic>;
}

/// A mock registry client loading local JSON files.
#[derive(Debug)]
pub struct MockRegistryClient {
    fixtures_dir: PathBuf,
}

impl MockRegistryClient {
    /// Creates a new `MockRegistryClient` reading from the specified directory.
    #[must_use]
    pub fn new(fixtures_dir: impl Into<PathBuf>) -> Self {
        Self {
            fixtures_dir: fixtures_dir.into(),
        }
    }
}

impl RegistryClient for MockRegistryClient {
    fn fetch_metadata(&self, name: &PackageName) -> Result<RegistryPackageMetadata, Diagnostic> {
        let safe_name = name.as_str().replace('/', "__").replace('@', "_");
        let path = self.fixtures_dir.join(format!("{safe_name}.json"));
        if !path.exists() {
            return Err(Diagnostic::new(
                ErrorFamily::Registry,
                1,
                format!("mock package metadata not found for `{}`", name.as_str()),
            )
            .with_help(format!("expected fixture file at `{}`", path.display())));
        }

        let content = std::fs::read_to_string(&path).map_err(|e| {
            Diagnostic::new(
                ErrorFamily::Registry,
                2,
                format!("failed to read mock fixture for `{}`: {e}", name.as_str()),
            )
        })?;

        let metadata: RegistryPackageMetadata = serde_json::from_str(&content).map_err(|e| {
            Diagnostic::new(
                ErrorFamily::Registry,
                3,
                format!("failed to parse mock fixture for `{}`: {e}", name.as_str()),
            )
        })?;

        Ok(metadata)
    }
}
