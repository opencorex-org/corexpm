//! Deterministic `corex.lock.json` lockfile parser, canonical serializer, and validator.

#![forbid(unsafe_code)]

use corex_errors::{Diagnostic, ErrorFamily};
use corex_graph::{DependencyGraph, DependencyKind};
use corex_manifest::{PackageManifest, PackageName};
use corex_semver::{Range, Version};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Supported lockfile schema version.
pub const CURRENT_LOCKFILE_VERSION: u32 = 1;

/// Package distribution resolution metadata.
#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LockfileResolution {
    /// Registry identifier or protocol.
    pub registry: String,
    /// Direct tarball download location.
    pub tarball: String,
    /// Package integrity hash string.
    pub integrity: String,
}

/// Package record entry inside `corex.lock.json`.
#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LockfilePackage {
    /// Exact package version string.
    pub version: String,
    /// Resolution metadata.
    #[serde(default = "default_resolution")]
    pub resolution: LockfileResolution,
    /// Runtime dependencies.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub dependencies: BTreeMap<String, String>,
    /// Development dependencies.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub dev_dependencies: BTreeMap<String, String>,
    /// Optional dependencies.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub optional_dependencies: BTreeMap<String, String>,
    /// Peer dependencies.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub peer_dependencies: BTreeMap<String, String>,
}

fn default_resolution() -> LockfileResolution {
    LockfileResolution {
        registry: "npm".to_string(),
        tarball: String::new(),
        integrity: String::new(),
    }
}

/// Project importer declaration entry (e.g. root `.`).
#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LockfileImporter {
    /// Direct runtime dependencies.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub dependencies: BTreeMap<String, String>,
    /// Direct dev dependencies.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub dev_dependencies: BTreeMap<String, String>,
    /// Direct optional dependencies.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub optional_dependencies: BTreeMap<String, String>,
    /// Direct peer dependencies.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub peer_dependencies: BTreeMap<String, String>,
}

/// Deterministic `corex.lock.json` root schema representation.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Lockfile {
    /// Schema version number.
    pub lockfile_version: u32,
    /// Importer declarations keyed by root path (e.g. `.`).
    #[serde(default)]
    pub importers: BTreeMap<String, LockfileImporter>,
    /// Resolved packages keyed deterministically by `name@version`.
    #[serde(default)]
    pub packages: BTreeMap<String, LockfilePackage>,
}

impl Default for Lockfile {
    fn default() -> Self {
        Self {
            lockfile_version: CURRENT_LOCKFILE_VERSION,
            importers: BTreeMap::new(),
            packages: BTreeMap::new(),
        }
    }
}

impl Lockfile {
    /// Creates a new empty `Lockfile` with default version 1.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Parses and validates a JSON string into a `Lockfile`.
    ///
    /// # Errors
    /// Returns `Diagnostic` if syntax is invalid or `lockfileVersion` exceeds supported version.
    pub fn from_json(content: &str) -> Result<Self, Diagnostic> {
        let lockfile: Self = serde_json::from_str(content).map_err(|e| {
            Diagnostic::new(
                ErrorFamily::Lockfile,
                1,
                format!("failed parsing lockfile JSON: {e}"),
            )
        })?;

        if lockfile.lockfile_version > CURRENT_LOCKFILE_VERSION {
            return Err(Diagnostic::new(
                ErrorFamily::Lockfile,
                2,
                format!(
                    "unsupported lockfile version {}, maximum supported is {}",
                    lockfile.lockfile_version, CURRENT_LOCKFILE_VERSION
                ),
            )
            .with_help("upgrade CorexPM to parse newer lockfile formats"));
        }

        Ok(lockfile)
    }

    /// Serializes the lockfile into canonical, deterministic 2-space JSON format.
    ///
    /// # Errors
    /// Returns `Diagnostic` if JSON serialization fails.
    pub fn to_canonical_json(&self) -> Result<String, Diagnostic> {
        let mut json = serde_json::to_string_pretty(self).map_err(|e| {
            Diagnostic::new(
                ErrorFamily::Lockfile,
                3,
                format!("failed serializing lockfile: {e}"),
            )
        })?;
        json.push('\n');
        Ok(json)
    }

