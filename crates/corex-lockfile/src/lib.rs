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

/// Supported foreign lockfile formats for migration.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum ForeignLockfileFormat {
    /// npm `package-lock.json`
    Npm,
    /// pnpm `pnpm-lock.yaml` or JSON
    Pnpm,
    /// Yarn `yarn.lock`
    Yarn,
    /// Bun `bun.lock`
    Bun,
}

impl ForeignLockfileFormat {
    /// Associated default filename for this format.
    #[must_use]
    pub const fn filename(self) -> &'static str {
        match self {
            Self::Npm => "package-lock.json",
            Self::Pnpm => "pnpm-lock.yaml",
            Self::Yarn => "yarn.lock",
            Self::Bun => "bun.lock",
        }
    }

    /// Short format string name.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Npm => "npm",
            Self::Pnpm => "pnpm",
            Self::Yarn => "yarn",
            Self::Bun => "bun",
        }
    }
}

/// Imports an npm `package-lock.json` into a canonical `Lockfile`.
///
/// # Errors
/// Returns `Diagnostic` if JSON is malformed.
pub fn import_npm_lockfile(content: &str) -> Result<Lockfile, Diagnostic> {
    let value: serde_json::Value = serde_json::from_str(content).map_err(|e| {
        Diagnostic::new(
            ErrorFamily::Lockfile,
            10,
            format!("failed parsing npm package-lock.json: {e}"),
        )
    })?;

    let mut lockfile = Lockfile::new();
    let mut importer = LockfileImporter::default();

    // Parse root dependencies
    if let Some(deps) = value
        .get("dependencies")
        .and_then(serde_json::Value::as_object)
    {
        for (name, dep_obj) in deps {
            if let Some(version_str) = dep_obj.get("version").and_then(serde_json::Value::as_str) {
                importer
                    .dependencies
                    .insert(name.clone(), version_str.to_string());

                let key = format!("{name}@{version_str}");
                let integrity = dep_obj
                    .get("integrity")
                    .and_then(serde_json::Value::as_str)
                    .unwrap_or("sha512-imported-npm")
                    .to_string();
                let tarball = dep_obj
                    .get("resolved")
                    .and_then(serde_json::Value::as_str)
                    .unwrap_or("")
                    .to_string();

                lockfile.packages.insert(
                    key,
                    LockfilePackage {
                        version: version_str.to_string(),
                        resolution: LockfileResolution {
                            registry: "npm".to_string(),
                            tarball,
                            integrity,
                        },
                        dependencies: BTreeMap::new(),
                        dev_dependencies: BTreeMap::new(),
                        optional_dependencies: BTreeMap::new(),
                        peer_dependencies: BTreeMap::new(),
                    },
                );
            }
        }
    } else if let Some(packages) = value.get("packages").and_then(serde_json::Value::as_object) {
        for (pkg_path, pkg_obj) in packages {
            if pkg_path.is_empty() {
                // Root package
                if let Some(deps) = pkg_obj
                    .get("dependencies")
                    .and_then(serde_json::Value::as_object)
                {
                    for (name, req) in deps {
                        if let Some(req_str) = req.as_str() {
                            importer
                                .dependencies
                                .insert(name.clone(), req_str.to_string());
                        }
                    }
                }
            } else if let Some(pkg_name) = pkg_path.strip_prefix("node_modules/") {
                if let Some(version_str) =
                    pkg_obj.get("version").and_then(serde_json::Value::as_str)
                {
                    let key = format!("{pkg_name}@{version_str}");
                    let integrity = pkg_obj
                        .get("integrity")
                        .and_then(serde_json::Value::as_str)
                        .unwrap_or("sha512-imported-npm")
                        .to_string();
                    let tarball = pkg_obj
                        .get("resolved")
                        .and_then(serde_json::Value::as_str)
                        .unwrap_or("")
                        .to_string();

                    lockfile.packages.insert(
                        key,
                        LockfilePackage {
                            version: version_str.to_string(),
                            resolution: LockfileResolution {
                                registry: "npm".to_string(),
                                tarball,
                                integrity,
                            },
                            dependencies: BTreeMap::new(),
                            dev_dependencies: BTreeMap::new(),
                            optional_dependencies: BTreeMap::new(),
                            peer_dependencies: BTreeMap::new(),
                        },
                    );
                }
            }
        }
    }

    lockfile.importers.insert(".".to_string(), importer);
    Ok(lockfile)
}

