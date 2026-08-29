//! `CorexPM` application orchestration.

use corex_config::ProjectConfig;
use corex_graph::DependencyGraph;
use corex_registry::{MockRegistryClient, RegistryPackageMetadata};

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
    let s_path = default_store_dir();
    let store_desc = if s_path.exists() {
        if let Ok(st) = store_stats(None) {
            format!("{} ({} packages)", s_path.display(), st.package_count)
        } else {
            format!("{} (initialized)", s_path.display())
        }
    } else {
        format!("{} (not initialized)", s_path.display())
    };

    DoctorInfo {
        version: format!("{VERSION}-dev"),
        platform: format!("{}-{}", std::env::consts::OS, std::env::consts::ARCH),
        tier: format!("{tier_name} ({tier_desc})"),
        workspace: "bootstrap".to_string(),
        store: store_desc,
        registry: "not checked".to_string(),
        status: "ready".to_string(),
    }
}

/// Returns a small environment report without performing network operations.
#[must_use]
pub fn doctor_report() -> String {
    doctor_info().to_string()
}

/// Resolves a package.json content to a resolved dependency graph.
///
/// # Errors
///
/// Returns a [`Diagnostic`] when parsing the manifest, loading mock registry metadata,
/// or resolving dependencies fails.
pub fn resolve_manifest(
    manifest_content: &str,
    context: &CorexContext,
    fixtures_dir: &std::path::Path,
) -> Result<DependencyGraph, corex_errors::Diagnostic> {
    use corex_manifest::PackageManifest;
    use corex_registry::MockRegistryClient;
    use corex_resolver::DependencyResolver;

    let manifest = PackageManifest::parse_json(manifest_content)?;
    let client = MockRegistryClient::new(fixtures_dir);
    let resolver = DependencyResolver::new(&client, &context.config);
    resolver.resolve(&manifest)
}

/// Fetches registry package metadata for the given package name.
///
/// # Errors
///
/// Returns a [`Diagnostic`] when the package name is invalid or the metadata is not found.
pub fn fetch_package_info(
    package_name: &str,
    fixtures_dir: &std::path::Path,
) -> Result<RegistryPackageMetadata, corex_errors::Diagnostic> {
    use corex_errors::ErrorFamily;
    use corex_manifest::PackageName;
    use corex_registry::RegistryClient;

    let name = PackageName::parse(package_name).map_err(|_| {
        corex_errors::Diagnostic::new(
            ErrorFamily::Registry,
            4,
            format!("invalid package name `{package_name}`"),
        )
    })?;

    let client = MockRegistryClient::new(fixtures_dir);
    client.fetch_metadata(&name)
}

/// Computes the default global Corex store path (`~/.corex/store/v1`).
#[must_use]
pub fn default_store_dir() -> std::path::PathBuf {
    dirs_home_or_temp().join(".corex").join("store").join("v1")
}

/// Computes the default global Corex cache path (`~/.corex/cache`).
#[must_use]
pub fn default_cache_dir() -> std::path::PathBuf {
    dirs_home_or_temp().join(".corex").join("cache")
}

fn dirs_home_or_temp() -> std::path::PathBuf {
    std::env::var_os("HOME")
        .or_else(|| std::env::var_os("USERPROFILE"))
        .map_or_else(std::env::temp_dir, std::path::PathBuf::from)
}

/// Returns store path string.
#[must_use]
pub fn store_path(custom_path: Option<&std::path::Path>) -> std::path::PathBuf {
    custom_path.map_or_else(default_store_dir, std::path::PathBuf::from)
}

/// Returns cache path string.
#[must_use]
pub fn cache_path(custom_path: Option<&std::path::Path>) -> std::path::PathBuf {
    custom_path.map_or_else(default_cache_dir, std::path::PathBuf::from)
}

/// Returns store statistics report.
///
/// # Errors
/// Returns `Diagnostic` if store scan fails.
pub fn store_stats(
    custom_path: Option<&std::path::Path>,
) -> Result<corex_store::StoreStats, corex_errors::Diagnostic> {
    let store = corex_store::Store::new(store_path(custom_path));
    store.stats()
}

