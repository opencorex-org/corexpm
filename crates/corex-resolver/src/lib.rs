//! Dependency resolver algorithm.

use corex_config::ProjectConfig;
use corex_errors::{Diagnostic, ErrorFamily};
use corex_graph::{DependencyGraph, DependencyKind, NodeId};
use corex_manifest::{PackageManifest, PackageName};
use corex_registry::{RegistryClient, RegistryVersionMetadata};
use corex_semver::{Range, Version};
use std::collections::BTreeMap;

/// Dependency resolver orchestrator.
pub struct DependencyResolver<'a> {
    client: &'a dyn RegistryClient,
    _config: &'a ProjectConfig,
    os: String,
    cpu: String,
}

impl<'a> DependencyResolver<'a> {
    /// Creates a new `DependencyResolver` with the given registry client and project configuration.
    #[must_use]
    pub fn new(client: &'a dyn RegistryClient, config: &'a ProjectConfig) -> Self {
        Self {
            client,
            _config: config,
            os: std::env::consts::OS.to_string(),
            cpu: std::env::consts::ARCH.to_string(),
        }
    }

    /// Configures custom target platform constraints for testing.
    #[must_use]
    pub fn with_platform(mut self, os: impl Into<String>, cpu: impl Into<String>) -> Self {
        self.os = os.into();
        self.cpu = cpu.into();
        self
    }

    fn platform_matches(list: &[String], target: &str) -> bool {
        if list.is_empty() {
            return true;
        }
        let mut matched = false;
        let mut has_positive = false;
        for item in list {
            if let Some(stripped) = item.strip_prefix('!') {
                if stripped == target {
                    return false;
                }
            } else {
                has_positive = true;
                if item == target {
                    matched = true;
                }
            }
        }
        if has_positive {
            matched
        } else {
            true
        }
    }

