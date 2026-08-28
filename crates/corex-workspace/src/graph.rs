//! Workspace dependency graph, protocol resolution, and cycle detection.

use crate::discovery::{WorkspaceMetadata, WorkspacePackage};
use corex_errors::{Diagnostic, ErrorFamily};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet, HashSet};

/// A node in the workspace dependency graph.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct WorkspaceNode {
    /// Package metadata.
    pub package: WorkspacePackage,
    /// Set of package names in the workspace that this package directly depends on.
    pub dependencies: BTreeSet<String>,
    /// Set of package names in the workspace that depend on this package.
    pub dependents: BTreeSet<String>,
}

/// Workspace dependency graph and resolution.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct WorkspaceGraph {
    /// Map of package name to workspace node.
    pub nodes: BTreeMap<String, WorkspaceNode>,
    /// Topologically ordered execution waves (parallel execution groups).
    pub execution_waves: Vec<Vec<String>>,
}

impl WorkspaceGraph {
    /// Builds a workspace graph from discovered workspace metadata.
    ///
    /// # Errors
    ///
    /// Returns a [`Diagnostic`] with error code `CXWORK0001` if a dependency cycle is detected.
    pub fn build(metadata: &WorkspaceMetadata) -> Result<Self, Diagnostic> {
        let mut nodes: BTreeMap<String, WorkspaceNode> = metadata
            .packages
            .iter()
            .map(|(name, pkg)| {
                (
                    name.clone(),
                    WorkspaceNode {
                        package: pkg.clone(),
                        dependencies: BTreeSet::new(),
                        dependents: BTreeSet::new(),
                    },
                )
            })
            .collect();

        // Populate dependency edges
        let package_names: HashSet<String> = metadata.packages.keys().cloned().collect();

        for (pkg_name, pkg) in &metadata.packages {
            let mut dep_names = BTreeSet::new();

            let all_deps = pkg
                .manifest
                .dependencies
                .iter()
                .chain(&pkg.manifest.dev_dependencies)
                .chain(&pkg.manifest.optional_dependencies)
                .chain(&pkg.manifest.peer_dependencies);

            for (dep_name, version_spec) in all_deps {
                let dep_str = dep_name.as_str();

                // Check workspace protocol or workspace package match
                if package_names.contains(dep_str) {
                    if version_spec.starts_with("workspace:") || is_workspace_match(version_spec) {
                        dep_names.insert(dep_str.to_string());
                    } else if !version_spec.starts_with("http") && !version_spec.starts_with("git")
                    {
                        // Default matching for local workspace packages when in monorepo
                        dep_names.insert(dep_str.to_string());
                    }
                }
            }

            // Remove self-dependency if any
            dep_names.remove(pkg_name);

            if let Some(node) = nodes.get_mut(pkg_name) {
                node.dependencies = dep_names;
            }
        }

        // Populate reverse edges (dependents)
        let edges: Vec<(String, String)> = nodes
            .iter()
            .flat_map(|(name, node)| {
                node.dependencies
                    .iter()
                    .map(|dep| (name.clone(), dep.clone()))
            })
            .collect();

        for (from, to) in edges {
            if let Some(node) = nodes.get_mut(&to) {
                node.dependents.insert(from);
            }
        }

        // Cycle detection and wave calculation via Kahn's Algorithm
        let execution_waves = compute_topological_waves(&nodes)?;

        Ok(Self {
            nodes,
            execution_waves,
        })
    }

    /// Returns a list of all package names in topological execution order.
    #[must_use]
    pub fn topological_order(&self) -> Vec<String> {
        self.execution_waves.iter().flatten().cloned().collect()
    }
}

fn is_workspace_match(spec: &str) -> bool {
    spec.starts_with("workspace:") || spec == "*"
}

fn compute_topological_waves(
    nodes: &BTreeMap<String, WorkspaceNode>,
) -> Result<Vec<Vec<String>>, Diagnostic> {
    let mut in_degree: BTreeMap<String, usize> = BTreeMap::new();
    for (name, node) in nodes {
        in_degree.insert(name.clone(), node.dependencies.len());
    }

    let mut waves = Vec::new();
    let mut processed_count = 0;
    let total_nodes = nodes.len();

    while processed_count < total_nodes {
        // Collect all nodes with 0 remaining in-degree
        let mut current_wave: Vec<String> = in_degree
            .iter()
            .filter(|(_, &deg)| deg == 0)
            .map(|(name, _)| name.clone())
            .collect();

        if current_wave.is_empty() {
            // Cycle detected! Perform DFS to extract exact cycle path for diagnostic
            let cycle_path = detect_cycle_path(nodes);
            return Err(Diagnostic::new(
                ErrorFamily::Workspace,
                1,
                format!(
                    "workspace dependency cycle detected: {}",
                    cycle_path.join(" -> ")
                ),
            )
            .with_help("refactor workspace package dependencies to break circular references"));
        }

        // Sort wave deterministically
        current_wave.sort();

        // Mark current wave nodes as processed (set in_degree to usize::MAX)
        for name in &current_wave {
            in_degree.insert(name.clone(), usize::MAX);
            processed_count += 1;
        }

        // Decrement in-degree for dependents of current wave nodes
        for name in &current_wave {
            if let Some(node) = nodes.get(name) {
                for dependent in &node.dependents {
                    if let Some(deg) = in_degree.get_mut(dependent) {
                        if *deg != usize::MAX && *deg > 0 {
                            *deg -= 1;
                        }
                    }
                }
            }
        }

        waves.push(current_wave);
    }

    Ok(waves)
}

fn dfs_cycle(
    curr: &str,
    nodes: &BTreeMap<String, WorkspaceNode>,
    visited: &mut HashSet<String>,
    stack: &mut Vec<String>,
    path: &mut Vec<String>,
) -> bool {
    visited.insert(curr.to_string());
    stack.push(curr.to_string());

    if let Some(node) = nodes.get(curr) {
        for dep in &node.dependencies {
            if stack.contains(dep) {
                let start_idx = stack.iter().position(|r| r == dep).unwrap_or(0);
                path.extend(stack[start_idx..].iter().cloned());
                path.push(dep.clone());
                return true;
            }
            if !visited.contains(dep) && dfs_cycle(dep, nodes, visited, stack, path) {
                return true;
            }
        }
    }

    stack.pop();
    false
}

fn detect_cycle_path(nodes: &BTreeMap<String, WorkspaceNode>) -> Vec<String> {
    let mut visited = HashSet::new();
    let mut stack = Vec::new();
    let mut path = Vec::new();

    for name in nodes.keys() {
        if !visited.contains(name) && dfs_cycle(name, nodes, &mut visited, &mut stack, &mut path) {
            return path;
        }
    }

    nodes.keys().cloned().collect()
}