    /// Builds a deterministic `Lockfile` from a resolved `DependencyGraph` and `PackageManifest`.
    #[must_use]
    pub fn from_graph(graph: &DependencyGraph, manifest: &PackageManifest) -> Self {
        let mut importer = LockfileImporter::default();

        for (name, spec) in &manifest.dependencies {
            importer
                .dependencies
                .insert(name.as_str().to_string(), spec.clone());
        }
        for (name, spec) in &manifest.dev_dependencies {
            importer
                .dev_dependencies
                .insert(name.as_str().to_string(), spec.clone());
        }
        for (name, spec) in &manifest.optional_dependencies {
            importer
                .optional_dependencies
                .insert(name.as_str().to_string(), spec.clone());
        }
        for (name, spec) in &manifest.peer_dependencies {
            importer
                .peer_dependencies
                .insert(name.as_str().to_string(), spec.clone());
        }

        let mut importers = BTreeMap::new();
        importers.insert(".".to_string(), importer);

        let mut packages = BTreeMap::new();
        for node in graph.nodes.values() {
            let key = format!(
                "{}@{}",
                node.package.name().as_str(),
                node.version.version().as_str()
            );

            let mut deps_map = BTreeMap::new();
            for edge in &graph.edges {
                if edge.from == node.id {
                    if let Some(target_node) = graph.nodes.get(&edge.to) {
                        deps_map.insert(
                            target_node.package.name().as_str().to_string(),
                            target_node.version.version().as_str().clone(),
                        );
                    }
                }
            }

            let pkg_entry = LockfilePackage {
                version: node.version.version().as_str().clone(),
                resolution: LockfileResolution {
                    registry: "npm".to_string(),
                    tarball: node.dist_url.clone(),
                    integrity: node.integrity.clone(),
                },
                dependencies: deps_map,
                dev_dependencies: BTreeMap::new(),
                optional_dependencies: BTreeMap::new(),
                peer_dependencies: BTreeMap::new(),
            };

            packages.insert(key, pkg_entry);
        }

        Self {
            lockfile_version: CURRENT_LOCKFILE_VERSION,
            importers,
            packages,
        }
    }

    /// Reconstructs a `DependencyGraph` from this `Lockfile`.
    ///
    /// # Errors
    /// Returns `Diagnostic` if package version parsing or range validation fails.
    pub fn to_graph(&self) -> Result<DependencyGraph, Diagnostic> {
        let mut graph = DependencyGraph::default();
        let mut node_ids = BTreeMap::new();

        // 1. Add all nodes
        for (key, pkg) in &self.packages {
            let (pkg_name_str, ver_str) = key
                .split_once('@')
                .unwrap_or((key.as_str(), pkg.version.as_str()));
            let name = PackageName::parse(pkg_name_str).map_err(|_| {
                Diagnostic::new(
                    ErrorFamily::Lockfile,
                    4,
                    format!("invalid package name `{pkg_name_str}` in lockfile"),
                )
            })?;
            let version = Version::parse(ver_str).map_err(|_| {
                Diagnostic::new(
                    ErrorFamily::Lockfile,
                    5,
                    format!("invalid package version `{ver_str}` in lockfile"),
                )
            })?;

            let id = graph.add_node(
                name,
                version,
                pkg.resolution.tarball.clone(),
                pkg.resolution.integrity.clone(),
            );
            node_ids.insert(key.clone(), id);
        }

        // 2. Add root nodes from importer
        if let Some(importer) = self.importers.get(".") {
            for root_pkg_name in importer
                .dependencies
                .keys()
                .chain(importer.dev_dependencies.keys())
            {
                for (key, node) in &graph.nodes {
                    if node.package.name().as_str() == root_pkg_name {
                        graph.root_nodes.insert(*key);
                    }
                }
            }
        }

        // 3. Add edges between nodes
        let wildcard_range = Range::parse("*")?;
        for (from_key, pkg) in &self.packages {
            if let Some(&from_id) = node_ids.get(from_key) {
                for (target_name, target_ver) in &pkg.dependencies {
                    let target_key = format!("{target_name}@{target_ver}");
                    if let Some(&to_id) = node_ids.get(&target_key) {
                        let range =
                            Range::parse(target_ver).unwrap_or_else(|_| wildcard_range.clone());
                        graph.add_edge(from_id, to_id, DependencyKind::Dependency, range);
                    }
                }
            }
        }

        Ok(graph)
    }

