//! Global Content-Addressed Store (CAS), locking, verification, metrics, and garbage collection.

#![forbid(unsafe_code)]

use corex_errors::{Diagnostic, ErrorFamily};
use fs2::FileExt;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{HashMap, HashSet};
use std::fs::{self, File};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

/// Metadata stored inside each committed CAS package directory in `.corex-pkg.json`.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct PackageMetadata {
    /// Package name (e.g. `react` or `@types/node`).
    pub name: String,
    /// Package version string.
    pub version: String,
    /// Canonical content-addressed SHA-256 key.
    pub cas_key: String,
    /// Registry integrity specification (e.g. `sha512-...` or `shasum`).
    pub expected_integrity: String,
    /// Unix timestamp in seconds when the package was committed.
    pub committed_at_secs: u64,
}

/// Project reference mapping record stored in `indexes/projects.json`.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, Default)]
pub struct ProjectIndex {
    /// Mapping of absolute project directory path to set of referenced CAS keys.
    pub projects: HashMap<String, Vec<String>>,
}

/// Diagnostic report generated when validating store integrity.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct VerificationReport {
    /// Total valid committed packages.
    pub valid_count: usize,
    /// Total corrupted package directories.
    pub corrupt_count: usize,
    /// Details of corrupted or tampered package keys.
    pub corrupt_details: Vec<String>,
}

/// Physical metrics and allocation statistics of the Content-Addressed Store.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct StoreStats {
    /// Root path of the store.
    pub store_path: PathBuf,
    /// Total count of unique committed package objects.
    pub package_count: usize,
    /// Total physical bytes occupied by committed packages on disk.
    pub physical_bytes: u64,
    /// Sum of bytes referenced across registered projects.
    pub logical_bytes: u64,
    /// Total physical bytes saved by package deduplication across projects.
    pub saved_bytes: u64,
    /// Total registered projects referencing packages in the store.
    pub project_count: usize,
}

/// Summary of packages pruned during garbage collection.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct PruneSummary {
    /// Total count of unreferenced package objects removed.
    pub removed_count: usize,
    /// Total physical bytes reclaimed.
    pub reclaimed_bytes: u64,
    /// CAS keys of pruned packages.
    pub pruned_keys: Vec<String>,
}

/// Corex Content-Addressed Store manager.
#[derive(Clone, Debug)]
pub struct Store {
    root: PathBuf,
}

