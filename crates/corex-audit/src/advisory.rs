//! Domain types for security vulnerability advisories and audit reports.

use serde::{Deserialize, Serialize};

/// Severity levels for security advisories.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum VulnerabilitySeverity {
    /// Low severity issue.
    Low = 1,
    /// Medium severity issue.
    Medium = 2,
    /// High severity issue.
    High = 3,
    /// Critical severity issue.
    Critical = 4,
}

impl VulnerabilitySeverity {
    /// Returns human-readable label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Low => "low",
            Self::Medium => "medium",
            Self::High => "high",
            Self::Critical => "critical",
        }
    }
}

impl std::fmt::Display for VulnerabilitySeverity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// A security advisory entry.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Advisory {
    /// Advisory identifier (e.g. `CX-ADV-2026-001`).
    pub id: String,
    /// Summary title of the vulnerability.
    pub title: String,
    /// Target package name.
    pub package_name: String,
    /// Semver range string matching vulnerable versions (e.g. `<1.2.0`).
    pub vulnerable_range: String,
    /// Safe patched version if available.
    pub patched_version: Option<String>,
    /// Assigned severity level.
    pub severity: VulnerabilitySeverity,
    /// Optional CVE reference.
    pub cve: Option<String>,
    /// Optional documentation URL.
    pub url: Option<String>,
}

/// A match between a package dependency and a vulnerability advisory.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct AuditMatch {
    /// Affected package name.
    pub package_name: String,
    /// Version present in the dependency graph.
    pub installed_version: String,
    /// Matched vulnerability advisory details.
    pub advisory: Advisory,
}

/// Summary report of an advisory security audit scan.
#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct AuditSummary {
    /// Total unique packages scanned in graph.
    pub scanned_packages: usize,
    /// Total vulnerability matches found.
    pub vulnerabilities_found: usize,
    /// Number of critical severity findings.
    pub critical_count: usize,
    /// Number of high severity findings.
    pub high_count: usize,
    /// Number of medium severity findings.
    pub medium_count: usize,
    /// Number of low severity findings.
    pub low_count: usize,
    /// Detailed list of vulnerability matches.
    pub matches: Vec<AuditMatch>,
}
