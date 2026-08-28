//! High-level installer, project state manager, and reconciliation engine.

#![forbid(unsafe_code)]

use corex_config::ProjectConfig;
use corex_errors::{Diagnostic, ErrorFamily};
use corex_linker::{IsolatedLinker, MaterializationSummary};
use corex_manifest::{PackageManifest, PackageName};
use corex_registry::MockRegistryClient;
use corex_resolver::DependencyResolver;
use corex_store::Store;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{Instant, SystemTime, UNIX_EPOCH};

/// Project state persisted inside `.corex/state.json`.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ProjectState {
    /// SHA-256 hash of the `package.json` content during last install.
    pub manifest_hash: String,
    /// Total resolved packages in the dependency graph.
    pub installed_packages: usize,
    /// Unix timestamp in seconds when the install completed.
    pub installed_at_secs: u64,
    /// Materialization summary from the linker.
    pub link_summary: MaterializationSummary,
}

/// Result details returned by `InstallerService::install`.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct InstallResult {
    /// True if warm fast reconciliation was performed without mutating `node_modules`.
    pub reconciled: bool,
    /// Name of the project from manifest.
    pub manifest_name: String,
    /// Number of resolved package instances.
    pub resolved_count: usize,
    /// Materialization details.
    pub summary: MaterializationSummary,
    /// Elapsed installation time in milliseconds.
    pub elapsed_ms: u128,
}

/// `CorexPM` Installer service handling project installation and reconciliation.
#[derive(Clone, Debug, Default)]
pub struct InstallerService;

impl InstallerService {
    /// Creates a new `InstallerService`.
    #[must_use]
    pub fn new() -> Self {
        Self
    }

    /// Reconciles and installs project dependencies inside `project_root`.
    ///
    /// # Errors
    /// Returns `Diagnostic` if manifest reading, resolution, CAS storage, or linking fails.
    pub fn install(
        &self,
        project_root: &Path,
        config: &ProjectConfig,
        fixtures_dir: &Path,
        custom_store_dir: Option<&Path>,
    ) -> Result<InstallResult, Diagnostic> {
        let start_time = Instant::now();
        let manifest_path = project_root.join("package.json");

        if !manifest_path.exists() {
            return Err(Diagnostic::new(
                ErrorFamily::Resolve,
                1,
                format!(
                    "`package.json` not found in project root `{}`",
                    project_root.display()
                ),
            )
            .with_help("run `corexpm init` or create a `package.json` file"));
        }

        let manifest_content = fs::read_to_string(&manifest_path).map_err(|e| {
            Diagnostic::new(
                ErrorFamily::Resolve,
                2,
                format!("failed reading `package.json`: {e}"),
            )
        })?;

        let manifest_hash = compute_sha256(&manifest_content);
        let state_path = project_root.join(".corex").join("state.json");
        let node_modules = project_root.join("node_modules");

        // Warm Fast Reconciliation Check
        if node_modules.exists() && state_path.exists() {
            if let Ok(state_str) = fs::read_to_string(&state_path) {
                if let Ok(saved_state) = serde_json::from_str::<ProjectState>(&state_str) {
                    if saved_state.manifest_hash == manifest_hash {
                        let manifest = PackageManifest::parse_json(&manifest_content)?;
                        let m_name = manifest
                            .name
                            .as_ref()
                            .map_or("unnamed", PackageName::as_str);
                        return Ok(InstallResult {
                            reconciled: true,
                            manifest_name: m_name.to_string(),
                            resolved_count: saved_state.installed_packages,
                            summary: saved_state.link_summary,
                            elapsed_ms: start_time.elapsed().as_millis(),
                        });
                    }
                }
            }
        }

        // Full Install Pipeline
        let manifest = PackageManifest::parse_json(&manifest_content)?;
        let client = MockRegistryClient::new(fixtures_dir);
        let resolver = DependencyResolver::new(&client, config);
        let graph = resolver.resolve(&manifest)?;

        let store_dir = custom_store_dir.map_or_else(
            || dirs_home_or_temp().join(".corex").join("store").join("v1"),
            PathBuf::from,
        );
        let store = Store::new(&store_dir);

        let linker = IsolatedLinker::new();
        let summary = linker.materialize(project_root, &graph, &store)?;

        // Register active CAS keys with global store
        let cas_keys: Vec<String> = graph
            .nodes
            .values()
            .map(|n| {
                format!(
                    "{}-{}",
                    n.package.name().as_str(),
                    n.version.version().as_str()
                )
            })
            .collect();
        let _ = store.register_project_references(project_root, &cas_keys);

        // Persist project state
        let now_secs = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let state = ProjectState {
            manifest_hash,
            installed_packages: graph.nodes.len(),
            installed_at_secs: now_secs,
            link_summary: summary.clone(),
        };

        let corex_dir = project_root.join(".corex");
        if !corex_dir.exists() {
            let _ = fs::create_dir_all(&corex_dir);
        }

        if let Ok(state_json) = serde_json::to_string_pretty(&state) {
            let _ = fs::write(&state_path, state_json);
        }

        let m_name = manifest
            .name
            .as_ref()
            .map_or("unnamed", PackageName::as_str);
        Ok(InstallResult {
            reconciled: false,
            manifest_name: m_name.to_string(),
            resolved_count: graph.nodes.len(),
            summary,
            elapsed_ms: start_time.elapsed().as_millis(),
        })
    }