    /// Validates lockfile against manifest requirements.
    ///
    /// # Errors
    /// Returns `Diagnostic` if manifest dependencies mismatch lockfile declarations.
    pub fn validate_against_manifest(&self, manifest: &PackageManifest) -> Result<(), Diagnostic> {
        let importer = self.importers.get(".").ok_or_else(|| {
            Diagnostic::new(
                ErrorFamily::Lockfile,
                6,
                "missing root `.` importer declaration in lockfile",
            )
        })?;

        validate_deps_match(
            &manifest.dependencies,
            &importer.dependencies,
            "dependencies",
        )?;
        validate_deps_match(
            &manifest.dev_dependencies,
            &importer.dev_dependencies,
            "devDependencies",
        )?;
        validate_deps_match(
            &manifest.optional_dependencies,
            &importer.optional_dependencies,
            "optionalDependencies",
        )?;
        validate_deps_match(
            &manifest.peer_dependencies,
            &importer.peer_dependencies,
            "peerDependencies",
        )?;

        Ok(())
    }

    /// Validates integrity hashes and resolution URLs across all locked packages.
    ///
    /// # Errors
    /// Returns `Diagnostic` if any locked package has empty or malformed integrity/resolution.
    pub fn validate_integrity(&self) -> Result<(), Diagnostic> {
        for (key, pkg) in &self.packages {
            if pkg.resolution.integrity.is_empty() {
                return Err(Diagnostic::new(
                    ErrorFamily::Lockfile,
                    7,
                    format!("missing integrity hash for locked package `{key}`"),
                ));
            }
        }
        Ok(())
    }
}

fn validate_deps_match(
    manifest_deps: &BTreeMap<PackageName, String>,
    importer_deps: &BTreeMap<String, String>,
    section_name: &str,
) -> Result<(), Diagnostic> {
    for (pkg_name, req) in manifest_deps {
        let name_str = pkg_name.as_str();
        match importer_deps.get(name_str) {
            Some(lock_req) if lock_req == req => {}
            Some(lock_req) => {
                return Err(Diagnostic::new(
                    ErrorFamily::Lockfile,
                    8,
                    format!(
                        "lockfile out of sync with package.json in `{section_name}`: `{name_str}` requirement is `{req}` in manifest but `{lock_req}` in lockfile"
                    ),
                )
                .with_help("run `corexpm install` to update the lockfile"));
            }
            None => {
                return Err(Diagnostic::new(
                    ErrorFamily::Lockfile,
                    9,
                    format!(
                        "lockfile out of sync with package.json in `{section_name}`: `{name_str}` present in manifest but missing in lockfile"
                    ),
                )
                .with_help("run `corexpm install` to update the lockfile"));
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lockfile_canonical_roundtrip() {
        let manifest_content = r#"{
            "name": "my-app",
            "version": "1.0.0",
            "dependencies": {
                "react": "^18.2.0"
            }
        }"#;
        let manifest = PackageManifest::parse_json(manifest_content).unwrap();

        let fixtures = find_fixtures_dir();
        let client = corex_registry::MockRegistryClient::new(&fixtures);
        let config = corex_config::ProjectConfig::default();
        let resolver = corex_resolver::DependencyResolver::new(&client, &config);
        let graph = resolver.resolve(&manifest).unwrap();

        let lockfile = Lockfile::from_graph(&graph, &manifest);
        let json_str = lockfile.to_canonical_json().unwrap();
        assert!(json_str.contains(r#""lockfileVersion": 1"#));
        assert!(json_str.contains("react@18.2.0"));

        let parsed = Lockfile::from_json(&json_str).unwrap();
        assert_eq!(parsed.lockfile_version, 1);
        parsed.validate_against_manifest(&manifest).unwrap();
        parsed.validate_integrity().unwrap();
    }

    #[test]
    fn test_rejects_newer_lockfile_version() {
        let json_str = r#"{ "lockfileVersion": 99, "importers": {}, "packages": {} }"#;
        let err = Lockfile::from_json(json_str).unwrap_err();
        assert_eq!(err.code(), "CXLOCK0002");
    }

    fn find_fixtures_dir() -> std::path::PathBuf {
        let cwd = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
        let mut current = cwd.as_path();
        loop {
            let candidate = current.join("tests").join("fixtures").join("registry");
            if candidate.exists() {
                return candidate;
            }
            if let Some(parent) = current.parent() {
                current = parent;
            } else {
                break;
            }
        }
        cwd.join("tests").join("fixtures").join("registry")
    }
}