/// Performs verification on committed store packages.
///
/// # Errors
/// Returns `Diagnostic` if store verification fails.
pub fn store_verify(
    custom_path: Option<&std::path::Path>,
) -> Result<corex_store::VerificationReport, corex_errors::Diagnostic> {
    let store = corex_store::Store::new(store_path(custom_path));
    store.verify()
}

/// Prunes unreferenced CAS packages from store after grace period.
///
/// # Errors
/// Returns `Diagnostic` if prune operation fails.
pub fn store_prune(
    custom_path: Option<&std::path::Path>,
    grace_period_secs: u64,
) -> Result<corex_store::PruneSummary, corex_errors::Diagnostic> {
    let store = corex_store::Store::new(store_path(custom_path));
    store.prune(grace_period_secs)
}

/// Returns cache status summary.
///
/// # Errors
/// Returns `Diagnostic` if reading cache directory fails.
pub fn cache_status(
    custom_path: Option<&std::path::Path>,
    mode: corex_cache::CacheMode,
) -> Result<corex_cache::CacheStatus, corex_errors::Diagnostic> {
    let manager = corex_cache::CacheManager::new(cache_path(custom_path), mode);
    manager.status()
}

/// Cleans cache contents.
///
/// # Errors
/// Returns `Diagnostic` if cleaning cache directory fails.
pub fn cache_clean(custom_path: Option<&std::path::Path>) -> Result<(), corex_errors::Diagnostic> {
    let manager =
        corex_cache::CacheManager::new(cache_path(custom_path), corex_cache::CacheMode::Online);
    manager.clean()
}

/// Performs full end-to-end installation and isolated materialization.
///
/// # Errors
/// Returns `Diagnostic` if installation or reconciliation fails.
pub fn install_project(
    project_root: &std::path::Path,
    context: &CorexContext,
    fixtures_dir: &std::path::Path,
) -> Result<corex_installer::InstallResult, corex_errors::Diagnostic> {
    let installer = corex_installer::InstallerService::new();
    installer.install(project_root, &context.config, fixtures_dir, None)
}

/// Adds a new dependency to `package.json` and reinstalls.
///
/// # Errors
/// Returns `Diagnostic` if modifying manifest or installing fails.
pub fn add_dependency(
    project_root: &std::path::Path,
    context: &CorexContext,
    fixtures_dir: &std::path::Path,
    package_name: &str,
    version_spec: Option<&str>,
    is_dev: bool,
) -> Result<corex_installer::InstallResult, corex_errors::Diagnostic> {
    let installer = corex_installer::InstallerService::new();
    installer.add_dependency(
        project_root,
        &context.config,
        fixtures_dir,
        None,
        package_name,
        version_spec,
        is_dev,
    )
}

/// Removes a dependency from `package.json` and reinstalls.
///
/// # Errors
/// Returns `Diagnostic` if modifying manifest or installing fails.
pub fn remove_dependency(
    project_root: &std::path::Path,
    context: &CorexContext,
    fixtures_dir: &std::path::Path,
    package_name: &str,
) -> Result<corex_installer::InstallResult, corex_errors::Diagnostic> {
    let installer = corex_installer::InstallerService::new();
    installer.remove_dependency(
        project_root,
        &context.config,
        fixtures_dir,
        None,
        package_name,
    )
}

/// Lists installed direct and transitive dependencies.
///
/// # Errors
/// Returns `Diagnostic` if resolving or inspecting dependencies fails.
pub fn list_dependencies(
    project_root: &std::path::Path,
    context: &CorexContext,
    fixtures_dir: &std::path::Path,
) -> Result<serde_json::Value, corex_errors::Diagnostic> {
    let installer = corex_installer::InstallerService::new();
    installer.list_dependencies(project_root, &context.config, fixtures_dir)
}

/// Locates script `script_name` command string in `package.json`.
///
/// # Errors
/// Returns `Diagnostic` if script is missing.
pub fn run_script(
    project_root: &std::path::Path,
    script_name: &str,
) -> Result<String, corex_errors::Diagnostic> {
    let installer = corex_installer::InstallerService::new();
    installer.run_script(project_root, script_name)
}

