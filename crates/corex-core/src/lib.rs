//! `CorexPM` application orchestration.

use corex_config::ProjectConfig;

/// Build-time version exposed by the CLI and diagnostic reports.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Runtime context passed to commands and, later, core services.
#[derive(Clone, Debug, Default)]
pub struct CorexContext {
    /// Effective project configuration.
    pub config: ProjectConfig,
}

/// Structured diagnostic information about the local `CorexPM` environment.
#[derive(Clone, Debug, serde::Serialize)]
pub struct DoctorInfo {
    /// `CorexPM` version.
    pub version: String,
    /// Operating system and architecture.
    pub platform: String,
    /// Support tier under the platform tier policy.
    pub tier: String,
    /// Workspace state.
    pub workspace: String,
    /// Global store state.
    pub store: String,
    /// Registry connectivity state.
    pub registry: String,
    /// General bootstrap status.
    pub status: String,
}

impl std::fmt::Display for DoctorInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "CorexPM       {}\nPlatform      {}\nPlatform Tier {}\nWorkspace     {}\nStore         {}\nRegistry      {}\nStatus        {}",
            self.version,
            self.platform,
            self.tier,
            self.workspace,
            self.store,
            self.registry,
            self.status,
        )
    }
}

/// Detects the compatibility and support tier of the current platform.
#[must_use]
pub fn platform_tier() -> (&'static str, &'static str) {
    let os = std::env::consts::OS;
    let arch = std::env::consts::ARCH;
    match (os, arch) {
        ("macos" | "linux", "x86_64" | "aarch64") | ("windows", "x86_64") => {
            ("Tier 1", "Fully Supported")
        }
        ("windows", "aarch64") | ("freebsd", "x86_64") => ("Tier 2", "Supported (Limited CI)"),
        _ => ("Tier 3", "Unsupported (Best Effort)"),
    }
}

/// Returns diagnostic information about the local development environment.
#[must_use]
pub fn doctor_info() -> DoctorInfo {
    let (tier_name, tier_desc) = platform_tier();
    DoctorInfo {
        version: format!("{VERSION}-dev"),
        platform: format!("{}-{}", std::env::consts::OS, std::env::consts::ARCH),
        tier: format!("{tier_name} ({tier_desc})"),
        workspace: "bootstrap".to_string(),
        store: "not initialized".to_string(),
        registry: "not checked".to_string(),
        status: "development scaffold".to_string(),
    }
}

/// Returns a small environment report without performing network operations.
#[must_use]
pub fn doctor_report() -> String {
    doctor_info().to_string()
}
