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

/// Returns a small environment report without performing network operations.
#[must_use]
pub fn doctor_report() -> String {
    format!(
        "CorexPM       {VERSION}-dev\nPlatform      {}-{}\nWorkspace     bootstrap\nStore         not initialized\nRegistry      not checked\nStatus        development scaffold",
        std::env::consts::OS,
        std::env::consts::ARCH,
    )
}