impl Store {
    /// Initializes or opens a `Store` at `root` directory (typically `~/.corex/store/v1`).
    #[must_use]
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }

    /// Root directory path.
    #[must_use]
    pub fn root(&self) -> &Path {
        &self.root
    }

    /// Packages storage directory (`packages/sha256/`).
    #[must_use]
    pub fn packages_dir(&self) -> PathBuf {
        self.root.join("packages").join("sha256")
    }

    /// Store indexes directory (`indexes/`).
    #[must_use]
    pub fn indexes_dir(&self) -> PathBuf {
        self.root.join("indexes")
    }

    /// Store temporary staging directory (`temp/`).
    #[must_use]
    pub fn temp_dir(&self) -> PathBuf {
        self.root.join("temp")
    }

    /// Store cross-process locks directory (`locks/`).
    #[must_use]
    pub fn locks_dir(&self) -> PathBuf {
        self.root.join("locks")
    }

    /// Project reference index file path (`indexes/projects.json`).
    #[must_use]
    pub fn projects_file(&self) -> PathBuf {
        self.indexes_dir().join("projects.json")
    }

    /// Acquires an exclusive cross-process lock for store mutation operations.
    fn lock(&self) -> Result<File, Diagnostic> {
        let locks_dir = self.locks_dir();
        fs::create_dir_all(&locks_dir).map_err(|e| {
            Diagnostic::new(
                ErrorFamily::Store,
                1,
                format!("failed creating locks dir `{}`: {e}", locks_dir.display()),
            )
        })?;

        let lock_path = locks_dir.join("store.lock");
        let file = File::create(&lock_path).map_err(|e| {
            Diagnostic::new(
                ErrorFamily::Store,
                1,
                format!("failed creating lock file `{}`: {e}", lock_path.display()),
            )
        })?;

        file.lock_exclusive().map_err(|e| {
            Diagnostic::new(
                ErrorFamily::Store,
                1,
                format!("failed acquiring cross-process store lock: {e}"),
            )
        })?;

        Ok(file)
    }

    /// Calculates the canonical CAS SHA-256 key for an extracted package directory.
    ///
    /// Iterates over all files in `dir` in sorted relative path order and hashes relative path + file contents.
    ///
    /// # Errors
    /// Returns `Diagnostic` if reading files fails.
    pub fn calculate_cas_key(dir: &Path) -> Result<String, Diagnostic> {
        let mut files = Vec::new();
        collect_dir_files(dir, dir, &mut files)?;
        files.sort_by(|a, b| a.0.cmp(&b.0));

        let mut hasher = Sha256::new();
        for (rel_path, abs_path) in files {
            hasher.update(rel_path.as_bytes());
            let content = fs::read(&abs_path).map_err(|e| {
                Diagnostic::new(
                    ErrorFamily::Store,
                    1,
                    format!(
                        "failed reading file `{}` for CAS key: {e}",
                        abs_path.display()
                    ),
                )
            })?;
            hasher.update(&content);
        }

        Ok(hex::encode(hasher.finalize()))
    }

    /// Atomically commits an extracted package from `staging_dir` into the immutable CAS store.
    ///
    /// # Errors
    /// Returns `Diagnostic` if integrity calculation, locking, or atomic move fails.
    pub fn commit_package(
        &self,
        staging_dir: &Path,
        name: &str,
        version: &str,
        expected_integrity: &str,
    ) -> Result<PackageMetadata, Diagnostic> {
        let cas_key = Self::calculate_cas_key(staging_dir)?;
        let prefix = &cas_key[..2];
        let target_dir = self.packages_dir().join(prefix).join(&cas_key);

        let now_secs = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let metadata = PackageMetadata {
            name: name.to_owned(),
            version: version.to_owned(),
            cas_key: cas_key.clone(),
            expected_integrity: expected_integrity.to_owned(),
            committed_at_secs: now_secs,
        };

        let _guard = self.lock()?;

        // If target already exists, validate metadata and reuse existing package (race condition win)
        if target_dir.exists() {
            let meta_file = target_dir.join(".corex-pkg.json");
            if meta_file.exists() {
                if let Ok(content) = fs::read_to_string(&meta_file) {
                    if let Ok(existing_meta) = serde_json::from_str::<PackageMetadata>(&content) {
                        let _ = fs::remove_dir_all(staging_dir);
                        return Ok(existing_meta);
                    }
                }
            }
        }

        // Write package metadata file into staging dir before moving
        let meta_json = serde_json::to_string_pretty(&metadata).map_err(|e| {
            Diagnostic::new(
                ErrorFamily::Store,
                1,
                format!("failed serializing package metadata: {e}"),
            )
        })?;
        fs::write(staging_dir.join(".corex-pkg.json"), meta_json).map_err(|e| {
            Diagnostic::new(
                ErrorFamily::Store,
                1,
                format!("failed writing `.corex-pkg.json` in staging dir: {e}"),
            )
        })?;

        // Ensure parent prefix dir exists
        if let Some(parent) = target_dir.parent() {
            fs::create_dir_all(parent).map_err(|e| {
                Diagnostic::new(
                    ErrorFamily::Store,
                    1,
                    format!(
                        "failed creating package parent dir `{}`: {e}",
                        parent.display()
                    ),
                )
            })?;
        }

        // Atomic move into CAS
        fs::rename(staging_dir, &target_dir).map_err(|e| {
            Diagnostic::new(
                ErrorFamily::Store,
                1,
                format!(
                    "failed committing package to CAS `{}`: {e}",
                    target_dir.display()
                ),
            )
        })?;

        // Set read-only permissions on committed target to guarantee immutability
        set_readonly_recursive(&target_dir, true)?;

        Ok(metadata)
    }

    /// Registers project reference association with CAS keys in `indexes/projects.json`.
    ///
    /// # Errors
    /// Returns `Diagnostic` if updating index file fails.
    pub fn register_project_references(
        &self,
        project_dir: &Path,
        cas_keys: &[String],
    ) -> Result<(), Diagnostic> {
        let _guard = self.lock()?;

        let mut index = self.read_project_index()?;
        let canonical_proj = project_dir
            .canonicalize()
            .unwrap_or_else(|_| project_dir.to_path_buf())
            .to_string_lossy()
            .to_string();

        index.projects.insert(canonical_proj, cas_keys.to_vec());
        self.write_project_index(&index)
    }

    /// Reads project index from disk.
    fn read_project_index(&self) -> Result<ProjectIndex, Diagnostic> {
        let p = self.projects_file();
        if p.exists() {
            let content = fs::read_to_string(&p).map_err(|e| {
                Diagnostic::new(
                    ErrorFamily::Store,
                    1,
                    format!("failed reading project index `{}`: {e}", p.display()),
                )
            })?;
            serde_json::from_str(&content).map_err(|e| {
                Diagnostic::new(
                    ErrorFamily::Store,
                    1,
                    format!("failed parsing project index `{}`: {e}", p.display()),
                )
            })
        } else {
            Ok(ProjectIndex::default())
        }
    }

    /// Writes project index to disk.
    fn write_project_index(&self, index: &ProjectIndex) -> Result<(), Diagnostic> {
        let p = self.projects_file();
        if let Some(parent) = p.parent() {
            fs::create_dir_all(parent).map_err(|e| {
                Diagnostic::new(
                    ErrorFamily::Store,
                    1,
                    format!(
                        "failed creating index directory `{}`: {e}",
                        parent.display()
                    ),
                )
            })?;
        }
        let json_str = serde_json::to_string_pretty(index).map_err(|e| {
            Diagnostic::new(
                ErrorFamily::Store,
                1,
                format!("failed serializing project index: {e}"),
            )
        })?;
        fs::write(&p, json_str).map_err(|e| {
            Diagnostic::new(
                ErrorFamily::Store,
                1,
                format!("failed writing project index `{}`: {e}", p.display()),
            )
        })
    }

    /// Scans the entire CAS store and verifies integrity of all committed packages.
    ///
    /// # Errors
    /// Returns `Diagnostic` if store scan fails.
    pub fn verify(&self) -> Result<VerificationReport, Diagnostic> {
        let pkgs_dir = self.packages_dir();
        let mut valid_count = 0usize;
        let mut corrupt_count = 0usize;
        let mut corrupt_details = Vec::new();

        if !pkgs_dir.exists() {
            return Ok(VerificationReport {
                valid_count: 0,
                corrupt_count: 0,
                corrupt_details: Vec::new(),
            });
        }

        let prefix_dirs = fs::read_dir(&pkgs_dir).map_err(|e| {
            Diagnostic::new(
                ErrorFamily::Store,
                1,
                format!(
                    "failed reading store packages directory `{}`: {e}",
                    pkgs_dir.display()
                ),
            )
        })?;

        for p_entry in prefix_dirs.flatten() {
            if !p_entry.path().is_dir() {
                continue;
            }
            if let Ok(pkg_dirs) = fs::read_dir(p_entry.path()) {
                for entry in pkg_dirs.flatten() {
                    let path = entry.path();
                    if !path.is_dir() {
                        continue;
                    }
                    let key = path
                        .file_name()
                        .map_or_else(String::new, |s| s.to_string_lossy().to_string());

                    let meta_file = path.join(".corex-pkg.json");
                    if !meta_file.exists() {
                        corrupt_count += 1;
                        corrupt_details.push(format!("{key}: missing `.corex-pkg.json`"));
                        continue;
                    }

                    match Self::calculate_cas_key(&path) {
                        Ok(actual_key) => {
                            if actual_key == key {
                                valid_count += 1;
                            } else {
                                corrupt_count += 1;
                                corrupt_details
                                    .push(format!("{key}: CAS key mismatch (got {actual_key})"));
                            }
                        }
                        Err(err) => {
                            corrupt_count += 1;
                            corrupt_details.push(format!("{key}: read failure: {err}"));
                        }
                    }
                }
            }
        }

        Ok(VerificationReport {
            valid_count,
            corrupt_count,
            corrupt_details,
        })
    }

    /// Computes summary statistics and physical space usage of the store.
    ///
    /// # Errors
    /// Returns `Diagnostic` if store scan fails.
    pub fn stats(&self) -> Result<StoreStats, Diagnostic> {
        let pkgs_dir = self.packages_dir();
        let mut package_count = 0usize;
        let mut physical_bytes = 0u64;

        let mut package_sizes = HashMap::new();

        if pkgs_dir.exists() {
            if let Ok(prefix_dirs) = fs::read_dir(&pkgs_dir) {
                for p_entry in prefix_dirs.flatten() {
                    if !p_entry.path().is_dir() {
                        continue;
                    }
                    if let Ok(pkg_dirs) = fs::read_dir(p_entry.path()) {
                        for entry in pkg_dirs.flatten() {
                            let path = entry.path();
                            if !path.is_dir() {
                                continue;
                            }
                            let key = path
                                .file_name()
                                .map_or_else(String::new, |s| s.to_string_lossy().to_string());

                            let size = compute_dir_size(&path);
                            package_count += 1;
                            physical_bytes += size;
                            package_sizes.insert(key, size);
                        }
                    }
                }
            }
        }

        let index = self.read_project_index().unwrap_or_default();
        let project_count = index.projects.len();
        let mut logical_bytes = 0u64;

        for keys in index.projects.values() {
            for key in keys {
                if let Some(&sz) = package_sizes.get(key) {
                    logical_bytes += sz;
                }
            }
        }

        let saved_bytes = logical_bytes.saturating_sub(physical_bytes);

        Ok(StoreStats {
            store_path: self.root.clone(),
            package_count,
            physical_bytes,
            logical_bytes,
            saved_bytes,
            project_count,
        })
    }

    /// Garbage collects unreferenced packages older than `grace_period_secs`.
    ///
    /// # Errors
    /// Returns `Diagnostic` if lock acquisition or directory deletion fails.
    pub fn prune(&self, grace_period_secs: u64) -> Result<PruneSummary, Diagnostic> {
        let _guard = self.lock()?;

        let index = self.read_project_index().unwrap_or_default();
        let mut active_keys = HashSet::new();
        for keys in index.projects.values() {
            for k in keys {
                active_keys.insert(k.clone());
            }
        }

        let now_secs = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let cutoff = now_secs.saturating_sub(grace_period_secs);

        let pkgs_dir = self.packages_dir();
        let mut removed_count = 0usize;
        let mut reclaimed_bytes = 0u64;
        let mut pruned_keys = Vec::new();

        if !pkgs_dir.exists() {
            return Ok(PruneSummary {
                removed_count: 0,
                reclaimed_bytes: 0,
                pruned_keys: Vec::new(),
            });
        }

        if let Ok(prefix_dirs) = fs::read_dir(&pkgs_dir) {
            for p_entry in prefix_dirs.flatten() {
                let p_path = p_entry.path();
                if !p_path.is_dir() {
                    continue;
                }
                if let Ok(pkg_dirs) = fs::read_dir(&p_path) {
                    for entry in pkg_dirs.flatten() {
                        let path = entry.path();
                        if !path.is_dir() {
                            continue;
                        }
                        let key = path
                            .file_name()
                            .map_or_else(String::new, |s| s.to_string_lossy().to_string());

                        if active_keys.contains(&key) {
                            continue;
                        }

                        let meta_file = path.join(".corex-pkg.json");
                        let mut committed_at = 0u64;
                        if meta_file.exists() {
                            if let Ok(content) = fs::read_to_string(&meta_file) {
                                if let Ok(meta) = serde_json::from_str::<PackageMetadata>(&content)
                                {
                                    committed_at = meta.committed_at_secs;
                                }
                            }
                        }

                        if committed_at <= cutoff {
                            let size = compute_dir_size(&path);
                            // Temporarily allow write perms to enable deletion
                            let _ = set_readonly_recursive(&path, false);
                            if fs::remove_dir_all(&path).is_ok() {
                                removed_count += 1;
                                reclaimed_bytes += size;
                                pruned_keys.push(key);
                            }
                        }
                    }
                }
            }
        }

        Ok(PruneSummary {
            removed_count,
            reclaimed_bytes,
            pruned_keys,
        })
    }
}

