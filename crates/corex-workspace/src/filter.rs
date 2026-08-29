//! Filtering options for workspace package selection.

use crate::graph::WorkspaceGraph;
use corex_errors::{Diagnostic, ErrorFamily};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

/// User-provided package selection filters.
#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct WorkspaceFilter {
    /// Explicit package targets (`--workspace` / `-w`).
    pub target_workspaces: Vec<String>,
    /// Inclusion patterns (`--include`).
    pub include_patterns: Vec<String>,
    /// Exclusion patterns (`--exclude`).
    pub exclude_patterns: Vec<String>,
    /// Whether all packages should be included (`--all`).
    pub all: bool,
}

impl WorkspaceFilter {
    /// Applies filters to select packages from the workspace graph.
    ///
    /// # Errors
    ///
    /// Returns a [`Diagnostic`] if the filter yields no matching packages.
    pub fn select_packages(
        &self,
        graph: &WorkspaceGraph,
        base_set: Option<&BTreeSet<String>>,
    ) -> Result<BTreeSet<String>, Diagnostic> {
        let all_names: BTreeSet<String> = base_set
            .cloned()
            .unwrap_or_else(|| graph.nodes.keys().cloned().collect());

        if all_names.is_empty() {
            return Err(Diagnostic::new(
                ErrorFamily::Workspace,
                2,
                "workspace contains no package members",
            ));
        }

        let mut selected = BTreeSet::new();

        if self.all
            || (self.target_workspaces.is_empty()
                && self.include_patterns.is_empty()
                && base_set.is_none())
        {
            selected.clone_from(&all_names);
        } else {
            // Apply base set or direct targets
            if let Some(base) = base_set {
                selected.clone_from(base);
            }

            for target in &self.target_workspaces {
                for name in &all_names {
                    if matches_pattern(name, target) {
                        selected.insert(name.clone());
                    }
                }
            }

            for inc in &self.include_patterns {
                for name in &all_names {
                    if matches_pattern(name, inc) {
                        selected.insert(name.clone());
                    }
                }
            }
        }

        // Apply exclusion filters
        if !self.exclude_patterns.is_empty() {
            selected.retain(|name| {
                !self
                    .exclude_patterns
                    .iter()
                    .any(|exc| matches_pattern(name, exc))
            });
        }

        if selected.is_empty() {
            return Err(Diagnostic::new(
                ErrorFamily::Workspace,
                2,
                "workspace filter matched 0 package members",
            )
            .with_help(
                "check package names or glob patterns passed to --workspace, --include, or --exclude",
            ));
        }

        Ok(selected)
    }
}

fn matches_pattern(name: &str, pattern: &str) -> bool {
    if pattern == "*" || pattern == name {
        return true;
    }
    if let Some(prefix) = pattern.strip_suffix('*') {
        return name.starts_with(prefix);
    }
    false
}
