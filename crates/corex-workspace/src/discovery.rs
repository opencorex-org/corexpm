//! Workspace discovery from root `package.json` and `corex.toml` manifests.

use corex_errors::{Diagnostic, ErrorFamily};
use corex_manifest::PackageManifest;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

/// A discovered workspace package member.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct WorkspacePackage {
    /// Package name as defined in its `package.json`.
    pub name: String,
    /// Relative path from workspace root (e.g. `packages/auth`).
    pub relative_path: PathBuf,
    /// Absolute path to the package directory.
    pub root_path: PathBuf,
    /// Parsed manifest of the package.
    pub manifest: PackageManifest,
}

/// The result of workspace discovery.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct WorkspaceMetadata {
    /// Absolute path to the workspace root.
    pub root_dir: PathBuf,
    /// Root `package.json` manifest.
    pub root_manifest: PackageManifest,
    /// Map from package name to workspace member package.
    pub packages: BTreeMap<String, WorkspacePackage>,
}

/// Helper service for workspace member discovery.
#[derive(Debug, Default)]
pub struct WorkspaceDiscovery;

impl WorkspaceDiscovery {
    /// Creates a new workspace discovery instance.
    #[must_use]
    pub fn new() -> Self {
        Self
    }

    /// Discovers workspace configuration and package members starting from `start_dir`.
    ///
    /// # Errors
    ///
    /// Returns a [`Diagnostic`] if no workspace root is found or parsing manifests fails.
    pub fn discover(&self, start_dir: &Path) -> Result<WorkspaceMetadata, Diagnostic> {
        let root_dir = find_workspace_root(start_dir)?;
        let root_manifest_path = root_dir.join("package.json");

        let root_manifest = if root_manifest_path.exists() {
            let content = fs::read_to_string(&root_manifest_path).map_err(|e| {
                Diagnostic::new(
                    ErrorFamily::Workspace,
                    2,
                    format!(
                        "failed to read root package.json at `{}`: {e}",
                        root_dir.display()
                    ),
                )
            })?;
            PackageManifest::parse_json(&content)?
        } else {
            PackageManifest::default()
        };

        let member_patterns = resolve_patterns(&root_dir, &root_manifest);
        let mut packages = BTreeMap::new();

        for pattern in member_patterns {
            let matched_dirs = expand_glob(&root_dir, &pattern);
            for member_dir in matched_dirs {
                if member_dir == root_dir {
                    continue; // Skip root itself if matched
                }

                let manifest_path = member_dir.join("package.json");
                if !manifest_path.exists() {
                    continue;
                }

                let Ok(content) = fs::read_to_string(&manifest_path) else {
                    continue;
                };

                let manifest = PackageManifest::parse_json(&content)?;

                let name = match &manifest.name {
                    Some(n) => n.as_str().to_string(),
                    None => {
                        return Err(Diagnostic::new(
                            ErrorFamily::Workspace,
                            2,
                            format!(
                                "workspace package at `{}` missing required `name` field in package.json",
                                member_dir.display()
                            ),
                        ));
                    }
                };

                let relative_path = member_dir
                    .strip_prefix(&root_dir)
                    .unwrap_or(&member_dir)
                    .to_path_buf();

                if packages.contains_key(&name) {
                    return Err(Diagnostic::new(
                        ErrorFamily::Workspace,
                        2,
                        format!(
                            "duplicate package name `{name}` found in workspace at `{}`",
                            member_dir.display()
                        ),
                    )
                    .with_help("workspace package names must be unique within a monorepo"));
                }

                packages.insert(
                    name.clone(),
                    WorkspacePackage {
                        name,
                        relative_path,
                        root_path: member_dir,
                        manifest,
                    },
                );
            }
        }

        Ok(WorkspaceMetadata {
            root_dir,
            root_manifest,
            packages,
        })
    }
}

