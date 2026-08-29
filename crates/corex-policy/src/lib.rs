//! Corex Guard baseline trust policy evaluation and permissions reporting.

#![forbid(unsafe_code)]

use corex_errors::{Diagnostic, ErrorFamily};
use corex_graph::DependencyGraph;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

/// Effective policy action for a package lifecycle script.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
pub enum ScriptPolicyAction {
    /// Script execution is explicitly allowed.
    Allow,
    /// Script execution is denied by default or policy.
    #[default]
    Deny,
    /// Interactive prompt requested (falls back to Deny in CI).
    Prompt,
}

/// Explicit trust decision for a package.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum TrustDecision {
    /// Package lifecycle scripts are approved.
    Approved,
    /// Package lifecycle scripts are denied.
    Denied,
}

/// Persisted trust configuration mapping packages to explicit trust decisions.
#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct TrustStore {
    /// Default policy action for unlisted packages.
    #[serde(default)]
    pub default_action: ScriptPolicyAction,
    /// Explicit package trust decisions.
    #[serde(default)]
    pub packages: BTreeMap<String, TrustDecision>,
}

impl TrustStore {
    /// Loads a `TrustStore` from JSON file at `path`.
    ///
    /// # Errors
    /// Returns `Diagnostic` if reading or parsing fails.
    pub fn load_from_path(path: &Path) -> Result<Self, Diagnostic> {
        if !path.exists() {
            return Ok(Self::default());
        }
        let content = fs::read_to_string(path).map_err(|e| {
            Diagnostic::new(
                ErrorFamily::Security,
                1,
                format!("failed reading trust file `{}`: {e}", path.display()),
            )
        })?;

        serde_json::from_str(&content).map_err(|e| {
            Diagnostic::new(
                ErrorFamily::Security,
                2,
                format!("failed parsing trust configuration: {e}"),
            )
        })
    }

    /// Saves `TrustStore` to JSON file at `path`.
    ///
    /// # Errors
    /// Returns `Diagnostic` if writing fails.
    pub fn save_to_path(&self, path: &Path) -> Result<(), Diagnostic> {
        if let Some(parent) = path.parent() {
            if !parent.exists() {
                let _ = fs::create_dir_all(parent);
            }
        }
        let content = serde_json::to_string_pretty(self).map_err(|e| {
            Diagnostic::new(
                ErrorFamily::Security,
                3,
                format!("failed serializing trust configuration: {e}"),
            )
        })?;

        fs::write(path, content).map_err(|e| {
            Diagnostic::new(
                ErrorFamily::Security,
                4,
                format!("failed writing trust file `{}`: {e}", path.display()),
            )
        })
    }
}

/// Summary entry for a package in `PermissionsReport`.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PermissionsEntry {
    /// Name of the package.
    pub package_name: String,
    /// True if package manifest declares lifecycle scripts.
    pub has_lifecycle_scripts: bool,
    /// List of declared lifecycle script names (e.g. `install`, `postinstall`).
    pub declared_scripts: Vec<String>,
    /// Effective script policy action (`Allow` or `Deny`).
    pub effective_action: ScriptPolicyAction,
    /// Source of policy decision (e.g. `project trust`, `global trust`, `deny-by-default`).
    pub policy_source: String,
    /// Actionable UX explanation and safe next steps.
    pub explanation: String,
}

/// Overall permissions report for a project dependency tree (`corexpm permissions`).
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct PermissionsReport {
    /// Default policy action applied to untrusted packages.
    pub default_policy: ScriptPolicyAction,
    /// Detailed permissions entries per package.
    pub entries: Vec<PermissionsEntry>,
}

/// Policy evaluation engine enforcing Corex Guard baseline security.
#[derive(Clone, Debug, Default)]
pub struct PolicyEngine;

impl PolicyEngine {
    /// Creates a new `PolicyEngine`.
    #[must_use]
    pub fn new() -> Self {
        Self
    }

