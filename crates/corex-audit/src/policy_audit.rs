//! Security policy compliance auditor for evaluating lifecycle scripts and policy risks.

use crate::advisory::VulnerabilitySeverity;
use corex_graph::DependencyGraph;
use corex_policy::{TrustDecision, TrustStore};
use serde::{Deserialize, Serialize};

/// A security policy compliance violation finding.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct PolicyViolation {
    /// Target package name.
    pub package_name: String,
    /// Policy rule identifier.
    pub rule: String,
    /// Detailed description of the violation.
    pub description: String,
    /// Assigned severity level.
    pub severity: VulnerabilitySeverity,
}

/// Report summarizing policy compliance audit findings.
#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct PolicyAuditReport {
    /// Total rules evaluated.
    pub total_rules_evaluated: usize,
    /// Total violations found.
    pub violations_found: usize,
    /// Detailed list of violations.
    pub violations: Vec<PolicyViolation>,
}

/// Policy auditor scanner.
#[derive(Debug, Default)]
pub struct PolicyAuditor;

impl PolicyAuditor {
    /// Creates a new policy auditor.
    #[must_use]
    pub fn new() -> Self {
        Self
    }

    /// Scans dependency graph and trust policy store for compliance risks.
    #[must_use]
    pub fn audit_policy(
        &self,
        graph: &DependencyGraph,
        trust_store: &TrustStore,
    ) -> PolicyAuditReport {
        let mut report = PolicyAuditReport::default();

        for node in graph.nodes.values() {
            let pkg_name = node.package.name().as_str();

            report.total_rules_evaluated += 1;

            // Check if package has explicitly denied script decisions
            if let Some(entry) = trust_store.packages.get(pkg_name) {
                if *entry == TrustDecision::Denied {
                    report.violations_found += 1;
                    report.violations.push(PolicyViolation {
                        package_name: pkg_name.to_string(),
                        rule: "policy.script.denied".to_string(),
                        description: format!(
                            "Package `{pkg_name}` declares lifecycle scripts that are explicitly denied by trust policy"
                        ),
                        severity: VulnerabilitySeverity::Medium,
                    });
                }
            }
        }

        report
    }
}