    #[allow(clippy::too_many_lines)]
    fn resolve_node(
        &self,
        name: &PackageName,
        range: &Range,
        kind: DependencyKind,
        parent_chain: &mut Vec<(PackageName, Version)>,
        graph: &mut DependencyGraph,
        resolved: &mut BTreeMap<PackageName, NodeId>,
    ) -> Result<Option<NodeId>, Diagnostic> {
        if let Some(&node_id) = resolved.get(name) {
            let version = graph.nodes.get(&node_id).unwrap().version.version();
            if range.satisfies(version) {
                return Ok(Some(node_id));
            }
        }

        let metadata = self.client.fetch_metadata(name)?;

        let mut candidates: Vec<&RegistryVersionMetadata> = metadata
            .versions
            .values()
            .filter(|v| range.satisfies(&v.version))
            .collect();

        candidates.sort_by(|a, b| b.version.cmp(&a.version));

        if candidates.is_empty() {
            return Err(Diagnostic::new(
                ErrorFamily::Resolve,
                1,
                format!(
                    "no satisfying version found for package `{}` matching range `{}`",
                    name.as_str(),
                    range
                ),
            ));
        }

        for candidate in candidates {
            let os_ok = Self::platform_matches(&candidate.os, &self.os);
            let cpu_ok = Self::platform_matches(&candidate.cpu, &self.cpu);

            if !os_ok || !cpu_ok {
                if kind == DependencyKind::OptionalDependency {
                    return Ok(None);
                }
                continue;
            }

            if parent_chain
                .iter()
                .any(|(p_name, p_ver)| p_name == name && p_ver == &candidate.version)
            {
                if let Some(&id) = resolved.get(name) {
                    return Ok(Some(id));
                }
                continue;
            }

            parent_chain.push((name.clone(), candidate.version.clone()));

            let node_id = graph.add_node(
                name.clone(),
                candidate.version.clone(),
                candidate.dist.tarball.clone(),
                candidate.dist.integrity.clone(),
            );

            resolved.insert(name.clone(), node_id);

            for (dep_name, dep_range_str) in &candidate.dependencies {
                let dep_range = Range::parse(dep_range_str)?;
                if let Some(dep_node_id) = self.resolve_node(
                    dep_name,
                    &dep_range,
                    DependencyKind::Dependency,
                    parent_chain,
                    graph,
                    resolved,
                )? {
                    graph.add_edge(node_id, dep_node_id, DependencyKind::Dependency, dep_range);
                }
            }

            for (dep_name, dep_range_str) in &candidate.optional_dependencies {
                let dep_range = Range::parse(dep_range_str)?;
                if let Some(dep_node_id) = self.resolve_node(
                    dep_name,
                    &dep_range,
                    DependencyKind::OptionalDependency,
                    parent_chain,
                    graph,
                    resolved,
                )? {
                    graph.add_edge(
                        node_id,
                        dep_node_id,
                        DependencyKind::OptionalDependency,
                        dep_range,
                    );
                }
            }

            for (peer_name, peer_range_str) in &candidate.peer_dependencies {
                let peer_range = Range::parse(peer_range_str)?;
                let mut peer_satisfied = false;
                for (parent_name, parent_ver) in parent_chain.iter().rev() {
                    if parent_name == peer_name && peer_range.satisfies(parent_ver) {
                        peer_satisfied = true;
                        break;
                    }
                }

                if !peer_satisfied {
                    if let Some(&sibling_id) = resolved.get(peer_name) {
                        let sibling_ver = graph.nodes.get(&sibling_id).unwrap().version.version();
                        if peer_range.satisfies(sibling_ver) {
                            peer_satisfied = true;
                            graph.add_edge(
                                node_id,
                                sibling_id,
                                DependencyKind::PeerDependency,
                                peer_range.clone(),
                            );
                        }
                    }
                }

                if !peer_satisfied {
                    return Err(Diagnostic::new(
                        ErrorFamily::Resolve,
                        3,
                        format!(
                            "peer dependency conflict: package `{}` version `{}` requires peer `{}` matching `{}` but it was not found in the dependency path",
                            name.as_str(),
                            candidate.version,
                            peer_name.as_str(),
                            peer_range
                        ),
                    ).with_help("verify that peer dependencies are declared in the root package.json"));
                }
            }

            parent_chain.pop();
            return Ok(Some(node_id));
        }

        Err(Diagnostic::new(
            ErrorFamily::Resolve,
            1,
            format!(
                "no compatible versions of package `{}` matching range `{}` found for target platform OS=`{}`, CPU=`{}`",
                name.as_str(),
                range,
                self.os,
                self.cpu
            ),
        ))
    }