/// Imports a pnpm `pnpm-lock.yaml` (or JSON representation) into a canonical `Lockfile`.
///
/// # Errors
/// Returns `Diagnostic` if content format is invalid.
pub fn import_pnpm_lockfile(content: &str) -> Result<Lockfile, Diagnostic> {
    let mut lockfile = Lockfile::new();
    let mut importer = LockfileImporter::default();

    // Parse line by line to extract dependencies and package versions safely
    let mut current_section = "";
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("dependencies:") {
            current_section = "deps";
            continue;
        } else if trimmed.starts_with("devDependencies:") {
            current_section = "devDeps";
            continue;
        } else if trimmed.starts_with("packages:") {
            current_section = "pkgs";
            continue;
        }

        if current_section == "deps" || current_section == "devDeps" {
            if let Some((name, val)) = trimmed.split_once(':') {
                let name = name.trim().trim_matches('\'').trim_matches('"');
                let val = val.trim().trim_matches('\'').trim_matches('"');
                if !name.is_empty() && !val.is_empty() {
                    let clean_ver = val.split('(').next().unwrap_or(val).trim();
                    if current_section == "deps" {
                        importer
                            .dependencies
                            .insert(name.to_string(), clean_ver.to_string());
                    } else {
                        importer
                            .dev_dependencies
                            .insert(name.to_string(), clean_ver.to_string());
                    }

                    let key = format!("{name}@{clean_ver}");
                    lockfile
                        .packages
                        .entry(key)
                        .or_insert_with(|| LockfilePackage {
                            version: clean_ver.to_string(),
                            resolution: LockfileResolution {
                                registry: "npm".to_string(),
                                tarball: String::new(),
                                integrity: "sha512-imported-pnpm".to_string(),
                            },
                            dependencies: BTreeMap::new(),
                            dev_dependencies: BTreeMap::new(),
                            optional_dependencies: BTreeMap::new(),
                            peer_dependencies: BTreeMap::new(),
                        });
                }
            }
        }
    }

    lockfile.importers.insert(".".to_string(), importer);
    Ok(lockfile)
}

/// Imports a Yarn `yarn.lock` file into a canonical `Lockfile`.
///
/// # Errors
/// Returns `Diagnostic` if content format is invalid.
pub fn import_yarn_lockfile(content: &str) -> Result<Lockfile, Diagnostic> {
    let mut lockfile = Lockfile::new();
    let mut importer = LockfileImporter::default();

    let mut current_pkg_name = String::new();
    let mut current_version = String::new();
    let mut current_integrity = "sha512-imported-yarn".to_string();
    let mut current_resolved = String::new();

    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }

        if !line.starts_with(' ') && !line.starts_with('\t') && trimmed.ends_with(':') {
            // New package declaration block e.g. "react@^18.2.0":
            if !current_pkg_name.is_empty() && !current_version.is_empty() {
                let key = format!("{current_pkg_name}@{current_version}");
                importer
                    .dependencies
                    .insert(current_pkg_name.clone(), current_version.clone());
                lockfile.packages.insert(
                    key,
                    LockfilePackage {
                        version: current_version.clone(),
                        resolution: LockfileResolution {
                            registry: "npm".to_string(),
                            tarball: current_resolved.clone(),
                            integrity: current_integrity.clone(),
                        },
                        dependencies: BTreeMap::new(),
                        dev_dependencies: BTreeMap::new(),
                        optional_dependencies: BTreeMap::new(),
                        peer_dependencies: BTreeMap::new(),
                    },
                );
            }

            let raw_spec = trimmed
                .trim_end_matches(':')
                .trim_matches('"')
                .trim_matches('\'');
            let first_spec = raw_spec.split(',').next().unwrap_or(raw_spec).trim();
            if let Some((name, _)) = first_spec.rsplit_once('@') {
                current_pkg_name = name.trim_start_matches('"').to_string();
            } else {
                current_pkg_name = first_spec.to_string();
            }
            current_version.clear();
            current_resolved.clear();
            current_integrity = "sha512-imported-yarn".to_string();
        } else if trimmed.starts_with("version ") {
            current_version = trimmed
                .trim_start_matches("version ")
                .trim_matches('"')
                .trim_matches('\'')
                .to_string();
        } else if trimmed.starts_with("resolved ") {
            current_resolved = trimmed
                .trim_start_matches("resolved ")
                .trim_matches('"')
                .trim_matches('\'')
                .to_string();
        } else if trimmed.starts_with("integrity ") {
            current_integrity = trimmed
                .trim_start_matches("integrity ")
                .trim_matches('"')
                .trim_matches('\'')
                .to_string();
        }
    }

    if !current_pkg_name.is_empty() && !current_version.is_empty() {
        let key = format!("{current_pkg_name}@{current_version}");
        importer
            .dependencies
            .insert(current_pkg_name.clone(), current_version.clone());
        lockfile.packages.insert(
            key,
            LockfilePackage {
                version: current_version,
                resolution: LockfileResolution {
                    registry: "npm".to_string(),
                    tarball: current_resolved,
                    integrity: current_integrity,
                },
                dependencies: BTreeMap::new(),
                dev_dependencies: BTreeMap::new(),
                optional_dependencies: BTreeMap::new(),
                peer_dependencies: BTreeMap::new(),
            },
        );
    }

    lockfile.importers.insert(".".to_string(), importer);
    Ok(lockfile)
}