    /// Evaluates effective script policy action for `package_name`.
    #[must_use]
    pub fn evaluate_script_policy(
        &self,
        project_root: &Path,
        package_name: &str,
        is_ci: bool,
    ) -> (ScriptPolicyAction, String) {
        let proj_trust_path = project_root.join(".corex").join("trust.json");
        if let Ok(store) = TrustStore::load_from_path(&proj_trust_path) {
            if let Some(decision) = store.packages.get(package_name) {
                return match decision {
                    TrustDecision::Approved => (
                        ScriptPolicyAction::Allow,
                        "project .corex/trust.json".to_string(),
                    ),
                    TrustDecision::Denied => (
                        ScriptPolicyAction::Deny,
                        "project .corex/trust.json (explicit deny)".to_string(),
                    ),
                };
            }
        }

        let global_trust_path = dirs_home_or_temp().join(".corex").join("trust.json");
        if let Ok(store) = TrustStore::load_from_path(&global_trust_path) {
            if let Some(decision) = store.packages.get(package_name) {
                return match decision {
                    TrustDecision::Approved => (
                        ScriptPolicyAction::Allow,
                        "global ~/.corex/trust.json".to_string(),
                    ),
                    TrustDecision::Denied => (
                        ScriptPolicyAction::Deny,
                        "global ~/.corex/trust.json (explicit deny)".to_string(),
                    ),
                };
            }
        }

        let source = if is_ci {
            "deny-by-default (CI mode)"
        } else {
            "deny-by-default (Corex Guard policy)"
        };

        (ScriptPolicyAction::Deny, source.to_string())
    }

    /// Explicitly approves lifecycle scripts for `package_name` in project trust configuration.
    ///
    /// # Errors
    /// Returns `Diagnostic` if reading or saving trust configuration fails.
    pub fn approve_package(
        &self,
        project_root: &Path,
        package_name: &str,
    ) -> Result<(), Diagnostic> {
        let trust_path = project_root.join(".corex").join("trust.json");
        let mut store = TrustStore::load_from_path(&trust_path)?;
        store
            .packages
            .insert(package_name.to_string(), TrustDecision::Approved);
        store.save_to_path(&trust_path)
    }

    /// Explicitly denies lifecycle scripts for `package_name` in project trust configuration.
    ///
    /// # Errors
    /// Returns `Diagnostic` if reading or saving trust configuration fails.
    pub fn deny_package(&self, project_root: &Path, package_name: &str) -> Result<(), Diagnostic> {
        let trust_path = project_root.join(".corex").join("trust.json");
        let mut store = TrustStore::load_from_path(&trust_path)?;
        store
            .packages
            .insert(package_name.to_string(), TrustDecision::Denied);
        store.save_to_path(&trust_path)
    }

    /// Generates a `PermissionsReport` explaining policy status across `graph` packages.
    #[must_use]
    pub fn generate_permissions_report(
        &self,
        project_root: &Path,
        graph: &DependencyGraph,
        is_ci: bool,
    ) -> PermissionsReport {
        let mut entries = Vec::new();

        for node in graph.nodes.values() {
            let pkg_name = node.package.name().as_str();
            let (action, source) = self.evaluate_script_policy(project_root, pkg_name, is_ci);

            let explanation = match action {
                ScriptPolicyAction::Allow => format!("Lifecycle scripts for `{pkg_name}` are approved via {source}."),
                ScriptPolicyAction::Deny => format!(
                    "Lifecycle scripts for `{pkg_name}` are denied by default. Run `corexpm trust approve {pkg_name}` to explicitly trust this dependency."
                ),
                ScriptPolicyAction::Prompt => format!("Interactive approval prompt required for `{pkg_name}`."),
            };

            entries.push(PermissionsEntry {
                package_name: pkg_name.to_string(),
                has_lifecycle_scripts: false,
                declared_scripts: Vec::new(),
                effective_action: action,
                policy_source: source,
                explanation,
            });
        }

        PermissionsReport {
            default_policy: ScriptPolicyAction::Deny,
            entries,
        }
    }
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
    fn test_policy_deny_by_default_and_approve() {
        let temp_proj = std::env::temp_dir().join("corex_test_policy_proj");
        let _ = fs::remove_dir_all(&temp_proj);
        fs::create_dir_all(&temp_proj).unwrap();

        let engine = PolicyEngine::new();

        // 1. Untrusted package is denied by default
        let (action1, source1) = engine.evaluate_script_policy(&temp_proj, "esbuild", false);
        assert_eq!(action1, ScriptPolicyAction::Deny);
        assert!(source1.contains("deny-by-default"));

        // 2. Approve package
        engine.approve_package(&temp_proj, "esbuild").unwrap();

        // 3. Approved package is allowed
        let (action2, source2) = engine.evaluate_script_policy(&temp_proj, "esbuild", false);
        assert_eq!(action2, ScriptPolicyAction::Allow);
        assert!(source2.contains("project .corex/trust.json"));

        let _ = fs::remove_dir_all(&temp_proj);
    }
}