/// Locates binary `binary_name` path in `node_modules/.bin/`.
///
/// # Errors
/// Returns `Diagnostic` if binary is missing.
pub fn exec_binary(
    project_root: &std::path::Path,
    binary_name: &str,
) -> Result<std::path::PathBuf, corex_errors::Diagnostic> {
    let installer = corex_installer::InstallerService::new();
    installer.exec_binary(project_root, binary_name)
}

/// Performs frozen installation without modifying lockfile or manifests (`corexpm ci`).
///
/// # Errors
/// Returns `Diagnostic` if lockfile is missing or out of sync with manifest.
pub fn install_project_frozen(
    project_root: &std::path::Path,
    context: &CorexContext,
    fixtures_dir: &std::path::Path,
) -> Result<corex_installer::InstallResult, corex_errors::Diagnostic> {
    let installer = corex_installer::InstallerService::new();
    installer.install_frozen(project_root, &context.config, fixtures_dir, None)
}

/// Verifies `corex.lock.json` lockfile integrity and sync with `package.json`.
///
/// # Errors
/// Returns `Diagnostic` if lockfile missing or mismatched.
pub fn verify_lockfile(
    project_root: &std::path::Path,
) -> Result<corex_lockfile::Lockfile, corex_errors::Diagnostic> {
    let lockfile_path = project_root.join("corex.lock.json");
    if !lockfile_path.exists() {
        return Err(corex_errors::Diagnostic::new(
            corex_errors::ErrorFamily::Lockfile,
            1,
            format!(
                "lockfile `corex.lock.json` not found in `{}`",
                project_root.display()
            ),
        ));
    }
    let content = std::fs::read_to_string(&lockfile_path).map_err(|e| {
        corex_errors::Diagnostic::new(
            corex_errors::ErrorFamily::Lockfile,
            2,
            format!("failed reading lockfile: {e}"),
        )
    })?;

    let lockfile = corex_lockfile::Lockfile::from_json(&content)?;
    let manifest_path = project_root.join("package.json");
    if manifest_path.exists() {
        if let Ok(manifest_content) = std::fs::read_to_string(&manifest_path) {
            if let Ok(manifest) = corex_manifest::PackageManifest::parse_json(&manifest_content) {
                lockfile.validate_against_manifest(&manifest)?;
            }
        }
    }
    lockfile.validate_integrity()?;
    Ok(lockfile)
}

/// Approves dependency lifecycle scripts for package `package_name`.
///
/// # Errors
/// Returns `Diagnostic` if saving trust store fails.
pub fn approve_trust(
    project_root: &std::path::Path,
    package_name: &str,
) -> Result<(), corex_errors::Diagnostic> {
    let engine = corex_policy::PolicyEngine::new();
    engine.approve_package(project_root, package_name)
}

/// Denies dependency lifecycle scripts for package `package_name`.
///
/// # Errors
/// Returns `Diagnostic` if saving trust store fails.
pub fn deny_trust(
    project_root: &std::path::Path,
    package_name: &str,
) -> Result<(), corex_errors::Diagnostic> {
    let engine = corex_policy::PolicyEngine::new();
    engine.deny_package(project_root, package_name)
}

/// Returns the project `TrustStore`.
///
/// # Errors
/// Returns `Diagnostic` if loading trust store fails.
pub fn list_trust(
    project_root: &std::path::Path,
) -> Result<corex_policy::TrustStore, corex_errors::Diagnostic> {
    let trust_path = project_root.join(".corex").join("trust.json");
    corex_policy::TrustStore::load_from_path(&trust_path)
}

