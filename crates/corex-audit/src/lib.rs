//! `CorexPM` advisory vulnerability matching and security policy auditing pipeline.

pub mod advisory;
pub mod engine;
pub mod policy_audit;

pub use advisory::{Advisory, AuditMatch, AuditSummary, VulnerabilitySeverity};
pub use engine::AuditEngine;
pub use policy_audit::{PolicyAuditReport, PolicyAuditor, PolicyViolation};

#[cfg(test)]
mod tests {
    use super::*;
    use corex_graph::{DependencyGraph, NodeId, PackageId, PackageNode, VersionId};
    use corex_manifest::PackageName;
    use corex_semver::Version;
    use std::collections::BTreeSet;

    fn build_test_graph(packages: &[(&str, &str)]) -> DependencyGraph {
        let mut graph = DependencyGraph::default();
        for (i, (name, version)) in packages.iter().enumerate() {
            let pkg_name = PackageName::parse(*name).unwrap();
            let ver = Version::parse(version).unwrap();
            let node_id = NodeId::new(i);
            let node = PackageNode {
                id: node_id,
                package: PackageId::new(pkg_name),
                version: VersionId::new(ver),
                peer_context: None,
                dist_url: String::new(),
                integrity: String::new(),
                dependencies: BTreeSet::new(),
            };
            graph.nodes.insert(node_id, node);
        }
        graph
    }

    #[test]
    fn test_audit_engine_vulnerability_matching() {
        let graph = build_test_graph(&[("legacy-tar", "1.0.0"), ("lodash", "4.17.20")]);

        let engine = AuditEngine::new();
        let summary = engine.audit_graph(&graph, None, &[]).unwrap();

        assert_eq!(summary.scanned_packages, 2);
        assert_eq!(summary.vulnerabilities_found, 2);
        assert_eq!(summary.critical_count, 1);
        assert_eq!(summary.high_count, 1);
    }

    #[test]
    fn test_audit_engine_severity_and_ignore_filtering() {
        let graph = build_test_graph(&[("legacy-tar", "1.0.0"), ("validator", "13.0.0")]);

        let engine = AuditEngine::new();

        // Filter min_severity High (should ignore Medium validator finding)
        let summary_high = engine
            .audit_graph(&graph, Some(VulnerabilitySeverity::High), &[])
            .unwrap();
        assert_eq!(summary_high.vulnerabilities_found, 1);

        // Ignore CX-ADV-2026-001
        let summary_ignored = engine
            .audit_graph(&graph, None, &["CX-ADV-2026-001".to_string()])
            .unwrap();
        assert_eq!(summary_ignored.vulnerabilities_found, 1);
        assert_eq!(summary_ignored.matches[0].advisory.id, "CX-ADV-2026-003");
    }
}
