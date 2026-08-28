//! Platform capability enforcement status evaluation across operating systems.

use serde::{Deserialize, Serialize};

/// Enforcement tier of a security capability control.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum EnforcementLevel {
    /// Enforced by kernel, container, or platform OS sandbox.
    Enforced,
    /// Monitored or detected by policy engine without hard OS sandbox.
    Detected,
    /// Advisory / declared capability requirement only.
    Advisory,
}

impl EnforcementLevel {
    /// Returns human-readable label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Enforced => "enforced",
            Self::Detected => "detected",
            Self::Advisory => "advisory",
        }
    }
}

impl std::fmt::Display for EnforcementLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Security capability control category.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CapabilityCategory {
    /// Process spawning capability (`process.spawn`).
    ProcessSpawn,
    /// Network socket and HTTP access (`network.access`).
    NetworkAccess,
    /// Project root filesystem write (`filesystem.project.write`).
    FilesystemProjectWrite,
    /// Filesystem write outside project root (`filesystem.outside_project`).
    FilesystemOutsideProject,
    /// Native binary execution (`native.execute`).
    NativeExecution,
    /// Process environment variable read (`environment.read`).
    EnvironmentRead,
}

impl CapabilityCategory {
    /// Returns canonical policy string representation.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::ProcessSpawn => "process.spawn",
            Self::NetworkAccess => "network.access",
            Self::FilesystemProjectWrite => "filesystem.project.write",
            Self::FilesystemOutsideProject => "filesystem.outside_project",
            Self::NativeExecution => "native.execute",
            Self::EnvironmentRead => "environment.read",
        }
    }
}

/// Status report for a specific capability category on the current platform.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct CapabilityStatus {
    /// Capability category.
    pub category: CapabilityCategory,
    /// Canonical capability key name.
    pub key: String,
    /// Effective enforcement level on current platform.
    pub level: EnforcementLevel,
    /// Explanation of platform enforcement mechanism.
    pub description: String,
}

/// Overall platform security capability report.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct PlatformEnforcementReport {
    /// Operating system name.
    pub os: String,
    /// Architecture name.
    pub arch: String,
    /// Status entries for all capabilities.
    pub capabilities: Vec<CapabilityStatus>,
}

/// Evaluator for platform capability enforcement levels.
#[derive(Debug, Default)]
pub struct CapabilityEnforcementEvaluator;

impl CapabilityEnforcementEvaluator {
    /// Creates a new evaluator instance.
    #[must_use]
    pub fn new() -> Self {
        Self
    }

    /// Evaluates effective security capability enforcement levels for the current host OS.
    #[must_use]
    pub fn evaluate_current_platform(&self) -> PlatformEnforcementReport {
        let os = std::env::consts::OS;
        let arch = std::env::consts::ARCH;

        let capabilities = vec![
            CapabilityStatus {
                category: CapabilityCategory::ProcessSpawn,
                key: CapabilityCategory::ProcessSpawn.as_str().to_string(),
                level: EnforcementLevel::Detected,
                description: "Process spawn is monitored by Corex script executor with policy consent checks.".to_string(),
            },
            CapabilityStatus {
                category: CapabilityCategory::NetworkAccess,
                key: CapabilityCategory::NetworkAccess.as_str().to_string(),
                level: EnforcementLevel::Detected,
                description: "Network access is filtered by Corex registry and offline configuration.".to_string(),
            },
            CapabilityStatus {
                category: CapabilityCategory::FilesystemProjectWrite,
                key: CapabilityCategory::FilesystemProjectWrite.as_str().to_string(),
                level: EnforcementLevel::Enforced,
                description: "Project writes are isolated within project overlay and strict lockfile constraints.".to_string(),
            },
            CapabilityStatus {
                category: CapabilityCategory::FilesystemOutsideProject,
                key: CapabilityCategory::FilesystemOutsideProject.as_str().to_string(),
                level: EnforcementLevel::Enforced,
                description: "Global CAS objects are read-only and immutable; staging paths are isolated.".to_string(),
            },
            CapabilityStatus {
                category: CapabilityCategory::NativeExecution,
                key: CapabilityCategory::NativeExecution.as_str().to_string(),
                level: EnforcementLevel::Detected,
                description: "Native builds require explicit trust policy approval prior to execution.".to_string(),
            },
            CapabilityStatus {
                category: CapabilityCategory::EnvironmentRead,
                key: CapabilityCategory::EnvironmentRead.as_str().to_string(),
                level: EnforcementLevel::Enforced,
                description: "Script process environments are sanitized and credential headers redacted.".to_string(),
            },
        ];

        PlatformEnforcementReport {
            os: os.to_string(),
            arch: arch.to_string(),
            capabilities,
        }
    }
}