/// Generates detailed `PermissionsReport` for all project dependencies.
///
/// # Errors
/// Returns `Diagnostic` if reading manifest or resolving graph fails.
pub fn get_permissions_report(
    project_root: &std::path::Path,
    context: &CorexContext,
    fixtures_dir: &std::path::Path,
) -> Result<corex_policy::PermissionsReport, corex_errors::Diagnostic> {
    let manifest_path = project_root.join("package.json");
    let manifest_text = std::fs::read_to_string(&manifest_path).map_err(|e| {
        corex_errors::Diagnostic::new(
            corex_errors::ErrorFamily::Resolve,
            1,
            format!("failed reading `package.json`: {e}"),
        )
    })?;

    let manifest = corex_manifest::PackageManifest::parse_json(&manifest_text)?;
    let client = corex_registry::MockRegistryClient::new(fixtures_dir);
    let resolver = corex_resolver::DependencyResolver::new(&client, &context.config);
    let graph = resolver.resolve(&manifest)?;

    let engine = corex_policy::PolicyEngine::new();
    Ok(engine.generate_permissions_report(project_root, &graph, false))
}

/// Executes project script `script_name` with secret redaction and sanitized environment.
///
/// # Errors
/// Returns `Diagnostic` if script is missing or execution fails.
pub fn run_project_script(
    project_root: &std::path::Path,
    script_name: &str,
) -> Result<corex_scripts::ScriptExecutionResult, corex_errors::Diagnostic> {
    let installer = corex_installer::InstallerService::new();
    let cmd_str = installer.run_script(project_root, script_name)?;
    let executor = corex_scripts::ScriptExecutor::new();
    executor.execute_script(project_root, project_root, "root", script_name, &cmd_str)
}

/// Discovers and lists workspace packages and metadata.
///
/// # Errors
/// Returns `Diagnostic` if workspace discovery fails.
pub fn list_workspace_members(
    start_dir: &std::path::Path,
) -> Result<corex_workspace::WorkspaceMetadata, corex_errors::Diagnostic> {
    let discovery = corex_workspace::WorkspaceDiscovery::new();
    discovery.discover(start_dir)
}

/// Lists workspace packages changed from git status or explicit file path list.
///
/// # Errors
/// Returns `Diagnostic` if workspace discovery or graph building fails.
pub fn list_changed_packages(
    start_dir: &std::path::Path,
    paths: Option<&[std::path::PathBuf]>,
) -> Result<std::collections::BTreeSet<String>, corex_errors::Diagnostic> {
    let discovery = corex_workspace::WorkspaceDiscovery::new();
    let metadata = discovery.discover(start_dir)?;
    let graph = corex_workspace::WorkspaceGraph::build(&metadata)?;
    let calculator = corex_workspace::WorkspaceChanged::new();

    let changed_paths = if let Some(p) = paths {
        p.to_vec()
    } else {
        calculator.detect_git_changed_paths(&metadata.root_dir)?
    };

    Ok(calculator.calculate_changed(&graph, &metadata.root_dir, &changed_paths))
}

/// Executes workspace script across target packages adhering to topological wave order.
///
/// # Errors
/// Returns `Diagnostic` if discovery, filtering, or task execution fails.
pub fn run_workspace_script(
    start_dir: &std::path::Path,
    script_name: &str,
    filter: &corex_workspace::WorkspaceFilter,
    options: &corex_workspace::TaskSchedulerOptions,
    changed_only: bool,
    affected_only: bool,
) -> Result<corex_workspace::TaskExecutionSummary, corex_errors::Diagnostic> {
    let discovery = corex_workspace::WorkspaceDiscovery::new();
    let metadata = discovery.discover(start_dir)?;
    let graph = corex_workspace::WorkspaceGraph::build(&metadata)?;
    let calculator = corex_workspace::WorkspaceChanged::new();

    let base_set = if changed_only || affected_only {
        let paths = calculator.detect_git_changed_paths(&metadata.root_dir)?;
        let changed = calculator.calculate_changed(&graph, &metadata.root_dir, &paths);
        if affected_only {
            Some(calculator.calculate_affected(&graph, &changed))
        } else {
            Some(changed)
        }
    } else {
        None
    };

    let target_packages = filter.select_packages(&graph, base_set.as_ref())?;
    let scheduler = corex_workspace::TaskScheduler::new();
    scheduler.execute_script(&graph, &target_packages, script_name, options)
}