    /// Resolves a package manifest to a resolved dependency graph.
    ///
    /// # Errors
    ///
    /// Returns a [`Diagnostic`] when resolution fails due to missing versions,
    /// incompatible platforms, or unsatisfied peer dependencies.
    pub fn resolve(&self, manifest: &PackageManifest) -> Result<DependencyGraph, Diagnostic> {
        let mut graph = DependencyGraph::default();
        let mut parent_chain = Vec::new();
        let mut resolved = BTreeMap::new();

        for (name, range_str) in &manifest.dependencies {
            let range = Range::parse(range_str)?;
            if let Some(node_id) = self.resolve_node(
                name,
                &range,
                DependencyKind::Dependency,
                &mut parent_chain,
                &mut graph,
                &mut resolved,
            )? {
                graph.root_nodes.insert(node_id);
            }
        }

        for (name, range_str) in &manifest.dev_dependencies {
            let range = Range::parse(range_str)?;
            if let Some(node_id) = self.resolve_node(
                name,
                &range,
                DependencyKind::DevDependency,
                &mut parent_chain,
                &mut graph,
                &mut resolved,
            )? {
                graph.root_nodes.insert(node_id);
            }
        }

        for (name, range_str) in &manifest.optional_dependencies {
            let range = Range::parse(range_str)?;
            if let Some(node_id) = self.resolve_node(
                name,
                &range,
                DependencyKind::OptionalDependency,
                &mut parent_chain,
                &mut graph,
                &mut resolved,
            )? {
                graph.root_nodes.insert(node_id);
            }
        }

        Ok(graph)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use corex_registry::MockRegistryClient;
    use std::path::PathBuf;

    fn get_fixtures_dir() -> PathBuf {
        let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        path.pop(); // crates/
        path.pop(); // root/
        path.join("tests").join("fixtures").join("registry")
    }

    #[test]
    fn test_resolve_simple() {
        let fixtures = get_fixtures_dir();
        let client = MockRegistryClient::new(fixtures);
        let config = ProjectConfig::default();
        let resolver = DependencyResolver::new(&client, &config);

        let mut manifest = PackageManifest::default();
        manifest.dependencies.insert(
            corex_manifest::PackageName::parse("react-dom").unwrap(),
            "^18.0.0".to_string(),
        );

        let graph = resolver.resolve(&manifest).unwrap();
        assert_eq!(graph.root_nodes.len(), 1);
        assert_eq!(graph.nodes.len(), 2); // react-dom and react
        assert_eq!(graph.edges.len(), 1);
    }

    #[test]
    fn test_resolve_cycles() {
        let fixtures = get_fixtures_dir();
        let client = MockRegistryClient::new(fixtures);
        let config = ProjectConfig::default();
        let resolver = DependencyResolver::new(&client, &config);

        let mut manifest = PackageManifest::default();
        manifest.dependencies.insert(
            corex_manifest::PackageName::parse("cyclic-a").unwrap(),
            "^1.0.0".to_string(),
        );

        let graph = resolver.resolve(&manifest).unwrap();
        assert_eq!(graph.nodes.len(), 2); // cyclic-a and cyclic-b
        assert_eq!(graph.edges.len(), 2); // a -> b and b -> a
    }

    #[test]
    fn test_resolve_peer_dependencies() {
        let fixtures = get_fixtures_dir();
        let client = MockRegistryClient::new(fixtures);
        let config = ProjectConfig::default();
        let resolver = DependencyResolver::new(&client, &config);

        let mut manifest = PackageManifest::default();
        manifest.dependencies.insert(
            corex_manifest::PackageName::parse("peer-dep-pkg").unwrap(),
            "^1.0.0".to_string(),
        );

        let graph = resolver.resolve(&manifest).unwrap();
        assert_eq!(graph.nodes.len(), 3); // peer-dep-pkg, react-dom, react
    }

    #[test]
    fn test_resolve_platform_tier_mismatch() {
        let fixtures = get_fixtures_dir();
        let client = MockRegistryClient::new(fixtures);
        let config = ProjectConfig::default();

        let resolver_linux =
            DependencyResolver::new(&client, &config).with_platform("linux", "x86_64");

        let mut manifest = PackageManifest::default();
        manifest.dependencies.insert(
            corex_manifest::PackageName::parse("platform-pkg").unwrap(),
            "^1.0.0".to_string(),
        );

        assert!(resolver_linux.resolve(&manifest).is_err());

        let resolver_macos =
            DependencyResolver::new(&client, &config).with_platform("darwin", "aarch64");
        assert!(resolver_macos.resolve(&manifest).is_ok());
    }

    #[test]
    fn test_resolve_optional_platform_mismatch() {
        let fixtures = get_fixtures_dir();
        let client = MockRegistryClient::new(fixtures);
        let config = ProjectConfig::default();

        let resolver_linux =
            DependencyResolver::new(&client, &config).with_platform("linux", "x86_64");

        let mut manifest = PackageManifest::default();
        manifest.optional_dependencies.insert(
            corex_manifest::PackageName::parse("platform-pkg").unwrap(),
            "^1.0.0".to_string(),
        );

        let graph = resolver_linux.resolve(&manifest).unwrap();
        assert_eq!(graph.nodes.len(), 0);
    }
}
