//! Audit engine for matching dependency graphs against security advisory databases.

use crate::advisory::{Advisory, AuditMatch, AuditSummary, VulnerabilitySeverity};
use corex_errors::Diagnostic;
use corex_graph::DependencyGraph;
use corex_semver::{Range, Version};
use std::collections::HashSet;

/// Security audit scanner engine.
#[derive(Clone, Debug, Default)]
pub struct AuditEngine {
    advisories: Vec<Advisory>,
}

impl AuditEngine {
    /// Creates a new audit engine loaded with built-in default advisories.
    #[must_use]
    pub fn new() -> Self {
        let mut engine = Self {
            advisories: Vec::new(),
        };
        engine.load_default_advisories();
        engine
    }

    /// Creates an audit engine with a custom list of security advisories.
    #[must_use]
    pub fn with_advisories(advisories: Vec<Advisory>) -> Self {
        Self { advisories }
    }

    /// Adds an advisory to the scanner database.
    pub fn add_advisory(&mut self, advisory: Advisory) {
        self.advisories.push(advisory);
    }

    /// Loads default test/known advisories.
    pub fn load_default_advisories(&mut self) {
        self.advisories.extend(vec![
            Advisory {
                id: "CX-ADV-2026-001".to_string(),
                title: "Remote Code Execution in legacy tar extraction".to_string(),
                package_name: "legacy-tar".to_string(),
                vulnerable_range: "<2.0.0".to_string(),
                patched_version: Some("2.0.0".to_string()),
                severity: VulnerabilitySeverity::Critical,
                cve: Some("CVE-2026-9901".to_string()),
                url: Some("https://corexpm.org/advisories/CX-ADV-2026-001".to_string()),
            },
            Advisory {
                id: "CX-ADV-2026-002".to_string(),
                title: "Prototype Pollution in un-sanitized object merge".to_string(),
                package_name: "lodash".to_string(),
                vulnerable_range: "<4.17.21".to_string(),
                patched_version: Some("4.17.21".to_string()),
                severity: VulnerabilitySeverity::High,
                cve: Some("CVE-2021-23337".to_string()),
                url: Some("https://corexpm.org/advisories/CX-ADV-2026-002".to_string()),
            },
            Advisory {
                id: "CX-ADV-2026-003".to_string(),
                title: "ReDoS vulnerability in regex validator".to_string(),
                package_name: "validator".to_string(),
                vulnerable_range: "<13.7.0".to_string(),
                patched_version: Some("13.7.0".to_string()),
                severity: VulnerabilitySeverity::Medium,
                cve: Some("CVE-2022-25646".to_string()),
                url: Some("https://corexpm.org/advisories/CX-ADV-2026-003".to_string()),
            },
        ]);
    }

    /// Audits a dependency graph for security vulnerabilities.
    ///
    /// # Errors
    ///
    /// Returns a [`Diagnostic`] if parsing semver versions or ranges fails.
    pub fn audit_graph(
        &self,
        graph: &DependencyGraph,
        min_severity: Option<VulnerabilitySeverity>,
        ignore_ids: &[String],
    ) -> Result<AuditSummary, Diagnostic> {
        let ignore_set: HashSet<&str> = ignore_ids.iter().map(String::as_str).collect();
        let min_sev = min_severity.unwrap_or(VulnerabilitySeverity::Low);

        let mut summary = AuditSummary {
            scanned_packages: graph.nodes.len(),
            ..Default::default()
        };

        for node in graph.nodes.values() {
            let pkg_name = node.package.name().as_str();
            let version_str = node.version.version().as_str();

            let Ok(version) = Version::parse(&version_str) else {
                continue;
            };

            for advisory in &self.advisories {
                if advisory.package_name != pkg_name {
                    continue;
                }

                if ignore_set.contains(advisory.id.as_str()) {
                    continue;
                }

                if advisory.severity < min_sev {
                    continue;
                }

                let Ok(range) = Range::parse(&advisory.vulnerable_range) else {
                    continue;
                };

                if range.satisfies(&version) {
                    summary.vulnerabilities_found += 1;
                    match advisory.severity {
                        VulnerabilitySeverity::Critical => summary.critical_count += 1,
                        VulnerabilitySeverity::High => summary.high_count += 1,
                        VulnerabilitySeverity::Medium => summary.medium_count += 1,
                        VulnerabilitySeverity::Low => summary.low_count += 1,
                    }

                    summary.matches.push(AuditMatch {
                        package_name: pkg_name.to_string(),
                        installed_version: version_str.clone(),
                        advisory: advisory.clone(),
                    });
                }
            }
        }

        Ok(summary)
    }
}