    /// Adds a new dependency to `package.json` and reinstalls project dependencies.
    ///
    /// # Errors
    /// Returns `Diagnostic` if modifying `package.json` or installing fails.
    #[allow(clippy::too_many_arguments)]
    pub fn add_dependency(
        &self,
        project_root: &Path,
        config: &ProjectConfig,
        fixtures_dir: &Path,
        custom_store_dir: Option<&Path>,
        package_name: &str,
        version_spec: Option<&str>,
        is_dev: bool,
    ) -> Result<InstallResult, Diagnostic> {
        let manifest_path = project_root.join("package.json");
        let content = fs::read_to_string(&manifest_path).map_err(|e| {
            Diagnostic::new(
                ErrorFamily::Resolve,
                1,
                format!("failed reading `package.json`: {e}"),
            )
        })?;

        let mut val: serde_json::Value = serde_json::from_str(&content).map_err(|e| {
            Diagnostic::new(
                ErrorFamily::Resolve,
                2,
                format!("failed parsing `package.json`: {e}"),
            )
        })?;

        let section = if is_dev {
            "devDependencies"
        } else {
            "dependencies"
        };
        if val.get(section).is_none() {
            val[section] = serde_json::json!({});
        }

        let ver = version_spec.unwrap_or("*");
        val[section][package_name] = serde_json::Value::String(ver.to_string());

        let new_content = serde_json::to_string_pretty(&val).map_err(|e| {
            Diagnostic::new(
                ErrorFamily::Resolve,
                2,
                format!("failed serializing updated `package.json`: {e}"),
            )
        })?;

        fs::write(&manifest_path, new_content).map_err(|e| {
            Diagnostic::new(
                ErrorFamily::Resolve,
                1,
                format!("failed writing updated `package.json`: {e}"),
            )
        })?;

        self.install(project_root, config, fixtures_dir, custom_store_dir)
    }

    /// Removes a dependency from `package.json` and reinstalls project dependencies.
    ///
    /// # Errors
    /// Returns `Diagnostic` if modifying `package.json` or installing fails.
    pub fn remove_dependency(
        &self,
        project_root: &Path,
        config: &ProjectConfig,
        fixtures_dir: &Path,
        custom_store_dir: Option<&Path>,
        package_name: &str,
    ) -> Result<InstallResult, Diagnostic> {
        let manifest_path = project_root.join("package.json");
        let content = fs::read_to_string(&manifest_path).map_err(|e| {
            Diagnostic::new(
                ErrorFamily::Resolve,
                1,
                format!("failed reading `package.json`: {e}"),
            )
        })?;

        let mut val: serde_json::Value = serde_json::from_str(&content).map_err(|e| {
            Diagnostic::new(
                ErrorFamily::Resolve,
                2,
                format!("failed parsing `package.json`: {e}"),
            )
        })?;

        if let Some(deps) = val.get_mut("dependencies").and_then(|d| d.as_object_mut()) {
            deps.remove(package_name);
        }
        if let Some(dev_deps) = val
            .get_mut("devDependencies")
            .and_then(|d| d.as_object_mut())
        {
            dev_deps.remove(package_name);
        }

        let new_content = serde_json::to_string_pretty(&val).map_err(|e| {
            Diagnostic::new(
                ErrorFamily::Resolve,
                2,
                format!("failed serializing updated `package.json`: {e}"),
            )
        })?;

        fs::write(&manifest_path, new_content).map_err(|e| {
            Diagnostic::new(
                ErrorFamily::Resolve,
                1,
                format!("failed writing updated `package.json`: {e}"),
            )
        })?;

        self.install(project_root, config, fixtures_dir, custom_store_dir)
    }