fn collect_dir_files(
    root: &Path,
    dir: &Path,
    acc: &mut Vec<(String, PathBuf)>,
) -> Result<(), Diagnostic> {
    let entries = fs::read_dir(dir).map_err(|e| {
        Diagnostic::new(
            ErrorFamily::Store,
            1,
            format!("failed reading dir `{}`: {e}", dir.display()),
        )
    })?;

    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_dir_files(root, &path, acc)?;
        } else if path.is_file() {
            if path
                .file_name()
                .is_some_and(|name| name == ".corex-pkg.json")
            {
                continue;
            }
            if let Ok(rel) = path.strip_prefix(root) {
                acc.push((rel.to_string_lossy().to_string(), path));
            }
        }
    }
    Ok(())
}

fn compute_dir_size(dir: &Path) -> u64 {
    let mut total = 0u64;
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                total += compute_dir_size(&path);
            } else if let Ok(meta) = path.metadata() {
                total += meta.len();
            }
        }
    }
    total
}

fn set_readonly_recursive(dir: &Path, readonly: bool) -> Result<(), Diagnostic> {
    if !readonly {
        if let Ok(meta) = dir.metadata() {
            let mut perms = meta.permissions();
            perms.set_readonly(false);
            let _ = fs::set_permissions(dir, perms);
        }
    }

    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                set_readonly_recursive(&path, readonly)?;
            } else if path.is_file() {
                if let Ok(meta) = path.metadata() {
                    let mut perms = meta.permissions();
                    perms.set_readonly(readonly);
                    let _ = fs::set_permissions(&path, perms);
                }
            }
        }
    }

    if readonly {
        if let Ok(meta) = dir.metadata() {
            let mut perms = meta.permissions();
            perms.set_readonly(true);
            let _ = fs::set_permissions(dir, perms);
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn clean_test_dir(dir: &Path) {
        if dir.exists() {
            let _ = set_readonly_recursive(dir, false);
            let _ = fs::remove_dir_all(dir);
        }
    }

    #[test]
    fn test_store_atomic_commit_and_verification() {
        let store_root = std::env::temp_dir().join("corex_test_store_commit");
        clean_test_dir(&store_root);

        let store = Store::new(&store_root);

        let staging = std::env::temp_dir().join("corex_test_staging_pkg");
        clean_test_dir(&staging);
        fs::create_dir_all(&staging).unwrap();
        fs::write(staging.join("index.js"), "console.log('hello');").unwrap();

        let meta = store
            .commit_package(&staging, "my-pkg", "1.0.0", "sha512-dummy")
            .unwrap();

        assert_eq!(meta.name, "my-pkg");
        assert_eq!(meta.version, "1.0.0");
        assert!(!staging.exists());

        let report = store.verify().unwrap();
        assert_eq!(report.valid_count, 1);
        assert_eq!(report.corrupt_count, 0);

        let stats = store.stats().unwrap();
        assert_eq!(stats.package_count, 1);
        assert!(stats.physical_bytes > 0);

        // Clean up
        clean_test_dir(&store_root);
    }

    #[test]
    fn test_store_prune() {
        let store_root = std::env::temp_dir().join("corex_test_store_prune");
        clean_test_dir(&store_root);

        let store = Store::new(&store_root);

        let staging = std::env::temp_dir().join("corex_test_staging_prune");
        clean_test_dir(&staging);
        fs::create_dir_all(&staging).unwrap();
        fs::write(staging.join("lib.js"), "module.exports = {};").unwrap();

        let meta = store
            .commit_package(&staging, "temp-pkg", "0.1.0", "sha512-test")
            .unwrap();

        let prune_res = store.prune(0).unwrap();
        assert_eq!(prune_res.removed_count, 1);
        assert_eq!(prune_res.pruned_keys, vec![meta.cas_key]);

        clean_test_dir(&store_root);
    }
}