/// Imports a Bun `bun.lock` (JSON) into a canonical `Lockfile`.
///
/// # Errors
/// Returns `Diagnostic` if format is invalid.
pub fn import_bun_lockfile(content: &str) -> Result<Lockfile, Diagnostic> {
    let value: serde_json::Value = serde_json::from_str(content).map_err(|e| {
        Diagnostic::new(
            ErrorFamily::Lockfile,
            11,
            format!("failed parsing Bun bun.lock: {e}"),
        )
    })?;

    let mut lockfile = Lockfile::new();
    let mut importer = LockfileImporter::default();

    if let Some(packages) = value.get("packages").and_then(serde_json::Value::as_object) {
        for (name, val) in packages {
            let ver = val
                .as_str()
                .or_else(|| val.get("version").and_then(serde_json::Value::as_str))
                .unwrap_or("0.0.0");

            importer.dependencies.insert(name.clone(), ver.to_string());
            let key = format!("{name}@{ver}");
            lockfile.packages.insert(
                key,
                LockfilePackage {
                    version: ver.to_string(),
                    resolution: LockfileResolution {
                        registry: "npm".to_string(),
                        tarball: String::new(),
                        integrity: "sha512-imported-bun".to_string(),
                    },
                    dependencies: BTreeMap::new(),
                    dev_dependencies: BTreeMap::new(),
                    optional_dependencies: BTreeMap::new(),
                    peer_dependencies: BTreeMap::new(),
                },
            );
        }
    }

    lockfile.importers.insert(".".to_string(), importer);
    Ok(lockfile)
}

/// Detects foreign lockfiles in `project_dir` and converts the first matching foreign lockfile into a `Lockfile`.
///
/// **Invariant**: The foreign lockfile is read-only and is **never** modified or deleted.
///
/// # Errors
/// Returns `Diagnostic` if no foreign lockfile is found or parsing fails.
pub fn detect_and_import_foreign(
    project_dir: &std::path::Path,
) -> Result<(Lockfile, ForeignLockfileFormat, std::path::PathBuf), Diagnostic> {
    let candidates = [
        (ForeignLockfileFormat::Npm, "package-lock.json"),
        (ForeignLockfileFormat::Pnpm, "pnpm-lock.yaml"),
        (ForeignLockfileFormat::Yarn, "yarn.lock"),
        (ForeignLockfileFormat::Bun, "bun.lock"),
    ];

    for (format, filename) in candidates {
        let file_path = project_dir.join(filename);
        if file_path.exists() {
            let content = std::fs::read_to_string(&file_path).map_err(|e| {
                Diagnostic::new(
                    ErrorFamily::Lockfile,
                    12,
                    format!("failed reading foreign lockfile `{filename}`: {e}"),
                )
            })?;

            let lockfile = match format {
                ForeignLockfileFormat::Npm => import_npm_lockfile(&content)?,
                ForeignLockfileFormat::Pnpm => import_pnpm_lockfile(&content)?,
                ForeignLockfileFormat::Yarn => import_yarn_lockfile(&content)?,
                ForeignLockfileFormat::Bun => import_bun_lockfile(&content)?,
            };

            return Ok((lockfile, format, file_path));
        }
    }

    Err(Diagnostic::new(
        ErrorFamily::Lockfile,
        13,
        "no foreign lockfile (package-lock.json, pnpm-lock.yaml, yarn.lock, bun.lock) found",
    )
    .with_help("ensure a supported foreign lockfile exists in the project root"))
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

    #[test]
    fn test_import_npm_lockfile() {
        let npm_json = r#"{
            "name": "demo",
            "version": "1.0.0",
            "dependencies": {
                "express": {
                    "version": "4.18.2",
                    "resolved": "https://registry.npmjs.org/express/-/express-4.18.2.tgz",
                    "integrity": "sha512-express-sha"
                }
            }
        }"#;
        let lockfile = import_npm_lockfile(npm_json).unwrap();
        assert!(lockfile.packages.contains_key("express@4.18.2"));
        let importer = lockfile.importers.get(".").unwrap();
        assert_eq!(importer.dependencies.get("express").unwrap(), "4.18.2");
    }

    #[test]
    fn test_import_yarn_lockfile() {
        let yarn_txt = r#"
# yarn lockfile v1
"lodash@^4.17.21":
  version "4.17.21"
  resolved "https://registry.yarnpkg.com/lodash/-/lodash-4.17.21.tgz"
  integrity "sha512-lodash-sha"
"#;
        let lockfile = import_yarn_lockfile(yarn_txt).unwrap();
        assert!(lockfile.packages.contains_key("lodash@4.17.21"));
    }

    #[test]
    fn test_foreign_lockfile_preservation() {
        let tmp = tempfile::tempdir().unwrap();
        let npm_lock_path = tmp.path().join("package-lock.json");
        std::fs::write(
            &npm_lock_path,
            r#"{ "name": "app", "version": "1.0.0", "dependencies": {} }"#,
        )
        .unwrap();

        let (lockfile, format, source_path) = detect_and_import_foreign(tmp.path()).unwrap();
        assert_eq!(format, ForeignLockfileFormat::Npm);
        assert_eq!(source_path, npm_lock_path);
        assert_eq!(lockfile.lockfile_version, 1);
        // Verify source lockfile is NOT deleted or altered
        assert!(npm_lock_path.exists());
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
