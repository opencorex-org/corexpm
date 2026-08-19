//! Compact resolved dependency graph.

use corex_manifest::PackageName;
use corex_semver::{Range, Version};
use std::collections::{BTreeMap, BTreeSet};

/// Unique identifier for a resolved graph node.
#[derive(
    Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, serde::Serialize, serde::Deserialize,
)]
pub struct NodeId(usize);

impl NodeId {
    /// Creates a `NodeId` from a raw index.
    #[must_use]
    pub const fn new(id: usize) -> Self {
        Self(id)
    }

    /// Returns the raw index value.
    #[must_use]
    pub const fn index(self) -> usize {
        self.0
    }
}

/// Identifies a package by name.
#[derive(
    Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, serde::Serialize, serde::Deserialize,
)]
pub struct PackageId(PackageName);

impl PackageId {
    /// Creates a `PackageId` from a `PackageName`.
    #[must_use]
    pub const fn new(name: PackageName) -> Self {
        Self(name)
    }

    /// Returns the underlying `PackageName` reference.
    #[must_use]
    pub const fn name(&self) -> &PackageName {
        &self.0
    }
}

/// Identifies a package version.
#[derive(
    Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, serde::Serialize, serde::Deserialize,
)]
pub struct VersionId(Version);

impl VersionId {
    /// Creates a `VersionId` from a `Version`.
    #[must_use]
    pub const fn new(version: Version) -> Self {
        Self(version)
    }

    /// Returns the underlying Version reference.
    #[must_use]
    pub const fn version(&self) -> &Version {
        &self.0
    }
}

/// Kind of dependency edge in the graph.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DependencyKind {
    /// Runtime dependency.
    Dependency,
    /// Development dependency.
    DevDependency,
    /// Optional dependency.
    OptionalDependency,
    /// Peer dependency.
    PeerDependency,
}

/// Edge connecting two package nodes.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct DependencyEdge {
    /// Origin node index.
    pub from: NodeId,
    /// Target node index.
    pub to: NodeId,
    /// The kind of dependency.
    pub kind: DependencyKind,
    /// The range requirement constraint.
    pub requirement: Range,
}

/// Node representing a resolved package instance.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct PackageNode {
    /// Unique node ID in the graph.
    pub id: NodeId,
    /// Package identifier (name).
    pub package: PackageId,
    /// Resolved package version.
    pub version: VersionId,
    /// Optional resolved peer context description (e.g. peer context version dependencies).
    pub peer_context: Option<String>,
    /// Package tarball distribution URL.
    pub dist_url: String,
    /// Package integrity hash.
    pub integrity: String,
    /// Outgoing dependency edges.
    pub dependencies: BTreeSet<NodeId>,
}

/// A deterministic resolved dependency graph.
#[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize)]
pub struct DependencyGraph {
    /// All resolved package nodes.
    pub nodes: BTreeMap<NodeId, PackageNode>,
    /// All dependency edges in the graph.
    pub edges: Vec<DependencyEdge>,
    /// The root package node IDs.
    pub root_nodes: BTreeSet<NodeId>,
}

impl DependencyGraph {
    /// Adds a new node to the graph.
    pub fn add_node(
        &mut self,
        package: PackageName,
        version: Version,
        dist_url: String,
        integrity: String,
    ) -> NodeId {
        let index = self.nodes.len();
        let id = NodeId::new(index);
        let node = PackageNode {
            id,
            package: PackageId::new(package),
            version: VersionId::new(version),
            peer_context: None,
            dist_url,
            integrity,
            dependencies: BTreeSet::new(),
        };
        self.nodes.insert(id, node);
        id
    }

    /// Adds an edge connecting two nodes.
    pub fn add_edge(&mut self, from: NodeId, to: NodeId, kind: DependencyKind, requirement: Range) {
        self.edges.push(DependencyEdge {
            from,
            to,
            kind,
            requirement,
        });
        if let Some(node) = self.nodes.get_mut(&from) {
            node.dependencies.insert(to);
        }
    }
}
