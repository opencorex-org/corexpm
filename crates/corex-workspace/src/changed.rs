//! Changed and affected package calculation from workspace file paths.

use crate::graph::WorkspaceGraph;
use corex_errors::{Diagnostic, ErrorFamily};
use std::collections::{BTreeSet, VecDeque};
use std::path::{Path, PathBuf};
use std::process::Command;

/// Service for calculating changed and downstream affected workspace packages.
#[derive(Debug, Default)]
pub struct WorkspaceChanged;

impl WorkspaceChanged {
    /// Creates a new workspace changed calculator.
    #[must_use]
    pub fn new() -> Self {
        Self
    }

    /// Maps a list of modified file paths to directly changed workspace packages.
    #[must_use]
    pub fn calculate_changed(
        &self,
        graph: &WorkspaceGraph,
        root_dir: &Path,
        changed_paths: &[PathBuf],
    ) -> BTreeSet<String> {
        let mut changed_packages = BTreeSet::new();

        for rel_or_abs in changed_paths {
            let abs_path = if rel_or_abs.is_absolute() {
                rel_or_abs.clone()
            } else {
                root_dir.join(rel_or_abs)
            };

            for (name, node) in &graph.nodes {
                if abs_path.starts_with(&node.package.root_path) {
                    changed_packages.insert(name.clone());
                }
            }
        }

        changed_packages
    }

    /// Calculates affected downstream packages (changed packages + transitive dependents).
    #[must_use]
    pub fn calculate_affected(
        &self,
        graph: &WorkspaceGraph,
        changed_packages: &BTreeSet<String>,
    ) -> BTreeSet<String> {
        let mut affected = changed_packages.clone();
        let mut queue: VecDeque<String> = changed_packages.iter().cloned().collect();

        while let Some(current) = queue.pop_front() {
            if let Some(node) = graph.nodes.get(&current) {
                for dependent in &node.dependents {
                    if affected.insert(dependent.clone()) {
                        queue.push_back(dependent.clone());
                    }
                }
            }
        }

        affected
    }

    /// Attempts to query git for modified or untracked paths in `root_dir`.
    ///
    /// # Errors
    ///
    /// Returns a [`Diagnostic`] if git command execution fails.
    pub fn detect_git_changed_paths(&self, root_dir: &Path) -> Result<Vec<PathBuf>, Diagnostic> {
        let output = Command::new("git")
            .arg("status")
            .arg("--porcelain")
            .current_dir(root_dir)
            .output()
            .map_err(|e| {
                Diagnostic::new(
                    ErrorFamily::Workspace,
                    2,
                    format!(
                        "failed to execute git status in `{}`: {e}",
                        root_dir.display()
                    ),
                )
            })?;

        if !output.status.success() {
            return Ok(Vec::new());
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let mut paths = Vec::new();

        for line in stdout.lines() {
            if line.len() > 3 {
                let path_str = line[3..].trim().trim_matches('"');
                if !path_str.is_empty() {
                    paths.push(PathBuf::from(path_str));
                }
            }
        }

        Ok(paths)
    }
}