fn find_workspace_root(start_dir: &Path) -> Result<PathBuf, Diagnostic> {
    let mut current = if start_dir.is_file() {
        start_dir.parent().unwrap_or(start_dir)
    } else {
        start_dir
    };

    loop {
        let manifest_path = current.join("package.json");
        if manifest_path.exists() {
            if let Ok(content) = fs::read_to_string(&manifest_path) {
                if let Ok(manifest) = PackageManifest::parse_json(&content) {
                    if !manifest.workspaces.is_empty() {
                        return Ok(current.to_path_buf());
                    }
                }
            }
        }

        let toml_path = current.join("corex.toml");
        if toml_path.exists() {
            if let Ok(content) = fs::read_to_string(&toml_path) {
                if content.contains("[workspace]") {
                    return Ok(current.to_path_buf());
                }
            }
        }

        if let Some(parent) = current.parent() {
            current = parent;
        } else {
            break;
        }
    }

    // If no root with explicit workspace pattern is found, fall back to start_dir if it has package.json
    if start_dir.join("package.json").exists() || start_dir.join("corex.toml").exists() {
        return Ok(start_dir.to_path_buf());
    }

    Err(Diagnostic::new(
        ErrorFamily::Workspace,
        2,
        format!(
            "no workspace root found starting from `{}`",
            start_dir.display()
        ),
    )
    .with_help(
        "ensure root package.json defines `workspaces` or corex.toml contains `[workspace]`",
    ))
}

fn resolve_patterns(root_dir: &Path, root_manifest: &PackageManifest) -> Vec<String> {
    if !root_manifest.workspaces.is_empty() {
        return root_manifest.workspaces.clone();
    }

    let toml_path = root_dir.join("corex.toml");
    if toml_path.exists() {
        if let Ok(content) = fs::read_to_string(&toml_path) {
            if let Ok(val) = toml::from_str::<toml::Value>(&content) {
                if let Some(ws) = val.get("workspace") {
                    if let Some(members) = ws.get("members").and_then(toml::Value::as_array) {
                        let mut patterns = Vec::new();
                        for item in members {
                            if let Some(s) = item.as_str() {
                                patterns.push(s.to_string());
                            }
                        }
                        if !patterns.is_empty() {
                            return patterns;
                        }
                    }
                }
            }
        }
    }

    Vec::new()
}

fn expand_glob(root_dir: &Path, pattern: &str) -> Vec<PathBuf> {
    let clean_pattern = pattern.trim().trim_start_matches("./");
    let mut results = Vec::new();

    if clean_pattern.ends_with("/*") {
        let parent_rel = clean_pattern.strip_suffix("/*").unwrap_or("");
        let base_dir = if parent_rel.is_empty() {
            root_dir.to_path_buf()
        } else {
            root_dir.join(parent_rel)
        };

        if base_dir.is_dir() {
            if let Ok(entries) = fs::read_dir(&base_dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.is_dir() && path.join("package.json").exists() {
                        results.push(path);
                    }
                }
            }
        }
    } else if clean_pattern.ends_with("/**") {
        let parent_rel = clean_pattern.strip_suffix("/**").unwrap_or("");
        let base_dir = if parent_rel.is_empty() {
            root_dir.to_path_buf()
        } else {
            root_dir.join(parent_rel)
        };

        let mut stack = vec![base_dir];
        while let Some(dir) = stack.pop() {
            if dir.is_dir() {
                if dir != root_dir && dir.join("package.json").exists() {
                    results.push(dir.clone());
                }
                if let Ok(entries) = fs::read_dir(&dir) {
                    for entry in entries.flatten() {
                        let path = entry.path();
                        if path.is_dir() {
                            let file_name = entry.file_name();
                            let name_str = file_name.to_string_lossy();
                            if name_str != "node_modules" && !name_str.starts_with('.') {
                                stack.push(path);
                            }
                        }
                    }
                }
            }
        }
    } else {
        let target_dir = root_dir.join(clean_pattern);
        if target_dir.is_dir() && target_dir.join("package.json").exists() {
            results.push(target_dir);
        }
    }

    results.sort();
    results
}