    /// Formats a list of installed dependencies for the current project.
    ///
    /// # Errors
    /// Returns `Diagnostic` if reading manifest or resolving graph fails.
    pub fn list_dependencies(
        &self,
        project_root: &Path,
        config: &ProjectConfig,
        fixtures_dir: &Path,
    ) -> Result<serde_json::Value, Diagnostic> {
        let manifest_path = project_root.join("package.json");
        let content = fs::read_to_string(&manifest_path).map_err(|e| {
            Diagnostic::new(
                ErrorFamily::Resolve,
                1,
                format!("failed reading `package.json`: {e}"),
            )
        })?;

        let manifest = PackageManifest::parse_json(&content)?;
        let client = MockRegistryClient::new(fixtures_dir);
        let resolver = DependencyResolver::new(&client, config);
        let graph = resolver.resolve(&manifest)?;

        let mut nodes_list = Vec::new();
        for node in graph.nodes.values() {
            nodes_list.push(serde_json::json!({
                "name": node.package.name().as_str(),
                "version": node.version.version().as_str(),
                "is_root": graph.root_nodes.contains(&node.id),
            }));
        }

        let m_name = manifest
            .name
            .as_ref()
            .map_or("unnamed", PackageName::as_str);
        Ok(serde_json::json!({
            "name": m_name,
            "dependencies": nodes_list,
        }))
    }

    /// Inspects `package.json` for script `script_name` and returns command.
    ///
    /// # Errors
    /// Returns `Diagnostic` if script is not found in manifest.
    pub fn run_script(&self, project_root: &Path, script_name: &str) -> Result<String, Diagnostic> {
        let manifest_path = project_root.join("package.json");
        let content = fs::read_to_string(&manifest_path).map_err(|e| {
            Diagnostic::new(
                ErrorFamily::Script,
                1,
                format!("failed reading `package.json`: {e}"),
            )
        })?;

        let val: serde_json::Value = serde_json::from_str(&content).map_err(|e| {
            Diagnostic::new(
                ErrorFamily::Script,
                2,
                format!("failed parsing `package.json`: {e}"),
            )
        })?;

        if let Some(cmd) = val
            .get("scripts")
            .and_then(|s| s.get(script_name))
            .and_then(|c| c.as_str())
        {
            Ok(cmd.to_string())
        } else {
            Err(Diagnostic::new(
                ErrorFamily::Script,
                3,
                format!("missing script `{script_name}` in `package.json`"),
            )
            .with_help("check the `scripts` section in package.json"))
        }
    }

    /// Inspects `node_modules/.bin/` for executable `binary_name`.
    ///
    /// # Errors
    /// Returns `Diagnostic` if binary shim is not found.
    pub fn exec_binary(
        &self,
        project_root: &Path,
        binary_name: &str,
    ) -> Result<PathBuf, Diagnostic> {
        let bin_path = project_root
            .join("node_modules")
            .join(".bin")
            .join(binary_name);

        if bin_path.exists() {
            Ok(bin_path)
        } else {
            Err(Diagnostic::new(
                ErrorFamily::Script,
                4,
                format!("binary `{binary_name}` not found in `node_modules/.bin`"),
            )
            .with_help("ensure the dependency providing this binary is installed"))
        }
    }
}

fn compute_sha256(data: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data.as_bytes());
    hex::encode(hasher.finalize())
}

fn dirs_home_or_temp() -> PathBuf {
    std::env::var_os("HOME")
        .or_else(|| std::env::var_os("USERPROFILE"))
        .map_or_else(std::env::temp_dir, PathBuf::from)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_installer_service_end_to_end() {
        let temp_proj = std::env::temp_dir().join("corex_test_installer_proj");
        let custom_store = std::env::temp_dir().join("corex_test_installer_store");

        let _ = fs::remove_dir_all(&temp_proj);
        let _ = fs::remove_dir_all(&custom_store);

        fs::create_dir_all(&temp_proj).unwrap();
        fs::write(
            temp_proj.join("package.json"),
            r#"{
                "name": "my-app",
                "version": "1.0.0",
                "dependencies": {
                    "react": "^18.2.0"
                }
            }"#,
        )
        .unwrap();

        let config = ProjectConfig::default();
        let fixtures = find_fixtures_dir();
        let installer = InstallerService::new();

        // 1. Initial cold install
        let res1 = installer
            .install(&temp_proj, &config, &fixtures, Some(&custom_store))
            .unwrap();

        assert!(!res1.reconciled);
        assert_eq!(res1.manifest_name, "my-app");
        assert!(temp_proj.join("node_modules").join("react").exists());

        // 2. Warm fast reconciliation
        let res2 = installer
            .install(&temp_proj, &config, &fixtures, Some(&custom_store))
            .unwrap();

        assert!(res2.reconciled);

        let _ = fs::remove_dir_all(&temp_proj);
        let _ = fs::remove_dir_all(&custom_store);
    }

    fn find_fixtures_dir() -> PathBuf {
        let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
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
